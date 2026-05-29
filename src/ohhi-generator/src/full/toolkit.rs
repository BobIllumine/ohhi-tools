//! "Our" generator — a randomized DFS sandbox for experimenting with alternative
//! full-board constructions.
//!
//! Unlike [`OgGenerator`](super::og::OgGenerator) which fills row-by-row from a
//! legal-line pool, this DFS visits cells in row-major order and tries Red/Blue in a
//! **randomly shuffled order** at each node. The result is a uniformly-varied complete
//! board rather than a lex-first one.
//!
//! # Tuning points
//!
//! Two aspects are intentionally left open for the user to experiment with:
//! - **Cell ordering**: currently row-major; try column-major, spiral, or random.
//! - **Restart policy**: currently pure DFS with no restart; an RNG-seeded restart
//!   after N backtracks can sometimes find solutions faster on large boards.
//!
//! See the `// TODO(user): experiment here` markers below.

use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use ohhi_solver::structs::SolverState;
use rand::Rng;
use rand::seq::SliceRandom;
use crate::FullBoardGenerator;

/// Randomized DFS full-board generator. A sandbox for alternative constructions.
pub struct ToolkitGenerator;

impl FullBoardGenerator for ToolkitGenerator {
    fn generate(&self, n: usize, rng: &mut impl Rng) -> BitBoard {
        toolkit_generate(n, rng)
    }
}

fn toolkit_generate(n: usize, rng: &mut impl Rng) -> BitBoard {
    let empty = BitBoard::new(n, n);
    let mut state = SolverState::new(&empty);

    // TODO(user): experiment here — alternative cell orderings:
    //   col-major:  (0..n).flat_map(|c| (0..n).map(move |r| (r, c)))
    //   random:     collect then shuffle
    let cells: Vec<(usize, usize)> = (0..n)
        .flat_map(|r| (0..n).map(move |c| (r, c)))
        .collect();

    if dfs(&cells, 0, &mut state, rng) {
        state.board_ref().clone()
    } else {
        // Should never happen on a valid n (even, n≤16).
        unreachable!("ToolkitGenerator DFS failed to find a solution for n={n}");
    }
}

/// Recursive DFS. Returns `true` when a complete legal board is found.
fn dfs(cells: &[(usize, usize)], idx: usize, state: &mut SolverState, rng: &mut impl Rng) -> bool {
    if idx == cells.len() {
        return true;
    }
    let (r, c) = cells[idx];

    // TODO(user): experiment here — try a different branch ordering or bias.
    let mut colors = [Cell::Red, Cell::Blue];
    colors.shuffle(rng);

    for color in colors {
        if state.place(color, (r, c)) {
            if dfs(cells, idx + 1, state, rng) {
                return true;
            }
        }
        state.unplace((r, c));
    }
    false
}
