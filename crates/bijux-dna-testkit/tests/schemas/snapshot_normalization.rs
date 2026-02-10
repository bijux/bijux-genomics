use std::env;

#[test]
fn snapshot_normalize_text_normalizes_windows_paths_and_line_endings() {
    let raw = "C:\\Users\\alice\\repo\\file.txt\r\nnext";
    let normalized = bijux_dna_testkit::snapshot_normalize_text(raw);
    assert!(normalized.contains("C:/Users/alice/repo/file.txt"));
    assert!(!normalized.contains('\r'));
}

#[test]
fn install_snapshot_env_enforces_locale_and_timezone() {
    env::set_var("TZ", "Europe/Berlin");
    env::set_var("LC_ALL", "de_DE.UTF-8");
    bijux_dna_testkit::install_snapshot_env();
    assert_eq!(env::var("TZ").unwrap_or_default(), "UTC");
    assert_eq!(env::var("LC_ALL").unwrap_or_default(), "C");
}
