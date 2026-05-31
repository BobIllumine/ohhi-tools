//! Threshold report for skeleton family #1: two Red anchors `d` apart, at the
//! left corner of an otherwise-empty line.
//!
//! For each gap `d` and length `n` it prints the red budgets at which the inner
//! neighbour cells are forced Blue. The structure it surfaces:
//!
//! - `gap = d-1 ≡ 0 (mod 3)` → never fires (the gap tiles comfortably with
//!   blocks of ≤2, so a red can always sit clear of both anchors).
//! - otherwise → fires at the anti-triple density floor, ≈ ⌊n/3⌋+1 reds: the
//!   point where blue is one cell away from a triple, so the inner cell can't
//!   be red.
//!
//! Usage: cargo run -p ohhi-data --bin thresholds

use ohhi_data::probe::firing_budgets;

// Masks are u16 → max line length 15.
const N_MAX: usize = 15;

fn main() {
    println!("Skeleton #1 — two reds at distance d, left corner.");
    println!("Cell = red budgets (tr) where the inner neighbours force Blue.\n");

    for d in 2..=9 {
        let gap = d - 1;
        let shape = format!("R{}R", ".".repeat(gap));
        let class = match gap % 3 {
            0 => "gap%3=0  (never)",
            1 => "gap%3=1",
            _ => "gap%3=2",
        };
        println!("d={d}  {shape:11} {class}");
        for n in (d + 1)..=N_MAX {
            let fires = firing_budgets(n, d);
            if fires.is_empty() {
                continue;
            }
            // Density floor splits by gap class: ⌊n/3⌋+1 for gap≡1, ⌊(n−1)/3⌋+1 for gap≡2.
            let pred = match gap % 3 { 1 => n / 3 + 1, 2 => (n - 1) / 3 + 1, _ => 0 };
            let hits = if fires.contains(&pred) { "✓" } else { " " };
            let rem: Vec<usize> = fires.iter().map(|tr| tr - 2).collect();
            println!(
                "    n={n:2}: tr={fires:?}  (reds-remaining {rem:?})   floor={pred} {hits}"
            );
        }
        println!();
    }
}
