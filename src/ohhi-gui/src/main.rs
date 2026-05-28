//! Interactive GUI for the 0h h1 research toolkit.
//!
//! This binary is a desktop application built with [egui] / [eframe]. It lets
//! you manually edit a 0h h1 board and run the solver, deduction engine, and
//! carver interactively.
//!
//! # Layout
//!
//! The window is divided into three vertical panels:
//!
//! | Panel | Contents |
//! |---|---|
//! | **Left toolbar** | Undo / Redo / Clear / Resize buttons; Load Seed dialog at the bottom |
//! | **Center canvas** | The board grid. Click a cell to cycle it through Nothing → Red → Blue → Nothing. Optional row/column signature overlay. |
//! | **Right panel** | Validate, Signatures, Deduction controls (technique checkboxes, Deduce / Apply trace / Clear overlay), and a step scrub slider |
//!
//! # Modules
//!
//! | Module | Purpose |
//! |---|---|
//! | [`app`] | Top-level [`eframe::App`] wrapper; owns [`GuiState`](state::GuiState) and delegates rendering |
//! | [`state`] | All mutable application state. UI fires [`Action`](state::Action) variants; [`apply`](state::apply) mutates [`GuiState`](state::GuiState) |
//! | [`render`] | Immediate-mode drawing for each panel |
//! | [`seed`] | Seed string encoding and parsing |
//!
//! # Seed format
//!
//! Seeds are plain text: `R` / `B` / `.` separated by spaces within each row,
//! rows separated by newlines. Use the "Load Seed" button to paste one in, or
//! "Show signatures" to inspect the current board's line identifiers.

pub mod app;
pub mod state;
pub mod render;
pub mod seed;
use crate::app::App;

fn main() -> eframe::Result {
    let native = eframe::NativeOptions::default();
    eframe::run_native("0h h1 Toolkit", native, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
