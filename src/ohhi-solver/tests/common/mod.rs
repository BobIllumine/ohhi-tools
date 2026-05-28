use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;

/// Builds a `BitBoard` from a slice of dense row strings.
/// Each character must be `R` (red), `B` (blue), or `.` (empty).
/// Width is inferred from `rows[0].len()`; height from `rows.len()`.
pub fn bb(rows: &[&str]) -> BitBoard {
    let height = rows.len();
    let width = rows[0].len();
    let mut board = BitBoard::new(width, height);
    for (r, row) in rows.iter().enumerate() {
        for (c, ch) in row.chars().enumerate() {
            let cell = match ch {
                'R' => Cell::Red,
                'B' => Cell::Blue,
                '.' => Cell::Nothing,
                other => panic!("bb: invalid character '{}' at row {}, col {}", other, r, c),
            };
            board.set((r, c), cell);
        }
    }
    board
}
