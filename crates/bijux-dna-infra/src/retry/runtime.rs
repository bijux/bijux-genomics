use super::{backoff_delay, Clock, RetryPolicy};

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
