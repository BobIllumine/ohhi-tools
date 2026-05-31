pub mod analysis;
pub mod history;
pub mod play;
pub mod practice;
pub mod timer;
pub mod trace;

use analysis::AnalysisSession;
use history::DrillRecord;
use play::PlaySession;
use practice::PracticeSession;

pub use play::GameRecord;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Screen {
    Play,
    Analysis,
    Practice,
    Patterns,
    Stats,
}

pub struct AppState {
    pub screen:       Screen,
    pub analysis:     AnalysisSession,
    pub play:             Option<PlaySession>,
    pub play_history:     Vec<GameRecord>,
    pub practice:         Option<PracticeSession>,
    pub practice_history: Vec<DrillRecord>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            screen:       Screen::Analysis,
            analysis:     AnalysisSession::new(4, 4),
            play:             None,
            play_history:     Vec::new(),
            practice:         None,
            practice_history: Vec::new(),
        }
    }

    /// If the active practice drill just finished, append its record to
    /// `practice_history` (once) and return it so the shell can persist it.
    pub fn record_drill_if_done(&mut self, epoch_ms: u64) -> Option<&DrillRecord> {
        let rec = self.practice.as_mut()?.take_record(epoch_ms)?;
        self.practice_history.push(rec);
        self.practice_history.last()
    }

    /// If the active play session is complete, move it into `play_history`
    /// and clear `play`. No-op if the session is absent or incomplete.
    pub fn record_completion_if_done(&mut self) {
        if self.play.as_ref().map(|s| s.is_complete()).unwrap_or(false) {
            let record = GameRecord::from_session(self.play.as_ref().unwrap());
            self.play_history.push(record);
            self.play = None;
        }
    }
}

impl Default for AppState {
    fn default() -> Self { Self::new() }
}
