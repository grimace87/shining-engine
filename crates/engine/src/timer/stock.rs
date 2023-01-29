
use crate::Timer;
use std::time::Instant;

pub struct StockTimer {
    last_update_time: Instant
}

impl StockTimer {
    pub fn new() -> Self {
        Self {
            last_update_time: Instant::now()
        }
    }
}

impl Timer for StockTimer {
    fn pull_time_step_millis(&mut self) -> u64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update_time);
        self.last_update_time = now;
        elapsed.as_millis() as u64
    }
}
