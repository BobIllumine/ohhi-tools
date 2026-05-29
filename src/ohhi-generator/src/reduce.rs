//! Puzzle reducers: strip cells from a complete board while preserving solvability.
//!
//! Two reducers are available:
//! - [`breakdown`]: faithful port of the game's `breakDown` (§4). Removes a cell only
//!   if the deduction engine alone (no guessing) can re-derive its value from the
//!   remaining clues. Pairs with the game's in-game solver.
//! - `ohhi_solver::carver::carve`: count-based minimal seed. Removes a cell whenever
//!   the board still has exactly one valid completion (guessing allowed). Produces
//!   different, often harder boards.
//!
//! The gap between what each reducer keeps is the Phase E.4 data point.

use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use ohhi_solver::v1::deduction::{deduce_with, TechniqueSet};
use rand::Rng;
use rand::seq::SliceRandom;

/// Faithful port of the game's `breakDown` reducer (§4 of board-generation.md).
///
/// Iterates cells in random order. For each cell, clears it from the working board
/// and runs the full deduction engine (no guessing). If the engine re-derives that
/// exact cell's original value, the hole is kept; otherwise the cell is restored.
/// **Holes accumulate** — each check runs against the current dug board.
///
/// Returns `(puzzle, empties)`.
pub fn breakdown(full: &BitBoard, rng: &mut impl Rng) -> (BitBoard, usize) {
    let (n_rows, n_cols) = (full.height(), full.width());
    let mut working = full.clone();

    // Build a cell pool and shuffle it once for this attempt.
    let mut cell_pool: Vec<(usize, usize)> = (0..n_rows)
        .flat_map(|r| (0..n_cols).map(move |c| (r, c)))
        .collect();
    cell_pool.shuffle(rng);

    for (r, c) in cell_pool {
        let original = working.get((r, c));
        if original == Cell::Nothing {
            continue; // already empty (shouldn't happen on a full board, but be safe)
        }

        // Clear the cell and run deduction from scratch.
        working.set((r, c), Cell::Nothing);

        let trace = deduce_with(&working, TechniqueSet::ALL);

        // Check if the engine re-derived the value of (r, c).
        let re_derived = trace.get_steps().iter().any(|s| s.at == (r, c) && s.cell == original);

        if !re_derived {
            // Deduction couldn't prove this cell — restore it.
            working.set((r, c), original);
        }
        // If re-derived, keep the hole (working already has the cell cleared).
    }

    let empties = (0..n_rows)
        .flat_map(|r| (0..n_cols).map(move |c| (r, c)))
        .filter(|&pos| working.get(pos) == Cell::Nothing)
        .count();

    (working, empties)
}
