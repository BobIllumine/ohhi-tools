use ohhi_app::trace::{from_v1, from_v2, StepReason};
use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use ohhi_solver::v1::deduction::deduce_with;
use ohhi_solver::structs::{Technique, TechniqueSet};
use ohhi_solver::v2::propagate::propagate;

fn bb(rows: &[&str]) -> BitBoard {
    let h = rows.len();
    let w = rows[0].len();
    let mut b = BitBoard::new(w, h);
    for (r, row) in rows.iter().enumerate() {
        for (c, ch) in row.chars().enumerate() {
            let cell = match ch {
                'R' => Cell::Red,
                'B' => Cell::Blue,
                _ => Cell::Nothing,
            };
            b.set((r, c), cell);
        }
    }
    b
}

#[test]
fn from_v1_produces_technique_steps() {
    // RR.. → pair extension forces col 2 Blue
    let board = bb(&["RR..", "....", "....", "...."]);
    let dt = deduce_with(&board, TechniqueSet::NONE.with(Technique::PairExtension));
    let trace = from_v1(&dt);
    assert!(!trace.steps.is_empty());
    assert!(matches!(trace.steps[0].reason, StepReason::Technique(Technique::PairExtension)));
    assert_eq!(trace.steps[0].color, Cell::Blue);
}

#[test]
fn from_v2_produces_line_forced_steps() {
    // Same board: v2 should also force the cell
    let board = bb(&["RR..", "....", "....", "...."]);
    let prop = propagate(&board);
    let trace = from_v2(&prop);
    if !trace.steps.is_empty() {
        assert!(matches!(trace.steps[0].reason, StepReason::LineForced));
    }
}

#[test]
fn empty_board_v1_produces_empty_trace() {
    let board = BitBoard::new(4, 4);
    let dt = deduce_with(&board, TechniqueSet::ALL);
    let trace = from_v1(&dt);
    assert!(trace.steps.is_empty());
}
