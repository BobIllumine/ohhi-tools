//! Parallel brute-force enumerator for valid 0h h1 boards.
//!
//! The binary in `src/main.rs` enumerates every possible coloring of an N×N
//! grid and validates each one against all three rules using a
//! [`ThreadPool`](thread_pool::ThreadPool). This is only practical for small
//! boards (≤ 6×6 in reasonable time), but serves as an independent oracle for
//! verifying the backtracking solver's solution counts.
//!
//! # Running
//!
//! ```sh
//! cargo run --release -p bruteforcer
//! ```
//!
//! Edit the `n` constant in `src/main.rs` to change the board size.

pub mod thread_pool;
