//! Board primitives: [`Cell`], [`Line`], and the legacy [`Board`] type.
//!
//! [`Cell`] is the canonical value type used everywhere in the crate.
//! [`Line`] and [`Board`] are the original row-of-cells representation;
//! new code should prefer [`BitBoard`](crate::bit_board::BitBoard) instead.

use std::ops::{Index, IndexMut};

/// Maximum supported board side length. Boards larger than 16×16 are rejected
/// by the GUI seed parser and are beyond the practical reach of the solver.
pub const BOARD_MAX_SIZE: u8 = 16;

/// A single cell value.
///
/// `Nothing` represents an unfilled (empty) cell. `Red` and `Blue` are the two
/// colors a player may place. The cycling order `Nothing → Red → Blue → Nothing`
/// (via [`next`](Cell::next)) matches the click behavior in the GUI.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Cell {
    Red,
    Blue,
    Nothing,
}

impl Cell {
    /// Returns the opposite color. `Nothing` maps to itself.
    pub fn flip(&self) -> Cell {
        match self {
            Cell::Red => Cell::Blue,
            Cell::Blue => Cell::Red,
            Cell::Nothing => Cell::Nothing,
        }
    }

    /// Advances to the next value in the cycle `Nothing → Red → Blue → Nothing`.
    /// Used by the GUI when the user clicks a cell to cycle its state.
    pub fn next(&self) -> Cell {
        match self {
            Cell::Red => Cell::Blue,
            Cell::Blue => Cell::Nothing,
            Cell::Nothing => Cell::Red,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    data: Vec<Cell>,
    len: usize,
}


impl Line {
    pub fn new(len: usize) -> Line {
        Line {
            data: vec![Cell::Nothing; len],
            len
        }
    }
    fn from_vec (data: &[Cell]) -> Line
    {
        Line {
            data: data.to_vec(),
            len: data.len(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> impl Iterator<Item=&Cell> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut Cell> {
        self.data.iter_mut()
    }
}

impl From<&Vec<Cell>> for Line {
    fn from(data: &Vec<Cell>) -> Line {
        Line::from_vec(data)
    }
}


impl Index<usize> for Line {
    type Output = Cell;
    fn index(&self, index: usize) -> &Cell {
        &self.data[index]
    }
}

impl IndexMut<usize> for Line {
    fn index_mut(&mut self, index: usize) -> &mut Cell {
        &mut self.data[index]
    }
}

impl AsRef<[Cell]> for Line {
    fn as_ref(&self) -> &[Cell] {
        &self.data
    }
}

impl AsMut<[Cell]> for Line {
    fn as_mut(&mut self) -> &mut [Cell] {
        &mut self.data
    }
}

impl FromIterator<Cell> for Line {
    fn from_iter<I: IntoIterator<Item=Cell>>(iter: I) -> Line {
        let data = Vec::from_iter(iter);
        Line::from_vec(&data)
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    board: Vec<Line>,
    width: usize,
    height: usize,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Board {
        Board {
            board: vec![Line::new(width); height],
            width,
            height,
        }
    }

    fn from_cells(board: &Vec<Vec<Cell>>) -> Board {
        assert!(!board.is_empty(), "Board is empty");
        Board {
            board: board.iter().map(|x| Line::from(x)).collect(),
            width: board[0].len(),
            height: board.len(),
        }
    }

    fn from_rows(board: &Vec<Line>) -> Board {
        assert!(!board.is_empty(), "Board is empty");
        Board {
            board: board.clone(),
            width: board[0].len(),
            height: board.len()
        }
    }

    pub fn as_flat(&self) -> Vec<Cell> {
        self.board
            .iter()
            .flat_map(
                |x| x.data.iter().cloned()
            )
            .collect()
    }

    pub fn as_cells(&self) -> Vec<Vec<Cell>> {
        self.board
            .iter()
            .map(
                |x| x.data.iter().cloned().collect()
            )
            .collect()
    }

    pub fn as_rows(&self) -> Vec<Line> {
        self.board.clone()
    }

    pub fn as_cols(&self) -> Vec<Line> {
        (0..self.width).map(
            |x| {
                self.board
                    .iter()
                    .map(|y| y[x])
                    .collect()
            }
        ).collect()
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn iter(&self) -> impl Iterator<Item=&Line> {
        self.board.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut Line> {
        self.board.iter_mut()
    }

    pub fn get_row(&self, index: usize) -> &Line {
        &self.board[index]
    }

    pub fn get_row_mut(&mut self, index: usize) -> &mut Line {
        &mut self.board[index]
    }

    pub fn get_col(&self, index: usize) -> Line {
        self.board.iter().map(|x| x[index]).collect()
    }
}

impl From<&Vec<Vec<Cell>>> for Board {
    fn from(data: &Vec<Vec<Cell>>) -> Board {
        Board::from_cells(data)
    }
}

impl From<&Vec<Line>> for Board {
    fn from(data: &Vec<Line>) -> Board {
        Board::from_rows(data)
    }
}

impl Index<usize> for Board {
    type Output = Line;
    fn index(&self, index: usize) -> &Line {
        &self.board[index]
    }
}
impl IndexMut<usize> for Board {
    fn index_mut(&mut self, index: usize) -> &mut Line {
        &mut self.board[index]
    }
}

impl Index<(usize, usize)> for Board {
    type Output = Cell;
    fn index(&self, index: (usize, usize)) -> &Cell {
        &self.board[index.0][index.1]
    }
}

impl IndexMut<(usize, usize)> for Board {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Cell {
        &mut self.board[index.0][index.1]
    }
}

impl AsRef<[Line]> for Board
{
    fn as_ref(&self) -> &[Line] {
        &self.board
    }
}

impl AsMut<[Line]> for Board {
    fn as_mut(&mut self) -> &mut [Line] {
        &mut self.board
    }
}