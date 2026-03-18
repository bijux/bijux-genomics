use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 4,
            base_delay: Duration::from_millis(50),
            max_delay: Duration::from_millis(500),
        }
    }
}

pub trait Clock {
    fn sleep(&self, duration: Duration);
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

/// Retry an operation with exponential backoff.
///
/// # Errors
/// Returns the last error from the operation after exhausting retries.
pub fn retry_with<T, E, F, C>(policy: &RetryPolicy, clock: &C, mut op: F) -> Result<T, E>
where
    F: FnMut(u32) -> Result<T, E>,
    C: Clock,
{
    let mut attempt = 0;
    loop {
        attempt += 1;
        match op(attempt) {
            Ok(value) => return Ok(value),
            Err(_err) if attempt < policy.max_attempts => {
                let delay = backoff_delay(policy, attempt);
                clock.sleep(delay);
            }
            Err(err) => return Err(err),
        }
    }
}

#[must_use]
pub fn backoff_delay(policy: &RetryPolicy, attempt: u32) -> Duration {
    let shift = attempt.saturating_sub(1).min(31);
    let pow = 1u32 << shift;
    let delay = policy.base_delay.saturating_mul(pow);
    delay.min(policy.max_delay)
}
