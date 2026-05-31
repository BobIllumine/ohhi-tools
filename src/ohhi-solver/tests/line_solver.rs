//! Integration tests for the per-line solver (engine v2 core).
//!
//! These touch only the public API. The `legal_lines` reference enumerator below
//! is recreated here *independently* (a dumb 2^n scan) so the cross-check against
//! `legal_completions` is a true independent witness — it can't share a bug with
//! the generator's `combos::legal_lines`.

use ohhi_core::board::Cell;
use ohhi_solver::v2::line_solver::{forced_cells, legal_completions, legal_completions_budget};

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

// ── legal_completions_budget (arbitrary red target, any parity) ──────────────

#[test]
fn budget_matches_balanced_at_half() {
    // With target = n/2 the budget variant must reproduce the balanced solver.
    for n in [2usize, 4, 6, 8] {
        assert_eq!(
            sorted(legal_completions_budget(0, 0, n, n / 2)),
            sorted(legal_completions(0, 0, n)),
            "n={n}"
        );
    }
}

#[test]
fn budget_allows_odd_lengths() {
    // n=3, exactly one red: the three single-red masks (no triple possible).
    assert_eq!(
        sorted(legal_completions_budget(0, 0, 3, 1)),
        vec![0b001, 0b010, 0b100]
    );
}

#[test]
fn budget_respects_anti_triple_at_extreme() {
    // n=3, zero reds → BBB blue triple → no legal completion.
    assert_eq!(legal_completions_budget(0, 0, 3, 0), Vec::<u16>::new());
    // n=3, three reds → RRR red triple → none.
    assert_eq!(legal_completions_budget(0, 0, 3, 3), Vec::<u16>::new());
}

#[test]
fn budget_reproduces_scarce_red_forcing() {
    // Ground truth from the line-atom probe: `R....R` forces its inner neighbours
    // Blue iff red is the scarce, near-exhausted colour.
    let r_dot_r = 0b100001u16; // R at 0 and 5
    // n=6 balanced (3 red): fires at 1,4.
    let comps = legal_completions_budget(r_dot_r, 0, 6, 3);
    assert_eq!(forced_cells(&comps, r_dot_r, 6), vec![(1, Cell::Blue), (4, Cell::Blue)]);

    // n=8 balanced (4 red): red has slack → forces nothing.
    let line8 = r_dot_r; // R....R.. , padding empty
    let comps = legal_completions_budget(line8, 0, 8, 4);
    assert_eq!(forced_cells(&comps, line8, 8), vec![]);

    // n=9, red minority (4 red / 5 blue): fires at 1,4 again.
    let comps = legal_completions_budget(r_dot_r, 0, 9, 4);
    assert_eq!(forced_cells(&comps, r_dot_r, 9), vec![(1, Cell::Blue), (4, Cell::Blue)]);
}
