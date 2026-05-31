//! Integration tests for the pattern catalog and minimal trigger mining.

use ohhi_data::catalog::{catalog, minimal_triggers, LineState, RuleClass};

#[test]
fn catalog_n4_includes_pair_extension() {
    // "R R . ." forces (2, Blue) — the canonical pair-extension state must appear.
    let states = catalog(4);
    let rr_dot_dot = LineState { n: 4, red: 0b0011, blue: 0 };
    let entry = states.iter().find(|e| e.state == rr_dot_dot);
    assert!(entry.is_some(), "RR.. not in n=4 catalog");
    let entry = entry.unwrap();
    assert!(
        entry.forced.iter().any(|&(pos, col)| pos == 2 && col == ohhi_core::board::Cell::Blue),
        "RR.. should force col 2 Blue, got {:?}", entry.forced
    );
}

#[test]
fn catalog_n4_includes_gap_fill() {
    // "R . R ." forces (1, Blue).
    let states = catalog(4);
    let r_dot_r_dot = LineState { n: 4, red: 0b0101, blue: 0 };
    let entry = states.iter().find(|e| e.state == r_dot_r_dot);
    assert!(entry.is_some(), "R.R. not in n=4 catalog");
    let entry = entry.unwrap();
    assert!(
        entry.forced.iter().any(|&(pos, col)| pos == 1 && col == ohhi_core::board::Cell::Blue),
        "R.R. should force col 1 Blue, got {:?}", entry.forced
    );
}

#[test]
fn catalog_only_contains_forcing_states() {
    // Every entry in the catalog must have at least one forced cell.
    for entry in catalog(4) {
        assert!(!entry.forced.is_empty(), "non-forcing state in catalog: {:?}", entry.state);
    }
}

#[test]
fn minimal_set_deduplicates_color_swap_symmetry() {
    // "R R . ." and "B B . ." are color-swaps of each other; the minimal set
    // collapses them to one canonical representative.
    let minimal = minimal_triggers(4);
    let rr = LineState { n: 4, red: 0b0011, blue: 0 };
    let bb = LineState { n: 4, red: 0, blue: 0b0011 };
    let has_rr = minimal.iter().any(|e| e.state == rr);
    let has_bb = minimal.iter().any(|e| e.state == bb);
    assert!(
        has_rr ^ has_bb,
        "both color-swap twins present in minimal set (only one should survive)"
    );
}

#[test]
fn minimal_set_deduplicates_reflection_symmetry() {
    // ". . R R" is the reflection of "R R . ."; only one should appear.
    let minimal = minimal_triggers(4);
    let rr_left  = LineState { n: 4, red: 0b0011, blue: 0 }; // cols 0,1
    let rr_right = LineState { n: 4, red: 0b1100, blue: 0 }; // cols 2,3
    let has_left  = minimal.iter().any(|e| e.state == rr_left);
    let has_right = minimal.iter().any(|e| e.state == rr_right);
    assert!(
        has_left ^ has_right,
        "both reflections present in minimal set (only one should survive)"
    );
}

#[test]
fn minimal_excludes_redundant_clues() {
    // "R R . B" forces col 2 Blue (pair-extension + equity: already 2 blues,
    // remaining must all be red... wait, 0 reds + 2 fixed means checking the state).
    // More concretely: "R R R ." is not valid (triple), so it won't appear.
    // The minimality test: if "R R . ." forces col 2, then "R R . B" also forces
    // col 2 (and more), but the B at col 3 is redundant for *that* deduction, so
    // "R R . B" should NOT appear in the minimal set if "R R . ." already covers it.
    let minimal = minimal_triggers(4);
    let rr_dot_b = LineState { n: 4, red: 0b0011, blue: 0b1000 }; // RR.B at cols 0,1,3
    assert!(
        !minimal.iter().any(|e| e.state == rr_dot_b),
        "redundant state RR.B should not be in minimal set (RR.. already forces the same cell)"
    );
}

#[test]
fn catalog_rule_classes_cover_all_known_techniques() {
    let states = catalog(4);
    let has_local_anti_triple = states.iter().any(|e| e.rule_class == RuleClass::LocalAntiTriple);
    let has_saturation = states.iter().any(|e| e.rule_class == RuleClass::Saturation);
    assert!(has_local_anti_triple, "no LocalAntiTriple entries in n=4 catalog");
    assert!(has_saturation, "no Saturation entries in n=4 catalog");
}
