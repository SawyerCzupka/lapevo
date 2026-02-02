use std::fmt::Display;
use tokio::sync::watch;

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
    pub fn new(rx: watch::Receiver<PositionState>) -> Self {
        Self { rx }
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
}
