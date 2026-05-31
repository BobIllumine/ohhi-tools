use ohhi_app::play::{GenMode, PlaySession};
use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;

// ── Shared fixtures ───────────────────────────────────────────────────────────

fn set_row(b: &mut BitBoard, r: usize, vals: [Cell; 4]) {
    for (c, &v) in vals.iter().enumerate() { b.set((r, c), v); }
}

fn tiny_puzzle() -> (BitBoard, BitBoard) {
    // Valid 4×4: all rows/cols distinct, balanced, no triples.
    // R R B B
    // B B R R
    // R B R B
    // B R B R
    let mut solution = BitBoard::new(4, 4);
    set_row(&mut solution, 0, [Cell::Red,  Cell::Red,  Cell::Blue, Cell::Blue]);
    set_row(&mut solution, 1, [Cell::Blue, Cell::Blue, Cell::Red,  Cell::Red ]);
    set_row(&mut solution, 2, [Cell::Red,  Cell::Blue, Cell::Red,  Cell::Blue]);
    set_row(&mut solution, 3, [Cell::Blue, Cell::Red,  Cell::Blue, Cell::Red ]);

    // Given: keep only first 2 cols.
    let mut given = solution.clone();
    for r in 0..4 {
        given.set((r, 2), Cell::Nothing);
        given.set((r, 3), Cell::Nothing);
    }
    (solution, given)
}

fn session_ext() -> PlaySession {
    let (sol, given) = tiny_puzzle();
    PlaySession::new(sol, given, GenMode::Extended)
}

// ── Locking ───────────────────────────────────────────────────────────────────

#[test]
fn given_cells_are_locked() {
    let s = session_ext();
    assert!(s.is_locked((0, 0)));
    assert!(!s.is_locked((0, 2)));
}

#[test]
fn set_cell_on_locked_position_is_rejected() {
    let mut s = session_ext();
    let result = s.set_cell((0, 0), Cell::Blue);
    assert!(result.is_err(), "writing a locked cell must return Err");
    assert_eq!(s.board().get((0, 0)), Cell::Red);
}

// ── Editing & history ─────────────────────────────────────────────────────────

#[test]
fn set_cell_writes_to_board_and_pushes_history() {
    let mut s = session_ext();
    s.set_cell((0, 2), Cell::Red).unwrap();
    assert_eq!(s.board().get((0, 2)), Cell::Red);
    assert_eq!(s.history_len(), 1);
}

#[test]
fn undo_restores_previous_board() {
    let mut s = session_ext();
    s.set_cell((0, 2), Cell::Red).unwrap();
    s.undo();
    assert_eq!(s.board().get((0, 2)), Cell::Nothing);
    assert_eq!(s.history_len(), 0);
}

#[test]
fn undo_on_empty_history_is_a_no_op() {
    let mut s = session_ext();
    s.undo();
    assert_eq!(s.board().get((0, 2)), Cell::Nothing);
}

// ── Mistake counting ──────────────────────────────────────────────────────────

#[test]
fn correct_move_does_not_increment_mistakes() {
    let (solution, given) = tiny_puzzle();
    let mut s = PlaySession::new(solution.clone(), given, GenMode::Extended);
    let sol_val = solution.get((0, 2));
    s.set_cell((0, 2), sol_val).unwrap();
    assert_eq!(s.mistakes(), 0);
}

#[test]
fn wrong_move_increments_mistakes() {
    let (solution, given) = tiny_puzzle();
    let mut s = PlaySession::new(solution.clone(), given, GenMode::Extended);
    let correct = solution.get((0, 2));
    let wrong = if correct == Cell::Red { Cell::Blue } else { Cell::Red };
    s.set_cell((0, 2), wrong).unwrap();
    assert_eq!(s.mistakes(), 1);
}

#[test]
fn mistakes_do_not_decrease_on_undo() {
    let (solution, given) = tiny_puzzle();
    let mut s = PlaySession::new(solution.clone(), given, GenMode::Extended);
    let correct = solution.get((0, 2));
    let wrong = if correct == Cell::Red { Cell::Blue } else { Cell::Red };
    s.set_cell((0, 2), wrong).unwrap();
    s.undo();
    assert_eq!(s.mistakes(), 1);
}

// ── Completion ────────────────────────────────────────────────────────────────

#[test]
fn is_complete_false_when_cells_remain_empty() {
    assert!(!session_ext().is_complete());
}

#[test]
fn is_complete_true_when_board_matches_solution() {
    let (solution, given) = tiny_puzzle();
    let mut s = PlaySession::new(solution.clone(), given, GenMode::Extended);
    for r in 0..4 {
        s.set_cell((r, 2), solution.get((r, 2))).unwrap();
        s.set_cell((r, 3), solution.get((r, 3))).unwrap();
    }
    assert!(s.is_complete());
}

// ── Timer ─────────────────────────────────────────────────────────────────────

#[test]
fn timer_starts_running_from_new() {
    assert!(session_ext().timer_running());
}

#[test]
fn tick_advances_elapsed_time() {
    let mut s = session_ext();
    s.tick(500);
    assert_eq!(s.elapsed_ms(), 500);
}

