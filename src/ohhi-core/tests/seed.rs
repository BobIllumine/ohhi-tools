use ohhi_core::seed;
use ohhi_core::board::Cell;

#[test]
fn parse_4x4_partial() {
    let s = "R . B .\n. R . B\nB . R .\n. B . R";
    let board = seed::parse(s).expect("parse failed");
    assert_eq!(board.width(), 4);
    assert_eq!(board.height(), 4);
    assert_eq!(board.get((0, 0)), Cell::Red);
    assert_eq!(board.get((0, 2)), Cell::Blue);
    assert_eq!(board.get((0, 1)), Cell::Nothing);
}

#[test]
fn encode_then_parse_roundtrip() {
    let s = "R B R B\nB R B R\nR B R B\nB R B R";
    let board = seed::parse(s).expect("parse failed");
    let encoded = seed::encode(&board);
    let board2 = seed::parse(&encoded).expect("re-parse failed");
    for r in 0..4 {
        for c in 0..4 {
            assert_eq!(board.get((r, c)), board2.get((r, c)));
        }
    }
}

#[test]
fn parse_rejects_forbidden_character() {
    let s = "R X B\n. . .";
    assert!(seed::parse(s).is_err());
}

#[test]
fn parse_rejects_inconsistent_row_widths() {
    let s = "R B\nR B R";
    assert!(seed::parse(s).is_err());
}

#[test]
fn parse_empty_cells_are_nothing() {
    let s = ". . . .\n. . . .";
    let board = seed::parse(s).expect("parse failed");
    for r in 0..2 {
        for c in 0..4 {
            assert_eq!(board.get((r, c)), Cell::Nothing);
        }
    }
}
