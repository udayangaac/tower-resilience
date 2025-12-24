use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Policy {
    pub timeout: Duration,
    pub max_retries: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub max_inflight: usize,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(2),
            max_retries: 2,
            base_delay: Duration::from_millis(50),
            max_delay: Duration::from_millis(500),
            max_inflight: 100,
        }
    }
}