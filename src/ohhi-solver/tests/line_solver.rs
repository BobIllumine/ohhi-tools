//! Integration tests for the per-line solver (engine v2 core).
//!
//! These touch only the public API. The `legal_lines` reference enumerator below
//! is recreated here *independently* (a dumb 2^n scan) so the cross-check against
//! `legal_completions` is a true independent witness — it can't share a bug with
//! the generator's `combos::legal_lines`.

use ohhi_core::board::Cell;
use ohhi_solver::v2::line_solver::{forced_cells, legal_completions};

fn sorted(mut v: Vec<u16>) -> Vec<u16> {
    v.sort();
    v
}

/// Independent reference: every balanced, no-triple full red-mask of width `n`.
/// Deliberately the most literal possible translation of rules 1 & 2.
fn legal_lines(n: usize) -> Vec<u16> {
    let full: u16 = (1 << n) - 1;
    let half = (n / 2) as u32;
    (0..=full)
        .filter(|&m| {
            let blue = (!m) & full;
            m.count_ones() == half
                && (m & (m >> 1) & (m >> 2)) == 0
                && (blue & (blue >> 1) & (blue >> 2)) == 0
        })
        .collect()
}

// ── legal_completions ────────────────────────────────────────────────────────

#[test]
fn empty_line_equals_independent_legal_lines() {
    // The headline cross-check: with no fixed cells, legal_completions must
    // enumerate exactly the balanced no-triple lines, for every board size.
    for n in [2usize, 4, 6, 8] {
        assert_eq!(
            sorted(legal_completions(0, 0, n)),
            sorted(legal_lines(n)),
            "mismatch at n={n}"
        );
    }
}

#[test]
fn legal_completions_n2() {
    assert_eq!(sorted(legal_completions(0, 0, 2)), vec![0b01, 0b10]);
}

#[test]
fn empty_4line_yields_the_six_balanced_no_triple_masks() {
    assert_eq!(
        sorted(legal_completions(0, 0, 4)),
        vec![0b0011, 0b0101, 0b0110, 0b1001, 0b1010, 0b1100]
    );
}

#[test]
fn two_fixed_reds_force_the_rest_blue() {
    // "R R . ." → both reds used, empties must be blue → only RRBB (0b0011).
    assert_eq!(legal_completions(0b0011, 0, 4), vec![0b0011]);
}

#[test]
fn one_fixed_red_leaves_three_completions() {
    // "R . . ." → RRBB, RBRB, RBBR.
    assert_eq!(
        sorted(legal_completions(0b0001, 0, 4)),
        vec![0b0011, 0b0101, 0b1001]
    );
}

#[test]
fn gap_between_two_reds_only_completion_keeps_it_blue() {
    // "R . R ." → reds fixed at 0,2, empties 1,3 blue → 0b0101.
    assert_eq!(legal_completions(0b0101, 0, 4), vec![0b0101]);
}

#[test]
fn no_triple_excludes_three_in_a_row() {
    for m in legal_completions(0, 0, 6) {
        let blue = (!m) & 0b111111;
        assert_eq!(m & (m >> 1) & (m >> 2), 0, "red triple in {m:06b}");
        assert_eq!(blue & (blue >> 1) & (blue >> 2), 0, "blue triple in {m:06b}");
    }
}

// ── forced_cells ───────────────────────────────────────────────────────────────

#[test]
fn pair_extension_forces_gap_blue() {
    let comps = legal_completions(0b0011, 0, 4);
    assert_eq!(
        forced_cells(&comps, 0b0011, 4),
        vec![(2, Cell::Blue), (3, Cell::Blue)]
    );
}

#[test]
fn gap_fill_forces_middle_blue() {
    let comps = legal_completions(0b0101, 0, 4);
    assert_eq!(
        forced_cells(&comps, 0b0101, 4),
        vec![(1, Cell::Blue), (3, Cell::Blue)]
    );
}

#[test]
fn two_fixed_blues_force_the_rest_red() {
    // Color mirror of the two-fixed-reds case: "B B . ." → both empties Red.
    let comps = legal_completions(0, 0b0011, 4);
    assert_eq!(comps, vec![0b1100]);
    assert_eq!(
        forced_cells(&comps, 0b0011, 4),
        vec![(2, Cell::Red), (3, Cell::Red)]
    );
}

#[test]
fn nothing_forced_when_completions_disagree() {
    // "R . . ." → 3 completions disagree on every empty cell → none forced.
    let comps = legal_completions(0b0001, 0, 4);
    assert_eq!(forced_cells(&comps, 0b0001, 4), vec![]);
}

#[test]
fn counting_deduction_forces_near_and_far_blue() {
    // "R R . . . ." forces col 2 (pair-extension) AND col 5 (counting: a red at 5
    // would strand the last red and force a blue triple at 2,3,4). Col 5 is the
    // deduction the four named techniques miss.
    let comps = legal_completions(0b000011, 0, 6);
    assert_eq!(
        forced_cells(&comps, 0b000011, 6),
        vec![(2, Cell::Blue), (5, Cell::Blue)]
    );
}

#[test]
fn counting_deduction_forces_near_and_far_red() {
    // Color mirror: "B B . . . ." forces Red at 2 and 5.
    let comps = legal_completions(0, 0b000011, 6);
    assert_eq!(
        forced_cells(&comps, 0b000011, 6),
        vec![(2, Cell::Red), (5, Cell::Red)]
    );
}

#[test]
fn no_completions_forces_nothing() {
    // A contradicted line (no legal completion) must force nothing, not fabricate
    // cells from the empty intersection.
    assert_eq!(forced_cells(&[], 0, 4), vec![]);
}
