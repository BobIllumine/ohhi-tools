//! Budget-probe regression tests — these pin the line-atom findings that
//! motivated the experiments: `R....R` forces its inner neighbours Blue iff red
//! is the scarce, near-exhausted colour, independent of parity.

use ohhi_core::board::Cell;
use ohhi_data::probe::{forced_line, forced_under, parse_line};

fn blues(cells: &[(usize, Cell)]) -> Vec<usize> {
    cells.iter().filter(|&&(_, c)| c == Cell::Blue).map(|&(p, _)| p).collect()
}

#[test]
fn parse_line_roundtrips_masks() {
    let (red, blue, n) = parse_line("R . . . . R");
    assert_eq!(n, 6);
    assert_eq!(red, 0b100001);
    assert_eq!(blue, 0);
}

#[test]
fn r_dot_r_fires_only_when_red_is_scarce() {
    // n=6 balanced: red scarce (3 of 6, 2 already placed) → forces 1,4.
    assert_eq!(blues(&forced_line("R....R", 3).unwrap()), vec![1, 4]);

    // n=8 balanced: red has slack (4 of 8) → forces nothing.
    assert_eq!(forced_line("R....R..", 4).unwrap(), vec![]);

    // n=9, red minority (4R/5B) → forces 1,4 again, same as n=6.
    assert_eq!(blues(&forced_line("R....R...", 4).unwrap()), vec![1, 4]);

    // n=9, red majority (5R/4B) → nothing.
    assert_eq!(forced_line("R....R...", 5).unwrap(), vec![]);
}

#[test]
fn odd_length_corner_forces_trailing_cell_too() {
    // n=7, 3 red / 4 blue: the scarce-red exhaustion reaches the trailing cell.
    assert_eq!(blues(&forced_line("R....R.", 3).unwrap()), vec![1, 4, 6]);
}

#[test]
fn contradictory_budget_returns_none() {
    // n=4 but demanding 4 reds → RRRR, an immediate triple → no completion.
    assert_eq!(forced_under(0, 0, 4, 4), None);
}

#[test]
fn half_budget_agrees_with_balanced_catalog_atom() {
    // Sanity: the embedded atom the user cited still reads out at n/2.
    assert_eq!(blues(&forced_line("R...R....R", 5).unwrap()), vec![5, 8]);
}