#[test]
fn timer_stops_on_completion() {
    let (solution, given) = tiny_puzzle();
    let mut s = PlaySession::new(solution.clone(), given, GenMode::Extended);
    s.tick(1000);
    for r in 0..4 {
        s.set_cell((r, 2), solution.get((r, 2))).unwrap();
        s.set_cell((r, 3), solution.get((r, 3))).unwrap();
    }
    let elapsed_at_win = s.elapsed_ms();
    s.tick(999);
    assert_eq!(s.elapsed_ms(), elapsed_at_win);
}

// ── Guesses ───────────────────────────────────────────────────────────────────

#[test]
fn wrong_move_is_not_a_guess() {
    let (solution, given) = tiny_puzzle();
    let mut s = PlaySession::new(solution.clone(), given, GenMode::Extended);
    let wrong = if solution.get((0, 2)) == Cell::Red { Cell::Blue } else { Cell::Red };
    s.set_cell((0, 2), wrong).unwrap();
    assert_eq!(s.guesses(), 0);
}

#[test]
fn correct_undeducible_move_is_a_guess() {
    // Completely empty board — v2 can deduce nothing, so any correct placement is a guess.
    let (solution, _) = tiny_puzzle();
    let empty_given = BitBoard::new(4, 4); // no givens at all
    let mut s = PlaySession::new(solution.clone(), empty_given, GenMode::Extended);
    s.set_cell((0, 0), solution.get((0, 0))).unwrap();
    assert_eq!(s.guesses(), 1);
}

#[test]
fn guesses_not_double_counted_on_undo_redo() {
    let (solution, _) = tiny_puzzle();
    let empty_given = BitBoard::new(4, 4);
    let mut s = PlaySession::new(solution.clone(), empty_given, GenMode::Extended);
    s.set_cell((0, 0), solution.get((0, 0))).unwrap(); // guess #1
    s.undo();
    s.set_cell((0, 0), solution.get((0, 0))).unwrap(); // same cell again — not counted again
    assert_eq!(s.guesses(), 1);
}

// ── OG difficulty ─────────────────────────────────────────────────────────────

// Build a 4×4 given where only column 3 is empty.
// Each row has 3 givens including 2 of one color → Saturation fires on every row.
// Difficulty_total should be 4 × cost(Saturation) = 4 × 3 = 12.
fn sat_puzzle() -> (BitBoard, BitBoard) {
    let mut solution = BitBoard::new(4, 4);
    // alternating: row i starts with (i%2==0 ? R : B)
    for r in 0..4 {
        for c in 0..4 {
            let cell = if (r + c) % 2 == 0 { Cell::Red } else { Cell::Blue };
            solution.set((r, c), cell);
        }
    }
    // Given: hide only col 3
    let mut given = solution.clone();
    for r in 0..4 { given.set((r, 3), Cell::Nothing); }
    (solution, given)
}

#[test]
fn og_mode_difficulty_total_nonzero_when_deduction_needed() {
    let (solution, given) = sat_puzzle();
    let s = PlaySession::new(solution, given, GenMode::Og);
    assert!(s.difficulty_total() > 0, "OG difficulty should be > 0 when saturation fires");
}

#[test]
fn og_mode_difficulty_done_increases_as_cells_are_correctly_placed() {
    let (solution, given) = sat_puzzle();
    let mut s = PlaySession::new(solution.clone(), given, GenMode::Og);
    let before = s.difficulty_done();
    for r in 0..4 { let _ = s.set_cell((r, 3), solution.get((r, 3))); }
    assert!(s.difficulty_done() > before, "difficulty_done should increase");
}

// ── Guess detection ───────────────────────────────────────────────────────────

#[test]
fn overwriting_with_correct_is_not_a_guess() {
    // When the LMB cycle goes Nothing→Red→Blue, the Blue placement overwrites
    // an already-filled cell. propagate() skips filled cells, so `pos` never
    // appears in steps → was wrongly counted as a guess.
    let (solution, given) = tiny_puzzle();
    let mut s = PlaySession::new(solution.clone(), given, GenMode::Extended);
    let correct = solution.get((0, 2));
    let wrong = if correct == Cell::Red { Cell::Blue } else { Cell::Red };
    s.set_cell((0, 2), wrong).unwrap();   // transit through wrong color
    s.set_cell((0, 2), correct).unwrap(); // place correct by overwriting
    assert_eq!(s.guesses(), 0, "overwriting a cell to the correct value is not a guess");
}

// ── Cells filled / total empties ──────────────────────────────────────────────

#[test]
fn total_empties_matches_initial_empty_count() {
    let (sol, given) = tiny_puzzle();
    let s = PlaySession::new(sol, given, GenMode::Extended);
    assert_eq!(s.total_empties(), 8); // 4 rows × 2 blank cols
}

#[test]
fn cells_filled_increases_after_placement() {
    let (solution, given) = tiny_puzzle();
    let mut s = PlaySession::new(solution.clone(), given, GenMode::Extended);
    assert_eq!(s.cells_filled(), 0);
    s.set_cell((0, 2), solution.get((0, 2))).unwrap();
    assert_eq!(s.cells_filled(), 1);
}

#[test]
fn cells_filled_decreases_after_undo() {
    let (solution, given) = tiny_puzzle();
    let mut s = PlaySession::new(solution.clone(), given, GenMode::Extended);
    s.set_cell((0, 2), solution.get((0, 2))).unwrap();
    s.undo();
    assert_eq!(s.cells_filled(), 0);
}
