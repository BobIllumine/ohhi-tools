//! Puzzle carver: turns a complete valid board into a minimal puzzle seed.
//!
//! [`carve`] removes cells from a complete board one at a time in random order.
//! After each removal it checks whether the partial board still has **exactly
//! one** valid completion (using [`calculate`] with `cap = 2`). If removing a
//! cell creates ambiguity the cell is put back and the next candidate is tried.
//! The result is a partially-filled board that is uniquely solvable and has no
//! redundant clues — every given cell is necessary for uniqueness.
//!
//! # Example use
//!
//! ```text
//! // Given a complete board `full`, produce a minimal puzzle seed:
//! let seed = carve(&full).unwrap();
//! assert_eq!(calculate(&seed, 2), 1); // exactly one solution
//! ```
//!
//! # Notes
//!
//! - The removal order is randomised, so repeated calls on the same board
//!   typically produce different minimal seeds of roughly similar difficulty.
//! - In the rare edge case where every cell can be removed while keeping
//!   uniqueness, `carve` returns an empty board.
//! - `carve` returns `Err` immediately if `board` is not itself a valid
//!   complete board.

use rand::RngExt;
use ohhi_core::bit_board::BitBoard;
use ohhi_core::validator::{Filter, Validator, Violation};
use crate::backtrack::calculate;
use crate::structs::SolverState;

/// Strips redundant clues from a complete valid board, returning a minimal seed.
///
/// Every empty cell in the returned board is necessary: filling it in would
/// produce a board with more than one valid completion. Returns `Err(violation)`
/// if `board` fails full validation.
pub fn carve(board: &BitBoard) -> Result<BitBoard, Violation> {
    let filter = Filter {
        rule_of_duplication: true,
        rule_of_equity: true,
        rule_of_3: true,
        incomplete: true
    };
    board.validate(&filter)?;
    let mut state = SolverState::new(board);
    let mut rng = rand::rng();
    let mut cell_pool: Vec<(usize, usize)> = vec![];
    for i in 0..board.height() * board.width() {
        cell_pool.push((i / board.width(), i % board.width()));
    }
    while !cell_pool.is_empty() {
        let idx = rng.random_range(0..cell_pool.len());
        let elem = cell_pool.swap_remove(idx);
        let cell = board.get(elem);
        state.unplace(elem);
        if calculate(state.board_ref(), 2) != 1 {
            state.place(cell, elem);
            return Ok(state.board_ref().clone());
        }
    }
    Ok(BitBoard::new(board.height(), board.width()))
}