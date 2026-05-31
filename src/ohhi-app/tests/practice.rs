use ohhi_app::practice::{Attempt, PracticeSession};
use ohhi_core::board::Cell;
use ohhi_generator::practice::{target_forced, Target};

fn session() -> PracticeSession {
    PracticeSession::new(Target::Saturation, 6, 3, 42).expect("session")
}

#[test]
fn starts_with_a_live_rep() {
    let s = session();
    assert!(s.available() >= 1, "a rep must have ≥1 valid answer");
    assert_eq!(s.current_rep(), 1);
    assert!(!s.is_complete());
    assert!(s.timer_running());
}

#[test]
fn correct_target_cell_is_a_hit() {
    let mut s = session();
    let ((r, c), color) = target_forced(s.board(), Target::Saturation)[0];
    assert_eq!(s.attempt((r, c), color), Attempt::Hit);
    assert_eq!(s.hits(), 1);
    assert_eq!(s.misses(), 0);
}

#[test]
fn guessing_correct_color_of_non_target_cell_is_a_miss() {
    // A cell that is empty and NOT in the target-forced set, filled with its
    // true solution colour, must still count as a guess (miss). Scan seeds for a
    // board that has such a (currently-unsolvable) empty cell — almost all do.
    for seed in 0..50 {
        let mut s = PracticeSession::new(Target::Saturation, 6, 3, seed).unwrap();
        let forced: Vec<(usize, usize)> = target_forced(s.board(), Target::Saturation)
            .iter().map(|&(p, _)| p).collect();
        let n = s.board().width();
        let mut found = None;
        for r in 0..s.board().height() {
            for c in 0..n {
                if s.board().get((r, c)) == Cell::Nothing && !forced.contains(&(r, c)) {
                    found = Some((r, c));
                }
            }
        }
        let Some((r, c)) = found else { continue };
        // Use the cell's true solution colour — still a guess, not a deduction.
        let truth = s.solution_color((r, c));
        assert_eq!(s.attempt((r, c), truth), Attempt::Miss);
        assert_eq!(s.hits(), 0);
        assert_eq!(s.misses(), 1);
        return;
    }
    panic!("no board with a non-target empty cell found in 50 seeds");
}

#[test]
fn any_forced_cell_is_accepted() {
    // If the rep has multiple valid answers, each is independently a hit.
    let mut s = PracticeSession::new(Target::Saturation, 6, 10, 5).expect("session");
    let forced = target_forced(s.board(), Target::Saturation);
    if forced.len() >= 2 {
        let ((r, c), color) = forced[1]; // not the first one
        assert_eq!(s.attempt((r, c), color), Attempt::Hit);
    }
}

#[test]
fn locked_cells_are_ignored() {
    let mut s = session();
    // find a filled (given) cell
    let mut locked = None;
    for r in 0..s.board().height() {
        for c in 0..s.board().width() {
            if s.board().get((r, c)) != Cell::Nothing { locked = Some((r, c)); }
        }
    }
    let pos = locked.expect("a given cell exists");
    assert_eq!(s.attempt(pos, Cell::Red), Attempt::Ignored);
    assert_eq!(s.misses(), 0);
}

#[test]
fn drill_completes_after_total_reps_and_records_stats() {
    let mut s = PracticeSession::new(Target::Saturation, 6, 3, 11).expect("session");
    let mut guard = 0;
    while !s.is_complete() && guard < 1000 {
        guard += 1;
        let forced = target_forced(s.board(), Target::Saturation);
        let ((r, c), color) = forced[0];
        s.tick(100); // simulate some recognition time
        assert_eq!(s.attempt((r, c), color), Attempt::Hit);
    }
    assert!(s.is_complete());
    assert_eq!(s.hits(), 3);
    assert!(s.mean_time_ms().is_some());
    assert_eq!(s.accuracy(), 1.0);
    assert!(!s.timer_running(), "timer stops on completion");
}
