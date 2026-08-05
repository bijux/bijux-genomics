#![cfg(unix)]

use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bijux_dna_environment::api::{
    apptainer_build_log_path, build_apptainer_image, classify_apptainer_build_failure,
    latest_apptainer_build_attempt, next_apptainer_build_attempt, run_apptainer_smoke,
    ApptainerBuildFailureClass, ApptainerBuildRequest, ApptainerSmokeRequest,
};

fn write_executable(path: &Path, body: &str) -> anyhow::Result<()> {
    std::fs::write(path, body)?;
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

fn shell_literal(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}

fn build_request(
    temp: &tempfile::TempDir,
    executable: PathBuf,
) -> anyhow::Result<ApptainerBuildRequest> {
    let definition_path = temp.path().join("tool.def");
    std::fs::write(&definition_path, "Bootstrap: docker\nFrom: scratch\n")?;
    Ok(ApptainerBuildRequest {
        executable,
        definition_path,
        image_path: temp.path().join("tool.sif"),
        evidence_root: temp.path().join("evidence"),
        tool_id: "samtools".to_string(),
        build_evidence_id: "a".repeat(64),
        maximum_invocations: 3,
        retry_delay: Duration::ZERO,
    })
}

fn smoke_request(
    temp: &tempfile::TempDir,
    executable: PathBuf,
    expected_version_regex: &str,
) -> anyhow::Result<ApptainerSmokeRequest> {
    let image_path = temp.path().join("tool.sif");
    let data_root = temp.path().join("input");
    let results_root = temp.path().join("output");
    std::fs::write(&image_path, b"sif")?;
    std::fs::create_dir_all(&data_root)?;
    std::fs::create_dir_all(&results_root)?;
    Ok(ApptainerSmokeRequest {
        executable,
        image_path,
        data_root,
        results_root,
        tool_id: "contract-tool".to_string(),
        stage_id: "catalog.retained".to_string(),
        build_evidence_id: "a".repeat(64),
        repository_commit: "b".repeat(40),
        registry_digest: "c".repeat(64),
        image_sha256: "d".repeat(64),
        version_command: "contract-tool --version".to_string(),
        expected_version_regex: expected_version_regex.to_string(),
        help_command: "contract-tool --help".to_string(),
        require_help: true,
        probe_commands: Vec::new(),
        java_heap_mb: None,
        upstream: "catalog-declared".to_string(),
        evidence_root: None,
        timeout: Duration::from_secs(2),
    })
}

#[test]
fn transient_build_failure_retries_with_immutable_attempt_logs() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let executable = temp.path().join("apptainer");
    let counter = temp.path().join("counter");
    write_executable(
        &executable,
        &format!(
            "#!/bin/sh\ncount=0\n[ ! -f {counter} ] || count=$(cat {counter})\ncount=$((count + 1))\nprintf '%s' \"$count\" > {counter}\nif [ \"$count\" -eq 1 ]; then\n  echo 'conveyor failed to get: unexpected end of JSON input' >&2\n  exit 1\nfi\nprintf 'sif' > \"$3\"\n",
            counter = shell_literal(&counter),
        ),
    )?;
    let request = build_request(&temp, executable)?;

    let report = build_apptainer_image(&request)?;

    assert_eq!(report.attempts.len(), 2);
    assert_eq!(
        report.attempts[0].failure_class,
        Some(ApptainerBuildFailureClass::TransientTransport)
    );
    assert_eq!(report.attempts[1].failure_class, None);
    assert!(request.image_path.is_file());
    assert_eq!(
        latest_apptainer_build_attempt(&request.evidence_root, "samtools", &"a".repeat(64))
            .map(|value| value.0),
        Some(2)
    );
    assert_eq!(
        next_apptainer_build_attempt(&request.evidence_root, "samtools", &"a".repeat(64))?,
        3
    );
    Ok(())
}

#[test]
fn permanent_build_failure_is_not_retried() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let executable = temp.path().join("apptainer");
    write_executable(&executable, "#!/bin/sh\necho 'definition parse error' >&2\nexit 1\n")?;
    let request = build_request(&temp, executable)?;

    let Err(error) = build_apptainer_image(&request) else {
        panic!("permanent failure must reject");
    };

    assert!(error.to_string().contains("attempt 1"));
    assert!(
        apptainer_build_log_path(&request.evidence_root, "samtools", &"a".repeat(64), 1).is_file()
    );
    assert!(
        !apptainer_build_log_path(&request.evidence_root, "samtools", &"a".repeat(64), 2).exists()
    );
    Ok(())
}

#[test]
fn transport_classifier_is_explicit_and_fail_closed() {
    for diagnostics in [
        "unexpected end of JSON input",
        "TOOMANYREQUESTS: You have reached your unauthenticated pull rate limit",
    ] {
        assert_eq!(
            classify_apptainer_build_failure(diagnostics),
            ApptainerBuildFailureClass::TransientTransport
        );
    }
    assert_eq!(
        classify_apptainer_build_failure("definition parse error"),
        ApptainerBuildFailureClass::Permanent
    );
}

#[test]
fn contract_regex_admits_real_non_semver_versions() -> anyhow::Result<()> {
    for (output, expected_regex) in
        [("ibdseq r1206", "ibdseq r1206"), ("seqpurge 2026_06", "2026_06")]
    {
        let temp = tempfile::tempdir()?;
        let executable = temp.path().join("apptainer");
        write_executable(
            &executable,
            &format!(
                "#!/bin/sh\ncase \"$*\" in\n  *--version*) printf '%s\\n' '{output}'; printf '%s\\n' 'runtime diagnostic' >&2 ;;\n  *--help*) printf '%s\\n' 'usage' ;;\n  *) exit 2 ;;\nesac\n"
            ),
        )?;
        let request = smoke_request(&temp, executable, expected_regex)?;

        let manifest = run_apptainer_smoke(&request)?;

        assert_eq!(manifest.status, "ok");
        assert!(manifest.version_matches_contract);
        assert_eq!(manifest.version, expected_regex);
    }
    Ok(())
}

#[test]
fn smoke_rejects_version_drift_invalid_regex_and_command_failure() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let executable = temp.path().join("apptainer");
    write_executable(
        &executable,
        "#!/bin/sh\ncase \"$*\" in\n  *--version*) echo 'tool 1.0' ;;\n  *--help*) exit 4 ;;\nesac\n",
    )?;
    let request = smoke_request(&temp, executable.clone(), "tool 2\\.0")?;
    let manifest = run_apptainer_smoke(&request)?;
    assert_eq!(manifest.status, "wrapper_failed");
    assert!(!manifest.version_matches_contract);
    assert!(!manifest.help_ok);

    let invalid_regex = smoke_request(&temp, executable, "[")?;
    assert!(run_apptainer_smoke(&invalid_regex).is_err());
    Ok(())
}

#[test]
fn smoke_bounds_execution_and_persists_immutable_streams() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let executable = temp.path().join("apptainer");
    write_executable(
        &executable,
        "#!/bin/sh\nprintf 'partial output\\n'\nprintf 'partial error\\n' >&2\nsleep 5\n",
    )?;
    let mut request = smoke_request(&temp, executable, "contract-tool 1[.]0")?;
    request.timeout = Duration::from_millis(100);
    request.evidence_root = Some(temp.path().join("evidence"));

    let manifest = run_apptainer_smoke(&request)?;

    assert_eq!(manifest.status, "wrapper_failed");
    assert!(manifest.probe_results.iter().all(|probe| probe.timed_out));
    assert_eq!(std::fs::read_dir(temp.path().join("evidence"))?.count(), 4);
    Ok(())
}
