mod common;

use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;

#[test]
fn new_is_all_nothing() {
    let board = BitBoard::new(4, 4);
    for r in 0..4 {
        for c in 0..4 {
            assert_eq!(board.get((r, c)), Cell::Nothing);
        }
    }
}

#[test]
fn set_then_get_red_blue_nothing() {
    let mut board = BitBoard::new(4, 4);
    board.set((1, 2), Cell::Red);
    assert_eq!(board.get((1, 2)), Cell::Red);
    board.set((1, 2), Cell::Blue);
    assert_eq!(board.get((1, 2)), Cell::Blue);
    board.set((1, 2), Cell::Nothing);
    assert_eq!(board.get((1, 2)), Cell::Nothing);
}

#[test]
fn set_overwrites_opposite_color() {
    let mut board = BitBoard::new(4, 4);
    board.set((0, 0), Cell::Red);
    board.set((0, 0), Cell::Blue);
    assert_eq!(board.get((0, 0)), Cell::Blue);
    // Red mask must be clear — no stale red bit after overwrite
    let (red_row, _) = board.get_row(0);
    let (red_col, _) = board.get_col(0);
    assert_eq!(red_row & 1, 0, "stale red bit in row mask");
    assert_eq!(red_col & 1, 0, "stale red bit in col mask");
}

#[test]
fn dual_storage_consistency() {
    let mut board = BitBoard::new(4, 4);
    board.set((1, 2), Cell::Red);
    // Row 1, col 2: bit 2 of red_rows[1] and bit 1 of red_cols[2]
    let (red_row, _) = board.get_row(1);
    let (red_col, _) = board.get_col(2);
    assert!(red_row & (1 << 2) != 0, "red not set in row mask");
    assert!(red_col & (1 << 1) != 0, "red not set in col mask");
}

#[test]
fn lsb_first_convention() {
    let mut board = BitBoard::new(4, 4);
    board.set((0, 0), Cell::Red);
    assert_eq!(board.get_row(0).0, 0b0001, "col 0 should be bit 0");

    let mut board = BitBoard::new(4, 4);
    board.set((0, 3), Cell::Red);
    assert_eq!(board.get_row(0).0, 0b1000, "col 3 should be bit 3");
}

#[test]
fn iter_row_matches_get() {
    let board = common::bb(&["RBRB", "BRBR", "RRBB", "BBRR"]);
    for r in 0..4 {
        let via_iter: Vec<Cell> = board.iter_row(r).collect();
        let via_get: Vec<Cell> = (0..4).map(|c| board.get((r, c))).collect();
        assert_eq!(via_iter, via_get, "row {} mismatch", r);
    }
}

#[test]
fn iter_col_matches_get() {
    let board = common::bb(&["RBRB", "BRBR", "RRBB", "BBRR"]);
    for c in 0..4 {
        let via_iter: Vec<Cell> = board.iter_col(c).collect();
        let via_get: Vec<Cell> = (0..4).map(|r| board.get((r, c))).collect();
        assert_eq!(via_iter, via_get, "col {} mismatch", c);
    }
}

#[test]
fn from_vec_of_vec() {
    let cells = vec![
        vec![Cell::Red, Cell::Blue],
        vec![Cell::Blue, Cell::Red],
    ];
    let board = BitBoard::from(&cells);
    assert_eq!(board.get((0, 0)), Cell::Red);
    assert_eq!(board.get((0, 1)), Cell::Blue);
    assert_eq!(board.get((1, 0)), Cell::Blue);
    assert_eq!(board.get((1, 1)), Cell::Red);
}

#[test]
fn width_height_after_new() {
    let board = BitBoard::new(5, 7);
    assert_eq!(board.width(), 5);
    assert_eq!(board.height(), 7);
}
