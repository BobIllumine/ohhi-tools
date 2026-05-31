//! Locks the invariance findings for the mined counting atoms (experiment #3).
//!
//! Two headline results:
//!  - atoms are **length-rigid** at balanced budget: they force only at their
//!    native `n` (padding to a longer balanced line breaks them), but
//!  - they re-fire at longer `n` once the red budget is dropped to the
//!    density floor — and most are then **position-free** (slide anywhere),
//!    while a few asymmetric ones are **position-dependent**.

use ohhi_core::board::Cell;
use ohhi_data::probe::window_survives;

const B: Cell = Cell::Blue;

/// Does the window fire (native forced cells preserved) at *some* budget when
/// embedded at `off` in length `n`?
fn fires_somewhere(red: u16, blue: u16, w: usize, forced: &[(usize, Cell)], off: usize, n: usize) -> bool {
    (0..=n).any(|tr| window_survives(red, blue, w, forced, off, n, tr))
}

// The R....R atom: reds at 0 and 5, inner cells 1 & 4 forced Blue.
const RDOTR: (u16, u16, usize) = (0b100001, 0, 6);
const RDOTR_FORCED: [(usize, Cell); 2] = [(1, B), (4, B)];

#[test]
fn atom_is_length_rigid_at_balanced_budget() {
    let (r, b, w) = RDOTR;
    // Native n=6 (3 reds): fires.
    assert!(window_survives(r, b, w, &RDOTR_FORCED, 0, 6, 3));
    // Padded to a longer *balanced* line: broken at every larger even n.
    for n in [8usize, 10, 12, 14] {
        assert!(
            !window_survives(r, b, w, &RDOTR_FORCED, 0, n, n / 2),
            "R....R unexpectedly survived balanced n={n}"
        );
    }
}

#[test]
fn atom_refires_at_density_floor() {
    let (r, b, w) = RDOTR;
    // Dropping to the scarce-red density floor re-fires it (corner).
    assert!(window_survives(r, b, w, &RDOTR_FORCED, 0, 12, 5)); // ⌊12/3⌋+1 = 5
    assert!(window_survives(r, b, w, &RDOTR_FORCED, 0, 14, 5));
}

#[test]
fn symmetric_atom_is_position_free() {
    // R....R fires at every offset in a roomy line (at its own firing budget).
    let (r, b, w) = RDOTR;
    let n = 12;
    for off in 0..=(n - w) {
        assert!(
            fires_somewhere(r, b, w, &RDOTR_FORCED, off, n),
            "R....R failed to fire at offset {off}"
        );
    }
}

#[test]
fn asymmetric_atom_is_position_dependent() {
    // R..B..R. (reds 0,6; blue 3; trailing empty) forces only its col-7 cell —
    // and that forcing depends on edge proximity, so the middle offset dies.
    let (r, b, w) = (0b1000001u16, 0b0001000u16, 8usize);
    let forced = [(7usize, B)];
    let n = 12;
    let fires: Vec<usize> = (0..=(n - w))
        .filter(|&off| fires_somewhere(r, b, w, &forced, off, n))
        .collect();
    assert!(fires.contains(&0), "should fire at the corner");
    assert!(
        fires.len() < (n - w + 1),
        "expected a dead offset (position-dependent), got all {fires:?}"
    );
}
