//! Fixpoint deduction engine for 0h h1.
//!
//! The engine applies four bitwise techniques in a loop until no technique
//! can fire (`find_forced` returns `None`) or a contradiction is reached.
//! Each iteration places at most one cell and records it in a `DeductionTrace`.
//!
//! Techniques are applied in this priority order within each row/column:
//! PairExtension → GapFill → Saturation → TwinCompletion.
//! Rows are scanned before columns.

use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use crate::structs::SolverState;
pub use crate::structs::{DeductionTrace, DeductionStep, Technique, TechniqueSet};

// fn find_pair_extension(state: &SolverState) -> Option<(usize, usize, Cell)> {
//     let mut result: Option<(usize, usize, Cell)> = None;
//
//     for r in 0..state.height {
//         let (red, blue) = state.board_ref().get_row(r);
//         let filled = red | blue;
//         let width_mask = (1 << state.width) - 1;
//         let forced_blue = pair_ext(red, filled, width_mask);
//         if forced_blue != usize::MAX {
//             result = Some((r, forced_blue, Cell::Blue));
//             return result;
//         }
//         let forced_red = pair_ext(blue, filled, width_mask);
//         if forced_red != usize::MAX {
//             result = Some((r, forced_red, Cell::Red));
//             return result;
//         }
//     }
//     for c in 0..state.width {
//         let (red, blue) = state.board_ref().get_col(c);
//         let filled = red | blue;
//         let height_mask = (1 << state.height) - 1;
//         let forced_blue = pair_ext(red, filled, height_mask);
//         if forced_blue != usize::MAX {
//             result = Some((forced_blue, c, Cell::Blue));
//             return result;
//         }
//         let forced_red = pair_ext(blue, filled, height_mask);
//         if forced_red != usize::MAX {
//             result = Some((forced_red, c, Cell::Red));
//             return result;
//         }
//     }
//     result
// }
//
// fn find_gap_fill(state: &SolverState) -> Option<(usize, usize, Cell)> {
//     let result: Option<(usize, usize, Cell)> = None;
//     let gap_fill = |selection: u16, filled: u16, mask: u16| {
//         let gaps = selection & (selection >> 2);
//         let forced = (gaps << 1) & mask & !filled;
//         if forced != 0 {
//             forced.trailing_zeros() as usize
//         }
//         else {
//             usize::MAX
//         }
//     };
//     for r in 0..state.height {
//         let (red, blue) = state.board_ref().get_row(r);
//         let filled = red | blue;
//         let width_mask = (1 << state.width) - 1;
//         let forced_blue = gap_fill(red, filled, width_mask);
//         if forced_blue != usize::MAX {
//             return Some((r, forced_blue, Cell::Blue));
//         }
//         let forced_red = gap_fill(blue, filled, width_mask);
//         if forced_red != usize::MAX {
//             return Some((r, forced_red, Cell::Red));
//         }
//     }
//     for c in 0..state.width {
//         let (red, blue) = state.board_ref().get_col(c);
//         let filled = red | blue;
//         let height_mask = (1 << state.height) - 1;
//         let forced_blue = gap_fill(red, filled, height_mask);
//         if forced_blue != usize::MAX {
//             return Some((forced_blue, c, Cell::Blue));
//         }
//         let forced_red = gap_fill(blue, filled, height_mask);
//         if forced_red != usize::MAX {
//             return Some((forced_red, c, Cell::Red));
//         }
//     }
//     result
// }
//
// fn find_saturation(state: &SolverState) -> Option<(usize, usize, Cell)> {
//     let result: Option<(usize, usize, Cell)> = None;
//     let saturation = |balance: u8, filled: u16, mask: u16| -> usize {
//         if balance == state.width as u8 {
//             let forced = mask & !filled;
//             if forced != 0 {
//                 forced.trailing_zeros() as usize
//             }
//             else {
//                 usize::MAX
//             }
//
//         } else {
//             usize::MAX
//         }
//     };
//     for r in 0..state.height {
//         let (red, blue) = state.board_ref().get_row(r);
//         let filled = red | blue;
//         let width_mask = (1 << state.width) - 1;
//         let forced_blue = saturation(red.count_ones() as u8, filled, width_mask);
//         if forced_blue != usize::MAX {
//             return Some((r, forced_blue, Cell::Blue));
//         }
//         let forced_red = saturation(blue.count_ones() as u8, filled, width_mask);
//         if forced_red != usize::MAX {
//             return Some((r, forced_blue, Cell::Red));
//         }
//     }
//     for c in 0..state.width {
//         let (red, blue) = state.board_ref().get_col(c);
//         let filled = red | blue;
//         let height_mask = (1 << state.height) - 1;
//         let forced_blue = saturation(red.count_ones() as u8, filled, height_mask);
//         if forced_blue != usize::MAX {
//             return Some((forced_blue, c, Cell::Blue));
//         }
//         let forced_red = saturation(blue.count_ones() as u8, filled, height_mask);
//         if forced_red != usize::MAX {
//             return Some((forced_red, c, Cell::Red));
//         }
//     }
//     result
// }
//
// fn find_twin_completion(state: &SolverState) -> Option<(usize, usize, Cell)> {
//     let result: Option<(usize, usize, Cell)> = None;
//     let twin_completion = |selection: u16, filled: u16, mask: u16| {
//         let empties = mask & !filled;
//         if empties.count_ones() == 2 {
//             let e1 = empties.trailing_zeros() as usize;
//             let e2 = (empties & (empties - 1)).trailing_zeros() as usize;
//             let cand_a = selection | (1 << e1);
//             let cand_b = selection | (1 << e2);
//             let twin_a = state.completed_rows().contains(&cand_a);
//             let twin_b = state.completed_rows().contains(&cand_b);
//             if twin_a && !twin_b {
//                 return e1;
//             }
//             if twin_b && !twin_a {
//                 return e2;
//             }
//         }
//         usize::MAX
//     };
//     for r in 0..state.height {
//         let (red, blue) = state.board_ref().get_row(r);
//         let filled = red | blue;
//         let width_mask = (1 << state.width) - 1;
//         let forced_blue = twin_completion(red, filled, width_mask);
//         if forced_blue != usize::MAX {
//             return Some((r, forced_blue, Cell::Blue));
//         }
//         let forced_red = twin_completion(blue, filled, width_mask);
//         if forced_red != usize::MAX {
//             return Some((r, forced_red, Cell::Red));
//         }
//     }
//     for c in 0..state.width {
//         let (red, blue) = state.board_ref().get_col(c);
//         let filled = red | blue;
//         let height_mask = (1 << state.height) - 1;
//         let forced_blue = twin_completion(red, filled, height_mask);
//         if forced_blue != usize::MAX {
//             return Some((forced_blue, c, Cell::Blue));
//         }
//         let forced_red = twin_completion(blue, filled, height_mask);
//         if forced_red != usize::MAX {
//             return Some((forced_red, c, Cell::Red));
//         }
//     }
//     result
// }

