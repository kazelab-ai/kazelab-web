//! Actor Supervision Strategies: One-For-One and One-For-All Restart Protocols.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestartStrategy {
    OneForOne,
    OneForAll,
    EscalateToRoot,
}

pub struct SupervisorDirective {
    pub max_restarts_within_window: usize,
    pub window_duration_seconds: u64,
    pub strategy: RestartStrategy,
}

impl SupervisorDirective {
    pub fn default_resilient() -> Self {
        Self {
            max_restarts_within_window: 5,
            window_duration_seconds: 60,
            strategy: RestartStrategy::OneForOne,
        }
    }

    pub fn should_restart(&self, consecutive_failures: usize) -> bool {
        consecutive_failures <= self.max_restarts_within_window
    }
}
