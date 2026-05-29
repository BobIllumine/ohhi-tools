//! Faithful port of the original game's `generateFast` algorithm (§3 of board-generation.md).
//!
//! # Algorithm
//!
//! 1. Build the combo pool: all legal complete lines of length `n` (balanced, no triples).
//! 2. Shuffle the pool with Fisher–Yates.
//! 3. Fill rows top→bottom. For each candidate, place its cells into `SolverState`;
//!    if any `place` call fails a rule, unplace that row and try the next pool entry.
//! 4. Dead-end backtrack: if a row exhausts the pool, clear rows `1..=y` (row 0 is the
//!    fixed anchor) and restart filling from row 1 with a re-shuffled pool.
//!
//! Column-uniqueness falls out for free: `SolverState::place` inserts completed-column
//! signatures and rejects duplicates, so no separate uniqueness scan is needed.

use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use ohhi_solver::structs::SolverState;
use rand::Rng;
use rand::seq::SliceRandom;
use crate::FullBoardGenerator;
use crate::combos::legal_lines;

/// Faithful port of the game's row-by-row combo-pool generator.
pub struct OgGenerator;

impl FullBoardGenerator for OgGenerator {
    fn generate(&self, n: usize, rng: &mut impl Rng) -> BitBoard {
        og_generate(n, rng)
    }
}

fn og_generate(n: usize, rng: &mut impl Rng) -> BitBoard {
    let mut pool = legal_lines(n);
    // The only randomness source: the order in which row patterns are tried.
    // A fixed seed therefore reproduces the same board.
    pool.shuffle(rng);

    let empty = BitBoard::new(n, n);
    let mut state = SolverState::new(&empty);

    if fill_row(&mut state, 0, n, &pool) {
        state.board_ref().clone()
    } else {
        // legal_lines(n) is non-empty for every valid even n, and a complete
        // board always exists, so the DFS cannot exhaust without a solution.
        unreachable!("OG generator found no valid board for n={n}");
    }
}

/// Recursively fills rows `r..n` by depth-first search over the shuffled pool.
///
/// For each candidate pattern, `try_place_row` enforces all three rules against
/// the columns and already-placed rows (including uniqueness), pruning illegal
/// branches immediately. On a dead end the row is unwound and the next pattern
/// is tried; if none fit, the caller backtracks.
fn fill_row(state: &mut SolverState, r: usize, n: usize, pool: &[u16]) -> bool {
    if r == n {
        return true;
    }
    for &pat in pool {
        if try_place_row(state, r, pat, n) {
            if fill_row(state, r + 1, n, pool) {
                return true;
            }
            // This row led to a dead end downstream — undo it and try the next.
            clear_row(state, r, n);
        }
    }
    false
}

/// Tries to place a complete row pattern; returns `true` if all cells were legal.
///
/// On the first illegal placement the successfully-placed cells of this row are
/// unwound and `false` is returned. The rejected cell needs no cleanup — `place`
/// rolls itself back on failure.
fn try_place_row(state: &mut SolverState, r: usize, pattern: u16, n: usize) -> bool {
    let mut placed_up_to = 0usize;
    for c in 0..n {
        let color = if (pattern >> c) & 1 == 1 { Cell::Red } else { Cell::Blue };
        if state.place(color, (r, c)) {
            placed_up_to = c + 1;
        } else {
            // Undo only the cells we successfully placed in this row.
            for uc in 0..placed_up_to {
                state.unplace((r, uc));
            }
            return false;
        }
    }
    true
}

/// Clears an entire row in `state`, removing it from the completed-line bookkeeping.
fn clear_row(state: &mut SolverState, r: usize, n: usize) {
    for c in 0..n {
        state.unplace((r, c));
    }
}