fn find_forced(state: &SolverState, enabled: TechniqueSet) -> Option<(usize, usize, Cell, Technique)> {
    // PairExtension (rule-of-3): given a color's bitmask `selection`, the
    // expression `selection & (selection >> 1)` has bit i set iff bits i AND
    // i+1 are both set — i.e. it marks the LEFT bit of every adjacent pair of
    // that color. Shifting that marker right by 1 gives the empty slot
    // immediately LEFT of the pair; shifting left by 2 gives the empty slot
    // immediately RIGHT. Intersecting with `mask & !filled` keeps only
    // positions that are in-bounds AND currently empty. The lowest set bit of
    // the result is one such forced cell (to be filled with the OPPOSITE color).
    let pair_ext = |selection: u16, filled: u16, mask: u16| {
        let pairs = selection & (selection >> 1);
        let forced = ((pairs >> 1) | (pairs << 2)) & mask & !filled;
        if forced != 0 {
            forced.trailing_zeros() as usize
        }
        else {
            usize::MAX
        }
    };

    // GapFill (rule-of-3): `selection & (selection >> 2)` has bit i set iff
    // bits i AND i+2 are set in `selection` — the X_X pattern starting at i.
    // Shifting left by 1 moves the marker to the middle slot (the gap).
    // Intersecting with `mask & !filled` ensures we only return an actual
    // empty cell. Result is filled with the OPPOSITE color.
    let gap_fill = |selection: u16, filled: u16, mask: u16| {
        let gaps = selection & (selection >> 2);
        let forced = (gaps << 1) & mask & !filled;
        if forced != 0 {
            forced.trailing_zeros() as usize
        }
        else {
            usize::MAX
        }
    };

    // Saturation (equity): when `balance` (count of one color in a line)
    // reaches half the line length, every remaining empty must be the
    // OPPOSITE color. `col` selects the correct half-length threshold
    // (height/2 for columns, width/2 for rows).
    let saturation = |balance: u8, filled: u16, mask: u16, col: bool| -> usize {
        let metric = if col { state.height as u8 / 2 } else { state.width as u8 / 2 };
        if balance == metric {
            let forced = mask & !filled;
            if forced != 0 {
                forced.trailing_zeros() as usize
            }
            else {
                usize::MAX
            }

        } else {
            usize::MAX
        }
    };

    // TwinCompletion (uniqueness): a line with exactly 2 empties has only 2
    // possible completions — one per assignment of the two empty positions.
    // If exactly one of those completions matches an already-completed line
    // elsewhere, that completion is forbidden, forcing the empties to the
    // OTHER assignment (return the index of the cell forced to opposite color).
    //
    // Bit idiom: `empties.trailing_zeros()` is the index of the lowest set
    // bit (first empty). `(empties & (empties - 1))` clears the lowest set
    // bit, so its trailing_zeros gives the SECOND empty's index.
    // Two candidate red masks are formed by OR-ing `selection` with each empty
    // bit individually; whichever candidate appears in the completed-line set
    // is the FORBIDDEN completion — return the OTHER empty's index.
    // `col` selects whether to check completed_cols or completed_rows.
    let twin_completion = |selection: u16, filled: u16, mask: u16, col: bool| {
        let empties = mask & !filled;
        if empties.count_ones() == 2 {
            let e1 = empties.trailing_zeros() as usize;
            let e2 = (empties & (empties - 1)).trailing_zeros() as usize;
            let cand_a = selection | (1 << e1);
            let cand_b = selection | (1 << e2);
            let (twin_a, twin_b): (bool, bool);
            if col {
                twin_a = state.completed_cols().contains(&cand_a);
                twin_b = state.completed_cols().contains(&cand_b);
            }
            else {
                twin_a = state.completed_rows().contains(&cand_a);
                twin_b = state.completed_rows().contains(&cand_b);
            }
            if twin_a && !twin_b {
                return e1;
            }
            if twin_b && !twin_a {
                return e2;
            }
        }
        usize::MAX
    };

    for r in 0..state.height {
        let (red, blue) = state.board_ref().get_row(r);
        let filled = red | blue;
        let width_mask = (1 << state.width) - 1;
        if enabled.contains(Technique::PairExtension) {
            let forced_blue = pair_ext(red, filled, width_mask);
            if forced_blue != usize::MAX {
                return Some((r, forced_blue, Cell::Blue, Technique::PairExtension));
            }
            let forced_red = pair_ext(blue, filled, width_mask);
            if forced_red != usize::MAX {
                return Some((r, forced_red, Cell::Red, Technique::PairExtension));
            }
        }
        if enabled.contains(Technique::GapFill) {
            let forced_blue = gap_fill(red, filled, width_mask);
            if forced_blue != usize::MAX {
                return Some((r, forced_blue, Cell::Blue, Technique::GapFill));
            }
            let forced_red = gap_fill(blue, filled, width_mask);
            if forced_red != usize::MAX {
                return Some((r, forced_red, Cell::Red, Technique::GapFill));
            }
        }
        if enabled.contains(Technique::Saturation) {
            let forced_blue = saturation(red.count_ones() as u8, filled, width_mask, false);
            if forced_blue != usize::MAX {
                return Some((r, forced_blue, Cell::Blue, Technique::Saturation));
            }
            let forced_red = saturation(blue.count_ones() as u8, filled, width_mask, false);
            if forced_red != usize::MAX {
                return Some((r, forced_red, Cell::Red, Technique::Saturation));
            }
        }
        if enabled.contains(Technique::TwinCompletion) {
            // Only call with the red mask. completed_rows/cols store red masks, so
            // twin_completion(blue, ...) would compare blue-derived values against red
            // masks — accidental collisions produce false-positive forced-Red placements.
            // The red arm covers both empties; Saturation forces the remaining Red after.
            let forced_blue = twin_completion(red, filled, width_mask, false);
            if forced_blue != usize::MAX {
                return Some((r, forced_blue, Cell::Blue, Technique::TwinCompletion));
            }
        }
    }

    for c in 0..state.width {
        let (red, blue) = state.board_ref().get_col(c);
        let filled = red | blue;
        let height_mask = (1 << state.height) - 1;
        if enabled.contains(Technique::PairExtension) {
            let forced_blue = pair_ext(red, filled, height_mask);
            if forced_blue != usize::MAX {
                return Some((forced_blue, c, Cell::Blue, Technique::PairExtension));
            }
            let forced_red = pair_ext(blue, filled, height_mask);
            if forced_red != usize::MAX {
                return Some((forced_red, c, Cell::Red, Technique::PairExtension));
            }
        }
        if enabled.contains(Technique::GapFill) {
            let forced_blue = gap_fill(red, filled, height_mask);
            if forced_blue != usize::MAX {
                return Some((forced_blue, c, Cell::Blue, Technique::GapFill));
            }
            let forced_red = gap_fill(blue, filled, height_mask);
            if forced_red != usize::MAX {
                return Some((forced_red, c, Cell::Red, Technique::GapFill));
            }
        }
        if enabled.contains(Technique::Saturation) {
            let forced_blue = saturation(red.count_ones() as u8, filled, height_mask, true);
            if forced_blue != usize::MAX {
                return Some((forced_blue, c, Cell::Blue, Technique::Saturation));
            }
            let forced_red = saturation(blue.count_ones() as u8, filled, height_mask, true);
            if forced_red != usize::MAX {
                return Some((forced_red, c, Cell::Red, Technique::Saturation));
            }
        }
        if enabled.contains(Technique::TwinCompletion) {
            let forced_blue = twin_completion(red, filled, height_mask, true);
            if forced_blue != usize::MAX {
                return Some((forced_blue, c, Cell::Blue, Technique::TwinCompletion));
            }
        }
    }
    None
}
/// Runs all four techniques to fixpoint on `board`.
/// Equivalent to `deduce_with(board, TechniqueSet::ALL)`.
pub fn deduce(board: &BitBoard) -> DeductionTrace {
    deduce_with(board, TechniqueSet::ALL)
}

/// Runs the enabled subset of techniques to fixpoint on `board`.
///
/// Returns a `DeductionTrace` containing every forced cell in order.
/// - `trace.stalled == true`: a technique fired but the placement was illegal
///   (contradiction). The trace contains steps up to but not including the
///   contradiction.
/// - `trace.stalled == false` and `steps.len() == 0`: no technique fired at
///   all; the board is either already complete or requires guessing.
/// - `trace.stalled == false` and `steps.len() > 0`: the engine ran to
///   fixpoint; check `steps.last().board_after` for the resulting board.
pub fn deduce_with(board: &BitBoard, set: TechniqueSet) -> DeductionTrace {
    let mut trace = DeductionTrace::new();
    let mut state = SolverState::new(board);
    loop {
        let next = find_forced(&state, set);
        match next {
            Some((r, c, cell, technique)) => {
                if !state.place(cell, (r, c)) {
                    trace.stalled = true;
                    return trace;
                }
                trace.add_step(DeductionStep {
                    at: (r, c),
                    cell,
                    technique,
                    board_after: state.board_ref().clone()
                });
            }
            None => break
        }
    }
    trace
}

