mod common;

use ohhi_core::stats::NumTransforms;

#[test]
fn signature_x_returns_red_mask() {
    // "RB.B": Red at col 0 only → bit 0 set = 0b0001
    let board = common::bb(&["RB.B"]);
    assert_eq!(board.signature_x(&0), 0b0001);
}

#[test]
fn signature_y_returns_red_mask() {
    // Col 0: R, B, R, B → reds at rows 0 and 2 → bits 0 and 2 = 0b0101
    let board = common::bb(&["R.", "B.", "R.", "B."]);
    assert_eq!(board.signature_y(&0), 0b0101);
}

#[test]
fn count_x_returns_red_blue_counts() {
    // "RBB.": 1 red, 2 blue
    let board = common::bb(&["RBB."]);
    assert_eq!(board.count_x(&0), (1, 2));
}

#[test]
fn count_y_returns_red_blue_counts() {
    // Col 0: R, B, B, . → 1 red, 2 blue
    let board = common::bb(&["R.", "B.", "B.", ".."]);
    assert_eq!(board.count_y(&0), (1, 2));
}

#[test]
fn has_consecutive_x_finds_run_of_3() {
    assert!(common::bb(&["RRR."]).has_consecutive_x(&0, 3));
    assert!(!common::bb(&["RR.."]).has_consecutive_x(&0, 3));
}

#[test]
fn has_consecutive_x_finds_run_of_2() {
    assert!(common::bb(&["RR.."]).has_consecutive_x(&0, 2));
    assert!(!common::bb(&["R.R."]).has_consecutive_x(&0, 2));
}

#[test]
fn has_consecutive_y_finds_vertical_run_of_3() {
    // Regression test: this codepath historically used get_row instead
    // of get_col, so it would silently check the wrong axis.
    let board = common::bb(&["R", "R", "R", "."]);
    assert!(board.has_consecutive_y(&0, 3));
    let board = common::bb(&["R", "R", ".", "."]);
    assert!(!board.has_consecutive_y(&0, 3));
}

#[test]
fn is_complete_x_only_when_full() {
    assert!(common::bb(&["RBRB"]).is_complete_x(&0));
    assert!(!common::bb(&["RB.B"]).is_complete_x(&0));
}

#[test]
fn is_complete_y_only_when_full() {
    let board = common::bb(&["R", "B", "R", "B"]);
    assert!(board.is_complete_y(&0));
    let board = common::bb(&["R", "B", ".", "B"]);
    assert!(!board.is_complete_y(&0));
}
