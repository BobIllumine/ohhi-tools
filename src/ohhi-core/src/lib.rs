//! Core data types and rules for the 0h h1 puzzle.
//!
//! # The puzzle
//!
//! 0h h1 is a binary-logic puzzle played on an N×N grid (N always even).
//! Each cell is either **Red**, **Blue**, or empty. A completed board is
//! valid when all three rules hold simultaneously:
//!
//! 1. **Rule of 3** — no three adjacent cells in the same row or column may
//!    share the same color.
//! 2. **Rule of equity** — every row and every column must contain exactly
//!    N/2 red cells and N/2 blue cells.
//! 3. **No duplicates** — no two rows may be identical, and no two columns
//!    may be identical.
//!
//! # Crate structure
//!
//! | Module | Purpose |
//! |---|---|
//! | [`bit_board`] | Primary board type. Stores the grid as bitpacked `u16` masks for O(1) row/column access. |
//! | [`stats`] | [`NumTransforms`](stats::NumTransforms) trait — bitwise line-level queries (signatures, counts, run detection, completeness). |
//! | [`validator`] | Checks a [`BitBoard`](bit_board::BitBoard) against any subset of the three rules via [`Validator::validate`](validator::Validator::validate). |
//! | [`board`] | Legacy `Board`/`Line` types (mostly superseded by `BitBoard`). Still home to the [`Cell`](board::Cell) enum used throughout the crate. |
//! | [`visualizer`] | `Display` impl for `BitBoard` that pretty-prints the grid. |

pub mod board;
pub mod visualizer;
pub mod validator;
pub mod stats;
pub mod bit_board;
pub mod seed;
