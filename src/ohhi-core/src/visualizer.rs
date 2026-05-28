//! `Display` implementations for board types.
//!
//! `BitBoard`'s `Display` outputs the seed format: each cell is `R`/`B`/`.`,
//! cells within a row are separated by spaces, rows are separated by newlines.
//! This is the format consumed by the seed parser in `ohhi-gui`.
//!
//! `Line` and `Board` `Display` impls are legacy (no spaces between cells);
//! prefer `BitBoard` for anything that needs to round-trip through the seed
//! parser.

use std::fmt::Display;
use crate::bit_board::BitBoard;
use crate::board::{Board, Cell, Line};

/// Blanket impl: any `Display` type gets `visualize() -> String` for free.
pub trait Visualizer : Display {
    fn visualize(&self) -> String;
}

impl<T: Display> Visualizer for T {
    fn visualize(&self) -> String {
        format!("{}", self)
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Cell::Blue => "B",
            Cell::Red => "R",
            Cell::Nothing => "."
        };
        write!(f, "{}", str)
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.iter().map(|r| r.to_string()).collect::<Vec<String>>().join(""))
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.iter().map(|r| r.to_string()).collect::<Vec<String>>().join("\n"))
    }
}

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = (0..self.height).map(
            |i| self.iter_row(i).map(|c| c.to_string()).collect::<Vec<String>>().join(" ")
        ).collect::<Vec<String>>().join("\n");

        write!(f, "{}", s)
    }
    
}