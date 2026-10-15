#![allow(clippy::expect_used)]

use crate::fixture_process::run_cli;

#[test]
fn fixtures_validate_expected_bam_haplogroup_truth_prints_expected_path() {
    let output = run_cli(&["fixtures", "validate-expected", "--corpus", "haplogroup-truth"]);

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "benchmarks/tests/fixtures/science/haplogroup-truth/expected.json"
    );
}
