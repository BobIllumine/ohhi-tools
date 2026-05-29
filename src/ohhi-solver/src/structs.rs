//! Solver auxiliary state and deduction trace types.
//!
//! `SolverState` tracks the board and the set of completed row/column
//! signatures so that `place`/`unplace` can detect uniqueness violations in
//! O(1) without rescanning the board. Both the backtracking solver and the
//! deduction engine share this type.
//!
//! `DeductionTrace`, `DeductionStep`, `Technique`, and `TechniqueSet` form
//! the output of the deduction engine.

use ohhi_core::stats::NumTransforms;
use std::collections::HashSet;
use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;

/// Mutable solver state: the current board plus bookkeeping for O(1) legality
/// checks. Shared by the backtracker and the deduction engine.
pub struct SolverState {
    board: BitBoard,
    pub width: usize,
    pub height: usize,
    /// Red masks of rows that are completely filled, used for uniqueness checks.
    completed_rows: HashSet<u16>,
    /// Red masks of columns that are completely filled, used for uniqueness checks.
    completed_cols: HashSet<u16>,
}

impl SolverState {
    
    /// Scans the board for completed rows and columns to saturate `completed_rows` and `completed_cols` correctly.
    /// May slow down some solver functions as a result, but it is a necessary fix and the overhead is not as crucial
    pub fn new(board: &BitBoard) -> SolverState {
        let board = board.clone();
        let mut completed_rows = HashSet::new();
        let mut completed_cols = HashSet::new();
        for r in 0..board.height() {
            if board.is_complete_x(&r) {
                completed_rows.insert(board.signature_x(&r));
            }
        }
        for c in 0..board.width() {
            if board.is_complete_y(&c) {
                completed_cols.insert(board.signature_y(&c));
            }
        }
        SolverState {
            width: board.width(),
            height: board.height(),
            completed_rows,
            completed_cols,
            board
        }
    }

    /// Places `color` at `(r, c)` and checks all three rules.
    ///
    /// Returns `true` if the placement is legal. On `false` the placement is
    /// fully rolled back — the cell is cleared and the completed-line
    /// bookkeeping is left exactly as it was before the call — so the caller
    /// must **not** call `unplace` after a `false` return.
    pub fn place(&mut self, color: Cell, (r, c): (usize, usize)) -> bool {
        self.board.set((r, c), color);
        let (red_x, blue_x) = self.board.count_x(&r);
        let (red_y, blue_y) = self.board.count_y(&c);
        let equity_ok = match color {
            Cell::Red => red_x as usize <= self.width / 2 && red_y as usize <= self.height / 2,
            Cell::Blue => blue_x as usize <= self.width / 2 && blue_y as usize <= self.height / 2,
            _ => true,
        };
        if !equity_ok
            || self.board.has_consecutive_x(&r, 3)
            || self.board.has_consecutive_y(&c, 3)
        {
            self.board.set((r, c), Cell::Nothing);
            return false;
        }
        // Uniqueness: register completed-line signatures, rolling everything
        // back (including the cell) if either line duplicates an existing one.
        // completed_rows/cols are keyed by signature *value*, so a transient
        // duplicate must never be left registered — otherwise a later `unplace`
        // would remove the shared key and silently disarm the duplicate guard.
        let mut inserted_row = false;
        if self.board.is_complete_x(&r) {
            if self.completed_rows.insert(self.board.signature_x(&r)) {
                inserted_row = true;
            } else {
                self.board.set((r, c), Cell::Nothing);
                return false;
            }
        }
        if self.board.is_complete_y(&c) && !self.completed_cols.insert(self.board.signature_y(&c)) {
            if inserted_row {
                self.completed_rows.remove(&self.board.signature_x(&r));
            }
            self.board.set((r, c), Cell::Nothing);
            return false;
        }
        true
    }

