//! v1 vs v2 engine diff — compares which cells each engine forces to fixpoint.
//!
//! `diff_engines` runs both engines on the same input and returns three sets:
//! - `only_v2` — cells forced by v2 but not v1 (the "gap win")
//! - `only_v1` — cells forced by v1 but not v2 (should always be empty; a
//!   non-empty `only_v1` signals a v2 soundness bug)
//! - `agree` — count of cells both engines forced to the same value

use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use crate::v1::deduction::{deduce_with, TechniqueSet};
use crate::v2::propagate::propagate;

/// Per-board comparison of what v1 and v2 deduced relative to the input.
pub struct EngineDiff {
    /// Cells forced by v2 but missed by v1 — the "gap win" set.
    pub only_v2: Vec<(usize, usize, Cell)>,
    /// Cells forced by v1 but not v2 — should always be empty; non-empty means
    /// a soundness bug in v2.
    pub only_v1: Vec<(usize, usize, Cell)>,
    /// Number of cells both engines placed to the same value.
    pub agree: usize,
}

/// Runs v1 (all techniques) and v2 (per-line propagation) to fixpoint on
/// `board` and compares the placements each made relative to the input.
pub fn diff_engines(board: &BitBoard) -> EngineDiff {
    // Collect every cell that was empty in `board` and is now filled in `after`,
    // as (r, c, color) triples.
    let placed = |after: &BitBoard| -> Vec<(usize, usize, Cell)> {
        let mut v = vec![];
        for r in 0..board.height() {
            for c in 0..board.width() {
                if board.get((r, c)) == Cell::Nothing {
                    let color = after.get((r, c));
                    if color != Cell::Nothing {
                        v.push((r, c, color));
                    }
                }
            }
        }
        v
    };

    let v1_trace = deduce_with(board, TechniqueSet::ALL);
    let v1_board = if let Some(last) = v1_trace.get_steps().last() {
        last.board_after.clone()
    } else {
        board.clone()
    };

    let v2_result = propagate(board);
    let v2_board = v2_result.board;

    let v1_placed = placed(&v1_board);
    let v2_placed = placed(&v2_board);

    // Build lookup: (r,c) → color for each engine.
    use std::collections::HashMap;
    let v1_map: HashMap<(usize, usize), Cell> = v1_placed.iter().map(|&(r, c, col)| ((r, c), col)).collect();
    let v2_map: HashMap<(usize, usize), Cell> = v2_placed.iter().map(|&(r, c, col)| ((r, c), col)).collect();

    let mut only_v2 = vec![];
    let mut only_v1 = vec![];
    let mut agree = 0usize;

    for &(r, c, col) in &v2_placed {
        if let Some(&v1_col) = v1_map.get(&(r, c)) {
            if v1_col == col { agree += 1; }
        } else {
            only_v2.push((r, c, col));
        }
    }
    for &(r, c, col) in &v1_placed {
        if !v2_map.contains_key(&(r, c)) {
            only_v1.push((r, c, col));
        }
    }

    EngineDiff { only_v2, only_v1, agree }
}
