//! Analysis mode session: free-edit board with deduction overlay and generation.

use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::{Cell, BOARD_MAX_SIZE};
use ohhi_core::validator::{Filter, Rule, Validator, Violation};
use ohhi_core::seed;
use ohhi_solver::structs::{Technique, TechniqueSet};
use ohhi_solver::v1::deduction::deduce_with;
use ohhi_solver::v2::propagate::propagate;
use ohhi_generator::full::og::OgGenerator;
use ohhi_generator::full::toolkit::ToolkitGenerator;
use ohhi_generator::reduce::breakdown;
use ohhi_generator::generate_puzzle;
use ohhi_solver::carver::carve;
use rand::rngs::SmallRng;
use rand::SeedableRng;

use crate::trace::{self, Trace};

/// Which full-board constructor to use during generation.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Constructor {
    /// Faithful port of the game's `generateFast`.
    Og,
    /// Randomized DFS sandbox.
    Toolkit,
}

/// Which reducer to use during generation.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Reducer {
    /// Deduction-only `breakDown` — game-accurate quality gate.
    Breakdown,
    /// Count-based minimal seed (uniqueness, guessing allowed).
    Carve,
}

/// Which deduction engine to run.
#[derive(Clone)]
pub enum Engine {
    /// v1 bitwise technique engine with a selectable technique set.
    V1(TechniqueSet),
    /// v2 per-line propagator.
    V2,
}

/// All mutable state for the Analysis mode. Framework-free.
pub struct AnalysisSession {
    pub board: BitBoard,
    pub history: Vec<BitBoard>,
    pub redo_stack: Vec<BitBoard>,
    pub overlay: Option<Trace>,
    pub overlay_step: usize,
    pub engine: Engine,
    pub gen_constructor: Constructor,
    pub gen_reducer: Reducer,
    pub filter: Filter,
    pub last_validation: Option<Result<(), Violation>>,
    pub show_signatures: bool,
    pub last_solve: Option<BitBoard>,
    pub last_quality: Option<f64>,
}

impl AnalysisSession {
    pub fn new(width: usize, height: usize) -> Self {
        AnalysisSession {
            board: BitBoard::new(width, height),
            history: vec![],
            redo_stack: vec![],
            overlay: None,
            overlay_step: 0,
            engine: Engine::V1(TechniqueSet::ALL),
            gen_constructor: Constructor::Og,
            gen_reducer: Reducer::Breakdown,
            filter: Filter {
                rule_of_3: true,
                rule_of_equity: true,
                rule_of_duplication: true,
                incomplete: true,
            },
            last_validation: None,
            show_signatures: false,
            last_solve: None,
            last_quality: None,
        }
    }
}

/// Every user action in Analysis mode.
pub enum AnalysisAction {
    /// Cycle cell through Nothing → Red → Blue → Nothing.
    CycleCell(usize, usize),
    /// Set cell to a specific color.
    SetCell(usize, usize, Cell),
    /// Clear the board to empty (same size).
    ClearBoard,
    /// Resize the board.
    Resize(usize, usize),
    /// Undo last board-mutating action.
    Undo,
    /// Redo last undone action.
    Redo,
    /// Load board from seed string.
    LoadSeed(String),
    /// Run the selected deduction engine and store the trace as overlay.
    Deduce,
    /// Apply the last overlay step to the board.
    ApplyDeduction,
    /// Discard the overlay.
    ClearOverlay,
    /// Validate the board against the active filter.
    Validate,
    /// Toggle row/column signature display.
    ToggleSignatures,
    /// Toggle a v1 technique (no-op for V2 engine).
    ToggleTechnique(Technique),
    /// Toggle a validation rule in the filter.
    ToggleRule(Rule),
    /// Set the active engine.
    SetEngine(Engine),
    /// Set the generator constructor.
    SetConstructor(Constructor),
    /// Set the generator reducer.
    SetReducer(Reducer),
    /// Generate a puzzle and replace the board.
    Generate { n: usize, seed: Option<u64> },
    /// Replace the board with the full solution from the last Generate.
    RevealSolution,
}

/// A human-readable error from [`apply`].
#[derive(Debug)]
pub struct AppError(pub String);

/// Discard any deduction overlay. Call this whenever the board is replaced
/// wholesale (load/generate/resize/clear/reveal): the overlay's steps reference
/// positions on the *previous* board and are meaningless against the new one.
fn clear_overlay(state: &mut AnalysisSession) {
    state.overlay = None;
    state.overlay_step = 0;
}

