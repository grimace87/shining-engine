pub mod stock;

pub trait Timer {
    fn pull_time_step_millis(&mut self) -> u64;
}
