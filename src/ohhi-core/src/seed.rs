//! Seed string encoding and parsing.
//!
//! # Seed format
//!
//! A seed is a plain-text representation of a (possibly partial) 0h h1 board:
//!
//! - Each cell is one of `R` (Red), `B` (Blue), or `.` (empty).
//! - Cells within a row are separated by **single spaces**.
//! - Rows are separated by **newlines** (`\n`).
//! - All rows must have the same number of cells.
//! - The board may be at most [`BOARD_MAX_SIZE`] × [`BOARD_MAX_SIZE`] (16×16).
//!
//! Example 4×4 seed (partial):
//!
//! ```text
//! R . B .
//! . R . B
//! B . R .
//! . B . R
//! ```

use crate::bit_board::BitBoard;
use crate::board::{Cell, BOARD_MAX_SIZE};

/// Encodes `board` as a seed string (space-delimited `R`/`B`/`.` per row,
/// newline-delimited rows).
pub fn encode(board: &BitBoard) -> String {
    board.to_string()
}

/// A seed parse error with a human-readable message.
#[derive(Debug)]
pub struct SeedError(pub String);

/// Parses a seed string into a `BitBoard`. See the module-level docs for the
/// expected format. Returns `Err(SeedError)` on malformed input or oversized
/// boards.
pub fn parse(str: &str) -> Result<BitBoard, SeedError> {
    let chars = str.lines().map(|x| x.split(" ").collect::<Vec<&str>>()).collect::<Vec<Vec<&str>>>();
    if chars.len() > BOARD_MAX_SIZE as usize || chars.iter().any(|x| x.len() > BOARD_MAX_SIZE as usize) {
        return Err(SeedError(format!("Loaded board is too large (max is {BOARD_MAX_SIZE}x{BOARD_MAX_SIZE})")))
    }
    let (dim_x, dim_y) = (chars[0].len(), chars.len());
    if chars.iter().any(|x| x.len() != dim_x) {
        return Err(SeedError("Board dimensions are inconsistent".to_string()))
    }
    let mut board = BitBoard::new(dim_x, dim_y);
    let res = chars.iter().enumerate().try_for_each(|(i, x)| {
        match x.iter().enumerate().try_for_each(|(j, y)| {
            return match y {
                &"R" => {
                    board.set((i, j), Cell::Red);
                    Ok(())
                }
                &"B" => {
                    board.set((i, j), Cell::Blue);
                    Ok(())
                }
                &"." => {
                    board.set((i, j), Cell::Nothing);
                    Ok(())
                }
                _ => { Err(SeedError(format!("Forbidden character is used at ({i}, {j})"))) }
            }
        }) {
            Ok(_) => Ok(()),
            Err(err) => { Err(err) }
        }
    });
    match res {
        Ok(_) => Ok(board),
        Err(err) => Err(err)
    }
}
