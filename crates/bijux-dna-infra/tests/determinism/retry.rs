use std::time::Duration;

#[test]
fn retry_backoff_plan_is_deterministic_without_executing_delays() {
    let policy = bijux_dna_infra::RetryPolicy {
        max_attempts: 5,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(40),
    };
    let plan = (1..policy.max_attempts)
        .map(|attempt| bijux_dna_infra::backoff_delay(&policy, attempt))
        .collect::<Vec<_>>();

    assert_eq!(
        plan,
        vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(40),
            Duration::from_millis(40),
        ]
    );
}
