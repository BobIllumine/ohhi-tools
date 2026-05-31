//! Practice mode: drill one technique. Each recognition is a rep.
//!
//! Correctness is membership in "what the target technique forces on the board
//! right now" — any such cell (with the right colour) is a hit; everything else,
//! including the correct colour of a cell the technique doesn't justify, is a
//! miss (a guess). A board may yield several reps; when the target stops firing
//! a fresh board is generated.

use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use ohhi_generator::practice::{target_forced, targeted_board, PracticeBoard};
pub use ohhi_generator::practice::Target;
use rand::rngs::SmallRng;
use rand::SeedableRng;

use crate::timer::Timer;

const GEN_TRIES: usize = 50_000;

/// Outcome of a single attempt.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Attempt {
    /// A cell the target technique forces — recorded, applied, timer reset.
    Hit,
    /// A guess: not a current target deduction (or wrong colour). No mutation.
    Miss,
    /// The attempt was on a locked/filled cell — ignored.
    Ignored,
}

pub struct PracticeSession {
    target: Target,
    n: usize,
    total_reps: usize,
    rng: SmallRng,
    board: BitBoard,
    solution: BitBoard,
    /// The cells the target forces on the current board (the valid answers).
    forced: Vec<((usize, usize), Cell)>,
    timer: Timer,
    times: Vec<u64>, // recognition time per hit
    hits: usize,
    misses: usize,
    recorded: bool,  // guard so the finished drill is logged to history once
}

impl PracticeSession {
    /// Starts a drill of `total_reps` recognitions of `target` on `n`×`n` boards.
    /// Returns `None` if no opening board could be generated.
    pub fn new(target: Target, n: usize, total_reps: usize, seed: u64) -> Option<Self> {
        let mut rng = SmallRng::seed_from_u64(seed);
        let pb = targeted_board(target, n, &mut rng, GEN_TRIES)?;
        let mut timer = Timer::new();
        timer.start();
        let PracticeBoard { given, solution, .. } = pb;
        let forced = target_forced(&given, target);
        Some(PracticeSession {
            target, n, total_reps, rng,
            board: given,
            solution,
            forced,
            timer,
            times: Vec::new(),
            hits: 0,
            misses: 0,
            recorded: false,
        })
    }

    // ── Accessors ───────────────────────────────────────────────────────────
    pub fn board(&self) -> &BitBoard { &self.board }
    pub fn target(&self) -> Target { self.target }
    pub fn hits(&self) -> usize { self.hits }
    pub fn misses(&self) -> usize { self.misses }
    pub fn total_reps(&self) -> usize { self.total_reps }
    pub fn current_rep(&self) -> usize { (self.hits + 1).min(self.total_reps) }
    pub fn elapsed_ms(&self) -> u64 { self.timer.elapsed_ms }
    pub fn timer_running(&self) -> bool { self.timer.running }
    /// Number of currently-available target deductions (≥1 while a rep is live).
    pub fn available(&self) -> usize { self.forced.len() }

    pub fn is_locked(&self, pos: (usize, usize)) -> bool {
        self.board.get(pos) != Cell::Nothing
    }

    /// The true colour of `pos` in the underlying solution (for tests / hints).
    pub fn solution_color(&self, pos: (usize, usize)) -> Cell {
        self.solution.get(pos)
    }

    pub fn is_complete(&self) -> bool {
        self.hits >= self.total_reps
    }

    pub fn mean_time_ms(&self) -> Option<u64> {
        if self.times.is_empty() {
            None
        } else {
            Some(self.times.iter().sum::<u64>() / self.times.len() as u64)
        }
    }

    pub fn accuracy(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 1.0 } else { self.hits as f64 / total as f64 }
    }

    pub fn tick(&mut self, delta_ms: u64) { self.timer.tick(delta_ms); }

    /// Builds a history record for a finished drill, once. Returns `None` if the
    /// drill isn't complete or was already recorded. `epoch_ms` is the caller's
    /// wall-clock timestamp (0 if unavailable).
    pub fn take_record(&mut self, epoch_ms: u64) -> Option<crate::history::DrillRecord> {
        if !self.is_complete() || self.recorded {
            return None;
        }
        self.recorded = true;
        Some(crate::history::DrillRecord {
            target: self.target,
            n: self.n,
            total_reps: self.total_reps,
            times: self.times.clone(),
            hits: self.hits,
            misses: self.misses,
            epoch_ms,
        })
    }

    // ── Play ────────────────────────────────────────────────────────────────

    /// Attempt to mark `pos` as `cell`.
    pub fn attempt(&mut self, pos: (usize, usize), cell: Cell) -> Attempt {
        if self.is_complete() {
            return Attempt::Ignored;
        }
        if self.is_locked(pos) {
            return Attempt::Ignored;
        }
        let is_target = self.forced.iter().any(|&(p, c)| p == pos && c == cell);
        if !is_target {
            self.misses += 1;
            return Attempt::Miss;
        }
        // Hit: record recognition time, apply, advance.
        self.times.push(self.timer.elapsed_ms);
        self.hits += 1;
        self.board.set(pos, cell);
        if !self.is_complete() {
            self.advance();
        } else {
            self.timer.stop();
        }
        Attempt::Hit
    }

    /// Recompute the forced set; if the current board is exhausted of target
    /// deductions, generate a fresh board. Resets the recognition timer.
    fn advance(&mut self) {
        self.forced = target_forced(&self.board, self.target);
        if self.forced.is_empty() {
            if let Some(pb) = targeted_board(self.target, self.n, &mut self.rng, GEN_TRIES) {
                self.board = pb.given;
                self.solution = pb.solution;
                self.forced = target_forced(&self.board, self.target);
            }
        }
        self.timer.reset();
        self.timer.start();
    }
}
