//! Experiment #3: translation / length invariance scan over the mined counting
//! atoms.
//!
//! For each atom we trim it to its active window and ask:
//!  - **Length scan** (corner, balanced budget n/2): at which even line lengths
//!    does the atom still force its native cells?
//!  - **Translation scan** (native length, balanced budget): at which offsets
//!    does it survive?
//!  - **Budget rescue**: when balanced n/2 fails, at which red budget *does* it
//!    fire? (Expected: the density floor ≈ n/3, per experiment #2.)
//!
//! Usage: cargo run -p ohhi-data --bin invariance

use ohhi_core::board::Cell;
use ohhi_data::patterns::nontrivial_patterns;
use ohhi_data::probe::window_survives;

const N_MAX: usize = 14; // even, ≤15 (u16 masks)

struct Window {
    red: u16,
    blue: u16,
    w: usize,
    forced: Vec<(usize, Cell)>,
    shape: String,
}

/// Trim a mined atom (1-row example + forced cells) to its active window.
fn window_of(red_full: u16, blue_full: u16, n: usize, forced_full: &[(usize, Cell)]) -> Window {
    let filled = red_full | blue_full;
    let positions: Vec<usize> = (0..n)
        .filter(|&p| filled & (1 << p) != 0)
        .chain(forced_full.iter().map(|&(p, _)| p))
        .collect();
    let lo = *positions.iter().min().unwrap();
    let hi = *positions.iter().max().unwrap();
    let w = hi - lo + 1;
    let mask = ((1u32 << w) - 1) as u16;
    let red = (red_full >> lo) & mask;
    let blue = (blue_full >> lo) & mask;
    let forced: Vec<(usize, Cell)> = forced_full.iter().map(|&(p, c)| (p - lo, c)).collect();
    let shape: String = (0..w)
        .map(|p| {
            if red & (1 << p) != 0 { 'R' }
            else if blue & (1 << p) != 0 { 'B' }
            else { '.' }
        })
        .collect();
    let _ = n;
    Window { red, blue, w, forced, shape }
}

/// Red budgets (0..=n) at which the window survives, embedded at `off` in `n`.
fn firing_budgets(win: &Window, off: usize, n: usize) -> Vec<usize> {
    (0..=n)
        .filter(|&tr| window_survives(win.red, win.blue, win.w, &win.forced, off, n, tr))
        .collect()
}

fn main() {
    println!("Experiment #3 — invariance scan of mined counting atoms.\n");

    for p in nontrivial_patterns() {
        // Extract masks from the 1-row example.
        let n0 = p.example.width();
        let mut red = 0u16;
        let mut blue = 0u16;
        for c in 0..n0 {
            match p.example.get((0, c)) {
                Cell::Red => red |= 1 << c,
                Cell::Blue => blue |= 1 << c,
                Cell::Nothing => {}
            }
        }
        let forced: Vec<(usize, Cell)> = p.forced.iter().map(|&((_, c), col)| (c, col)).collect();
        let win = window_of(red, blue, n0, &forced);

        // Length scan at balanced budget (corner).
        let mut len_fires = Vec::new();
        let mut rescue = Vec::new();
        for n in (win.w..=N_MAX).filter(|n| n % 2 == 0) {
            let balanced = window_survives(win.red, win.blue, win.w, &win.forced, 0, n, n / 2);
            if balanced {
                len_fires.push(n);
            } else {
                // Where does it fire instead?
                let fb = firing_budgets(&win, 0, n);
                if let Some(&tr) = fb.first() {
                    rescue.push((n, tr));
                }
            }
        }

        // Position scan: embed the window in a roomy line and, at each offset,
        // see whether *some* red budget makes it fire (corner vs middle vs side).
        let n_test = (win.w + 4).min(N_MAX) & !1; // even, room to slide
        let n_test = if n_test <= win.w { win.w + 2 } else { n_test };
        let slots = n_test - win.w + 1;
        let pos_fires: Vec<usize> = (0..slots)
            .filter(|&off| (0..=n_test).any(|tr| {
                window_survives(win.red, win.blue, win.w, &win.forced, off, n_test, tr)
            }))
            .collect();

        let len_label = if len_fires.len() == 1 { "native-only" } else { "multi-n" };
        let pos_label = if pos_fires.len() == slots { "position-free" }
            else if pos_fires.is_empty() { "never(rigid-n)" }
            else { "position-dependent" };

        println!("{:14} (n0={n0}, w={})", win.shape, win.w);
        println!("   length (bal n/2): fires at n={len_fires:?}  [{len_label}]");
        println!("   position (n={n_test}): fires at offsets={pos_fires:?}/{slots} [{pos_label}]");
        if !rescue.is_empty() {
            println!("   budget rescue   : non-firing n fire at (n,tr)={rescue:?}");
        }
        println!();
    }
}