/// Apply an action to the session, returning `Ok(())` or `Err(AppError)`.
pub fn apply(state: &mut AnalysisSession, action: AnalysisAction) -> Result<(), AppError> {
    match action {
        AnalysisAction::CycleCell(r, c) => {
            state.history.push(state.board.clone());
            state.redo_stack.clear();
            state.board.set((r, c), state.board.get((r, c)).next());
        }
        AnalysisAction::SetCell(r, c, cell) => {
            state.history.push(state.board.clone());
            state.redo_stack.clear();
            state.board.set((r, c), cell);
        }
        AnalysisAction::ClearBoard => {
            state.history.push(state.board.clone());
            state.redo_stack.clear();
            state.board = BitBoard::new(state.board.width(), state.board.height());
            clear_overlay(state);
        }
        AnalysisAction::Resize(w, h) => {
            state.history.push(state.board.clone());
            state.redo_stack.clear();
            state.board = BitBoard::new(w, h);
            clear_overlay(state);
        }
        AnalysisAction::Undo => {
            if let Some(board) = state.history.pop() {
                state.redo_stack.push(state.board.clone());
                state.board = board;
            }
        }
        AnalysisAction::Redo => {
            if let Some(board) = state.redo_stack.pop() {
                state.history.push(state.board.clone());
                state.board = board;
            }
        }
        AnalysisAction::LoadSeed(s) => {
            state.history.push(state.board.clone());
            state.redo_stack.clear();
            match seed::parse(&s) {
                Ok(board) => {
                    state.board = board;
                    clear_overlay(state);
                }
                Err(e) => {
                    state.history.pop();
                    return Err(AppError(format!("Failed to parse seed: {}", e.0)));
                }
            }
        }
        AnalysisAction::Deduce => {
            let t = match &state.engine {
                Engine::V1(techniques) => trace::from_v1(&deduce_with(&state.board, *techniques)),
                Engine::V2 => trace::from_v2(&propagate(&state.board)),
            };
            state.overlay_step = t.steps.len();
            state.overlay = Some(t);
        }
        AnalysisAction::ApplyDeduction => {
            if let Some(trace) = &state.overlay {
                if let Some(last_step) = trace.steps.last() {
                    // Re-derive the board at the last step by replaying from history
                    // We need the board after all steps; rebuild from current + steps.
                    let mut b = state.board.clone();
                    for step in &trace.steps {
                        b.set(step.at, step.color);
                    }
                    state.history.push(state.board.clone());
                    state.redo_stack.clear();
                    state.board = b;
                    let _ = last_step; // suppress unused warning
                }
            }
            state.overlay = None;
            state.overlay_step = 0;
        }
        AnalysisAction::ClearOverlay => {
            state.overlay = None;
            state.overlay_step = 0;
        }
        AnalysisAction::Validate => {
            state.last_validation = Some(state.board.validate(&state.filter));
        }
        AnalysisAction::ToggleSignatures => {
            state.show_signatures = !state.show_signatures;
        }
        AnalysisAction::ToggleTechnique(t) => {
            if let Engine::V1(ref mut ts) = state.engine {
                *ts = ts.toggle(t);
            }
        }
        AnalysisAction::ToggleRule(rule) => {
            match rule {
                Rule::RuleOf3 => state.filter.rule_of_3 = !state.filter.rule_of_3,
                Rule::RuleOfEquity => state.filter.rule_of_equity = !state.filter.rule_of_equity,
                Rule::RuleOfDuplication => state.filter.rule_of_duplication = !state.filter.rule_of_duplication,
                Rule::Incomplete => state.filter.incomplete = !state.filter.incomplete,
            }
        }
        AnalysisAction::SetEngine(e) => {
            state.engine = e;
        }
        AnalysisAction::SetConstructor(c) => {
            state.gen_constructor = c;
        }
        AnalysisAction::SetReducer(r) => {
            state.gen_reducer = r;
        }
        AnalysisAction::Generate { n, seed: rng_seed } => {
            if n == 0 || n % 2 != 0 || n > BOARD_MAX_SIZE as usize {
                return Err(AppError(format!(
                    "N must be even and between 2 and {}: got {n}",
                    BOARD_MAX_SIZE
                )));
            }
            let mut rng: SmallRng = match rng_seed {
                Some(s) => SmallRng::seed_from_u64(s),
                None => SmallRng::seed_from_u64(rand::random()),
            };
            let carve_adapter = |full: &BitBoard, _rng: &mut SmallRng| {
                match carve(full) {
                    Ok(puzzle) => {
                        let empties = (0..full.height())
                            .flat_map(|r| (0..full.width()).map(move |c| (r, c)))
                            .filter(|&pos| puzzle.get(pos) == Cell::Nothing)
                            .count();
                        (puzzle, empties)
                    }
                    Err(_) => (full.clone(), 0),
                }
            };
            let puzzle = match (state.gen_constructor, state.gen_reducer) {
                (Constructor::Og, Reducer::Breakdown) =>
                    generate_puzzle(&OgGenerator, n, &mut rng, breakdown),
                (Constructor::Og, Reducer::Carve) =>
                    generate_puzzle(&OgGenerator, n, &mut rng, carve_adapter),
                (Constructor::Toolkit, Reducer::Breakdown) =>
                    generate_puzzle(&ToolkitGenerator, n, &mut rng, breakdown),
                (Constructor::Toolkit, Reducer::Carve) =>
                    generate_puzzle(&ToolkitGenerator, n, &mut rng, carve_adapter),
            };
            state.history.push(state.board.clone());
            state.redo_stack.clear();
            state.board = puzzle.puzzle;
            state.last_solve = Some(puzzle.full);
            state.last_quality = Some(puzzle.quality);
            clear_overlay(state);
        }
        AnalysisAction::RevealSolution => {
            if let Some(full) = state.last_solve.clone() {
                state.history.push(state.board.clone());
                state.redo_stack.clear();
                state.board = full;
                clear_overlay(state);
            }
        }
    }
    Ok(())
}
