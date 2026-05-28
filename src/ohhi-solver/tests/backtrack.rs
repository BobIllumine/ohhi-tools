mod common;

use ohhi_core::bit_board::BitBoard;
use ohhi_solver::backtrack::{backtrack_one, calculate};

#[test]
fn count_empty_2x2_returns_2() {
    // Only two valid 2×2 boards exist: RB/BR and BR/RB.
    assert_eq!(calculate(&BitBoard::new(2, 2), usize::MAX), 2);
}

#[test]
fn count_empty_4x4_returns_known_value() {
    // 72 is the correct count for 4×4 0h h1 boards.
    // Verified by independent enumeration (see differential.rs).
    assert_eq!(calculate(&BitBoard::new(4, 4), usize::MAX), 72);
}

#[test]
fn count_cap_stops_early() {
    // With cap=2 on a 4×4 board, the solver must return exactly 2.
    let result = calculate(&BitBoard::new(4, 4), 2);
    assert_eq!(result, 2);
}

#[test]
fn count_on_unique_seed_returns_1() {
    // RRBB/BBRR/RBBR/BRR. has exactly one completion: the B at (3,3).
    let seed = common::bb(&["RRBB", "BBRR", "RBBR", "BRR."]);
    assert_eq!(calculate(&seed, 2), 1);
}

#[test]
fn count_on_ambiguous_seed_returns_at_least_2() {
    // Empty 4×4 has 72 solutions, so cap=2 must return 2.
    assert_eq!(calculate(&BitBoard::new(4, 4), 2), 2);
}

#[test]
fn backtrack_one_returns_solution_for_unique_seed() {
    let seed = common::bb(&["RRBB", "BBRR", "RBBR", "BRR."]);
    let expected = common::bb(&["RRBB", "BBRR", "RBBR", "BRRB"]);
    assert_eq!(backtrack_one(&seed), Some(expected));
}

#[test]
fn backtrack_one_returns_none_for_impossible_seed() {
    // Three reds in a row is immediately illegal — the solver prunes and
    // returns None. This should pass even with the missing base case bug.
    let seed = common::bb(&["RRR.", "....", "....", "...."]);
    assert_eq!(backtrack_one(&seed), None);
}
