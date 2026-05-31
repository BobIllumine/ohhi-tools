//! Curated library of *non-trivial* forcing patterns.
//!
//! The point of this module is to surface the deductions a casual solver (and
//! the v1 technique engine) tends to miss: the ones that need a genuine
//! counting argument across the whole line, plus the cross-line uniqueness
//! ("twin") family that no per-line catalog can produce.
//!
//! - [`nontrivial_patterns`] mines [`crate::catalog::minimal_triggers`] for the
//!   `CountingAntiTriple` class only — dropping the trivial local anti-triple
//!   and saturation atoms — and dedupes the same atom across board sizes.
//! - [`twin_patterns`] is a small hand-built enumeration of the uniqueness
//!   family.
//! - [`all_patterns`] concatenates both with stable ids assigned.

use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;

use crate::catalog::{minimal_triggers, LineState, RuleClass};

/// Board sizes mined for line-counting atoms.
pub const MINED_SIZES: [usize; 5] = [4, 6, 8, 10, 12];

/// What kind of reasoning a pattern demonstrates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternClass {
    /// A single-line equity-counting deduction (the category v1 misses).
    Counting,
    /// A cross-line uniqueness deduction (the "twin" family).
    Twin,
}

impl PatternClass {
    /// Short badge text for the UI.
    pub fn badge(&self) -> &'static str {
        match self {
            PatternClass::Counting => "counting",
            PatternClass::Twin => "twin",
        }
    }
}

/// One library entry: an example board plus the cells it forces.
#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub id: usize,
    pub name: String,
    /// A small example board showing the clue cells (forced cells left empty).
    pub example: BitBoard,
    /// The cells this pattern deduces: `(row, col) -> color`.
    pub forced: Vec<((usize, usize), Cell)>,
    pub class: PatternClass,
    pub description: String,
}

/// The full curated library of counting atoms, with sequential ids.
pub fn all_patterns() -> Vec<Pattern> {
    let mut out = nontrivial_patterns();
    for (i, p) in out.iter_mut().enumerate() {
        p.id = i;
    }
    out
}

/// Mines the counting-only minimal triggers across [`MINED_SIZES`], deduped by
/// canonical atom shape (so the same atom appearing at several board sizes is
/// listed once, keyed on the smallest size it occurs at).
pub fn nontrivial_patterns() -> Vec<Pattern> {
    use std::collections::HashSet;
    let mut seen: HashSet<Vec<u8>> = HashSet::new();
    let mut out = Vec::new();

    for n in MINED_SIZES {
        for entry in minimal_triggers(n) {
            if entry.rule_class != RuleClass::CountingAntiTriple {
                continue;
            }
            let key = canonical_atom(&entry.state, &entry.forced);
            if !seen.insert(key) {
                continue; // same atom already captured at a smaller n
            }
            out.push(pattern_from_line(&entry.state, &entry.forced));
        }
    }
    out
}

