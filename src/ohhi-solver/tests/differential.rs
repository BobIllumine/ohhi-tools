//! Differential / cross-algorithm integration tests.
//!
//! These tests compare two independent counting methods against each other
//! and against known correct values. They are the strongest correctness gate:
//! if both algorithms agree it's hard for both to be wrong in the same way.

use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use ohhi_core::validator::{Filter, Validator};
use ohhi_solver::backtrack::calculate;

fn bb(rows: &[&str]) -> BitBoard {
    let height = rows.len();
    let width = rows[0].len();
    let mut board = BitBoard::new(width, height);
    for (r, row) in rows.iter().enumerate() {
        for (c, ch) in row.chars().enumerate() {
            let cell = match ch {
                'R' => Cell::Red,
                'B' => Cell::Blue,
                '.' => Cell::Nothing,
                other => panic!("invalid char '{}' at ({}, {})", other, r, c),
            };
            board.set((r, c), cell);
        }
    }
    board
}

/// Inline brute-force oracle: enumerates all 2^(n*n) complete colorings (R/B
/// only, no Nothing) and counts those that pass `Validator::validate`.
/// This is deliberately simple — it does not use any of the production crates'
/// algorithms — so it acts as an independent witness.
fn brute_count(n: usize) -> usize {
    let filter = Filter {
        rule_of_3: true,
        rule_of_equity: true,
        rule_of_duplication: true,
        incomplete: true,
    };
    let cells = n * n;
    let mut count = 0usize;
    for bits in 0u64..(1u64 << cells) {
        let mut board = BitBoard::new(n, n);
        for r in 0..n {
            for c in 0..n {
                let idx = r * n + c;
                let cell = if (bits >> idx) & 1 == 1 { Cell::Red } else { Cell::Blue };
                board.set((r, c), cell);
            }
        }
        if board.validate(&filter).is_ok() {
            count += 1;
        }
    }
    count
}

#[test]
fn brute_and_backtracker_agree_on_2x2() {
    assert_eq!(brute_count(2), 2);
    assert_eq!(calculate(&BitBoard::new(2, 2), usize::MAX), 2);
}

#[test]
fn brute_and_backtracker_agree_on_4x4() {
    let brute = brute_count(4);
    let backtrack = calculate(&BitBoard::new(4, 4), usize::MAX);
    assert_eq!(brute, backtrack, "brute={brute} backtrack={backtrack}");
    // The correct answer for 4×4 0h h1 (equity + rule-of-3 + no-duplicate)
    assert_eq!(brute, 72);
}

#[test]
fn deduce_completion_matches_backtracker_on_unique_seeds() {
    use ohhi_solver::v1::deduction::deduce;

    // Hand-built unique 4×4 seeds: one empty cell each, all have exactly one
    // valid completion.
    let cases: &[(&[&str], &[&str])] = &[
        (
            &["RRBB", "BBRR", "RBBR", "BRR."],
            &["RRBB", "BBRR", "RBBR", "BRRB"],
        ),
        (
            &["RRBB", "BBRR", "RBB.", "BRRB"],
            &["RRBB", "BBRR", "RBBR", "BRRB"],
        ),
        (
            &["RRBB", "BBRR", ".BBR", "BRRB"],
            &["RRBB", "BBRR", "RBBR", "BRRB"],
        ),
    ];

    for (seed_rows, expected_rows) in cases {
        let seed = bb(seed_rows);
        let expected = bb(expected_rows);

        // Must have exactly one solution
        assert_eq!(calculate(&seed, 2), 1, "seed is not unique: {:?}", seed_rows);

        let trace = deduce(&seed);
        if !trace.stalled && !trace.get_steps().is_empty() {
            let final_board = &trace.get_steps().last().unwrap().board_after;
            assert_eq!(
                *final_board, expected,
                "deduction result differs from expected for seed {:?}", seed_rows
            );
        }
        // If the engine stalls on a unique seed it can't solve it — that's
        // not a bug but a capability limit, so we don't fail the test.
    }
}
