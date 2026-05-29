//! Application state and the action-based reducer.
//!
//! # Architecture
//!
//! The GUI follows a simple action-reducer pattern:
//!
//! 1. **Render** functions read from `GuiState` and emit `Action` values when
//!    the user interacts with the UI (button clicks, cell clicks, etc.).
//! 2. **`apply(state, action)`** is the single point of mutation. It updates
//!    `GuiState` and returns `Ok(())` or `Err(AppError)` (errors are surfaced
//!    in the UI).
//!
//! # Undo / Redo
//!
//! Every action that changes the board (`CycleCell`, `SetCell`, `ClearBoard`,
//! `Resize`, `Undo`, `Redo`, `LoadSeed`, `ApplyDeduction`) pushes a snapshot
//! onto the history stack before mutating the board. `Undo` pops from history
//! and pushes to the redo stack; `Redo` does the reverse.

use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use ohhi_core::validator::{Filter, Rule, Validator, Violation};
use ohhi_solver::v1::deduction::{deduce_with, DeductionTrace, Technique, TechniqueSet};
use crate::seed;

/// Which full-board constructor to use during generation.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Constructor {
    /// Faithful port of the game's `generateFast` (combo-pool shuffle + row fill).
    Og,
    /// Randomized DFS sandbox.
    Toolkit,
}

/// Which reducer to use during generation.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Reducer {
    /// Faithful deduction-only `breakDown` — game-accurate quality gate.
    Breakdown,
    /// Count-based minimal seed (uniqueness, guessing allowed).
    Carve,
}

/// All mutable application state. Render functions borrow this read-only;
/// all writes go through [`apply`].

#[allow(dead_code)]
pub(crate) struct GuiState {
    board: BitBoard,

    history: Vec<BitBoard>,
    redo_stack: Vec<BitBoard>,

    overlay: Option<DeductionTrace>,
    overlay_step: usize,
    techniques: TechniqueSet,
    show_signatures: bool,
    show_counts: bool,

    last_count: Option<usize>,
    last_solve: Option<BitBoard>,
    last_validation: Option<Result<(), Violation>>,

    pending: Option<PendingTask>,

    dialogs: Dialogs
}

/// Inline dialog states. Render functions open/close these directly rather
/// than going through `apply`, because dialogs are pure UI state with no undo
/// semantics.
pub(crate) struct Dialogs {
    pub resize: ResizeDialog,
    pub load_seed: LoadSeedDialog,
    pub export: ExportDialog,
    pub filter_rules: FilterRulesDialog,
    pub generate: GenerateDialog,
}

/// Generate dialog: controls for the two-phase board generator.
pub(crate) struct GenerateDialog {
    pub open: bool,
    /// Board size N (must be even, ≤ BOARD_MAX_SIZE). Buffer for text input.
    pub size_buf: String,
    /// Optional RNG seed. Empty string = random.
    pub seed_buf: String,
    /// Which full-board constructor to use.
    pub constructor: Constructor,
    /// Which reducer to use.
    pub reducer: Reducer,
    /// Quality fraction of the last generated puzzle, for display.
    pub last_quality: Option<f64>,
    /// Error from the last generation attempt, if any.
    pub error: Option<String>,
}

/// Export dialog: displays the encoded seed string and offers a copy-to-clipboard button.
pub(crate) struct ExportDialog {
    pub open: bool,
    /// The encoded seed, generated fresh each time the dialog opens.
    pub seed: String,
}

/// Inline Resize dialog: text buffers for width and height before Apply.
pub(crate) struct LoadSeedDialog {
    pub open: bool,
    pub seed: String,
    pub error: Option<String>,
}
/// Inline Load Seed dialog: the raw text, and the last parse error if any.
pub(crate) struct ResizeDialog {
    pub open: bool,
    pub w_buf: String,
    pub h_buf: String,
}

