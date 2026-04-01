use std::time::Duration;

use super::RetryPolicy;

#[must_use]
pub fn backoff_delay(policy: &RetryPolicy, attempt: u32) -> Duration {
    let shift = attempt.saturating_sub(1).min(31);
    let pow = 1u32 << shift;
    let delay = policy.base_delay.saturating_mul(pow);
    delay.min(policy.max_delay)
}
