//! Depth-first backtracking solver for 0h h1.
//!
//! Branches on `Cell::Nothing` cells in row-major order, trying `Red` then
//! `Blue` at each position. `SolverState::place` performs incremental
//! legality checks so illegal branches are pruned immediately.

use ohhi_core::board::{Cell};
use crate::structs::SolverState;
use ohhi_core::bit_board::BitBoard;

/// Recursively counts valid completions of `state`, stopping early once
/// `cap` solutions are found.
///
/// `cap - result` is passed to recursive calls so the remaining budget
/// shrinks across both color branches — the second color can't push the
/// total past the cap.
fn count(state: &mut SolverState, cap: usize) -> usize {
    for r in 0..state.height {
        for c in 0..state.width {
            if state.board_ref().get((r, c)) != Cell::Nothing { continue; }
            let mut result = 0;
            for color in [Cell::Red, Cell::Blue] {
                if state.place(color, (r, c)) {
                    result += count(state, cap - result);
                }
                state.unplace((r, c));
                if result >= cap { return result; }
            }
            return result;
        }
    }
    // All cells are filled and no rule was violated → this leaf is one solution.
    1
}

/// Recursively finds one valid completion of `state`, returning it if found.
///
fn backtrack(state: &mut SolverState) -> Option<BitBoard> {
    for r in 0..state.height {
        for c in 0..state.width {
            if state.board_ref().get((r, c)) != Cell::Nothing { continue; }
            for color in [Cell::Red, Cell::Blue] {
                if state.place(color, (r, c)) {
                    if let Some(board) = backtrack(state) {
                        return Some(board);
                    }
                    state.unplace((r, c));
                }
            }
        }
    }
    Some(state.board_ref().clone())
}

/// Returns the number of valid completions of `board`, stopping at `cap`.
///
/// Pass `usize::MAX` for an exact count. Pass `2` to cheaply test for
/// uniqueness: a return value of `1` means the board has exactly one solution.
pub fn calculate(board: &BitBoard, cap: usize) -> usize {
    let mut state = SolverState::new(board);
    count(&mut state, cap)
}

/// Returns one valid completion of `board`, or `None` if none exists.
///
/// See the FIXME on `backtrack` — when called on an already-complete board
/// this currently returns `None` instead of `Some(board.clone())`.
pub fn backtrack_one(board: &BitBoard) -> Option<BitBoard> {
    let mut state = SolverState::new(board);
    backtrack(&mut state)
}
