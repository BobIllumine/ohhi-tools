//! Headless generation driver — reproduces the §10 measurement path.
//!
//! For each board size (4, 6, 8) generates a batch of puzzles with a seeded RNG and
//! prints, per reducer:
//! - empties: min / mean / max
//! - empties histogram (each count → frequency)
//! - % deduction-solvable from scratch (oracle §11.3)
//! - % unique solution (oracle §11.2)
//!
//! Usage:
//!   cargo run --release -p ohhi-generator --bin gen -- [SEED] [BATCH]
//!
//! Defaults: SEED=42, BATCH=500

use ohhi_core::validator::{Filter, Validator};
use ohhi_generator::full::og::OgGenerator;
use ohhi_generator::full::toolkit::ToolkitGenerator;
use ohhi_generator::reduce::breakdown;
use ohhi_generator::{generate_puzzle, FullBoardGenerator};
use ohhi_solver::backtrack::calculate;
use ohhi_solver::carver::carve;
use ohhi_solver::v1::deduction::{deduce_with, TechniqueSet};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::collections::HashMap;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let seed: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(42);
    let batch: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(500);

    println!("Seed: {seed}  Batch: {batch} per size/reducer\n");

    for n in [4usize, 6, 8] {
        println!("=== n={n} ===");
        run_batch("OG + Breakdown", n, seed, batch, &OgGenerator, breakdown);
        run_batch("OG + Carve    ", n, seed, batch, &OgGenerator, carve_adapter);
        run_batch("Toolkit + Breakdown", n, seed, batch, &ToolkitGenerator, breakdown);
        println!();
    }
}

fn carve_adapter(full: &ohhi_core::bit_board::BitBoard, _rng: &mut impl rand::Rng) -> (ohhi_core::bit_board::BitBoard, usize) {
    match carve(full) {
        Ok(puzzle) => {
            let empties = (0..full.height())
                .flat_map(|r| (0..full.width()).map(move |c| (r, c)))
                .filter(|&pos| puzzle.get(pos) == ohhi_core::board::Cell::Nothing)
                .count();
            (puzzle, empties)
        }
        Err(_) => (full.clone(), 0),
    }
}

fn run_batch<G: FullBoardGenerator>(
    label: &str,
    n: usize,
    seed: u64,
    batch: usize,
    generator: &G,
    reducer: fn(&ohhi_core::bit_board::BitBoard, &mut rand::rngs::SmallRng) -> (ohhi_core::bit_board::BitBoard, usize),
) {
    let mut rng = SmallRng::seed_from_u64(seed);
    let filter_all = Filter {
        rule_of_3: true,
        rule_of_equity: true,
        rule_of_duplication: true,
        incomplete: true,
    };

    let mut empties_list: Vec<usize> = Vec::with_capacity(batch);
    let mut deduction_solvable = 0usize;
    let mut unique = 0usize;

    for _ in 0..batch {
        let puzzle = generate_puzzle(generator, n, &mut rng, reducer);

        // Oracle §11.1 — full board legality (spot-check, not counted)
        debug_assert!(puzzle.full.validate(&filter_all).is_ok());

        empties_list.push(puzzle.empties);

        // Oracle §11.3 — deduction-only solvable from scratch
        let trace = deduce_with(&puzzle.puzzle, TechniqueSet::ALL);
        let last_board = trace.get_steps().last().map(|s| &s.board_after);
        let is_deduction_solvable = if let Some(board) = last_board {
            board.validate(&filter_all).is_ok()
        } else {
            false
        };
        if is_deduction_solvable {
            deduction_solvable += 1;
        }

        // Oracle §11.2 — unique solution
        if calculate(&puzzle.puzzle, 2) == 1 {
            unique += 1;
        }
    }

    let min = *empties_list.iter().min().unwrap();
    let max = *empties_list.iter().max().unwrap();
    let mean = empties_list.iter().sum::<usize>() as f64 / batch as f64;

    let mut hist: HashMap<usize, usize> = HashMap::new();
    for &e in &empties_list {
        *hist.entry(e).or_default() += 1;
    }
    let mut hist_sorted: Vec<(usize, usize)> = hist.into_iter().collect();
    hist_sorted.sort_by_key(|&(k, _)| k);

    println!("  [{label}]");
    println!("    empties: min={min}  mean={mean:.2}  max={max}");
    println!("    deduction-solvable: {deduction_solvable}/{batch} = {:.1}%",
        deduction_solvable as f64 / batch as f64 * 100.0);
    println!("    unique: {unique}/{batch} = {:.1}%",
        unique as f64 / batch as f64 * 100.0);
    let hist_str: Vec<String> = hist_sorted.iter()
        .map(|(k, v)| format!("{k}:{v}"))
        .collect();
    println!("    histogram: {}", hist_str.join("  "));
}
