use ohhi_data::patterns::{all_patterns, nontrivial_patterns, PatternClass};
use ohhi_core::board::Cell;

#[test]
fn nontrivial_list_is_counting_only() {
    // Every mined atom must be a counting deduction — never local/saturation.
    let pats = nontrivial_patterns();
    assert!(!pats.is_empty(), "expected some counting atoms");
    assert!(pats.iter().all(|p| p.class == PatternClass::Counting));
}

#[test]
fn contains_the_double_anchor_counting_atom() {
    // The canonical `R....R → cols 1,4 Blue` double-anchor atom must be present:
    // two same-colour anchors at the ends of a 6-line forcing both inner... cells.
    let pats = nontrivial_patterns();
    let found = pats.iter().any(|p| {
        p.example.width() == 6
            && p.example.height() == 1
            && p.example.get((0, 0)) != Cell::Nothing
            && p.example.get((0, 5)) == p.example.get((0, 0))
            && p.forced.len() >= 1
    });
    assert!(found, "expected the R....R double-anchor counting atom");
}

#[test]
fn no_canonical_duplicates() {
    // Dedup keys (size-independent canonical shape) must be unique.
    use std::collections::HashSet;
    let pats = nontrivial_patterns();
    let mut keys: HashSet<(usize, usize, u16, u16, Vec<((usize, usize), Cell)>)> = HashSet::new();
    for p in &pats {
        // Reconstruct a comparable signature from the example + forced cells.
        let (w, h) = (p.example.width(), p.example.height());
        let mut red = 0u16;
        let mut blue = 0u16;
        for c in 0..w {
            match p.example.get((0, c)) {
                Cell::Red => red |= 1 << c,
                Cell::Blue => blue |= 1 << c,
                Cell::Nothing => {}
            }
        }
        let sig = (w, h, red, blue, p.forced.clone());
        assert!(keys.insert(sig), "duplicate pattern shape in list");
    }
}

#[test]
fn all_patterns_have_sequential_ids() {
    let all = all_patterns();
    for (i, p) in all.iter().enumerate() {
        assert_eq!(p.id, i, "ids must be sequential");
    }
    assert!(all.iter().all(|p| p.class == PatternClass::Counting));
}
