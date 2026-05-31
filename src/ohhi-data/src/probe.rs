//! Budget-parametrized line probes for pattern research.
//!
//! The mined [`crate::catalog`] always assumes a balanced line (`n/2` of each
//! colour). These probes drop that assumption: they let you ask "what does this
//! partial line force if exactly `target_red` of its cells end up Red?" — for
//! any length (including odd `n`) and any colour budget.
//!
//! This is the foundation for the experiments on *why* a counting atom fires:
//! the controlling variable is the remaining colour budget, not geometry, so
//! every higher-level probe (threshold finding, translation-invariance scans)
//! reduces to [`forced_under`].

use ohhi_core::board::Cell;
use ohhi_solver::v2::line_solver::{forced_cells, legal_completions_budget};

/// Parses a textual line like `"R..B.R"` or `"R . . . . R"` into
/// `(red_mask, blue_mask, n)`. Whitespace is ignored; `.` (or `_`) is empty.
///
/// Panics on any other character — these are test/research inputs, not user data.
pub fn parse_line(s: &str) -> (u16, u16, usize) {
    let mut red = 0u16;
    let mut blue = 0u16;
    let mut n = 0usize;
    for ch in s.chars().filter(|c| !c.is_whitespace()) {
        match ch {
            'R' | 'r' => red |= 1 << n,
            'B' | 'b' => blue |= 1 << n,
            '.' | '_' => {}
            other => panic!("parse_line: unexpected character {other:?}"),
        }
        n += 1;
    }
    (red, blue, n)
}

/// The cells forced in a partial line if exactly `target_red` cells are Red.
///
/// Returns `None` when the state is **contradictory** (no legal completion at
/// that budget) — distinct from `Some(vec![])`, which means "consistent but
/// nothing is forced". `red`/`blue` are disjoint masks of the fixed cells.
pub fn forced_under(
    red: u16,
    blue: u16,
    n: usize,
    target_red: usize,
) -> Option<Vec<(usize, Cell)>> {
    let comps = legal_completions_budget(red, blue, n, target_red);
    if comps.is_empty() {
        None
    } else {
        Some(forced_cells(&comps, red | blue, n))
    }
}

/// Convenience wrapper over [`forced_under`] taking a textual line and red budget.
pub fn forced_line(line: &str, target_red: usize) -> Option<Vec<(usize, Cell)>> {
    let (red, blue, n) = parse_line(line);
    forced_under(red, blue, n, target_red)
}

/// Skeleton family #1: two Red anchors `d` apart, the left one at `off`, in an
/// otherwise-empty line of length `n`. Reports whether the two **inner**
/// neighbour cells (just inside each anchor) are both forced Blue under the
/// given red budget.
///
/// `Some(true)`  — both inner cells forced Blue (the atom *fires*).
/// `Some(false)` — a legal completion exists but the inner cells aren't pinned.
/// `None`        — contradictory: no legal completion at this budget.
pub fn two_reds_inner_forced(
    n: usize,
    off: usize,
    d: usize,
    target_red: usize,
) -> Option<bool> {
    debug_assert!(off + d < n, "anchors must fit inside the line");
    let red = (1u16 << off) | (1u16 << (off + d));
    let forced = forced_under(red, 0, n, target_red)?;
    let inner = [off + 1, off + d - 1];
    Some(inner.iter().all(|&p| {
        forced.iter().any(|&(q, c)| q == p && c == Cell::Blue)
    }))
}

/// The critical red budgets at which a corner-anchored two-red skeleton of gap
/// `d` fires, scanning all budgets `0..=n`. Empty ⇒ never fires at this `n`.
pub fn firing_budgets(n: usize, d: usize) -> Vec<usize> {
    (0..=n)
        .filter(|&tr| two_reds_inner_forced(n, 0, d, tr) == Some(true))
        .collect()
}

/// Embeds a window — relative red/blue masks of width `w` — at absolute offset
/// `off` in an otherwise-empty line of length `n`, and returns the forced cells
/// under `target_red` (absolute positions). `None` ⇒ contradictory.
///
/// This is the workhorse for invariance scans: slide a fixed atom shape across
/// positions and lengths and see whether its deductions survive.
pub fn embed_forced(
    win_red: u16,
    win_blue: u16,
    w: usize,
    off: usize,
    n: usize,
    target_red: usize,
) -> Option<Vec<(usize, Cell)>> {
    debug_assert!(off + w <= n, "window must fit inside the line");
    forced_under(win_red << off, win_blue << off, n, target_red)
}

/// Does the window's native forced set (relative positions `win_forced`) survive
/// when embedded at `off` in length `n` under `target_red`? I.e. is every native
/// deduction still forced to the same colour?
pub fn window_survives(
    win_red: u16,
    win_blue: u16,
    w: usize,
    win_forced: &[(usize, Cell)],
    off: usize,
    n: usize,
    target_red: usize,
) -> bool {
    match embed_forced(win_red, win_blue, w, off, n, target_red) {
        None => false,
        Some(forced) => win_forced.iter().all(|&(p, color)| {
            forced.iter().any(|&(q, c)| q == p + off && c == color)
        }),
    }
}
