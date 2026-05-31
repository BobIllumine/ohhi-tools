//! Per-line forcing-state enumeration and minimal trigger mining.
//!
//! # How it works
//!
//! For each line length `n`, we enumerate all `3^n` partial line states
//! (each cell is Red, Blue, or empty). For each state we call
//! `legal_completions` + `forced_cells` from engine v2. A state is a
//! **forcing state** if at least one empty cell is forced.
//!
//! # Rule classes
//!
//! Each forcing state is classified by the *shape* of reasoning that produces
//! the forced cell:
//! - [`RuleClass::LocalAntiTriple`] — forced purely by an adjacent pair or gap
//!   (`XX_`, `_XX`, `X_X`). Pair-extension and gap-fill territory.
//! - [`RuleClass::Saturation`] — forced because one color already has `n/2`
//!   occurrences in the line, so all remaining empties must be the other color.
//! - [`RuleClass::CountingAntiTriple`] — forced by the *combination* of equity
//!   counting and the anti-triple rule, but not by either alone. The "v1 gap"
//!   category: the col-7 style deductions that v1 misses.
//!
//! Classification is determined by running the same `forced_cells` call with
//! restrictions: if the deduction survives on a single empty near a pair →
//! LocalAntiTriple; if saturation alone suffices → Saturation; otherwise →
//! CountingAntiTriple. When a state has multiple forced cells with different
//! classes the strongest (most general) class wins.
//!
//! # Minimal trigger set
//!
//! A forcing state is **minimal** if removing any single fixed cell makes it
//! no longer forcing. Two symmetries are quotiented out before minimality:
//! - **Color-swap**: R↔B (the rules are symmetric in color).
//! - **Reflection**: bit-reverse the `n` positions (left↔right).
//!
//! The canonical representative is the lexicographically smaller state after
//! both symmetries are applied.

use ohhi_core::board::Cell;
use ohhi_solver::v2::line_solver::{forced_cells, legal_completions};

/// A partial line state: the fixed red/blue cells in a line of length `n`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LineState {
    pub n: usize,
    /// Bit mask of Red cells (bit `i` = position `i`).
    pub red: u16,
    /// Bit mask of Blue cells (bit `i` = position `i`).
    pub blue: u16,
}

/// Classification of the reasoning that produces a forced cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleClass {
    /// Forced by a local adjacent-pair or gap pattern alone.
    LocalAntiTriple,
    /// Forced because one color has reached `n/2` in this line.
    Saturation,
    /// Forced by the combination of equity counting and the anti-triple rule;
    /// requires neither alone. The category v1 misses.
    CountingAntiTriple,
}

/// One entry in the forcing-state catalog.
#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub state: LineState,
    /// The forced cells: `(position, color)`, ascending position order.
    pub forced: Vec<(usize, Cell)>,
    /// Dominant rule class for this state.
    pub rule_class: RuleClass,
}

/// Returns every forcing partial-line state for lines of length `n`.
///
/// Enumerates all `3^n` states, computes `legal_completions` + `forced_cells`
/// for each, and returns only those with at least one forced cell.
pub fn catalog(n: usize) -> Vec<CatalogEntry> {
    assert!(n <= 16 && n % 2 == 0, "n must be even and ≤16");
    let full: u16 = (1 << n) - 1;
    let mut entries = vec![];

    // Enumerate all (red, blue) pairs with red & blue == 0 and (red|blue) ⊆ full.
    // We iterate over all subsets of `full` as `filled`, then over all
    // assignments of the filled bits to Red/Blue.
    for filled in 0u32..=(full as u32) {
        let filled = filled as u16;
        if filled & !full != 0 { continue; }
        // Enumerate all red assignments of the filled bits.
        let mut sub = filled;
        loop {
            let red = sub;
            let blue = filled & !red;
            // Skip states where no cell is empty — nothing to force.
            if filled != full {
                let completions = legal_completions(red, blue, n);
                let forced = forced_cells(&completions, filled, n);
                if !forced.is_empty() {
                    let rule_class = classify(red, blue, n, &forced);
                    entries.push(CatalogEntry {
                        state: LineState { n, red, blue },
                        forced,
                        rule_class,
                    });
                }
            }
            if sub == 0 { break; }
            sub = (sub - 1) & filled;
        }
    }
    entries
}

