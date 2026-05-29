//! Solving algorithms for 0h h1 puzzles built on [`ohhi_core`].
//!
//! # Modules
//!
//! | Module | What it does |
//! |---|---|
//! | [`backtrack`] | Depth-first solver. Counts valid completions of a partial board (`calculate`) or finds one solution (`backtrack_one`). Use `cap=2` as a fast uniqueness test. |
//! | [`deduction`] | Fixpoint deduction engine. Given a board, repeatedly applies logical techniques until the board is complete or no more moves can be forced. Returns a [`DeductionTrace`](structs::DeductionTrace) describing every forced cell and the technique that forced it. |
//! | [`carver`] | Puzzle generator. Given a complete valid board, removes cells one-by-one (in random order) while the solution remains unique. Returns the minimal seed that still has exactly one completion. |
//! | [`structs`] | Shared types: [`SolverState`](structs::SolverState) (incremental legality checker used by both the backtracker and the deduction engine), plus the trace output types [`DeductionTrace`](structs::DeductionTrace), [`DeductionStep`](structs::DeductionStep), [`Technique`](structs::Technique), and [`TechniqueSet`](structs::TechniqueSet). |
//!
//! # Typical workflows
//!
//! **Count solutions (uniqueness check):**
//! ```rust
//! use ohhi_core::bit_board::BitBoard;
//! use ohhi_solver::backtrack::calculate;
//!
//! let board = BitBoard::new(4, 4); // empty 4×4
//! assert_eq!(calculate(&board, 2), 2); // at least 2 — not unique
//! ```
//!
//! **Run the deduction engine:**
//! ```rust
//! use ohhi_solver::v1::deduction::{deduce, Technique};
//! // board with one obvious forced cell
//! // let trace = deduce(&board);
//! // for step in trace.get_steps() { ... }
//! ```
//!
//! **Generate a puzzle from a complete board:**
//! ```rust
//! use ohhi_solver::carver::carve;
//! // let seed = carve(&complete_board).unwrap();
//! ```
pub use v1::deduction;
pub mod backtrack;
pub mod carver;
pub mod structs;
pub mod v1;
pub mod v2;
