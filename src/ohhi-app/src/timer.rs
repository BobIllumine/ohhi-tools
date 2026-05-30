//! Injection-based timer. No system-clock calls — WASM-safe.

/// A millisecond-resolution timer driven by caller-supplied deltas.
pub struct Timer {
    pub elapsed_ms: u64,
    pub running: bool,
}

impl Timer {
    pub fn new() -> Self {
        Timer { elapsed_ms: 0, running: false }
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn reset(&mut self) {
        self.elapsed_ms = 0;
        self.running = false;
    }

    /// Advance the timer by `delta_ms` milliseconds. No-op when stopped.
    pub fn tick(&mut self, delta_ms: u64) {
        if self.running {
            self.elapsed_ms += delta_ms;
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