    /// Removes the cell at `(r, c)` and cleans up any completed-line
    /// signatures that were added for that cell's row or column.
    pub fn unplace(&mut self, (r, c): (usize, usize)) {
        if self.board.is_complete_x(&r) {
            self.completed_rows.remove(&self.board.signature_x(&r));
        }
        if self.board.is_complete_y(&c) {
            self.completed_cols.remove(&self.board.signature_y(&c));
        }
        self.board.set((r, c), Cell::Nothing);
    }

    pub fn board_ref(&self) -> &BitBoard {
        &self.board
    }
    pub fn completed_rows(&self) -> &HashSet<u16> {
        &self.completed_rows
    }
    pub fn completed_cols(&self) -> &HashSet<u16> {
        &self.completed_cols
    }
}

/// The output of a deduction engine run: an ordered list of forced cells.
pub struct DeductionTrace {
    steps: Vec<DeductionStep>,
    /// `true` if a technique fired but the resulting placement was illegal
    /// (a contradiction). `false` when the engine ran to fixpoint normally
    /// (either all cells filled or no technique fired — use `steps.len()` to
    /// distinguish).
    pub stalled: bool,
}

impl DeductionTrace {
    pub fn new() -> DeductionTrace {
        DeductionTrace {
            steps: Vec::new(),
            stalled: false,
        }
    }
    pub fn add_step(&mut self, step: DeductionStep) {
        self.steps.push(step);
    }
    pub fn get_steps(&self) -> &Vec<DeductionStep> {
        &self.steps
    }
}

/// One forced cell deduced by the engine, including the board state after
/// the placement.
pub struct DeductionStep {
    pub at: (usize, usize),
    pub cell: Cell,
    pub technique: Technique,
    /// Snapshot of the full board immediately after this cell was placed.
    /// Useful for the GUI scrub slider.
    pub board_after: BitBoard,
}

/// The deduction technique that forced a cell.
///
/// Discriminants are used as bit positions in `TechniqueSet`, so the order
/// is load-bearing — do not reorder variants without updating `TechniqueSet`.
///
/// - `PairExtension`: `XX_` or `_XX` → fill gap with opposite color.
/// - `GapFill`: `X_X` → fill middle with opposite color.
/// - `Saturation`: a line has N/2 of one color → force remaining to the other.
/// - `TwinCompletion`: a line with exactly 2 empties matches a complete line
///   everywhere else → the 2 empties must differ from the twin's values.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Technique {
    PairExtension,
    GapFill,
    Saturation,
    TwinCompletion
}

/// A bitset over the four `Technique` variants.
///
/// Bit `i` corresponds to the `Technique` variant whose discriminant equals
/// `i` (i.e. `PairExtension=0`, `GapFill=1`, `Saturation=2`,
/// `TwinCompletion=3`). This ordering is load-bearing.
#[derive(Copy, Clone)]
pub struct TechniqueSet(u8);

impl TechniqueSet {
    /// All four techniques enabled.
    // bit 0..=3 correspond to PairExtension..=TwinCompletion
    pub const ALL: TechniqueSet = TechniqueSet(0b1111);
    /// No techniques enabled.
    pub const NONE: TechniqueSet = TechniqueSet(0);

    /// Returns a copy with `technique` disabled.
    pub fn without(&self, technique: Technique) -> TechniqueSet {
        TechniqueSet(self.0 & !(1 << technique as u8))
    }

    /// Returns a copy with `technique` enabled.
    pub fn with(&self, technique: Technique) -> TechniqueSet {
        TechniqueSet(self.0 | (1 << technique as u8))
    }

    /// Returns a copy with `technique` toggled.
    pub fn toggle(&self, technique: Technique) -> TechniqueSet {
        TechniqueSet(self.0 ^ (1 << technique as u8))
    }

    /// Returns `true` if `technique` is enabled in this set.
    pub fn contains(&self, technique: Technique) -> bool {
        self.0 & (1 << technique as usize) != 0
    }
}

impl Default for TechniqueSet {
    fn default() -> Self {
        TechniqueSet::ALL
    }
}
