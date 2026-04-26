#[test]
fn guardrails() {
    let clock = bijux_dna_testkit::FixedClock::unix_s(42);
    assert_eq!(clock.now(), std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(42));
}

#[test]
fn resolve_under_rejects_absolute_paths() {
    let result = std::panic::catch_unwind(|| bijux_dna_testkit::resolve_under("/outside"));
    assert!(result.is_err(), "absolute paths must not escape the temp root");
}

#[test]
fn resolve_under_rejects_parent_traversal() {
    let result = std::panic::catch_unwind(|| bijux_dna_testkit::resolve_under("../outside"));
    assert!(result.is_err(), "parent traversal must not escape the temp root");
}

#[test]
fn test_paths_child_rejects_absolute_paths() {
    let paths = bijux_dna_testkit::TestPaths::new("absolute-child");
    let result = std::panic::catch_unwind(|| paths.child("/outside"));
    assert!(result.is_err(), "absolute child paths must not escape the test root");
}

#[test]
fn test_paths_child_rejects_parent_traversal() {
    let paths = bijux_dna_testkit::TestPaths::new("parent-child");
    let result = std::panic::catch_unwind(|| paths.child("../outside"));
    assert!(result.is_err(), "parent traversal must not escape the test root");
}
