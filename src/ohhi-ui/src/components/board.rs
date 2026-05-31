use dioxus::prelude::*;
use ohhi_core::bit_board::BitBoard;
use ohhi_core::board::Cell;
use ohhi_app::trace::{Trace, StepReason};
use ohhi_solver::structs::Technique;

pub const CELL: usize = 56;
pub const GAP:  usize = 4;

#[derive(Props, Clone, PartialEq)]
pub struct BoardProps {
    pub board: BitBoard,
    #[props(default)]
    pub locked: Option<BitBoard>,
    /// Cells to draw with a gold highlight outline (any non-`Nothing` position).
    #[props(default)]
    pub outline: Option<BitBoard>,
    #[props(default)]
    pub overlay: Option<Trace>,
    #[props(default)]
    pub overlay_step: usize,
    #[props(default)]
    pub show_signatures: bool,
    pub on_cell_click: EventHandler<(usize, usize)>,
    #[props(default)]
    pub on_right_click: Option<EventHandler<(usize, usize)>>,
}

#[component]
pub fn Board(props: BoardProps) -> Element {
    let board   = props.board.clone();
    let n_rows  = board.height();
    let n_cols  = board.width();

    // Collect forced cells up to overlay_step.
    let forced: Vec<(usize, usize, Cell, &'static str)> =
        props.overlay.as_ref().map(|t| {
            t.steps.iter().take(props.overlay_step).map(|s| {
                (s.at.0, s.at.1, s.color, technique_label(&s.reason))
            }).collect()
        }).unwrap_or_default();

    rsx! {
        div {
            style: "display: inline-flex; flex-direction: column; gap: {GAP}px;",

            // ── Grid row + row-sig column ──────────────────────────────────
            div {
                style: "display: inline-flex; flex-direction: row; gap: {GAP}px; align-items: flex-start;",

                // Board grid
                div {
                    style: "display: grid; grid-template-columns: repeat({n_cols}, {CELL}px); gap: {GAP}px;",
                    for idx in 0..n_rows * n_cols {
                        {
                            let r = idx / n_cols;
                            let c = idx % n_cols;
                            let f = forced.iter().find(|(fr, fc, _, _)| *fr == r && *fc == c)
                                .map(|(_, _, color, label)| (*color, *label));
                            rsx! {
                                CellView {
                                    key: "{r}-{c}",
                                    r, c,
                                    cell: board.get((r, c)),
                                    locked: props.locked.as_ref()
                                        .map(|lb| lb.get((r, c)) != Cell::Nothing)
                                        .unwrap_or(false),
                                    outlined: props.outline.as_ref()
                                        .map(|ob| ob.get((r, c)) != Cell::Nothing)
                                        .unwrap_or(false),
                                    forced: f,
                                    on_click: props.on_cell_click.clone(),
                                    on_right_click: props.on_right_click.clone(),
                                }
                            }
                        }
                    }
                }

                // Row signatures — one number per row, right of the grid
                if props.show_signatures {
                    div {
                        style: "display: flex; flex-direction: column; gap: {GAP}px; padding-top: 0;",
                        for r in 0..n_rows {
                            {
                                let sig = row_sig(&board, r);
                                rsx! {
                                    div {
                                        key: "rs-{r}",
                                        style: "
                                            height: {CELL}px;
                                            display: flex; align-items: center;
                                            padding-left: 8px;
                                            font-size: 13px;
                                            font-family: monospace;
                                            color: #909090;
                                            white-space: nowrap;
                                            min-width: 36px;
                                        ",
                                        "{sig}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── Column signatures — diagonal text below each column ───────
            if props.show_signatures {
                // Rotated text extends upward by ≈ n_rows*8*sin(45°) ≈ n_rows*6 px;
                // margin-top compensates so it doesn't overlap the board cells.
                {
                let col_sig_margin = n_rows * 6;
                rsx! {
                div {
                    style: "display: flex; flex-direction: row; gap: {GAP}px; margin-top: {col_sig_margin}px;",
                    for c in 0..n_cols {
                        {
                            let sig = col_sig(&board, c);
                            // Each cell is CELL px wide; the rotated text pivots from top-left.
                            rsx! {
                                div {
                                    key: "cs-{c}",
                                    style: "
                                        width: {CELL}px;
                                        height: 72px;
                                        position: relative;
                                        overflow: visible;
                                    ",
                                    span {
                                        style: "
                                            position: absolute;
                                            top: 4px;
                                            left: 6px;
                                            font-size: 13px;
                                            font-family: monospace;
                                            color: #909090;
                                            white-space: nowrap;
                                            transform: rotate(-45deg);
                                            transform-origin: top left;
                                        ",
                                        "{sig}"
                                    }
                                }
                            }
                        }
                    }
                }
                }} // close rsx! block and col_sig_margin let-block
            }
        }
    }
}

fn row_sig(board: &BitBoard, r: usize) -> String {
    (0..board.width()).map(|c| match board.get((r, c)) {
        Cell::Red     => '1',
        Cell::Blue    => '0',
        Cell::Nothing => '·',
    }).collect()
}

fn col_sig(board: &BitBoard, c: usize) -> String {
    (0..board.height()).map(|r| match board.get((r, c)) {
        Cell::Red     => '1',
        Cell::Blue    => '0',
        Cell::Nothing => '·',
    }).collect()
}

fn technique_label(reason: &StepReason) -> &'static str {
    match reason {
        StepReason::Technique(Technique::PairExtension)  => "PE",
        StepReason::Technique(Technique::GapFill)        => "GF",
        StepReason::Technique(Technique::Saturation)     => "Sat",
        StepReason::Technique(Technique::TwinCompletion) => "TC",
        StepReason::LineForced                           => "V2",
    }
}

// ── Cell ──────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct CellViewProps {
    r: usize,
    c: usize,
    cell: Cell,
    locked: bool,
    /// Draw a gold highlight outline around this cell (e.g. a forced deduction).
    #[props(default)]
    outlined: bool,
    /// When `Some`, the cell is part of the overlay: (forced color, technique label).
    forced: Option<(Cell, &'static str)>,
    on_click: EventHandler<(usize, usize)>,
    #[props(default)]
    on_right_click: Option<EventHandler<(usize, usize)>>,
}

#[component]
fn CellView(props: CellViewProps) -> Element {
    let r = props.r;
    let c = props.c;
    let locked = props.locked;

    // If forced, display the forced color; otherwise the board's current color.
    let display_cell = props.forced.map(|(col, _)| col).unwrap_or(props.cell);

    let bg = cell_bg(display_cell, locked);
    let outline = if props.forced.is_some() || props.outlined {
        "2px solid #f0c040"
    } else if locked {
        "2px solid #484848"
    } else {
        "2px solid #383838"
    };
    let opacity = if locked && props.cell == Cell::Nothing { "0.45" } else { "1.0" };
    let cursor  = if locked { "default" } else { "pointer" };
    let style = format!(
        "width: {CELL}px; height: {CELL}px; \
         background: {bg}; outline: {outline}; outline-offset: -2px; \
         border-radius: 10px; opacity: {opacity}; cursor: {cursor}; \
         position: relative; \
         display: flex; align-items: center; justify-content: center; flex-direction: column; gap: 2px;"
    );

    rsx! {
        div {
            style: "{style}",
            // Handle both buttons on mousedown so there's no contextmenu round-trip delay.
            onmousedown: move |e| {
                use dioxus::html::input_data::MouseButton;
                if locked { return; }
                match e.trigger_button() {
                    Some(MouseButton::Primary)   => props.on_click.call((r, c)),
                    Some(MouseButton::Secondary) => {
                        if let Some(h) = &props.on_right_click { h.call((r, c)); }
                    }
                    _ => {}
                }
            },
            // Suppress the native context menu without any game logic here.
            oncontextmenu: move |e| { e.prevent_default(); },

            // Technique label + forced-color chip
            if let Some((forced_color, label)) = props.forced {
                span {
                    style: "font-size: 11px; font-weight: 700; letter-spacing: .03em; color: #ffffff; text-shadow: 0 1px 3px rgba(0,0,0,0.7); pointer-events: none;",
                    "{label}"
                }
                span {
                    style: "font-size: 10px; font-weight: 600; color: {color_name(forced_color)}; text-shadow: 0 1px 2px rgba(0,0,0,0.6); pointer-events: none;",
                    "{cell_char(forced_color)}"
                }
            }
        }
    }
}

fn cell_bg(cell: Cell, locked: bool) -> &'static str {
    match cell {
        Cell::Red     => if locked { "#c03040" } else { "#ff3355" },
        Cell::Blue    => if locked { "#1466bb" } else { "#1a80ff" },
        Cell::Nothing => "#2e2e2e",
    }
}

fn color_name(cell: Cell) -> &'static str {
    match cell {
        Cell::Red     => "#ffaaaa",
        Cell::Blue    => "#aad4ff",
        Cell::Nothing => "#888888",
    }
}

fn cell_char(cell: Cell) -> &'static str {
    match cell { Cell::Red => "R", Cell::Blue => "B", Cell::Nothing => "." }
}
