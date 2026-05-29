//! Integration tests for the v1-vs-v2 engine diff.
//!
//! Key invariants:
//! - `only_v1` must always be empty: v2 is strictly ≥ v1 in per-line power.
//! - `only_v2` is non-empty on boards with counting deductions v1 misses.
//! - Both empty on boards where v1 already reaches fixpoint.

use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use ohhi_solver::v2::diff::diff_engines;

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
fn diff_only_v1_always_empty_on_trivial_board() {
    // v1 can handle a plain pair-extension; v2 also handles it (and possibly
    // more). The soundness invariant: nothing v1 forces should be missed by v2.
    let d = diff_engines(&bb(&["RR..", "....", "....", "...."]));
    assert!(d.only_v1.is_empty(), "v2 missed a v1 deduction: {:?}", d.only_v1);
}

#[test]
fn diff_only_v2_nonempty_on_counting_board() {
    // "R R . . . ." in a 6-wide board. v1 forces (0,2) Blue via pair-extension,
    // then stalls. v2 also forces (0,5) Blue via counting — the far-end cell
    // that v1's four named techniques cannot reach.
    let d = diff_engines(&bb(&[
        "RR....", "......", "......", "......", "......", "......",
    ]));
    assert!(d.only_v1.is_empty(), "v2 missed a v1 deduction: {:?}", d.only_v1);
    assert!(
        !d.only_v2.is_empty(),
        "expected v2 to force cells v1 misses on the counting board"
    );
    assert!(
        d.only_v2.iter().any(|&(r, c, col)| r == 0 && c == 5 && col == Cell::Blue),
        "expected (0,5,Blue) in only_v2, got {:?}", d.only_v2
    );
}

#[test]
fn diff_empty_on_blank_board() {
    // Neither engine forces anything on a completely empty board.
    let d = diff_engines(&BitBoard::new(4, 4));
    assert!(d.only_v1.is_empty());
    assert!(d.only_v2.is_empty());
    assert_eq!(d.agree, 0);
}

#[test]
fn diff_agree_count_correct_on_simple_pair() {
    // Both engines force (0,2) Blue and (0,3) Blue on "RR..". agree should be 2.
    let d = diff_engines(&bb(&["RR..", "....", "....", "...."]));
    assert!(d.only_v1.is_empty());
    assert_eq!(d.agree, 2, "expected 2 agreed cells, got {}", d.agree);
}

#[test]
fn diff_only_v1_invariant_on_uniqueness_board() {
    // A board requiring uniqueness deduction: row 0 complete, row 1 near-twin.
    // Regardless of which engine fires which cell, v2 must never miss what v1 forces.
    let d = diff_engines(&bb(&["RBRB", "RB..", "....", "...."]));
    assert!(d.only_v1.is_empty(), "soundness violation: v2 missed {:?}", d.only_v1);
}
