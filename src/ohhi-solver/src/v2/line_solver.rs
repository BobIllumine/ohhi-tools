//! Complete single-line deduction.
//!
//! For one row or column in isolation, the anti-triple rule (1) and the equity
//! rule (2) fully determine which completions are legal. [`legal_completions`]
//! enumerates every legal full red-mask consistent with the cells already
//! placed; [`forced_cells`] intersects them — any empty cell that takes the
//! same color in *all* completions is forced.
//!
//! This subsumes pair-extension, gap-fill, saturation, and the counting
//! deductions (e.g. "only one placement of the remaining colors avoids a
//! triple") in a single operation. Uniqueness (rule 3) is global and handled by
//! the caller, which filters out completions that duplicate a completed line.

use ohhi_core::board::Cell;

/// Returns every rule-1&2-legal full red-mask of an `n`-cell line consistent
/// with the fixed cells.
///
/// `red` / `blue` are disjoint bit masks of the cells already fixed to that
/// color (bit `i` = position `i`). A returned mask `m` keeps every fixed red
/// (`red ⊆ m`) and every fixed blue (`m & blue == 0`), has exactly `n/2` reds,
/// and contains no three consecutive cells of the same color.
pub fn legal_completions(red: u16, blue: u16, n: usize) -> Vec<u16> {
    legal_completions_budget(red, blue, n, n / 2)
}

/// Like [`legal_completions`] but with an explicit red target instead of the
/// equity rule's `n/2`. The blue target is implied: `n - target_red`.
///
/// This decouples line solving from the even-`n` / balanced assumption so
/// research probes can sweep arbitrary colour budgets and odd line lengths
/// (where the standard equity rule doesn't apply). With `target_red == n/2` it
/// is identical to [`legal_completions`].
pub fn legal_completions_budget(red: u16, blue: u16, n: usize, target_red: usize) -> Vec<u16> {
    let full = (1 << n) - 1;
    let empties = full & !red & !blue;
    let mut sub = empties;
    let mut res = vec![];
    loop {
        let m = red | sub;
        let b = full & !m;
        let consec = (m & (m >> 1) & (m >> 2) == 0) && (b & (b >> 1) & (b >> 2) == 0);
        let balance = m.count_ones() == target_red as u32;
        if consec && balance {
            res.push(m);
        }
        if sub == 0 {
            break res;
        }
        sub = (sub - 1) & empties;
    }
}

/// Given the legal completions of a line and the mask of already-filled cells
/// (`filled == red | blue`), returns the empty cells whose color is identical
/// across every completion, paired with that forced color, in ascending
/// position order.
pub fn forced_cells(completions: &[u16], filled: u16, n: usize) -> Vec<(usize, Cell)> {
    let mut cells = vec![];
    if completions.is_empty() {
        return cells;
    }
    for p in 0..n {
        if filled & (1 << p) != 0 { continue; }
        let m = (completions[0] >> p) & 1;
        if completions.iter().all(|&c| (c >> p) & 1 == m) {
           cells.push((p, if m == 1 { Cell::Red } else { Cell::Blue }));
        }
    }
    cells
}
