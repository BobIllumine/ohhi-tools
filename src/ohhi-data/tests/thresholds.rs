//! Locks the threshold law for skeleton family #1 (two reds at distance `d`,
//! left corner). Everything is governed by the triple length, 3.

use ohhi_data::probe::{firing_budgets, two_reds_inner_forced};

// Masks are u16, so the line solver supports lengths up to 15.
const N_MAX: usize = 15;

/// The anti-triple density floor — the critical red budget — split by gap class.
fn density_floor(n: usize, d: usize) -> usize {
    match (d - 1) % 3 {
        1 => n / 3 + 1,
        2 => (n - 1) / 3 + 1,
        _ => unreachable!("gap ≡ 0 (mod 3) never fires"),
    }
}

/// Gap ≡ 0 (mod 3) — the gap tiles comfortably, so the inner cells are never
/// forced, at any length or budget.
#[test]
fn gap_multiple_of_three_never_fires() {
    for d in [4usize, 7, 10] {
        // d-1 ∈ {3,6,9}
        for n in (d + 1)..=N_MAX {
            assert!(
                firing_budgets(n, d).is_empty(),
                "d={d} (gap {}) unexpectedly fired at n={n}: {:?}",
                d - 1,
                firing_budgets(n, d)
            );
        }
    }
}

/// Gap ≢ 0 (mod 3) — fires at the anti-triple density floor (one red per ~3
/// cells), `⌊n/3⌋+1` for gap≡1, `⌊(n−1)/3⌋+1` for gap≡2.
#[test]
fn non_multiple_gap_fires_at_density_floor() {
    for d in [3usize, 5, 6, 8, 9] {
        // gaps 2,4,5,7,8 — all ≢ 0 (mod 3)
        for n in (d + 1)..=N_MAX {
            let fires = firing_budgets(n, d);
            assert!(!fires.is_empty(), "d={d} should fire at n={n}");
            assert!(
                fires.contains(&density_floor(n, d)),
                "d={d} n={n}: density floor {} not in {fires:?}",
                density_floor(n, d)
            );
        }
    }
}

/// At the firing budget red is scarce: never more than ~one red per three cells.
#[test]
fn firing_budget_is_scarce_red() {
    for d in [5usize, 8] {
        // single-budget gap≡1 cases
        for n in (d + 1)..=N_MAX {
            for tr in firing_budgets(n, d) {
                assert!(tr <= n / 3 + 1, "d={d} n={n}: tr={tr} not scarce");
            }
        }
    }
}

/// Below the density floor the line is contradictory (too few reds → blue
/// triple); above it there's slack and nothing is forced. The atom fires only
/// on the boundary.
#[test]
fn slack_budget_does_not_force() {
    // n=12, R....R (d=5): fires at tr=5 only; tr=6,7 have red slack → no force.
    assert_eq!(two_reds_inner_forced(12, 0, 5, 5), Some(true));
    assert_eq!(two_reds_inner_forced(12, 0, 5, 6), Some(false));
    assert_eq!(two_reds_inner_forced(12, 0, 5, 7), Some(false));
}
