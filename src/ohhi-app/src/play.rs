use std::collections::{HashMap, HashSet};
use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use ohhi_core::seed;
use ohhi_core::validator::{Filter, Validator};
use ohhi_generator::full::og::OgGenerator;
use ohhi_generator::full::toolkit::ToolkitGenerator;
use ohhi_generator::reduce::breakdown;
use ohhi_generator::generate_puzzle;
use ohhi_data::metrics::difficulty as ext_difficulty;
use ohhi_solver::carver::carve;
use ohhi_solver::structs::{Technique, TechniqueSet};
use ohhi_solver::v1::deduction::deduce_with;
use ohhi_solver::v2::propagate::propagate;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use crate::timer::Timer;

// ── Generation mode ───────────────────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum GenMode {
    /// OgGenerator + breakdown reducer. V1-deducible puzzles.
    /// Difficulty is the weighted sum of v1 technique steps.
    Og,
    /// ToolkitGenerator + carve reducer. May require guessing.
    /// Difficulty is empty-cells − v2-forced count.
    Extended,
}

// ── Technique cost weights ────────────────────────────────────────────────────

fn technique_cost(t: Technique) -> u32 {
    match t {
        Technique::PairExtension  => 1,
        Technique::GapFill        => 1,
        Technique::Saturation     => 3,
        Technique::TwinCompletion => 7,
    }
}

// ── Session ───────────────────────────────────────────────────────────────────

pub struct PlaySession {
    solution:         BitBoard,
    given:            BitBoard,
    board:            BitBoard,
    history:          Vec<BitBoard>,
    timer:            Timer,
    gen_mode:         GenMode,
    // difficulty
    technique_map:    HashMap<(usize, usize), u32>, // OG only
    difficulty_total: u32,
    difficulty_done:  u32,
    discovered:       HashSet<(usize, usize)>, // correctly-placed cells seen so far
    // counters
    total_empties:    usize,
    mistakes:         usize,
    guesses:          usize,
}

impl PlaySession {
    pub fn new(solution: BitBoard, given: BitBoard, mode: GenMode) -> Self {
        let total_empties = count_empties(&given);
        let (technique_map, difficulty_total) = match mode {
            GenMode::Og => build_og_difficulty(&given),
            GenMode::Extended => {
                let d = ext_difficulty(&given) as u32;
                (HashMap::new(), d)
            }
        };
        let board = given.clone();
        let mut timer = Timer::new();
        timer.start();
        PlaySession {
            solution,
            given,
            board,
            history: vec![],
            timer,
            gen_mode: mode,
            technique_map,
            difficulty_total,
            difficulty_done: 0,
            discovered: HashSet::new(),
            total_empties,
            mistakes: 0,
            guesses: 0,
        }
    }

    pub fn new_from_puzzle(puzzle: ohhi_generator::Puzzle, mode: GenMode) -> Self {
        Self::new(puzzle.full, puzzle.puzzle, mode)
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn board(&self) -> &BitBoard { &self.board }
    pub fn given(&self) -> &BitBoard { &self.given }
    pub fn gen_mode(&self) -> GenMode { self.gen_mode }

    pub fn is_locked(&self, pos: (usize, usize)) -> bool {
        self.given.get(pos) != Cell::Nothing
    }

    pub fn is_complete(&self) -> bool {
        // Filled AND valid under all three rules (allows alternative solutions).
        let filter = Filter {
            rule_of_3: true,
            rule_of_equity: true,
            rule_of_duplication: true,
            incomplete: true,
        };
        self.board.validate(&filter).is_ok()
    }

    pub fn n(&self) -> usize { self.board.width() }

    pub fn mistakes(&self)         -> usize { self.mistakes }
    pub fn guesses(&self)          -> usize { self.guesses }
    pub fn history_len(&self)      -> usize { self.history.len() }
    pub fn elapsed_ms(&self)       -> u64   { self.timer.elapsed_ms }
    pub fn timer_running(&self)    -> bool  { self.timer.running }
    pub fn total_empties(&self)    -> usize { self.total_empties }
    pub fn difficulty_total(&self) -> u32   { self.difficulty_total }
    pub fn difficulty_done(&self)  -> u32   { self.difficulty_done }

    pub fn cells_filled(&self) -> usize {
        let n_r = self.board.height();
        let n_c = self.board.width();
        (0..n_r).flat_map(|r| (0..n_c).map(move |c| (r, c)))
            .filter(|&pos| !self.is_locked(pos) && self.board.get(pos) != Cell::Nothing)
            .count()
    }

    pub fn given_seed(&self) -> String { seed::encode(&self.given) }

    // ── Mutators ──────────────────────────────────────────────────────────────

    /// Place `cell` at `pos`. Returns `Err` if the position is locked.
    pub fn set_cell(&mut self, pos: (usize, usize), cell: Cell) -> Result<(), &'static str> {
        if self.is_locked(pos) {
            return Err("cell is locked");
        }
        // Overwriting an already-filled cell is never a fresh guess: propagate()
        // skips non-empty cells so pos would never appear in its steps.
        let deducible = self.board.get(pos) != Cell::Nothing
            || is_v2_deducible(&self.board, pos);

        self.history.push(self.board.clone());
        self.board.set(pos, cell);

        let correct = self.solution.get(pos) == cell && cell != Cell::Nothing;
        if self.solution.get(pos) != cell && cell != Cell::Nothing {
            self.mistakes += 1;
        }
        if correct {
            // Guess: correct placement that v2 couldn't deduce.
            if !deducible && !self.discovered.contains(&pos) {
                self.guesses += 1;
            }
            // Difficulty progress (monotone — no double-counting after undo/redo).
            if !self.discovered.contains(&pos) {
                self.discovered.insert(pos);
                if let Some(&cost) = self.technique_map.get(&pos) {
                    self.difficulty_done += cost;
                }
            }
        }

        if self.is_complete() {
            self.timer.stop();
        }
        Ok(())
    }

