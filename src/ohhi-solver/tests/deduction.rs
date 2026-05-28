mod common;

use ohhi_core::board::Cell;
use ohhi_solver::deduction::{deduce, deduce_with, Technique, TechniqueSet};

// ── PairExtension ─────────────────────────────────────────────────────────────

#[test]
fn pair_ext_xx_dot_forces_blue_to_right() {
    // RR.. — pair at cols 0,1 forces col 2 to Blue
    let board = common::bb(&["RR..", "....", "....", "...."]);
    let trace = deduce_with(&board, TechniqueSet::NONE.with(Technique::PairExtension));
    let step = &trace.get_steps()[0];
    assert_eq!(step.at, (0, 2));
    assert_eq!(step.cell, Cell::Blue);
    assert_eq!(step.technique, Technique::PairExtension);
}

#[test]
fn pair_ext_dot_xx_forces_blue_to_left() {
    // .RR. — pair at cols 1,2 forces col 0 to Blue
    let board = common::bb(&[".RR.", "....", "....", "...."]);
    let trace = deduce_with(&board, TechniqueSet::NONE.with(Technique::PairExtension));
    let step = &trace.get_steps()[0];
    assert_eq!(step.at, (0, 0));
    assert_eq!(step.cell, Cell::Blue);
    assert_eq!(step.technique, Technique::PairExtension);
}

#[test]
fn pair_ext_vertical() {
    // Column 0 has reds at rows 0 and 1; col-scan forces row 2 to Blue
    let board = common::bb(&["R...", "R...", "....", "...."]);
    let trace = deduce_with(&board, TechniqueSet::NONE.with(Technique::PairExtension));
    let step = &trace.get_steps()[0];
    assert_eq!(step.at, (2, 0));
    assert_eq!(step.cell, Cell::Blue);
    assert_eq!(step.technique, Technique::PairExtension);
}

// ── GapFill ───────────────────────────────────────────────────────────────────

#[test]
fn gap_fill_x_dot_x_forces_middle() {
    // R.R. — gap at col 1 forced Blue
    let board = common::bb(&["R.R.", "....", "....", "...."]);
    let trace = deduce_with(&board, TechniqueSet::NONE.with(Technique::GapFill));
    let step = &trace.get_steps()[0];
    assert_eq!(step.at, (0, 1));
    assert_eq!(step.cell, Cell::Blue);
    assert_eq!(step.technique, Technique::GapFill);
}

#[test]
fn gap_fill_vertical() {
    // Col 0 has reds at rows 0 and 2; row 1 is the gap
    let board = common::bb(&["R...", "....", "R...", "...."]);
    let trace = deduce_with(&board, TechniqueSet::NONE.with(Technique::GapFill));
    let step = &trace.get_steps()[0];
    assert_eq!(step.at, (1, 0));
    assert_eq!(step.cell, Cell::Blue);
    assert_eq!(step.technique, Technique::GapFill);
}

// ── Saturation ────────────────────────────────────────────────────────────────

#[test]
fn saturation_half_reds_forces_remaining_blue_in_row() {
    // 4-wide row with 2 reds (= width/2) → remaining 2 cells forced Blue
    let board = common::bb(&["RR..", "....", "....", "...."]);
    let trace = deduce_with(&board, TechniqueSet::NONE.with(Technique::Saturation));
    let sat_steps: Vec<_> = trace.get_steps().iter()
        .filter(|s| s.technique == Technique::Saturation)
        .collect();
    assert_eq!(sat_steps.len(), 2, "expected 2 saturation steps");
    for step in &sat_steps {
        assert_eq!(step.cell, Cell::Blue);
        assert_eq!(step.at.0, 0);
    }
}

#[test]
fn saturation_col_uses_height_threshold() {
    // 4-tall col with 2 reds (= height/2) → remaining 2 cells forced Blue
    let board = common::bb(&["R...", "R...", "....", "...."]);
    let trace = deduce_with(&board, TechniqueSet::NONE.with(Technique::Saturation));
    let sat_steps: Vec<_> = trace.get_steps().iter()
        .filter(|s| s.technique == Technique::Saturation && s.at.1 == 0)
        .collect();
    assert_eq!(sat_steps.len(), 2, "expected 2 saturation steps in col 0");
    for step in &sat_steps {
        assert_eq!(step.cell, Cell::Blue);
    }
}

// ── TwinCompletion ────────────────────────────────────────────────────────────

#[test]
fn twin_completion_row() {
    // Row 0 will be completed by PairExt+Saturation to RRBB (sig=0b0011).
    // Row 1 (.R.B) has 2 empties; filling e0=Red would duplicate row 0,
    // so col 0 of row 1 is forced Blue via TwinCompletion.
    let board = common::bb(&["RR..", ".R.B", "....", "...."]);
    let trace = deduce_with(&board, TechniqueSet::ALL);
    let twin_step = trace.get_steps().iter()
        .find(|s| s.technique == Technique::TwinCompletion);
    assert!(twin_step.is_some(), "expected a TwinCompletion step");
    let step = twin_step.unwrap();
    assert_eq!(step.at, (1, 0));
    assert_eq!(step.cell, Cell::Blue);
}

#[test]
fn twin_completion_col() {
    // Col 0 will be completed by PairExt+Saturation to RRBB (sig=0b0011).
    // Col 1 (.R.B vertically) has 2 empties; one completion would
    // duplicate col 0, so row 0 of col 1 is forced Blue via TwinCompletion.
    let board = common::bb(&["R...", "RR..", "....", ".B.."]);
    let trace = deduce_with(&board, TechniqueSet::ALL);
    let twin_step = trace.get_steps().iter()
        .find(|s| s.technique == Technique::TwinCompletion);
    assert!(twin_step.is_some(), "expected a TwinCompletion step");
    let step = twin_step.unwrap();
    assert_eq!(step.at, (0, 1));
    assert_eq!(step.cell, Cell::Blue);
}

// ── Engine behaviour ──────────────────────────────────────────────────────────

#[test]
fn engine_stalls_on_contradictory_board() {
    // RRR. violates rule-of-3; PairExtension fires and tries to place Blue
    // at col 3, but SolverState::place immediately rejects it.
    let board = common::bb(&["RRR.", "....", "....", "...."]);
    let trace = deduce(&board);
    assert!(trace.stalled);
    assert_eq!(trace.get_steps().len(), 0);
}

#[test]
fn engine_disabled_techniques_produce_fewer_deductions() {
    let board = common::bb(&["R...", "RR..", "....", ".B.."]);
    let trace_all = deduce_with(&board, TechniqueSet::ALL);
    let trace_no_twin = deduce_with(&board, TechniqueSet::ALL.without(Technique::TwinCompletion));
    assert!(trace_all.get_steps().len() > trace_no_twin.get_steps().len());
}

#[test]
fn engine_full_solve_unique_seed() {
    // RRBB/BBRR/RBBR/BRR. — one empty cell.
    // Row 3 has RR at positions 1-2; PairExtension fires first and forces (3,3)=Blue.
    let seed = common::bb(&["RRBB", "BBRR", "RBBR", "BRR."]);
    let expected = common::bb(&["RRBB", "BBRR", "RBBR", "BRRB"]);
    let trace = deduce(&seed);
    assert!(!trace.stalled);
    assert_eq!(trace.get_steps().len(), 1);
    let step = &trace.get_steps()[0];
    assert_eq!(step.at, (3, 3));
    assert_eq!(step.cell, Cell::Blue);
    assert_eq!(step.technique, Technique::PairExtension);
    assert_eq!(step.board_after, expected);
}
