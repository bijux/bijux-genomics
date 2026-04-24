use std::sync::{Arc, Mutex};
use std::time::Duration;

struct FakeClock {
    sleeps: Arc<Mutex<Vec<Duration>>>,
}

impl bijux_dna_infra::Clock for FakeClock {
    fn sleep(&self, duration: Duration) {
        if let Ok(mut guard) = self.sleeps.lock() {
            guard.push(duration);
        }
    }
}

#[test]
fn retry_policy_is_deterministic() -> anyhow::Result<()> {
    let sleeps = Arc::new(Mutex::new(Vec::new()));
    let clock = FakeClock { sleeps: sleeps.clone() };
    let policy = bijux_dna_infra::RetryPolicy {
        max_attempts: 3,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(40),
    };
    let mut attempts = 0;
    let _ = bijux_dna_infra::retry_with(&policy, &clock, |_| {
        attempts += 1;
        Err::<(), _>(bijux_dna_infra::IoError::new(bijux_dna_infra::IoErrorKind::Transient, "fail"))
    });
    assert_eq!(attempts, 3);
    let recorded = sleeps.lock().map_err(|_| anyhow::anyhow!("lock poisoned"))?.clone();
    assert_eq!(recorded, vec![Duration::from_millis(10), Duration::from_millis(20)]);
    Ok(())
}
