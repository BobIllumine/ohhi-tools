//! Integration tests for engine v2's fixpoint propagator.
//!
//! Hand-verified boards, public API only. `bb` builds a board from dense rows of
//! `R`/`B`/`.` (no spaces). Assertions target the specific forced cells rather
//! than the whole board, so they stay robust to incidental cells the engine also
//! fills on the way to fixpoint.

use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use ohhi_solver::v2::propagate::propagate;

fn bb(rows: &[&str]) -> BitBoard {
    let grid: Vec<Vec<Cell>> = rows
        .iter()
        .map(|line| {
            line.chars()
                .map(|ch| match ch {
                    'R' => Cell::Red,
                    'B' => Cell::Blue,
                    _ => Cell::Nothing,
                })
                .collect()
        })
        .collect();
    BitBoard::from(&grid)
}

#[test]
fn propagate_solves_pair_extension() {
    // Row 0 "R R . ." → pair-extension forces (0,2) Blue, then equity forces
    // (0,3) Blue. Nothing else is forced (every column has a single filled cell).
    let p = propagate(&bb(&["RR..", "....", "....", "...."]));
    assert!(!p.stalled);
    assert_eq!(p.board.get((0, 2)), Cell::Blue);
    assert_eq!(p.board.get((0, 3)), Cell::Blue);
}

#[test]
fn propagate_cracks_counting_case() {
    // Row 0 "R R . . . ." in a 6-wide board. Pair-extension forces (0,2) Blue,
    // but (0,5) is *also* forced Blue: coloring it Red would strand the last red
    // and force a blue triple at cols 2,3,4. v1's four techniques never see the
    // far cell — this is the headline v2 capability.
    let p = propagate(&bb(&[
        "RR....", "......", "......", "......", "......", "......",
    ]));
    assert!(!p.stalled);
    assert_eq!(p.board.get((0, 2)), Cell::Blue);
    assert_eq!(p.board.get((0, 5)), Cell::Blue, "the counting cell v1 misses");
}

#[test]
fn propagate_honors_uniqueness() {
    // Row 0 is complete (RBRB). Row 1 "R B . ." has two empties; filling them as
    // R,B would duplicate row 0, so that completion is forbidden and the cells are
    // forced the other way → (1,2) Blue, (1,3) Red. Without the uniqueness rule
    // neither cell would be forced (both completions are otherwise legal).
    let p = propagate(&bb(&["RBRB", "RB..", "....", "...."]));
    assert!(!p.stalled);
    assert_eq!(p.board.get((1, 2)), Cell::Blue);
    assert_eq!(p.board.get((1, 3)), Cell::Red);
}

#[test]
fn propagate_reaches_fixpoint_idempotent() {
    // Propagating an already-propagated board forces nothing more.
    let first = propagate(&bb(&["RR..", "....", "....", "...."]));
    let second = propagate(&first.board);
    assert!(second.steps.is_empty(), "fixpoint output should be stable");
    assert_eq!(second.board, first.board);
}

#[test]
fn propagate_blank_board_forces_nothing() {
    // An empty board has no single-line forced cell — propagation is a no-op,
    // not a stall.
    let p = propagate(&BitBoard::new(4, 4));
    assert!(!p.stalled);
    assert!(p.steps.is_empty());
}
