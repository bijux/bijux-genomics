#![cfg(unix)]

use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bijux_dna_containers_qa::api::{run_apptainer_smoke, ApptainerSmokeRequest};

fn write_executable(path: &Path, body: &str) -> anyhow::Result<()> {
    std::fs::write(path, body)?;
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

fn smoke_request(
    root: &Path,
    executable: PathBuf,
    expected_version_regex: &str,
) -> anyhow::Result<ApptainerSmokeRequest> {
    let image_path = root.join("tool.sif");
    let data_root = root.join("input");
    let results_root = root.join("output");
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
        timeout: Duration::from_secs(10),
    })
}

#[test]
fn contract_regex_admits_real_non_semver_versions() -> anyhow::Result<()> {
    for (output, expected_regex) in
        [("ibdseq r1206", "ibdseq r1206"), ("seqpurge 2026_06", "2026_06")]
    {
        let temp = bijux_dna_testkit::tempdir_for("apptainer-smoke-non-semver");
        let executable = temp.path().join("apptainer");
        write_executable(
            &executable,
            &format!(
                "#!/bin/sh\ncase \"$*\" in\n  *--version*) printf '%s\\n' '{output}'; printf '%s\\n' 'runtime diagnostic' >&2 ;;\n  *--help*) printf '%s\\n' 'usage' ;;\n  *) exit 2 ;;\nesac\n"
            ),
        )?;
        let request = smoke_request(temp.path(), executable, expected_regex)?;

        let manifest = run_apptainer_smoke(&request)?;

        assert_eq!(manifest.status, "ok");
        assert!(manifest.version_matches_contract);
        assert_eq!(manifest.version, expected_regex);
    }
    Ok(())
}

#[test]
fn smoke_rejects_version_drift_invalid_regex_and_command_failure() -> anyhow::Result<()> {
    let temp = bijux_dna_testkit::tempdir_for("apptainer-smoke-contract-failure");
    let executable = temp.path().join("apptainer");
    write_executable(
        &executable,
        "#!/bin/sh\ncase \"$*\" in\n  *--version*) echo 'tool 1.0' ;;\n  *--help*) exit 4 ;;\nesac\n",
    )?;
    let request = smoke_request(temp.path(), executable.clone(), "tool 2\\.0")?;
    let manifest = run_apptainer_smoke(&request)?;
    assert_eq!(manifest.status, "wrapper_failed");
    assert!(!manifest.version_matches_contract);
    assert!(!manifest.help_ok);

    let invalid_regex = smoke_request(temp.path(), executable, "[")?;
    assert!(run_apptainer_smoke(&invalid_regex).is_err());
    Ok(())
}

#[test]
fn smoke_bounds_execution_and_persists_immutable_streams() -> anyhow::Result<()> {
    let temp = bijux_dna_testkit::tempdir_for("apptainer-smoke-timeout-evidence");
    let executable = temp.path().join("apptainer");
    write_executable(
        &executable,
        "#!/bin/sh\nprintf 'partial output\\n'\nprintf 'partial error\\n' >&2\nexec tail -f /dev/null\n",
    )?;
    let mut request = smoke_request(temp.path(), executable, "contract-tool 1[.]0")?;
    request.timeout = Duration::from_millis(100);
    request.evidence_root = Some(temp.path().join("evidence"));

    let manifest = run_apptainer_smoke(&request)?;

    assert_eq!(manifest.status, "wrapper_failed");
    assert!(manifest.probe_results.iter().all(|probe| probe.timed_out));
    assert_eq!(std::fs::read_dir(temp.path().join("evidence"))?.count(), 4);
    Ok(())
}