/// Inline Filter Rules dialog: which validation rules are active.
pub(crate) struct FilterRulesDialog {
    pub open: bool,
    pub filter: Filter,
}
impl GuiState {
    pub fn new((width, height): (usize, usize), filter: Option<Filter>) -> Self {
        GuiState {
            board: BitBoard::new(width, height),
            history: vec![],
            redo_stack: vec![],
            overlay: None,
            overlay_step: 0,
            techniques: TechniqueSet::ALL,
            show_signatures: false,
            show_counts: false,
            last_solve: None,
            last_count: None,
            last_validation: None,
            pending: None,
            dialogs: Dialogs {
                resize: ResizeDialog {
                    open: false,
                    h_buf: String::new(),
                    w_buf: String::new()
                },
                load_seed: LoadSeedDialog {
                    open: false,
                    seed: String::new(),
                    error: None,
                },
                export: ExportDialog {
                    open: false,
                    seed: String::new(),
                },
                filter_rules: FilterRulesDialog {
                    open: false,
                    filter: filter.unwrap_or(Filter {
                        rule_of_3: true,
                        rule_of_equity: true,
                        rule_of_duplication: true,
                        incomplete: true
                    }),
                },
                generate: GenerateDialog {
                    open: false,
                    size_buf: String::from("6"),
                    seed_buf: String::new(),
                    constructor: Constructor::Og,
                    reducer: Reducer::Breakdown,
                    last_quality: None,
                    error: None,
                },
            }
        }
    }

    pub fn last_solve(&self) -> Option<&BitBoard> {
        self.last_solve.as_ref()
    }
    pub fn dims(&self) -> (usize, usize) {
        (self.board.width(), self.board.height())
    }
    pub fn board(&self) -> &BitBoard {
        &self.board
    }

    pub fn dialogs(&mut self) -> &mut Dialogs {
        &mut self.dialogs
    }

    pub fn overlay(&self) -> Option<&DeductionTrace> {
        self.overlay.as_ref()
    }

    pub fn overlay_step(&self) -> usize {
        self.overlay_step
    }

    pub fn overlay_step_mut(&mut self) -> &mut usize {
        &mut self.overlay_step
    }

    pub fn techniques(&self) -> TechniqueSet {
        self.techniques
    }

    pub fn show_signatures(&self) -> bool {
        self.show_signatures
    }

    pub fn _filter(&self) -> &Filter {
        &self.dialogs.filter_rules.filter
    }

    pub fn last_validation(&self) -> Option<&Result<(), Violation>> {
        self.last_validation.as_ref()
    }
}

/// Every user interaction that can change application state.
///
/// Actions that modify the board (marked **↩ history**) push a snapshot to the
/// undo history before mutating, so they can be reversed with [`Action::Undo`].
pub enum Action {
    /// Advance cell `(r, c)` through Nothing → Red → Blue → Nothing. **↩ history**
    CycleCell(usize, usize),
    /// Set cell `(r, c)` to a specific color. **↩ history**
    SetCell(usize, usize, Cell),
    /// Replace the board with an empty board of the same size. **↩ history**
    ClearBoard,
    /// Replace the board with a fresh empty board of the given dimensions. **↩ history**
    Resize(usize, usize),

    /// Pop the most recent board snapshot and restore it. **↩ history** (pushes to redo stack)
    Undo,
    /// Re-apply the most recently undone snapshot. **↩ history** (pushes to undo stack)
    Redo,

    /// Count valid completions of the current board up to `cap` (not yet implemented).
    CountSolutions { cap: usize },
    /// Find one valid completion and display it (not yet implemented).
    SolveOne,
    /// Run the deduction engine with the currently enabled technique set and
    /// store the resulting [`DeductionTrace`] as an overlay.
    Deduce,
    /// Strip redundant clues from the current board (not yet implemented).
    Carve,

    /// Toggle the row/column signature display on the canvas.
    ToggleSignatures,
    /// Toggle cell-count labels (not yet implemented).
    ToggleCount,
    /// Discard the current deduction overlay.
    ClearOverlay,

    /// Enable or disable one validation rule in the active filter.
    ToggleRule(Rule),
    /// Enable or disable one deduction technique.
    ToggleTechnique(Technique),
    /// Apply the last step of the current overlay to the board. **↩ history**
    ///
    /// Replaces the board with the `board_after` of the final deduction step,
    /// then discards the overlay.
    ApplyDeduction,
    /// Validate the current board against the active filter and store the result.
    Validate,

    /// Parse `String` as a seed and load it as the current board. **↩ history**
    ///
    /// Returns `Err(AppError)` if the string cannot be parsed. The seed format
    /// is `R`/`B`/`.` separated by spaces within rows, rows separated by newlines.
    LoadSeed(String),

    /// Generate a puzzle and replace the board with it. **↩ history**
    ///
    /// Runs the two-phase generator with the given constructor, reducer, and optional
    /// RNG seed. The full solution is stashed in `last_solve` for later reveal.
    Generate {
        n: usize,
        constructor: Constructor,
        reducer: Reducer,
        seed: Option<u64>,
    },

    /// Replace the board with the full solution from the last `Generate`. **↩ history**
    ///
    /// Does nothing if no generation has been performed.
    RevealSolution,
}

