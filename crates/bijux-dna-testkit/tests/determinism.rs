#[test]
fn fixed_clock_returns_configured_unix_time() {
    let clock = bijux_dna_testkit::FixedClock::unix_s(1_234);

    assert_eq!(
        clock.now(),
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_234)
    );
}

#[test]
fn fixed_rng_replays_seeded_sequence() {
    use rand::RngCore;

    let mut left = bijux_dna_testkit::fixed_rng(42);
    let mut right = bijux_dna_testkit::fixed_rng(42);

    assert_eq!(left.next_u64(), right.next_u64());
    assert_eq!(left.next_u64(), right.next_u64());
}

#[test]
fn strip_timestamp_fields_removes_nested_configured_keys() {
    let raw = serde_json::json!({
        "stable": true,
        "created_at": "2024-01-01T00:00:00Z",
        "nested": {
            "updated_at": "2024-01-02T00:00:00Z",
            "value": 7
        }
    });

    let stripped = bijux_dna_testkit::strip_timestamp_fields(&raw, &["created_at", "updated_at"]);

    assert_eq!(
        stripped,
        serde_json::json!({
            "stable": true,
            "nested": {
                "value": 7
            }
        })
    );
}

#[test]
fn assert_stable_ordering_accepts_sorted_values() {
    bijux_dna_testkit::assert_stable_ordering(&[1, 2, 3]);
}

#[test]
fn assert_stable_ordering_rejects_unsorted_values() {
    let result = std::panic::catch_unwind(|| bijux_dna_testkit::assert_stable_ordering(&[2, 1, 3]));

    assert!(result.is_err(), "unsorted values must be rejected");
}