    /// Undo the last edit. Mistake and guess counts are permanent.
    pub fn undo(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.board = prev;
        }
    }

    pub fn tick(&mut self, delta_ms: u64) { self.timer.tick(delta_ms); }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn count_empties(given: &BitBoard) -> usize {
    let (nr, nc) = (given.height(), given.width());
    (0..nr).flat_map(|r| (0..nc).map(move |c| (r, c)))
        .filter(|&pos| given.get(pos) == Cell::Nothing)
        .count()
}

fn build_og_difficulty(given: &BitBoard) -> (HashMap<(usize, usize), u32>, u32) {
    let trace = deduce_with(given, TechniqueSet::ALL);
    let mut map = HashMap::new();
    let mut total = 0u32;
    for step in trace.get_steps() {
        let cost = technique_cost(step.technique);
        map.insert(step.at, cost);
        total += cost;
    }
    (map, total)
}

fn is_v2_deducible(board: &BitBoard, pos: (usize, usize)) -> bool {
    propagate(board).steps.iter().any(|&(r, c, _)| (r, c) == pos)
}

// ── Completed-game record ─────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct GameRecord {
    pub n:                usize,
    pub mode:             GenMode,
    pub elapsed_ms:       u64,
    pub mistakes:         usize,
    pub guesses:          usize,
    pub difficulty_total: u32,
    pub difficulty_done:  u32,
    pub given_seed:       String,
}

impl GameRecord {
    pub fn from_session(s: &PlaySession) -> Self {
        GameRecord {
            n:                s.n(),
            mode:             s.gen_mode(),
            elapsed_ms:       s.elapsed_ms(),
            mistakes:         s.mistakes(),
            guesses:          s.guesses(),
            difficulty_total: s.difficulty_total(),
            difficulty_done:  s.difficulty_done(),
            given_seed:       s.given_seed(),
        }
    }
}

// ── Actions ───────────────────────────────────────────────────────────────────

pub enum PlayAction {
    NewGame { n: usize, seed: Option<u64>, mode: GenMode },
    SetCell(usize, usize, Cell),
    Undo,
}

pub fn apply(play: &mut Option<PlaySession>, action: PlayAction) {
    match action {
        PlayAction::NewGame { n, seed, mode } => {
            let mut rng = match seed {
                Some(s) => SmallRng::seed_from_u64(s),
                None    => SmallRng::seed_from_u64(rand::random()),
            };
            let puzzle = match mode {
                GenMode::Og => generate_puzzle(&OgGenerator, n, &mut rng, breakdown),
                GenMode::Extended => {
                    let carve_adapter = |full: &BitBoard, _rng: &mut SmallRng| {
                        match carve(full) {
                            Ok(carved) => {
                                let empties = count_empties(&carved);
                                (carved, empties)
                            }
                            Err(_) => (full.clone(), 0),
                        }
                    };
                    generate_puzzle(&ToolkitGenerator, n, &mut rng, carve_adapter)
                }
            };
            *play = Some(PlaySession::new_from_puzzle(puzzle, mode));
        }
        PlayAction::SetCell(r, c, cell) => {
            if let Some(s) = play.as_mut() { let _ = s.set_cell((r, c), cell); }
        }
        PlayAction::Undo => {
            if let Some(s) = play.as_mut() { s.undo(); }
        }
    }
}