/// Hand-built uniqueness / twin patterns — cross-line deductions that the
/// per-line catalog cannot express.
pub fn twin_patterns() -> Vec<Pattern> {
    // The twin rule: two rows (or columns) may not be identical when complete.
    // If a row is one cell away from duplicating an already-complete twin row,
    // that cell is forced to the opposite color.
    //
    // Minimal 4-wide demo:
    //   row 0 (complete): R B R B
    //   row 1 (partial) : R B R .   → the last cell can't be B (that would
    //                                  duplicate row 0), so it's forced Red.
    let n = 4;
    let mut board = BitBoard::new(n, 2);
    for (c, cell) in [Cell::Red, Cell::Blue, Cell::Red, Cell::Blue].into_iter().enumerate() {
        board.set((0, c), cell);
    }
    for (c, cell) in [Cell::Red, Cell::Blue, Cell::Red].into_iter().enumerate() {
        board.set((1, c), cell);
    }
    let twin = Pattern {
        id: 0,
        name: "twin completion".to_string(),
        example: board,
        forced: vec![((1, 3), Cell::Red)],
        class: PatternClass::Twin,
        description:
            "Row 1 would duplicate the completed row 0 if its last cell were Blue, \
             so uniqueness forces it Red."
                .to_string(),
    };

    vec![twin]
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Builds a [`Pattern`] from a single-line forcing state.
fn pattern_from_line(state: &LineState, forced: &[(usize, Cell)]) -> Pattern {
    let mut example = BitBoard::new(state.n, 1);
    for pos in 0..state.n {
        if state.red & (1 << pos) != 0 {
            example.set((0, pos), Cell::Red);
        } else if state.blue & (1 << pos) != 0 {
            example.set((0, pos), Cell::Blue);
        }
    }
    let forced_cells: Vec<((usize, usize), Cell)> =
        forced.iter().map(|&(p, c)| ((0, p), c)).collect();

    Pattern {
        id: 0,
        name: atom_name(state, forced),
        example,
        forced: forced_cells,
        class: PatternClass::Counting,
        description: atom_description(state, forced),
    }
}

/// Canonical, size-independent key for a line atom.
///
/// Trims the empty padding around the active window (fixed clues + forced
/// cells), then takes the lexicographically smallest encoding over the four
/// symmetries (identity, colour-swap, reflection, both). The forced cells are
/// part of the encoding, so two windows that look alike but force different
/// cells (e.g. the same shape under a different `n/2`) stay distinct.
fn canonical_atom(state: &LineState, forced: &[(usize, Cell)]) -> Vec<u8> {
    let filled = state.red | state.blue;
    let lo = (0..state.n).find(|&p| filled & (1 << p) != 0).unwrap_or(0);
    let hi = forced
        .iter()
        .map(|&(p, _)| p)
        .chain((0..state.n).filter(|&p| filled & (1 << p) != 0))
        .max()
        .unwrap_or(0)
        .max((0..state.n).rev().find(|&p| filled & (1 << p) != 0).unwrap_or(0));
    let w = hi - lo + 1;

    // Per-position code: clue (0 none / 1 R / 2 B) * 3 + forced (0 none / 1 R / 2 B).
    let code_at = |p: usize| -> u8 {
        let clue = if state.red & (1 << p) != 0 { 1 }
            else if state.blue & (1 << p) != 0 { 2 }
            else { 0 };
        let f = forced.iter().find(|&&(fp, _)| fp == p).map(|&(_, c)| match c {
            Cell::Red => 1u8,
            Cell::Blue => 2,
            Cell::Nothing => 0,
        }).unwrap_or(0);
        clue * 3 + f
    };
    let base: Vec<u8> = (lo..=hi).map(code_at).collect();

    let swap = |code: u8| -> u8 {
        let clue = code / 3;
        let f = code % 3;
        let sc = match clue { 1 => 2, 2 => 1, x => x };
        let sf = match f { 1 => 2, 2 => 1, x => x };
        sc * 3 + sf
    };

    let id: Vec<u8> = base.clone();
    let cs: Vec<u8> = base.iter().map(|&c| swap(c)).collect();
    let mut rev: Vec<u8> = base.clone();
    rev.reverse();
    let mut csrev: Vec<u8> = cs.clone();
    csrev.reverse();

    let mut best = [id, cs, rev, csrev].into_iter().min().unwrap();
    best.insert(0, w as u8); // include width so different spans never collide
    best
}

/// A templated name based on the number of clue anchors.
fn atom_name(state: &LineState, _forced: &[(usize, Cell)]) -> String {
    let anchors = (state.red | state.blue).count_ones();
    let prefix = match anchors {
        2 => "double-anchor".to_string(),
        3 => "triple-anchor".to_string(),
        4 => "quad-anchor".to_string(),
        k => format!("{k}-anchor"),
    };
    format!("{prefix} counting")
}

/// Human description: the line shape plus the forced cells.
fn atom_description(state: &LineState, forced: &[(usize, Cell)]) -> String {
    let shape: String = (0..state.n)
        .map(|p| {
            if state.red & (1 << p) != 0 { 'R' }
            else if state.blue & (1 << p) != 0 { 'B' }
            else { '.' }
        })
        .collect();
    let mut by_color: Vec<String> = Vec::new();
    for color in [Cell::Red, Cell::Blue] {
        let cols: Vec<String> = forced
            .iter()
            .filter(|&&(_, c)| c == color)
            .map(|&(p, _)| p.to_string())
            .collect();
        if !cols.is_empty() {
            let name = if color == Cell::Red { "Red" } else { "Blue" };
            by_color.push(format!("{name} at col{} {}", if cols.len() > 1 { "s" } else { "" }, cols.join(", ")));
        }
    }
    format!("{shape} → {}", by_color.join("; "))
}
