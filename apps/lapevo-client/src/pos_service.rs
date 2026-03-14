use std::fmt::Display;
use tokio::sync::watch;

#[derive(Clone)]
pub struct PositionService {
    rx: watch::Receiver<PositionState>,
    // last_state: Box<PositionState>,
}

#[derive(Debug, Clone, Default)]
pub struct PositionState {
    pub lap_dist_pct: f32,
    pub lap_number: i32,
}

impl Display for PositionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PosState (Pct: {}, Num: {})",
            self.lap_dist_pct, self.lap_number
        )
    }
}

impl PositionService {
    pub fn new() -> (Self, tokio::sync::watch::Sender<PositionState>) {
        let (tx, rx) = watch::channel(PositionState::default());

        (Self { rx }, tx)
    }

    pub async fn wait_until_position(&mut self, target_pct: f32) -> PositionState {
        loop {
            if self.rx.changed().await.is_err() {
                break self.rx.borrow().clone();
            }

            let state = self.rx.borrow().clone();
            if state.lap_dist_pct >= target_pct {
                return state;
            }
        }
    }

    /// Wait until the lap number advances beyond the current lap.
    /// Useful for scheduling actions on the next lap.
    pub async fn wait_for_next_lap(&mut self) -> PositionState {
        let current_lap = self.rx.borrow().lap_number;
        loop {
            if self.rx.changed().await.is_err() {
                return self.rx.borrow().clone();
            }
            let state = self.rx.borrow().clone();
            if state.lap_number > current_lap {
                return state;
            }
        }
    }
}
