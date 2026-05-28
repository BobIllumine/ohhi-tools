//! Board validation against the three 0h h1 rules.
//!
//! `Validator::validate` judges a board against a `Filter`. Uniqueness
//! (duplication) is gated on line completeness — a partial line's red mask
//! is not a valid identifier and must not be inserted into the duplicate-check
//! set.

use std::collections::HashSet;
use crate::bit_board::BitBoard;
use crate::stats::NumTransforms;

/// Which rule violations were detected. All fields default to `false`
/// (no violation). Check `is_valid()` for a one-shot test.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Violation {
    rule_of_3: bool,
    rule_of_equity: bool,
    rule_of_duplication: bool,
    /// `true` if the board has at least one empty cell.
    /// This is not a "rule" violation per se — it just signals that
    /// the board is incomplete. Controlled by `Filter::incomplete`.
    incomplete: bool,
}
impl Violation {
    pub fn new() -> Violation {
        Violation {
            rule_of_3: false,
            rule_of_equity: false,
            rule_of_duplication: false,
            incomplete: false,
        }
    }
    pub fn rule_of_3(&self) -> bool {
        self.rule_of_3
    }
    pub fn rule_of_equity(&self) -> bool {
        self.rule_of_equity
    }
    pub fn rule_of_duplication(&self) -> bool {
        self.rule_of_duplication
    }
    pub fn incomplete(&self) -> bool {
        self.incomplete
    }
    /// Returns `true` only if no violation flags are set.
    pub fn is_valid(&self) -> bool {
        !self.rule_of_3 && !self.rule_of_equity && !self.rule_of_duplication && !self.incomplete
    }
}

/// Selects which rules to enforce during `validate`. Set a field to `false`
/// to skip that rule entirely (useful for research experiments that want to
/// count boards solvable without uniqueness, for example).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Filter {
    pub rule_of_3: bool,
    pub rule_of_equity: bool,
    pub rule_of_duplication: bool,
    /// When `true`, any board with empty cells is reported as `incomplete`.
    pub incomplete: bool,
}

/// Identifies a single rule for use with `ToggleRule` actions.
pub enum Rule {
    RuleOf3,
    RuleOfEquity,
    RuleOfDuplication,
    Incomplete,
}

/// Validates a board against the 0h h1 rules selected by `filter`.
pub trait Validator {
    fn validate(&self, filter: &Filter) -> Result<(), Violation>;
}

impl Validator for BitBoard {
    fn validate(&self, filter: &Filter) -> Result<(), Violation> {
        let mut vio = Violation::new();
        let mut rows_hset: HashSet<u16> = HashSet::new();
        for r in 0..self.height {
            if filter.incomplete && !self.is_complete_x(&r) {
                vio.incomplete = true;
            }
            if filter.rule_of_3 && self.has_consecutive_x(&r, 3) {
                vio.rule_of_3 = true;
            }
            if filter.rule_of_equity && (self.count_x(&r).0 * 2 != self.width as u8) {
                vio.rule_of_equity = true;
            }
            // Only insert the signature of a *complete* row. A partial row's
            // red mask is not a valid line identifier and would cause false
            // uniqueness positives between two incomplete rows.
            if filter.rule_of_duplication && self.is_complete_x(&r) {
                vio.rule_of_duplication = !rows_hset.insert(self.signature_x(&r));
            }
        }
        let mut cols_hset: HashSet<u16> = HashSet::new();
        for c in 0..self.width {
            if filter.incomplete && !self.is_complete_y(&c) {
                vio.incomplete = true;
            }
            if filter.rule_of_3 && self.has_consecutive_y(&c, 3) {
                vio.rule_of_3 = true;
            }
            if filter.rule_of_equity && (self.count_y(&c).0 * 2 != self.width as u8) {
                vio.rule_of_equity = true;
            }
            // Same gate as the row branch above: only check uniqueness for
            // complete columns.
            if filter.rule_of_duplication && self.is_complete_y(&c) {
                vio.rule_of_duplication = !cols_hset.insert(self.signature_y(&c));
            }
        }

        match vio.is_valid() {
            true => Ok(()),
            false => Err(vio)
        }
    }
}
