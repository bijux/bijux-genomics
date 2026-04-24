#[test]
fn guardrails() {
    let clock = bijux_dna_testkit::FixedClock::unix_s(42);
    assert_eq!(clock.now(), std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(42));
}
