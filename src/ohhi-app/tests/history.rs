use ohhi_app::history::{summaries, trend, DrillRecord};
use ohhi_app::{AppState, Screen};
use ohhi_generator::practice::{target_forced, Target};
use ohhi_app::practice::PracticeSession;

fn rec(target: Target, times: Vec<u64>, hits: usize, misses: usize, epoch: u64) -> DrillRecord {
    DrillRecord { target, n: 6, total_reps: hits, times, hits, misses, epoch_ms: epoch }
}

#[test]
fn record_stats_math() {
    let d = rec(Target::Saturation, vec![100, 300, 200], 3, 1, 0);
    assert_eq!(d.mean_ms(), 200);
    assert_eq!(d.best_ms(), 100);
    assert!((d.accuracy() - 0.75).abs() < 1e-9);
}

#[test]
fn summaries_group_and_rank_by_drill_count() {
    let hist = vec![
        rec(Target::Saturation, vec![200], 1, 0, 1),
        rec(Target::Saturation, vec![100], 1, 0, 2),
        rec(Target::GapFill, vec![500], 1, 1, 3),
    ];
    let s = summaries(&hist);
    assert_eq!(s[0].target, Target::Saturation); // most drills first
    assert_eq!(s[0].drills, 2);
    assert_eq!(s[0].best_mean_ms, 100);
    assert_eq!(s[0].avg_mean_ms, 150);
    assert_eq!(s[1].target, Target::GapFill);
}

#[test]
fn trend_filters_and_orders_by_target() {
    let hist = vec![
        rec(Target::Saturation, vec![300], 1, 0, 1),
        rec(Target::GapFill, vec![400], 1, 0, 2),
        rec(Target::Saturation, vec![200], 1, 0, 3),
    ];
    let t = trend(&hist, Target::Saturation);
    assert_eq!(t.len(), 2);
    assert_eq!(t[0].mean_ms(), 300);
    assert_eq!(t[1].mean_ms(), 200);
}

#[test]
fn drill_is_recorded_once_into_app_history() {
    let mut app = AppState::new();
    app.screen = Screen::Practice;
    app.practice = Some(PracticeSession::new(Target::Saturation, 6, 2, 11).unwrap());

    // Drive the drill to completion.
    let mut guard = 0;
    while !app.practice.as_ref().unwrap().is_complete() && guard < 1000 {
        guard += 1;
        let board = app.practice.as_ref().unwrap().board().clone();
        let ((r, c), color) = target_forced(&board, Target::Saturation)[0];
        app.practice.as_mut().unwrap().tick(50);
        app.practice.as_mut().unwrap().attempt((r, c), color);
    }

    assert!(app.record_drill_if_done(1234).is_some());
    assert_eq!(app.practice_history.len(), 1);
    // Idempotent: a second call doesn't double-log.
    assert!(app.record_drill_if_done(1234).is_none());
    assert_eq!(app.practice_history.len(), 1);
    assert_eq!(app.practice_history[0].epoch_ms, 1234);
    assert_eq!(app.practice_history[0].hits, 2);
}
