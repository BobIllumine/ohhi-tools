use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use ohhi_data::metrics::{difficulty, forced_cell_count};

// Build a 4×4 given where every cell is already given (no empties).
fn fully_given_4x4() -> BitBoard {
    let mut b = BitBoard::new(4, 4);
    // R B R B / B R B R / R B R B / B R B R
    for r in 0..4 {
        for c in 0..4 {
            let cell = if (r + c) % 2 == 0 { Cell::Red } else { Cell::Blue };
            b.set((r, c), cell);
        }
    }
    b
}

// Completely empty board — nothing is forced, but we can still call the function.
fn empty_4x4() -> BitBoard {
    BitBoard::new(4, 4)
}

#[test]
fn fully_given_board_has_zero_difficulty() {
    // No empty cells → difficulty == 0
    let b = fully_given_4x4();
    assert_eq!(difficulty(&b), 0);
}

#[test]
fn empty_board_forced_count_is_zero() {
    // Nothing given → v2 can't deduce anything → forced_cell_count == 0
    let b = empty_4x4();
    assert_eq!(forced_cell_count(&b), 0);
}

#[test]
fn forced_cell_count_does_not_exceed_empty_count() {
    // Whatever v2 deduces, it can't fill more cells than are empty
    let mut b = BitBoard::new(4, 4);
    // Give only the first row
    b.set((0, 0), Cell::Red);
    b.set((0, 1), Cell::Blue);
    b.set((0, 2), Cell::Red);
    b.set((0, 3), Cell::Blue);
    let empties = (0..4).flat_map(|r| (0..4).map(move |c| (r, c)))
        .filter(|&pos| b.get(pos) == Cell::Nothing)
        .count();
    let forced = forced_cell_count(&b);
    assert!(forced <= empties, "forced={forced} > empties={empties}");
}

#[test]
fn difficulty_equals_empties_minus_forced() {
    let mut b = BitBoard::new(4, 4);
    b.set((0, 0), Cell::Red);
    b.set((0, 1), Cell::Blue);
    b.set((0, 2), Cell::Red);
    b.set((0, 3), Cell::Blue);
    let empties = (0..4).flat_map(|r| (0..4).map(move |c| (r, c)))
        .filter(|&pos| b.get(pos) == Cell::Nothing)
        .count();
    let forced = forced_cell_count(&b);
    assert_eq!(difficulty(&b), empties - forced);
}
