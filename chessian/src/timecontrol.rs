use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug)]
pub struct TimeControl {
    stop_flag: Option<Arc<AtomicBool>>,
    mode: TCMode,
}

#[derive(Clone, Debug)]
pub enum TCMode {
    MoveTime(u128),
    Depth(usize),
    Infinite,
}

impl TimeControl {
    pub fn new(stop_flag: Option<Arc<AtomicBool>>, mode: TCMode) -> Self {
        Self { stop_flag, mode }
    }

    pub fn should_stop(&self, elapsed: u128, reached_depth: usize) -> bool {
        if self
            .stop_flag
            .as_ref()
            .map(|b| b.load(Ordering::Relaxed))
            .unwrap_or(false)
        {
            true
        } else {
            match self.mode {
                TCMode::MoveTime(millis) => elapsed >= millis,
                TCMode::Depth(depth) => reached_depth >= depth,
                TCMode::Infinite => false,
            }
        }
    }
}
