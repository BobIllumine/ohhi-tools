use dioxus::prelude::*;
use ohhi_core::board::Cell;
use ohhi_data::patterns::{all_patterns, Pattern, PatternClass};

use super::board::Board;

const PANEL_BG: &str = "#181818";
const BORDER:   &str = "#2e2e2e";
const MUTED:    &str = "#909090";

/// Patterns mode: a scrollable, enumerated library of non-trivial forcing
/// patterns (single-line counting atoms + the cross-line twin family).
#[component]
pub fn PatternsView() -> Element {
    // Mining `all_patterns()` enumerates 3^n line states up to n=12, so compute
    // it once and keep it for the lifetime of the view.
    let patterns = use_signal(all_patterns);

    // Group by board width, preserving order (counting atoms ascend 6→12, the
    // twin family trails). Each group gets a section header.
    let mut groups: Vec<(usize, PatternClass, Vec<Pattern>)> = Vec::new();
    for p in patterns.read().iter() {
        let w = p.example.width();
        match groups.last_mut() {
            Some((gw, _, v)) if *gw == w => v.push(p.clone()),
            _ => groups.push((w, p.class, vec![p.clone()])),
        }
    }

    rsx! {
        div {
            style: "height: 100%; overflow-y: auto; background: #1e1e1e; padding: 20px;",
            div {
                style: "max-width: 920px; margin: 0 auto; display: flex; flex-direction: column; gap: 14px;",

                h2 { style: "margin: 0 0 4px; font-size: 18px; font-weight: 600;", "Non-trivial patterns" }
                p { style: "margin: 0 0 12px; font-size: 13px; color: {MUTED};",
                    "Deductions that need a full-line counting argument or cross-line uniqueness — the cases saturation, gap-fill and pair-extension miss."
                }

                for (w, class, group) in groups.iter() {
                    SectionHeader { width: *w, class: *class, count: group.len() }
                    for p in group.iter() {
                        PatternRow { key: "{p.id}", pattern: p.clone() }
                    }
                }
            }
        }
    }
}

/// A divider + label introducing a group of patterns of the same width.
#[component]
fn SectionHeader(width: usize, class: PatternClass, count: usize) -> Element {
    let title = match class {
        PatternClass::Twin => "Twin family".to_string(),
        PatternClass::Counting => format!("Line width {width}"),
    };
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 12px; margin-top: 10px;",
            span { style: "font-size: 13px; font-weight: 700; letter-spacing: .04em; text-transform: uppercase; color: #b0b0b0;",
                "{title}"
            }
            span { style: "font-size: 12px; color: {MUTED};", "{count}" }
            div { style: "flex: 1; height: 1px; background: {BORDER};" }
        }
    }
}

#[component]
fn PatternRow(pattern: Pattern) -> Element {
    let (badge_bg, badge_fg) = match pattern.class {
        PatternClass::Counting => ("#2a3a4a", "#9cc4ff"),
        PatternClass::Twin     => ("#3a2a4a", "#d0a0ff"),
    };

    // Step 2 board: clues with the forced cells filled in, plus an outline board
    // marking which cells were deduced (so they stand out from the given anchors).
    let mut solved = pattern.example.clone();
    let mut outline = ohhi_core::bit_board::BitBoard::new(
        pattern.example.width(),
        pattern.example.height(),
    );
    for &(pos, color) in &pattern.forced {
        solved.set(pos, color);
        outline.set(pos, color);
    }

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 20px; padding: 14px 16px; \
                    background: {PANEL_BG}; border: 1px solid {BORDER}; border-radius: 10px;",

            // Index (1-based for display)
            div { style: "font-size: 13px; color: {MUTED}; font-family: monospace; min-width: 28px;",
                "#{pattern.id + 1}"
            }

            // Two-step schematic: clues above, forced result below.
            div { style: "display: flex; flex-direction: column; gap: 6px; flex-shrink: 0; width: 400px;",
                MiniBoardRow { caption: "pattern", board: pattern.example.clone() }
                MiniBoardRow { caption: "forced",  board: solved, outline }
            }

            // Text block
            div { style: "display: flex; flex-direction: column; gap: 5px; min-width: 0;",
                div { style: "display: flex; align-items: center; gap: 8px;",
                    span { style: "font-size: 15px; font-weight: 600;", "{pattern.name}" }
                    span {
                        style: "font-size: 11px; font-weight: 600; padding: 2px 8px; border-radius: 999px; \
                                background: {badge_bg}; color: {badge_fg};",
                        "{pattern.class.badge()}"
                    }
                }
                span { style: "font-size: 13px; color: #c8c8c8;",
                    "{forced_summary(&pattern)}"
                }
            }
        }
    }
}

/// One captioned, scaled-down board row in the schematic.
#[component]
fn MiniBoardRow(
    caption: &'static str,
    board: ohhi_core::bit_board::BitBoard,
    #[props(default)] outline: Option<ohhi_core::bit_board::BitBoard>,
) -> Element {
    // The board renders at its native 56px cell size; clip + scale into a slot.
    // Scale chosen so the widest mined line (n=12 ≈ 716px) fits the slot.
    const SCALE: f64 = 0.45;
    let slot_h = board.height() as f64 * 56.0 * SCALE + 4.0;
    rsx! {
        div { style: "display: flex; align-items: center; gap: 10px;",
            span { style: "font-size: 11px; color: {MUTED}; width: 52px; text-align: right; flex-shrink: 0;",
                "{caption}"
            }
            div {
                style: "width: 340px; height: {slot_h}px; overflow: hidden;",
                div { style: "transform: scale({SCALE}); transform-origin: top left;",
                    Board { board, outline, on_cell_click: move |_| {} }
                }
            }
        }
    }
}

/// Human-readable forced-cell summary, columns **and rows 1-indexed for the UI**
/// (the data layer keeps everything 0-based).
fn forced_summary(pattern: &Pattern) -> String {
    let multi_row = pattern.example.height() > 1;
    let parts: Vec<String> = pattern
        .forced
        .iter()
        .map(|&((r, c), color)| {
            let name = match color {
                Cell::Red => "Red",
                Cell::Blue => "Blue",
                Cell::Nothing => "?",
            };
            if multi_row {
                format!("r{} c{} → {name}", r + 1, c + 1)
            } else {
                format!("col {} → {name}", c + 1)
            }
        })
        .collect();
    parts.join(",  ")
}
