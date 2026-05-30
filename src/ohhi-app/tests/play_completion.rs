use ohhi_app::play::{GenMode, PlaySession};
use ohhi_app::{AppState, GameRecord};
use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;

// ── Two distinct valid 4×4 boards ─────────────────────────────────────────────
//
// Board A            Board B
// R R B B            R B R B
// B B R R            R B B R
// R B R B            B R R B
// B R B R            B R B R
//
// Both satisfy all three 0h h1 rules (no triple, balanced, no duplicates).

fn set_row(b: &mut BitBoard, r: usize, vals: [Cell; 4]) {
    for (c, &v) in vals.iter().enumerate() { b.set((r, c), v); }
}

fn board_a() -> BitBoard {
    let mut b = BitBoard::new(4, 4);
    set_row(&mut b, 0, [Cell::Red,  Cell::Red,  Cell::Blue, Cell::Blue]);
    set_row(&mut b, 1, [Cell::Blue, Cell::Blue, Cell::Red,  Cell::Red ]);
    set_row(&mut b, 2, [Cell::Red,  Cell::Blue, Cell::Red,  Cell::Blue]);
    set_row(&mut b, 3, [Cell::Blue, Cell::Red,  Cell::Blue, Cell::Red ]);
    b
}

fn board_b() -> BitBoard {
    let mut b = BitBoard::new(4, 4);
    set_row(&mut b, 0, [Cell::Red,  Cell::Blue, Cell::Red,  Cell::Blue]);
    set_row(&mut b, 1, [Cell::Red,  Cell::Blue, Cell::Blue, Cell::Red ]);
    set_row(&mut b, 2, [Cell::Blue, Cell::Red,  Cell::Red,  Cell::Blue]);
    set_row(&mut b, 3, [Cell::Blue, Cell::Red,  Cell::Blue, Cell::Red ]);
    b
}

// ── Validity-based completion ─────────────────────────────────────────────────

#[test]
fn is_complete_true_for_valid_filled_board_matching_solution() {
    let sol = board_a();
    let given = BitBoard::new(4, 4); // no givens
    let mut s = PlaySession::new(sol.clone(), given, GenMode::Extended);
    // Fill in sol exactly.
    for r in 0..4 {
        for c in 0..4 { s.set_cell((r, c), sol.get((r, c))).unwrap(); }
    }
    assert!(s.is_complete());
}

#[test]
fn is_complete_true_for_valid_board_that_differs_from_stored_solution() {
    // Solution stored as board_a; player fills board_b (also valid).
    let sol = board_a();
    let given = BitBoard::new(4, 4);
    let mut s = PlaySession::new(sol, given, GenMode::Extended);
    let b = board_b();
    for r in 0..4 {
        for c in 0..4 { s.set_cell((r, c), b.get((r, c))).unwrap(); }
    }
    assert!(s.is_complete(), "a different valid completion should also count as done");
}

#[test]
fn is_complete_false_for_invalid_board() {
    // Fill all cells Red — invalid (wrong balance, triples, duplicates).
    let sol = board_a();
    let given = BitBoard::new(4, 4);
    let mut s = PlaySession::new(sol, given, GenMode::Extended);
    for r in 0..4 {
        for c in 0..4 { s.set_cell((r, c), Cell::Red).unwrap(); }
    }
    assert!(!s.is_complete());
}

#[test]
fn timer_stops_when_valid_completion_is_detected() {
    let sol = board_a();
    let given = BitBoard::new(4, 4);
    let mut s = PlaySession::new(sol.clone(), given, GenMode::Extended);
    s.tick(500);
    for r in 0..4 {
        for c in 0..4 { s.set_cell((r, c), sol.get((r, c))).unwrap(); }
    }
    let stopped_at = s.elapsed_ms();
    s.tick(999);
    assert_eq!(s.elapsed_ms(), stopped_at, "timer must stop on valid completion");
}

// ── Game history ──────────────────────────────────────────────────────────────

fn app_with_completed_game() -> AppState {
    let mut app = AppState::new();
    let sol = board_a();
    let given = BitBoard::new(4, 4);
    app.play = Some(PlaySession::new(sol.clone(), given, GenMode::Extended));
    app.play.as_mut().unwrap().tick(3000);
    // Fill correctly → complete.
    for r in 0..4 {
        for c in 0..4 {
            app.play.as_mut().unwrap().set_cell((r, c), sol.get((r, c))).ok();
        }
    }
    // Record the completion.
    app.record_completion_if_done();
    app
}

#[test]
fn history_is_empty_on_new_appstate() {
    assert!(AppState::new().play_history.is_empty());
}

#[test]
fn record_completion_adds_entry_to_history() {
    let app = app_with_completed_game();
    assert_eq!(app.play_history.len(), 1);
}

#[test]
fn record_completion_clears_play_session() {
    let app = app_with_completed_game();
    assert!(app.play.is_none(), "play session should be cleared after recording");
}

#[test]
fn history_entry_captures_elapsed_time() {
    let app = app_with_completed_game();
    assert!(app.play_history[0].elapsed_ms >= 3000);
}

#[test]
fn history_entry_captures_board_size() {
    let app = app_with_completed_game();
    assert_eq!(app.play_history[0].n, 4);
}

#[test]
fn history_entry_captures_gen_mode() {
    let app = app_with_completed_game();
    assert_eq!(app.play_history[0].mode, GenMode::Extended);
}

#[test]
fn record_completion_is_no_op_when_game_not_complete() {
    let mut app = AppState::new();
    let sol = board_a();
    let given = BitBoard::new(4, 4);
    app.play = Some(PlaySession::new(sol, given, GenMode::Extended));
    // Board is NOT complete — don't fill it.
    app.record_completion_if_done();
    assert!(app.play_history.is_empty());
    assert!(app.play.is_some(), "incomplete game should not be cleared");
}

#[test]
fn multiple_games_accumulate_in_history() {
    let mut app = AppState::new();
    for _ in 0..3 {
        let sol = board_a();
        let given = BitBoard::new(4, 4);
        app.play = Some(PlaySession::new(sol.clone(), given, GenMode::Extended));
        for r in 0..4 {
            for c in 0..4 {
                app.play.as_mut().unwrap().set_cell((r, c), sol.get((r, c))).ok();
            }
        }
        app.record_completion_if_done();
    }
    assert_eq!(app.play_history.len(), 3);
}
