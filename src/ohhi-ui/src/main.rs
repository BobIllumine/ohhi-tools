use dioxus::prelude::*;
use ohhi_app::{AppState, Screen};

mod components;
mod storage;
use components::analysis::AnalysisView;
use components::play::PlayView;
use components::patterns::PatternsView;
use components::practice::PracticeView;
use components::stats::StatsView;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut state = use_signal(|| {
        let mut s = AppState::new();
        s.practice_history = storage::load();
        s
    });
    // Light mode for outdoor / bright-sun use. The whole UI is styled dark with
    // hardcoded colours, so rather than re-theme every component we invert the
    // root; hue-rotate(180) keeps red/blue cells roughly their original hue.
    let mut light = use_signal(|| false);
    let root_filter = if light() { "filter: invert(1) hue-rotate(180deg);" } else { "" };

    rsx! {
        style { r#"
            *, *::before, *::after {{ box-sizing: border-box; }}
            html, body {{
                margin: 0; padding: 0;
                width: 100vw; height: 100vh;
                overflow: hidden;
                background: #1e1e1e;
                color: #e8e8e8;
                font-family: system-ui, sans-serif;
                font-size: 14px;
            }}
            button {{
                font-size: 13px;
                font-family: inherit;
                cursor: pointer;
            }}
            input, textarea {{
                font-family: monospace;
                font-size: 13px;
                color: #e8e8e8;
            }}
            ::-webkit-scrollbar {{ width: 6px; height: 6px; }}
            ::-webkit-scrollbar-track {{ background: #161616; }}
            ::-webkit-scrollbar-thumb {{ background: #404040; border-radius: 3px; }}
        "# }

        div {
            style: "display: flex; flex-direction: column; width: 100vw; height: 100vh; background: #1e1e1e; {root_filter}",

            // Top tab bar
            div {
                style: "display: flex; align-items: center; gap: 4px; padding: 6px 14px; background: #141414; border-bottom: 1px solid #2e2e2e; flex-shrink: 0;",
                span { style: "font-weight: 700; color: #e0e0e0; margin-right: 14px; font-size: 15px; letter-spacing: -0.3px;", "0h h1 Toolkit" }
                Tab { label: "Analysis", screen: Screen::Analysis, current: state.read().screen.clone(), onclick: move |_| state.write().screen = Screen::Analysis }
                Tab { label: "Play",     screen: Screen::Play,     current: state.read().screen.clone(), onclick: move |_| state.write().screen = Screen::Play }
                Tab { label: "Practice", screen: Screen::Practice, current: state.read().screen.clone(), onclick: move |_| state.write().screen = Screen::Practice }
                Tab { label: "Patterns", screen: Screen::Patterns, current: state.read().screen.clone(), onclick: move |_| state.write().screen = Screen::Patterns }
                Tab { label: "Stats",    screen: Screen::Stats,    current: state.read().screen.clone(), onclick: move |_| state.write().screen = Screen::Stats }
                // Push the theme toggle to the far right.
                div { style: "flex: 1;" }
                button {
                    style: "padding: 5px 12px; border-radius: 6px; border: 1px solid #404040; background: #2a2a2a; color: #e8e8e8;",
                    onclick: move |_| { let v = light(); light.set(!v); },
                    if light() { "🌙 Dark" } else { "☀ Light" }
                }
            }

            div {
                style: "flex: 1; overflow: hidden;",
                match state.read().screen {
                    Screen::Analysis => rsx! { AnalysisView { state } },
                    Screen::Play     => rsx! { PlayView { state } },
                    Screen::Practice => rsx! { PracticeView { state } },
                    Screen::Patterns => rsx! { PatternsView {} },
                    Screen::Stats    => rsx! { StatsView { state } },
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct TabProps {
    label: &'static str,
    screen: Screen,
    current: Screen,
    onclick: EventHandler<()>,
}

#[component]
fn Tab(props: TabProps) -> Element {
    let active = props.screen == props.current;
    let bg  = if active { "#2a2a2a" } else { "transparent" };
    let fg  = if active { "#e8e8e8" } else { "#707070" };
    let fw  = if active { "600" } else { "400" };
    let bb  = if active { "2px solid #a0a0a0" } else { "2px solid transparent" };
    let sty = format!("padding: 5px 16px; border-radius: 6px 6px 0 0; border: none; background: {bg}; color: {fg}; font-weight: {fw}; border-bottom: {bb};");
    rsx! {
        button { style: "{sty}", onclick: move |_| props.onclick.call(()), "{props.label}" }
    }
}

#[component]
fn Placeholder(title: &'static str) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: center; height: 100%; color: #555;",
            h2 { "{title} — coming soon" }
        }
    }
}
