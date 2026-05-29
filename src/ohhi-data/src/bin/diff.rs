//! Headless v1-vs-v2 engine comparison.
//!
//! Generates a seeded batch of puzzles per size and prints, for each size:
//! - boards where v2 forced more cells than v1 (the "gap" count)
//! - mean `only_v2` cell count across all boards
//! - one example board where they differ (the diff printed)
//!
//! Usage: cargo run -p ohhi-data --bin diff -- [SEED] [BATCH]
//! Defaults: SEED=42, BATCH=500

use ohhi_generator::full::og::OgGenerator;
use ohhi_generator::reduce::breakdown;
use ohhi_generator::generate_puzzle;
use ohhi_solver::carver::carve;
use ohhi_solver::v2::diff::diff_engines;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::env;

fn carve_adapter(full: &ohhi_core::bit_board::BitBoard, _rng: &mut impl rand::Rng) -> (ohhi_core::bit_board::BitBoard, usize) {
    match carve(full) {
        Ok(puzzle) => {
            let empties = (0..full.height()).flat_map(|r| (0..full.width()).map(move |c| (r, c)))
                .filter(|&pos| puzzle.get(pos) == ohhi_core::board::Cell::Nothing)
                .count();
            (puzzle, empties)
        }
        Err(_) => (full.clone(), 0),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let seed: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(42);
    let batch: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(500);

    println!("Seed: {seed}  Batch: {batch} per size\n");
    println!("Note: breakdown puzzles are v1-solvable by construction (that's the point of breakdown).");
    println!("      carve puzzles are uniqueness-minimal only — not guaranteed v1-solvable.");
    println!("      The gap (only_v2) should appear on carve boards.\n");

    for n in [4usize, 6, 8] {
        println!("=== n={n} ===");
        for (label, use_carve) in [("breakdown", false), ("carve    ", true)] {
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut gap_count = 0usize;
            let mut total_only_v2 = 0usize;
            let mut example: Option<String> = None;

            for _ in 0..batch {
                let p = if use_carve {
                    generate_puzzle(&OgGenerator, n, &mut rng, carve_adapter)
                } else {
                    generate_puzzle(&OgGenerator, n, &mut rng, breakdown)
                };
                let d = diff_engines(&p.puzzle);

                assert!(d.only_v1.is_empty(), "soundness violation at n={n} [{label}]: v2 missed {:?}", d.only_v1);

                if !d.only_v2.is_empty() {
                    gap_count += 1;
                    total_only_v2 += d.only_v2.len();
                    if example.is_none() {
                        example = Some(format_board(&p.puzzle, n));
                    }
                }
            }

            let mean_gap = if gap_count > 0 { total_only_v2 as f64 / gap_count as f64 } else { 0.0 };
            println!("  [{label}] v2 > v1: {gap_count}/{batch} ({:.1}%)  mean only_v2: {mean_gap:.2}",
                gap_count as f64 / batch as f64 * 100.0);
            if let Some(board_str) = &example {
                println!("    example:\n{board_str}");
            }
        }
        println!();
    }
}

fn format_board(board: &ohhi_core::bit_board::BitBoard, n: usize) -> String {
    let mut s = String::new();
    for r in 0..n {
        s.push_str("    ");
        for c in 0..n {
            s.push(match board.get((r, c)) {
                ohhi_core::board::Cell::Red => 'R',
                ohhi_core::board::Cell::Blue => 'B',
                ohhi_core::board::Cell::Nothing => '.',
            });
            s.push(' ');
        }
        s.push('\n');
    }
    s
}
