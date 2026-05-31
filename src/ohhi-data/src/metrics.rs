use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use ohhi_solver::v2::propagate::propagate;

/// Number of cells v2 can force from the given clues alone.
pub fn forced_cell_count(given: &BitBoard) -> usize {
    propagate(given).steps.len()
}

/// How many empty cells remain after v2 exhausts single-line deduction.
/// Higher = harder (more guessing required).
pub fn difficulty(given: &BitBoard) -> usize {
    let n = given.width();
    let empties = (0..n).flat_map(|r| (0..n).map(move |c| (r, c)))
        .filter(|&pos| given.get(pos) == Cell::Nothing)
        .count();
    empties.saturating_sub(forced_cell_count(given))
}