/// Returns the minimal canonical forcing states for lines of length `n`.
///
/// Reduces `catalog(n)` by:
/// 1. **Minimality**: only keeps states where removing any single fixed cell
///    loses at least one forced deduction.
/// 2. **Symmetry quotient**: color-swap (R↔B) and reflection (left↔right);
///    within each equivalence class keeps only the canonical (smallest) one.
pub fn minimal_triggers(n: usize) -> Vec<CatalogEntry> {
    let all = catalog(n);

    // Step 1: minimality filter.
    let is_minimal = |entry: &CatalogEntry| -> bool {
        let LineState { n, red, blue } = entry.state;
        let filled = red | blue;
        // Try removing each fixed cell.
        for pos in 0..n {
            let bit = 1u16 << pos;
            if filled & bit == 0 { continue; } // already empty
            let new_red  = red  & !bit;
            let new_blue = blue & !bit;
            let new_completions = legal_completions(new_red, new_blue, n);
            let new_forced = forced_cells(&new_completions, new_red | new_blue, n);
            // If every forced cell from the original is still forced → this clue
            // is redundant → not minimal.
            if entry.forced.iter().all(|fc| new_forced.contains(fc)) {
                return false;
            }
        }
        true
    };

    let minimal: Vec<CatalogEntry> = all.into_iter().filter(|e| is_minimal(e)).collect();

    // Step 2: symmetry quotient — keep only the canonical representative.
    let canonical = |state: &LineState| -> (u16, u16) {
        let LineState { n, red, blue } = *state;
        // Identity.
        let (r0, b0) = (red, blue);
        // Color-swap.
        let (r1, b1) = (blue, red);
        // Reflection.
        let r2 = reverse_bits(red, n);
        let b2 = reverse_bits(blue, n);
        // Color-swap + reflection.
        let r3 = reverse_bits(blue, n);
        let b3 = reverse_bits(red, n);
        // Return smallest (red, blue) pair in lex order.
        [(r0,b0),(r1,b1),(r2,b2),(r3,b3)].into_iter().min().unwrap()
    };

    use std::collections::HashSet;
    let mut seen: HashSet<(u16, u16)> = HashSet::new();
    let mut result = vec![];
    for entry in minimal {
        let key = canonical(&entry.state);
        if seen.insert(key) {
            result.push(entry);
        }
    }
    result
}

/// Reverses the lowest `n` bits of `x`.
fn reverse_bits(x: u16, n: usize) -> u16 {
    let mut out = 0u16;
    for i in 0..n {
        if x & (1 << i) != 0 {
            out |= 1 << (n - 1 - i);
        }
    }
    out
}

/// Classifies the dominant rule class for a forcing state.
///
/// Tries weaker rule sets in ascending order of power; whichever fires first
/// gets the label:
/// 1. If any forced cell survives with only the 2-empty-or-less pattern
///    (adjacent pair / gap) → `LocalAntiTriple`.
/// 2. If the color count alone (`n/2` saturation) forces any cell → `Saturation`.
/// 3. Otherwise → `CountingAntiTriple`.
fn classify(red: u16, blue: u16, n: usize, forced: &[(usize, Cell)]) -> RuleClass {
    let filled = red | blue;
    let full: u16 = (1 << n) - 1;

    // Local anti-triple: pair `XX_` / `_XX` or gap `X_X` — fires when there
    // is an adjacent same-color pair with an empty neighbor, or a color
    // sandwiching an empty. Check by seeing if a forced cell appears in a
    // position immediately adjacent to a 2-in-a-row or across a gap.
    let pairs_r = red & (red >> 1);
    let pairs_b = blue & (blue >> 1);
    let gaps_r  = red & (red >> 2);
    let gaps_b  = blue & (blue >> 2);
    let local_mask = ((pairs_r >> 1) | (pairs_r << 2) | (pairs_b >> 1) | (pairs_b << 2)
        | (gaps_r << 1) | (gaps_b << 1)) & full & !filled;
    if forced.iter().any(|&(p, _)| local_mask & (1 << p) != 0) {
        return RuleClass::LocalAntiTriple;
    }

    // Saturation: one color is at n/2 — all remaining empties are the other color.
    let half = (n / 2) as u32;
    if red.count_ones() == half || blue.count_ones() == half {
        return RuleClass::Saturation;
    }

    RuleClass::CountingAntiTriple
}
