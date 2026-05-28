mod common;

use ohhi_core::validator::{Filter, Validator};

fn filter_only_rule_of_3() -> Filter {
    Filter { rule_of_3: true, rule_of_equity: false, rule_of_duplication: false, incomplete: false }
}
fn filter_only_equity() -> Filter {
    Filter { rule_of_3: false, rule_of_equity: true, rule_of_duplication: false, incomplete: false }
}
fn filter_only_duplication() -> Filter {
    Filter { rule_of_3: false, rule_of_equity: false, rule_of_duplication: true, incomplete: false }
}
fn filter_only_incomplete() -> Filter {
    Filter { rule_of_3: false, rule_of_equity: false, rule_of_duplication: false, incomplete: true }
}
fn filter_all() -> Filter {
    Filter { rule_of_3: true, rule_of_equity: true, rule_of_duplication: true, incomplete: true }
}

#[test]
fn valid_2x2_rb_br() {
    let board = common::bb(&["RB", "BR"]);
    assert_eq!(board.validate(&filter_all()), Ok(()));
}

#[test]
fn valid_2x2_br_rb() {
    let board = common::bb(&["BR", "RB"]);
    assert_eq!(board.validate(&filter_all()), Ok(()));
}

#[test]
fn invalid_equity_two_reds_in_row() {
    let board = common::bb(&["RR", "BB"]);
    let result = board.validate(&filter_only_equity());
    assert!(result.is_err());
    assert!(result.unwrap_err().rule_of_equity());
}

#[test]
fn invalid_horizontal_three_reds() {
    let board = common::bb(&["RRR.", "....", "....", "...."]);
    let result = board.validate(&filter_only_rule_of_3());
    assert!(result.is_err());
    assert!(result.unwrap_err().rule_of_3());
}

#[test]
fn invalid_vertical_three_reds() {
    // Regression: this goes through has_consecutive_y; previously the
    // column axis was checked using the row bitmask instead of the column.
    let board = common::bb(&["R...", "R...", "R...", "...."]);
    let result = board.validate(&filter_only_rule_of_3());
    assert!(result.is_err());
    assert!(result.unwrap_err().rule_of_3());
}

#[test]
fn valid_complete_board_no_duplicate_rows() {
    // RRBB / BBRR / RBBR / BRRB — all rows and cols distinct
    let board = common::bb(&["RRBB", "BBRR", "RBBR", "BRRB"]);
    assert_eq!(board.validate(&filter_only_duplication()), Ok(()));
}

#[test]
fn invalid_duplicate_complete_rows() {
    // Rows 0 and 2 are both RBRB
    let board = common::bb(&["RBRB", "BRBR", "RBRB", "BRBR"]);
    let result = board.validate(&filter_only_duplication());
    assert!(result.is_err());
    assert!(result.unwrap_err().rule_of_duplication());
}

#[test]
fn partial_lines_do_not_trip_uniqueness() {
    // Two incomplete rows share the same partial red mask — this must NOT
    // count as a uniqueness violation. Regression for the Phase A.2 bug.
    let board = common::bb(&["R...", "R...", "....", "...."]);
    assert_eq!(board.validate(&filter_only_duplication()), Ok(()));
}

#[test]
fn incomplete_flag_fires_on_partial_board() {
    let board = common::bb(&["R...", "....", "....", "...."]);
    let result = board.validate(&filter_only_incomplete());
    assert!(result.is_err());
    assert!(result.unwrap_err().incomplete());
}

#[test]
fn incomplete_off_passes_partial_board() {
    let board = common::bb(&["R...", "....", "....", "...."]);
    let filter = Filter { rule_of_3: false, rule_of_equity: false, rule_of_duplication: false, incomplete: false };
    assert_eq!(board.validate(&filter), Ok(()));
}

#[test]
fn disabling_rule_of_3_suppresses_only_that_violation() {
    // Board violates rule-of-3 (3 horizontal reds) AND equity
    let board = common::bb(&["RRR.", "....", "....", "...."]);
    let filter = Filter { rule_of_3: false, rule_of_equity: true, rule_of_duplication: false, incomplete: false };
    let result = board.validate(&filter);
    match result {
        Err(v) => {
            assert!(!v.rule_of_3(), "rule_of_3 should be suppressed");
            assert!(v.rule_of_equity(), "equity should still fire");
        }
        Ok(()) => panic!("expected equity violation"),
    }
}

#[test]
fn disabling_equity_suppresses_only_that_violation() {
    let board = common::bb(&["RRR.", "....", "....", "...."]);
    let filter = Filter { rule_of_3: true, rule_of_equity: false, rule_of_duplication: false, incomplete: false };
    let result = board.validate(&filter);
    match result {
        Err(v) => {
            assert!(v.rule_of_3(), "rule_of_3 should still fire");
            assert!(!v.rule_of_equity(), "equity should be suppressed");
        }
        Ok(()) => panic!("expected rule_of_3 violation"),
    }
}
