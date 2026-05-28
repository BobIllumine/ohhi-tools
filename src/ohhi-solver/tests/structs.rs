mod common;

use ohhi_core::board::Cell;
use ohhi_solver::structs::{SolverState, Technique, TechniqueSet};

// ── TechniqueSet ─────────────────────────────────────────────────────────────

#[test]
fn all_contains_every_variant() {
    assert!(TechniqueSet::ALL.contains(Technique::PairExtension));
    assert!(TechniqueSet::ALL.contains(Technique::GapFill));
    assert!(TechniqueSet::ALL.contains(Technique::Saturation));
    assert!(TechniqueSet::ALL.contains(Technique::TwinCompletion));
}

#[test]
fn none_contains_nothing() {
    assert!(!TechniqueSet::NONE.contains(Technique::PairExtension));
    assert!(!TechniqueSet::NONE.contains(Technique::GapFill));
    assert!(!TechniqueSet::NONE.contains(Technique::Saturation));
    assert!(!TechniqueSet::NONE.contains(Technique::TwinCompletion));
}

#[test]
fn with_then_contains() {
    let set = TechniqueSet::NONE.with(Technique::GapFill);
    assert!(set.contains(Technique::GapFill));
    assert!(!set.contains(Technique::PairExtension));
}

#[test]
fn without_then_not_contains() {
    let set = TechniqueSet::ALL.without(Technique::TwinCompletion);
    assert!(!set.contains(Technique::TwinCompletion));
    assert!(set.contains(Technique::PairExtension));
    assert!(set.contains(Technique::GapFill));
    assert!(set.contains(Technique::Saturation));
}

#[test]
fn toggle_is_involution() {
    for &t in &[Technique::PairExtension, Technique::GapFill, Technique::Saturation, Technique::TwinCompletion] {
        let set = TechniqueSet::ALL;
        assert_eq!(set.toggle(t).toggle(t).contains(t), set.contains(t));
        let set = TechniqueSet::NONE;
        assert_eq!(set.toggle(t).toggle(t).contains(t), set.contains(t));
    }
}

#[test]
fn with_is_idempotent() {
    for &t in &[Technique::PairExtension, Technique::GapFill, Technique::Saturation, Technique::TwinCompletion] {
        let set = TechniqueSet::NONE.with(t).with(t);
        assert!(set.contains(t));
    }
}

#[test]
fn without_only_affects_target() {
    let all = [Technique::PairExtension, Technique::GapFill, Technique::Saturation, Technique::TwinCompletion];
    for &a in &all {
        let set = TechniqueSet::ALL.without(a);
        for &b in &all {
            if a != b {
                assert!(set.contains(b), "{:?} should still be in set after removing {:?}", b, a);
            }
        }
    }
}

// ── SolverState ──────────────────────────────────────────────────────────────

#[test]
fn place_succeeds_on_first_cell() {
    let board = common::bb(&["....", "....", "....", "...."]);
    let mut state = SolverState::new(&board);
    assert!(state.place(Cell::Red, (0, 0)));
    assert_eq!(state.board_ref().get((0, 0)), Cell::Red);
}

#[test]
fn place_rejects_third_consecutive_horizontal() {
    let board = common::bb(&["RR..", "....", "....", "...."]);
    let mut state = SolverState::new(&board);
    state.place(Cell::Red, (0, 0));
    state.place(Cell::Red, (0, 1));
    assert!(!state.place(Cell::Red, (0, 2)));
}

#[test]
fn place_rejects_third_consecutive_vertical() {
    let board = common::bb(&["....", "....", "....", "...."]);
    let mut state = SolverState::new(&board);
    state.place(Cell::Red, (0, 0));
    state.place(Cell::Red, (1, 0));
    assert!(!state.place(Cell::Red, (2, 0)));
}

#[test]
fn place_rejects_equity_overflow_row() {
    let board = common::bb(&["....", "....", "....", "...."]);
    let mut state = SolverState::new(&board);
    state.place(Cell::Red, (0, 0));
    state.place(Cell::Red, (0, 1));
    assert!(!state.place(Cell::Red, (0, 3)));
}

#[test]
fn place_rejects_equity_overflow_col() {
    let board = common::bb(&["....", "....", "....", "...."]);
    let mut state = SolverState::new(&board);
    state.place(Cell::Red, (0, 0));
    state.place(Cell::Red, (1, 0));
    assert!(!state.place(Cell::Red, (3, 0)));
}

#[test]
fn place_rejects_duplicate_complete_row() {
    let board = common::bb(&["....", "....", "....", "...."]);
    let mut state = SolverState::new(&board);
    // Row 0: R R B B (sig = 0b0011)
    assert!(state.place(Cell::Red, (0, 0)));
    assert!(state.place(Cell::Red, (0, 1)));
    assert!(state.place(Cell::Blue, (0, 2)));
    assert!(state.place(Cell::Blue, (0, 3)));
    // Row 1: R R B B — same signature, must be rejected on the final cell
    assert!(state.place(Cell::Red, (1, 0)));
    assert!(state.place(Cell::Red, (1, 1)));
    assert!(state.place(Cell::Blue, (1, 2)));
    assert!(!state.place(Cell::Blue, (1, 3)));
}

#[test]
fn unplace_restores_cell_to_nothing() {
    let board = common::bb(&["....", "....", "....", "...."]);
    let mut state = SolverState::new(&board);
    state.place(Cell::Red, (0, 0));
    state.unplace((0, 0));
    assert_eq!(state.board_ref().get((0, 0)), Cell::Nothing);
}

#[test]
fn unplace_removes_completed_row_signature() {
    let board = common::bb(&["....", "....", "....", "...."]);
    let mut state = SolverState::new(&board);
    // Complete row 0 as RRBB
    state.place(Cell::Red, (0, 0));
    state.place(Cell::Red, (0, 1));
    state.place(Cell::Blue, (0, 2));
    state.place(Cell::Blue, (0, 3));
    // Remove one cell → row incomplete, signature removed from set
    state.unplace((0, 3));
    // Now we should be able to complete row 1 with the same RRBB signature
    state.place(Cell::Red, (1, 0));
    state.place(Cell::Red, (1, 1));
    state.place(Cell::Blue, (1, 2));
    // Re-complete row 0
    assert!(state.place(Cell::Blue, (0, 3)));
    // Row 0 is RRBB again; now row 1 final cell should be rejected (dup)
    assert!(!state.place(Cell::Blue, (1, 3)));
}
