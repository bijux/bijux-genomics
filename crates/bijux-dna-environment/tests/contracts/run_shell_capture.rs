use bijux_dna_environment::resolve::run_shell_capture;

#[test]
fn run_shell_capture_preserves_stderr_on_success() {
    let output = run_shell_capture("printf 'stdout\\n'; printf 'stderr\\n' >&2")
        .unwrap_or_else(|err| panic!("capture success output: {err}"));
    assert!(output.contains("stdout"));
    assert!(output.contains("stderr"));
}

#[test]
fn run_shell_capture_preserves_stderr_on_failure() {
    let error = run_shell_capture("printf 'stdout\\n'; printf 'stderr\\n' >&2; exit 7")
        .err()
        .unwrap_or_else(|| panic!("expected command failure"));
    let message = error.to_string();
    assert!(message.contains("stdout"));
    assert!(message.contains("stderr"));
}

#[test]
fn run_shell_capture_rejects_empty_command() {
    let error = run_shell_capture("  ")
        .err()
        .unwrap_or_else(|| panic!("expected empty command rejection"));
    assert!(error.to_string().contains("empty command"));
}
