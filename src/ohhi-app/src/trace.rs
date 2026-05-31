//! Engine-agnostic deduction trace used by the step-scrubber UI.

use ohhi_core::board::Cell;
use ohhi_solver::structs::{DeductionTrace, Technique};
use ohhi_solver::v2::propagate::Propagation;

/// Why a cell was forced.
#[derive(Clone, Debug, PartialEq)]
pub enum StepReason {
    /// Forced by a v1 named technique.
    Technique(Technique),
    /// Forced by per-line enumeration in v2 (no single technique name).
    LineForced,
}

/// One forced cell in a deduction sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub at: (usize, usize),
    pub color: Cell,
    pub reason: StepReason,
}

/// An ordered list of forced cells produced by a deduction run.
#[derive(Clone, Debug, PartialEq)]
pub struct Trace {
    pub steps: Vec<Step>,
}

impl Trace {
    pub fn empty() -> Self {
        Trace { steps: vec![] }
    }
}

/// Convert a v1 [`DeductionTrace`] into a [`Trace`].
pub fn from_v1(t: &DeductionTrace) -> Trace {
    Trace {
        steps: t.get_steps().iter().map(|s| Step {
            at: s.at,
            color: s.cell,
            reason: StepReason::Technique(s.technique),
        }).collect(),
    }
}

/// Convert a v2 [`Propagation`] into a [`Trace`].
pub fn from_v2(p: &Propagation) -> Trace {
    Trace {
        steps: p.steps.iter().map(|&(r, c, color)| Step {
            at: (r, c),
            color,
            reason: StepReason::LineForced,
        }).collect(),
    }
}
