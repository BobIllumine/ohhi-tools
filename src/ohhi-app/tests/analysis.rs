use ohhi_app::analysis::{AnalysisSession, AnalysisAction, Engine, Constructor, Reducer, apply};
use ohhi_core::board::Cell;
use ohhi_solver::structs::{Technique, TechniqueSet};

fn session() -> AnalysisSession {
    AnalysisSession::new(4, 4)
}

#[test]
fn cycle_cell_changes_cell_and_pushes_history() {
    let mut s = session();
    assert_eq!(s.board.get((0, 0)), Cell::Nothing);
    apply(&mut s, AnalysisAction::CycleCell(0, 0)).unwrap();
    assert_eq!(s.board.get((0, 0)), Cell::Red);
    assert_eq!(s.history.len(), 1);
}

#[test]
fn set_cell_stores_value() {
    let mut s = session();
    apply(&mut s, AnalysisAction::SetCell(1, 2, Cell::Blue)).unwrap();
    assert_eq!(s.board.get((1, 2)), Cell::Blue);
}

#[test]
fn undo_restores_previous_board() {
    let mut s = session();
    apply(&mut s, AnalysisAction::SetCell(0, 0, Cell::Red)).unwrap();
    apply(&mut s, AnalysisAction::Undo).unwrap();
    assert_eq!(s.board.get((0, 0)), Cell::Nothing);
}

#[test]
fn redo_reapplies_action() {
    let mut s = session();
    apply(&mut s, AnalysisAction::SetCell(0, 0, Cell::Red)).unwrap();
    apply(&mut s, AnalysisAction::Undo).unwrap();
    apply(&mut s, AnalysisAction::Redo).unwrap();
    assert_eq!(s.board.get((0, 0)), Cell::Red);
}

#[test]
fn clear_board_empties_board() {
    let mut s = session();
    apply(&mut s, AnalysisAction::SetCell(0, 0, Cell::Red)).unwrap();
    apply(&mut s, AnalysisAction::ClearBoard).unwrap();
    assert_eq!(s.board.get((0, 0)), Cell::Nothing);
    assert_eq!(s.history.len(), 2);
}

#[test]
fn resize_changes_dimensions() {
    let mut s = session();
    apply(&mut s, AnalysisAction::Resize(6, 6)).unwrap();
    assert_eq!(s.board.width(), 6);
    assert_eq!(s.board.height(), 6);
}

#[test]
fn load_seed_parses_and_sets_board() {
    let mut s = session();
    apply(&mut s, AnalysisAction::LoadSeed("R B R B\nB R B R\nR B R B\nB R B R".to_string())).unwrap();
    assert_eq!(s.board.get((0, 0)), Cell::Red);
    assert_eq!(s.board.get((0, 1)), Cell::Blue);
}

#[test]
fn load_seed_invalid_returns_error() {
    let mut s = session();
    let history_len = s.history.len();
    let result = apply(&mut s, AnalysisAction::LoadSeed("X X X".to_string()));
    assert!(result.is_err());
    assert_eq!(s.history.len(), history_len);
}

#[test]
fn deduce_with_v1_fills_overlay() {
    let mut s = session();
    // Set up a board with a forced cell via pair extension: RR.. in row 0
    apply(&mut s, AnalysisAction::SetCell(0, 0, Cell::Red)).unwrap();
    apply(&mut s, AnalysisAction::SetCell(0, 1, Cell::Red)).unwrap();
    s.engine = Engine::V1(TechniqueSet::NONE.with(Technique::PairExtension));
    apply(&mut s, AnalysisAction::Deduce).unwrap();
    assert!(s.overlay.is_some());
    let trace = s.overlay.as_ref().unwrap();
    assert!(!trace.steps.is_empty());
}

#[test]
fn deduce_with_v2_fills_overlay() {
    let mut s = session();
    apply(&mut s, AnalysisAction::SetCell(0, 0, Cell::Red)).unwrap();
    apply(&mut s, AnalysisAction::SetCell(0, 1, Cell::Red)).unwrap();
    s.engine = Engine::V2;
    apply(&mut s, AnalysisAction::Deduce).unwrap();
    assert!(s.overlay.is_some());
}

#[test]
fn clear_overlay_removes_overlay() {
    let mut s = session();
    apply(&mut s, AnalysisAction::SetCell(0, 0, Cell::Red)).unwrap();
    apply(&mut s, AnalysisAction::SetCell(0, 1, Cell::Red)).unwrap();
    s.engine = Engine::V1(TechniqueSet::ALL);
    apply(&mut s, AnalysisAction::Deduce).unwrap();
    apply(&mut s, AnalysisAction::ClearOverlay).unwrap();
    assert!(s.overlay.is_none());
    assert_eq!(s.overlay_step, 0);
}

#[test]
fn generate_produces_valid_sized_board() {
    let mut s = session();
    s.gen_constructor = Constructor::Og;
    s.gen_reducer = Reducer::Breakdown;
    apply(&mut s, AnalysisAction::Generate { n: 4, seed: Some(42) }).unwrap();
    assert_eq!(s.board.width(), 4);
    assert_eq!(s.board.height(), 4);
    assert!(s.last_solve.is_some());
    assert!(s.last_quality.is_some());
}

#[test]
fn generate_rejects_odd_n() {
    let mut s = session();
    let result = apply(&mut s, AnalysisAction::Generate { n: 3, seed: None });
    assert!(result.is_err());
}
