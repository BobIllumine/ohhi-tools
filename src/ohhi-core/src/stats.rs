//! Bit-level analytic surface for `BitBoard`.
//!
//! The `NumTransforms` trait is the canonical place for all line-level
//! operations on a `BitBoard`. New operations should be added here rather
//! than duplicated in callers.

use crate::bit_board::BitBoard;

/// Analytic operations on rows and columns of a board.
///
/// `signature_x/y` returns the **red** bitmask for that line. On a complete
/// line the blue mask is the complement of the red mask within the line width,
/// so the red mask alone uniquely identifies the line's color pattern.
/// On partial lines the red mask alone is not sufficient.
pub trait NumTransforms {
    type AxisX;
    type AxisY;
    /// Red bitmask of row `input` (bit `c` = column `c`).
    fn signature_x(&self, input: &Self::AxisX) -> u16;
    /// Red bitmask of column `input` (bit `r` = row `r`).
    fn signature_y(&self, input: &Self::AxisY) -> u16;
    /// `(red_count, blue_count)` for row `input`.
    fn count_x(&self, input: &Self::AxisX) -> (u8, u8);
    /// `(red_count, blue_count)` for column `input`.
    fn count_y(&self, input: &Self::AxisY) -> (u8, u8);
    /// Returns `true` if row `input` contains `num` or more consecutive cells
    /// of the same color.
    fn has_consecutive_x(&self, input: &Self::AxisX, num: usize) -> bool;
    /// Returns `true` if column `input` contains `num` or more consecutive cells
    /// of the same color.
    fn has_consecutive_y(&self, input: &Self::AxisY, num: usize) -> bool;
    /// Returns `true` if every cell in row `input` is non-empty.
    fn is_complete_x(&self, input: &Self::AxisX) -> bool;
    /// Returns `true` if every cell in column `input` is non-empty.
    fn is_complete_y(&self, input: &Self::AxisY) -> bool;
}

impl NumTransforms for BitBoard {
    type AxisX = usize;
    type AxisY = usize;

    fn signature_x(&self, input: &Self::AxisX) -> u16 {
        self.get_row(*input).0
    }

    fn signature_y(&self, input: &Self::AxisY) -> u16 {
        self.get_col(*input).0
    }

    fn count_x(&self, input: &Self::AxisX) -> (u8, u8) {
        let (red, blue) = self.get_row(*input);
        (red.count_ones() as u8, blue.count_ones() as u8)
    }

    fn count_y(&self, input: &Self::AxisY) -> (u8, u8) {
        let (red, blue) = self.get_col(*input);
        (red.count_ones() as u8, blue.count_ones() as u8)
    }

    fn has_consecutive_x(&self, input: &Self::AxisX, num: usize) -> bool {
        let (mut start_red, mut start_blue) = self.get_row(*input);
        // AND-cascade-shift idiom: `mask &= mask >> 1` collapses each run of
        // set bits by one, so repeating it `num-1` times leaves a set bit
        // only where a run of at least `num` consecutive bits existed.
        for _ in 1..num {
            start_red &= start_red >> 1;
            start_blue &= start_blue >> 1;
        }
        start_red != 0 || start_blue != 0
    }

    fn has_consecutive_y(&self, input: &Self::AxisY, num: usize) -> bool {
        let (mut start_red, mut start_blue) = self.get_col(*input);
        // Same AND-cascade-shift idiom as has_consecutive_x, applied to the
        // column's bitmask (bit `r` = row `r`).
        for _ in 1..num {
            start_red &= start_red >> 1;
            start_blue &= start_blue >> 1;
        }
        start_red != 0 || start_blue != 0
    }

    fn is_complete_x(&self, input: &Self::AxisX) -> bool {
        let (red, blue) = self.get_row(*input);
        (red | blue).count_ones() as usize == self.width
    }

    fn is_complete_y(&self, input: &Self::AxisY) -> bool {
        let (red, blue) = self.get_col(*input);
        (red | blue).count_ones() as usize == self.height
    }
}
