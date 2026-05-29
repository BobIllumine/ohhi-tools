//! Engine v2: complete per-line deduction run to fixpoint.
//!
//! Each pass scans rows then columns of the *current* board. For one line it
//! enumerates every legal completion ([`legal_completions`]), drops those that
//! would duplicate an already-complete parallel line (uniqueness, rule 3), then
//! intersects the survivors ([`forced_cells`]) — any empty cell that is the same
//! color in all of them is forced. The first forced cell is placed and the scan
//! restarts; when a full pass forces nothing, the board is at the v2 fixpoint.
//!
//! Unlike v1's four hand-written techniques, the per-line intersection is
//! *complete* for a single line under rules 1 (anti-triple) and 2 (equity), so
//! it also catches the counting deductions v1 misses (e.g. "only one placement
//! of the remaining colors avoids a triple"). It is still **not** a full solver:
//! deductions needing cross-line reasoning or case-splits leave a fixpoint with
//! empty cells remaining.

use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use crate::structs::SolverState;
use crate::v2::line_solver::{forced_cells, legal_completions};

/// The result of running [`propagate`] to fixpoint.
pub struct Propagation {
    /// The board after every forced cell has been placed.
    pub board: BitBoard,
    /// The forced cells, in the order they were placed: `(row, col, color)`.
    pub steps: Vec<(usize, usize, Cell)>,
    /// `true` if a forced placement contradicted the rules (an inconsistent
    /// input board); propagation stops at that point.
    pub stalled: bool,
}

/// Runs the complete per-line deduction engine to fixpoint on `board`.
///
/// Returns a [`Propagation`] whose `board` is the input with all forced cells
/// filled, `steps` the placements in order, and `stalled` set only if the input
/// was contradictory. A fixpoint with empty cells remaining means the board
/// cannot be finished by single-line reasoning alone (it needs guessing).
pub fn propagate(board: &BitBoard) -> Propagation {
    let mut state = SolverState::new(board);
    let mut prop = Propagation {
        board: board.clone(),
        steps: vec![],
        stalled: false
    };
    'outer: loop {
        for r in 0..board.height() {
            let (red, blue) = state.board_ref().get_row(r);
            let current = state.completed_rows();
            let completions = legal_completions(red, blue, board.width())
                .iter()
                .filter(|&&x| !current.contains(&x))
                .copied()
                .collect::<Vec<u16>>();
            let forced = forced_cells(completions.as_slice(), red | blue, board.width());
            if let Some((c, cell)) = forced.first() {
                if state.place(*cell, (r, *c)) {
                    prop.board = state.board_ref().clone();
                    prop.steps.push((r, *c, *cell));
                    continue 'outer;
                }
                else {
                    prop.stalled = true;
                    prop.board = state.board_ref().clone();
                    break 'outer prop;
                }
            }
        }
        for c in 0..board.width() {
            let (red, blue) = state.board_ref().get_col(c);
            let current = state.completed_cols();
            let completions = legal_completions(red, blue, board.height())
                .iter()
                .filter(|&&x| !current.contains(&x))
                .map(|x| *x)
                .collect::<Vec<u16>>();
            let forced = forced_cells(completions.as_slice(), red | blue, board.height());
            if let Some((r, cell)) = forced.get(0) {
                if state.place(*cell, (*r, c)) {
                    prop.board = state.board_ref().clone();
                    prop.steps.push((*r, c, *cell));
                    continue 'outer;
                }
                else {
                    prop.board = state.board_ref().clone();
                    prop.stalled = true;
                    break 'outer prop;
                }
            }
        }
        return prop;
    }
}