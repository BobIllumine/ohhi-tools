//! Desktop persistence for practice drill history (JSON on disk).
//!
//! WASM builds get no-op stubs — browser persistence (localStorage) can be added
//! later without touching callers.

use ohhi_app::history::DrillRecord;

#[cfg(not(target_arch = "wasm32"))]
fn history_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = std::path::Path::new(&home).join(".ohhi-tools");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("drill_history.json")
}

/// Loads the saved history, or an empty list if absent/unreadable.
#[cfg(not(target_arch = "wasm32"))]
pub fn load() -> Vec<DrillRecord> {
    let path = history_path();
    let Ok(text) = std::fs::read_to_string(&path) else { return Vec::new() };
    serde_json::from_str(&text).unwrap_or_default()
}

/// Writes the full history to disk (best-effort; errors are swallowed).
#[cfg(not(target_arch = "wasm32"))]
pub fn save(history: &[DrillRecord]) {
    if let Ok(text) = serde_json::to_string_pretty(history) {
        let _ = std::fs::write(history_path(), text);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn load() -> Vec<DrillRecord> {
    Vec::new()
}

#[cfg(target_arch = "wasm32")]
pub fn save(_history: &[DrillRecord]) {}
