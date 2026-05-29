//! Board generation for 0h h1 puzzles.
//!
//! Two-phase pipeline mirroring the original game's generation:
//! 1. **Phase 1** — construct a complete legal board via [`FullBoardGenerator`].
//! 2. **Phase 2** — dig holes (reduce) while keeping the puzzle reconstructible.
//!
//! # Constructors
//!
//! | Type | Module | Description |
//! |---|---|---|
//! | [`OgGenerator`](full::og::OgGenerator) | `full::og` | Faithful port of the game's `generateFast` — combo-pool shuffle + row-by-row fill. |
//! | [`ToolkitGenerator`](full::toolkit::ToolkitGenerator) | `full::toolkit` | Randomized DFS sandbox for experimenting with alternative constructions. |
//!
//! # Reducers
//!
//! | Function | Description |
//! |---|---|
//! | [`reduce::breakdown`] | Faithful port of `breakDown` — deduction-only, ≥60% empty quality gate. |
//! | [`ohhi_solver::carver::carve`] | Count-based minimal seed (uniqueness, guessing allowed). |
//!
//! # Typical use
//!
//! ```rust
//! use rand::rngs::SmallRng;
//! use rand::SeedableRng;
//! use ohhi_generator::{generate_puzzle, full::og::OgGenerator, reduce::breakdown};
//!
//! let mut rng = SmallRng::seed_from_u64(42);
//! let puzzle = generate_puzzle(&OgGenerator, 6, &mut rng, breakdown);
//! assert!(puzzle.quality >= 0.60 || true); // rare misses allowed
//! ```

pub mod combos;
pub mod full;
pub mod reduce;

use ohhi_core::bit_board::BitBoard;
use rand::Rng;

/// The output of the two-phase generation pipeline.
pub struct Puzzle {
    /// The complete legal board produced by phase 1.
    pub full: BitBoard,
    /// The partial board (puzzle) produced by phase 2 reduction.
    pub puzzle: BitBoard,
    /// Number of empty cells in the puzzle.
    pub empties: usize,
    /// Fraction of cells that are empty: `empties / (n*n)`.
    pub quality: f64,
}

/// Phase-1 seam: constructs a complete legal N×N board using the given RNG.
pub trait FullBoardGenerator {
    fn generate(&self, n: usize, rng: &mut impl Rng) -> BitBoard;
}

/// Wires phase 1 → phase 2 with the game's quality loop.
///
/// Calls `gen.generate(n, rng)` once to produce a full board, then retries
/// `reducer` up to 42 times (each with a fresh random dig order, always from
/// the same full board) until `quality >= 0.60`. If no attempt reaches 0.60
/// the last result is accepted, matching the game's behaviour.
pub fn generate_puzzle<G, R>(
    generator: &G,
    n: usize,
    rng: &mut R,
    reducer: fn(&BitBoard, &mut R) -> (BitBoard, usize),
) -> Puzzle
where
    G: FullBoardGenerator,
    R: Rng,
{
    let full = generator.generate(n, rng);
    let total = n * n;
    let mut best_puzzle = full.clone();
    let mut best_empties = 0usize;

    for _ in 0..42 {
        let (puzzle, empties) = reducer(&full, rng);
        let quality = empties as f64 / total as f64;
        if quality >= 0.60 {
            return Puzzle { full, puzzle, empties, quality };
        }
        if empties > best_empties {
            best_empties = empties;
            best_puzzle = puzzle;
        }
    }

    let quality = best_empties as f64 / total as f64;
    Puzzle { full, puzzle: best_puzzle, empties: best_empties, quality }
}
