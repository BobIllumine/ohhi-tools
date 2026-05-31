//! Completed-drill records and aggregate stats for the Stats view.
//!
//! Records are framework-free and `serde`-serializable so the desktop shell can
//! persist them to disk; the aggregation helpers are pure and WASM-safe.

use ohhi_generator::practice::Target;
use serde::{Deserialize, Serialize};

/// One finished practice drill.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrillRecord {
    pub target: Target,
    pub n: usize,
    pub total_reps: usize,
    /// Recognition time (ms) for each successful rep, in order.
    pub times: Vec<u64>,
    pub hits: usize,
    pub misses: usize,
    /// Unix epoch millis when the drill finished (0 if unknown). Injected by the
    /// shell — `ohhi-app` has no clock.
    pub epoch_ms: u64,
}

impl DrillRecord {
    pub fn mean_ms(&self) -> u64 {
        if self.times.is_empty() {
            0
        } else {
            self.times.iter().sum::<u64>() / self.times.len() as u64
        }
    }

    pub fn best_ms(&self) -> u64 {
        self.times.iter().copied().min().unwrap_or(0)
    }

    pub fn accuracy(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 1.0 } else { self.hits as f64 / total as f64 }
    }
}

/// Aggregate stats for one technique across all its drills.
#[derive(Clone, Debug, PartialEq)]
pub struct TargetSummary {
    pub target: Target,
    pub drills: usize,
    pub best_mean_ms: u64,
    pub avg_mean_ms: u64,
    pub avg_accuracy: f64,
}

/// Per-technique summaries over `history`, ordered by descending drill count.
pub fn summaries(history: &[DrillRecord]) -> Vec<TargetSummary> {
    use std::collections::BTreeMap;
    // BTreeMap keyed by the discriminant order for determinism.
    let mut by_target: BTreeMap<usize, Vec<&DrillRecord>> = BTreeMap::new();
    for d in history {
        by_target.entry(target_order(d.target)).or_default().push(d);
    }
    let mut out: Vec<TargetSummary> = by_target
        .into_values()
        .map(|drills| {
            let n = drills.len() as u64;
            let best_mean_ms = drills.iter().map(|d| d.mean_ms()).min().unwrap_or(0);
            let avg_mean_ms = drills.iter().map(|d| d.mean_ms()).sum::<u64>() / n.max(1);
            let avg_accuracy = drills.iter().map(|d| d.accuracy()).sum::<f64>() / n as f64;
            TargetSummary {
                target: drills[0].target,
                drills: drills.len(),
                best_mean_ms,
                avg_mean_ms,
                avg_accuracy,
            }
        })
        .collect();
    out.sort_by(|a, b| b.drills.cmp(&a.drills));
    out
}

/// Drills of one technique in completion order (for trend charts).
pub fn trend<'a>(history: &'a [DrillRecord], target: Target) -> Vec<&'a DrillRecord> {
    history.iter().filter(|d| d.target == target).collect()
}

fn target_order(t: Target) -> usize {
    match t {
        Target::Saturation => 0,
        Target::TwinCompletion => 1,
        Target::PairExtension => 2,
        Target::GapFill => 3,
        Target::Counting => 4,
    }
}