/// A human-readable error message from [`apply`]. Displayed in the UI.
pub struct AppError(pub String);

/// The single point of state mutation. Applies `action` to `state` and returns
/// `Ok(())` on success or `Err(AppError)` when the action cannot be completed
/// (e.g. a seed that fails to parse).
///
/// All board-mutating actions push to `state.history` before they run, so
/// they can be reversed with [`Action::Undo`].
pub(crate) fn apply(state: &mut GuiState, action: Action) -> Result<(), AppError> {
    match action {
        Action::CycleCell(r, c) => {
            state.history.push(state.board.clone());
            state.board.set((r,c), state.board.get((r, c)).next());
        }
        Action::SetCell(r, c, cell) => {
            state.history.push(state.board.clone());
            state.board.set((r, c), cell);
        }
        Action::Undo => {
            if let Some(board) = state.history.pop() {
                state.redo_stack.push(state.board.clone());
                state.board = board;
            }
        }
        Action::Redo => {
            if let Some(board) = state.redo_stack.pop() {
                state.history.push(state.board.clone());
                state.board = board;
            }
        }
        Action::Resize(w, h) => {
            state.history.push(state.board.clone());
            state.board = BitBoard::new(w, h);
        }
        Action::ClearBoard => {
            state.history.push(state.board.clone());
            state.board = BitBoard::new(state.board.width(), state.board.height());
        }
        Action::LoadSeed(s) => {
            state.history.push(state.board.clone());
            match seed::parse(&s) {
                Ok(board) => state.board = board,
                Err(err) => {
                    return Err(AppError(format!("Failed to parse seed: {}", err.0)));
                }
            }
        }
        Action::CountSolutions { .. } => {
            todo!("Implement count solutions")
        }
        Action::ToggleSignatures => {
            state.show_signatures = !state.show_signatures;
        }
        Action::ToggleCount => {
            state.show_counts = !state.show_counts;
        }
        Action::Deduce => {
            let trace = deduce_with(&state.board, state.techniques);
            state.overlay_step = trace.get_steps().len();
            state.overlay = Some(trace);
        }
        Action::ClearOverlay => {
            state.overlay = None;
            state.overlay_step = 0;
        }
        Action::ApplyDeduction => {
            let last = state.overlay.as_ref()
                .and_then(|t| t.get_steps().last())
                .map(|s| s.board_after.clone());
            if let Some(board) = last {
                state.history.push(state.board.clone());
                state.board = board;
                state.overlay = None;
                state.overlay_step = 0;
            }
        }
        Action::ToggleTechnique(t) => {
            state.techniques = state.techniques.toggle(t);
        }
        Action::Validate => {
            let filter = state.dialogs.filter_rules.filter;
            state.last_validation = Some(state.board.validate(&filter));
        }
        Action::Generate { n, constructor, reducer, seed } => {
            use ohhi_generator::full::og::OgGenerator;
            use ohhi_generator::full::toolkit::ToolkitGenerator;
            use ohhi_generator::reduce::breakdown;
            use ohhi_generator::generate_puzzle;
            use ohhi_solver::carver::carve;
            use rand::rngs::SmallRng;
            use rand::SeedableRng;

            if n == 0 || n % 2 != 0 || n > ohhi_core::board::BOARD_MAX_SIZE as usize {
                return Err(AppError(format!(
                    "N must be even and between 2 and {}: got {n}",
                    ohhi_core::board::BOARD_MAX_SIZE
                )));
            }

            let mut rng: SmallRng = match seed {
                Some(s) => SmallRng::seed_from_u64(s),
                None => SmallRng::seed_from_u64(rand::random()),
            };

            let carve_adapter = |full: &ohhi_core::bit_board::BitBoard, _rng: &mut SmallRng| {
                match carve(full) {
                    Ok(puzzle) => {
                        let empties = (0..full.height())
                            .flat_map(|r| (0..full.width()).map(move |c| (r, c)))
                            .filter(|&pos| puzzle.get(pos) == ohhi_core::board::Cell::Nothing)
                            .count();
                        (puzzle, empties)
                    }
                    Err(_) => (full.clone(), 0),
                }
            };

            let puzzle = match (constructor, reducer) {
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
            state.board = puzzle.puzzle;
            state.last_solve = Some(puzzle.full);
            state.dialogs.generate.last_quality = Some(puzzle.quality);
            state.dialogs.generate.error = None;
        }
        Action::RevealSolution => {
            if let Some(full) = state.last_solve.clone() {
                state.history.push(state.board.clone());
                state.board = full;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(crate) struct PendingTask {

}