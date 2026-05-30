use ohhi_app::timer::Timer;

#[test]
fn tick_only_while_running() {
    let mut t = Timer::new();
    t.tick(100);
    assert_eq!(t.elapsed_ms, 0);
    t.start();
    t.tick(50);
    assert_eq!(t.elapsed_ms, 50);
    t.stop();
    t.tick(999);
    assert_eq!(t.elapsed_ms, 50);
}

#[test]
fn reset_clears_elapsed_and_stops() {
    let mut t = Timer::new();
    t.start();
    t.tick(200);
    t.reset();
    assert_eq!(t.elapsed_ms, 0);
    assert!(!t.running);
    t.tick(100);
    assert_eq!(t.elapsed_ms, 0);
}

#[test]
fn start_then_stop_then_start_accumulates() {
    let mut t = Timer::new();
    t.start();
    t.tick(100);
    t.stop();
    t.tick(50);
    t.start();
    t.tick(30);
    assert_eq!(t.elapsed_ms, 130);
}
