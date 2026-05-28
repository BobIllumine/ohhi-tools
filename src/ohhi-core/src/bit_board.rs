//! Bitpacked board representation for 0h h1.
//!
//! `BitBoard` stores the board state in four `Vec<u16>` masks — one pair for
//! rows (`red_rows`, `blue_rows`) and one pair for columns (`red_cols`,
//! `blue_cols`). The redundancy is intentional: it gives O(1) access to any
//! row or column as a bitmask without scanning the whole board.
//!
//! **Bit ordering (LSB-first):** bit `i` of a row mask corresponds to column
//! `i`; bit `i` of a column mask corresponds to row `i`. The least-significant
//! bit is the leftmost column / topmost row.

use crate::board::Cell;

/// Bitpacked 0h h1 board. See the module-level docs for the storage layout.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BitBoard {
    pub(crate) width: usize,
    pub(crate) height: usize,
    red_cols: Vec<u16>,
    blue_cols: Vec<u16>,
    red_rows: Vec<u16>,
    blue_rows: Vec<u16>,
}

impl BitBoard {
    /// Creates an empty board of the given dimensions (all cells `Nothing`).
    pub fn new(width: usize, height: usize) -> BitBoard {
        BitBoard {
            width,
            height,
            red_cols: vec![0; width],
            blue_cols: vec![0; width],
            red_rows: vec![0; height],
            blue_rows: vec![0; height],
        }
    }

    /// Returns the `Cell` at `(row, col)`.
    pub fn get(&self, (r, c): (usize, usize)) -> Cell {
        if (self.red_cols[c] & (1 << r)) != 0 {
            Cell::Red
        }
        else if (self.blue_cols[c] & (1 << r)) != 0 {
            Cell::Blue
        }
        else {
            Cell::Nothing
        }
    }

    /// Writes `cell` to `(row, col)`.
    ///
    /// Both colors are cleared in both axes before the new value is written.
    /// This prevents the red/blue row and column masks from diverging when a
    /// cell is overwritten — a bug that caused subtle corruption in an earlier
    /// version of this code.
    pub fn set(&mut self, (r, c): (usize, usize), cell: Cell) {
        // Always clear both colors in both row and column representations
        // first, so a Red→Blue overwrite doesn't leave a stale red bit.
        self.red_cols[c] &= !(1 << r);
        self.red_rows[r] &= !(1 << c);
        self.blue_cols[c] &= !(1 << r);
        self.blue_rows[r] &= !(1 << c);
        match cell {
            Cell::Red => {
                self.red_cols[c] |= 1 << r;
                self.red_rows[r] |= 1 << c;
            }
            Cell::Blue => {
                self.blue_cols[c] |= 1 << r;
                self.blue_rows[r] |= 1 << c;
            }
            _ => {}
        }
    }

    /// Returns `(red_mask, blue_mask)` for column `c`.
    /// Bit `r` of each mask corresponds to row `r`.
    pub fn get_col(&self, c: usize) -> (u16, u16) {
        (self.red_cols[c], self.blue_cols[c])
    }

    /// Returns `(red_mask, blue_mask)` for row `r`.
    /// Bit `c` of each mask corresponds to column `c`.
    pub fn get_row(&self, r: usize) -> (u16, u16) {
        (self.red_rows[r], self.blue_rows[r])
    }

    /// Iterates cells in row `r` from left to right.
    pub fn iter_row(&self, r: usize) -> impl Iterator<Item=Cell> + '_ {
        (0..self.width).map(move |c| self.get((r, c)))
    }

    /// Iterates cells in column `c` from top to bottom.
    pub fn iter_col(&self, c: usize) -> impl Iterator<Item=Cell> + '_ {
        (0..self.height).map(move |r| self.get((r, c)))
    }

    /// Returns the number of columns.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the number of rows.
    pub fn height(&self) -> usize {
        self.height
    }
}

/// Constructs a `BitBoard` from a row-major `Vec<Vec<Cell>>`.
/// `Cell::Nothing` entries produce empty cells, allowing partial-fill inputs.
impl From<&Vec<Vec<Cell>>> for BitBoard {
    fn from(input: &Vec<Vec<Cell>>) -> Self {
        let mut board = BitBoard::new(input[0].len(), input.len());
        for (r, row) in input.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                board.set((r, c), *cell);
            }
        }
        board
    }
}
