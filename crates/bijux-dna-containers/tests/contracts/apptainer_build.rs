#![cfg(unix)]

use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use bijux_dna_containers::api::{
    apptainer_build_log_path, build_apptainer_image, classify_apptainer_build_failure,
    latest_apptainer_build_attempt, next_apptainer_build_attempt, ApptainerBuildFailureClass,
    ApptainerBuildRequest,
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

#[test]
fn bbmap_recipes_remove_group_and_other_write_bits() -> anyhow::Result<()> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for path in [
        "containers/apptainer/bbduk.def",
        "containers/apptainer/bbmerge.def",
        "containers/apptainer/clumpify.def",
        "containers/docker/Dockerfile.arm64.bbduk",
        "containers/docker/Dockerfile.arm64.bbmerge",
        "containers/docker/Dockerfile.arm64.clumpify",
    ] {
        let recipe = std::fs::read_to_string(workspace.join(path))?;
        assert!(
            recipe.contains("chmod -R go-w /opt/bbmap"),
            "BBMap recipe does not normalize archive permissions: {path}"
        );
    }
    Ok(())
}

#[test]
fn bijux_dna_apptainer_recipe_removes_build_toolchains() -> anyhow::Result<()> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let recipe = std::fs::read_to_string(workspace.join("containers/apptainer/bijux_dna.def"))?;

    assert!(recipe.contains("rm -rf /opt/bijux-src /opt/rust /root/.cargo"));
    Ok(())
}

#[test]
fn seqpurge_recipe_runs_help_probe_from_private_writable_directory() -> anyhow::Result<()> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let recipe = std::fs::read_to_string(workspace.join("containers/apptainer/seqpurge.def"))?;
    let test_section = recipe
        .split_once("%test\n")
        .map(|(_, section)| section.split_once("\n%help").map_or(section, |(body, _)| body))
        .ok_or_else(|| anyhow::anyhow!("SeqPurge recipe must retain a test section"))?;

    assert!(test_section.contains("mktemp -d /tmp/bijux-seqpurge-test.XXXXXX"));
    assert!(test_section.contains("cd \"$seqpurge_test_dir\""));
    assert!(test_section.contains("seqpurge --help >/dev/null"));
    assert!(!test_section.contains("|| true"));
    Ok(())
}

#[test]
fn seqpurge_wrapper_rejects_an_unusable_inherited_tmpdir() -> anyhow::Result<()> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let wrapper = workspace.join("containers/tools/seqpurge/seqpurge");
    let temporary = tempfile::tempdir()?;
    let unusable_tmpdir = temporary.path().join("not-a-directory");
    std::fs::write(&unusable_tmpdir, b"hostile scheduler TMPDIR")?;

    let output = Command::new("sh")
        .arg(wrapper)
        .arg("--version")
        .env("TMPDIR", unusable_tmpdir)
        .env_remove("XDG_RUNTIME_DIR")
        .output()?;

    assert!(output.status.success());
    assert_eq!(output.stdout, b"seqpurge 2026_06\n");
    assert!(output.stderr.is_empty());
    Ok(())
}

#[test]
fn alientrimmer_source_download_retries_transient_resets_within_a_bounded_build(
) -> anyhow::Result<()> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let recipe = std::fs::read_to_string(workspace.join("containers/apptainer/alientrimmer.def"))?;

    assert!(recipe.contains("--retry 5 --retry-delay 3 --retry-all-errors"));
    assert!(recipe.contains("--connect-timeout 20 --max-time 300 --retry-max-time 300"));
    assert!(recipe.contains(
        "echo \"${ALIENTRIMMER_SHA256}  /tmp/alientrimmer.tar.gz\" | sha256sum --check --strict"
    ));
    Ok(())
}

#[test]
fn alientrimmer_license_authorities_preserve_upstream_agpl_identity() -> anyhow::Result<()> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let catalog: toml::Value = toml::from_str(&std::fs::read_to_string(
        workspace.join("containers/contracts/catalog.toml"),
    )?)?;
    let catalog_tool = catalog
        .get("tools")
        .and_then(toml::Value::as_array)
        .and_then(|tools| {
            tools
                .iter()
                .find(|tool| tool.get("id").and_then(toml::Value::as_str) == Some("alientrimmer"))
        })
        .ok_or_else(|| anyhow::anyhow!("canonical catalog omits alientrimmer"))?;
    assert_eq!(catalog_tool.get("license").and_then(toml::Value::as_str), Some("AGPL-3.0-only"));

    let versions: toml::Value = toml::from_str(&std::fs::read_to_string(
        workspace.join("containers/versions/versions.toml"),
    )?)?;
    assert_eq!(
        versions
            .get("alientrimmer")
            .and_then(|tool| tool.get("upstream_license"))
            .and_then(toml::Value::as_str),
        Some("AGPL-3.0-only")
    );

    for path in
        ["containers/apptainer/alientrimmer.def", "containers/docker/Dockerfile.arm64.alientrimmer"]
    {
        let recipe = std::fs::read_to_string(workspace.join(path))?;
        assert!(
            recipe.contains("org.opencontainers.image.licenses AGPL-3.0-only")
                || recipe.contains("org.opencontainers.image.licenses=\"AGPL-3.0-only\""),
            "AlienTrimmer recipe does not preserve upstream AGPL identity: {path}"
        );
    }
    Ok(())
}

#[test]
fn bowtie2_wrapper_derives_and_removes_a_scratch_reference_index() -> anyhow::Result<()> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let wrapper = std::fs::read_to_string(workspace.join("containers/tools/bowtie2/bowtie2"))?;

    assert!(wrapper.contains("--bijux-align-reference"));
    assert!(wrapper.contains("mktemp -d /scratch/bijux-bowtie2-index.XXXXXX"));
    assert!(wrapper.contains("trap 'rm -rf \"$index_root\"' EXIT HUP INT TERM"));
    assert!(wrapper.contains("/opt/bowtie2/bowtie2-build --threads \"$threads\""));
    assert!(wrapper.contains("/opt/bowtie2/bowtie2 -x \"$index_root/reference\" \"$@\""));
    for recipe in ["containers/apptainer/bowtie2.def", "containers/docker/Dockerfile.arm64.bowtie2"]
    {
        let recipe = std::fs::read_to_string(workspace.join(recipe))?;
        assert!(recipe.contains("containers/tools/bowtie2/bowtie2"));
        assert!(recipe.contains("bowtie2-build"));
    }
    Ok(())
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
    })
}

#[test]
fn apptainer_build_lifecycle_is_one_shot() -> anyhow::Result<()> {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/build/apptainer.rs"),
    )?;

    assert!(!source.contains("thread::sleep"));
    assert!(!source.contains("maximum_invocations"));
    assert!(!source.contains("retry_delay"));
    Ok(())
}

#[test]
fn transient_build_failure_requires_a_new_one_shot_invocation() -> anyhow::Result<()> {
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

    let Err(first_error) = build_apptainer_image(&request) else {
        panic!("transient failure must return after one attempt");
    };
    assert!(first_error.to_string().contains("TransientTransport"));
    let first_log =
        apptainer_build_log_path(&request.evidence_root, "samtools", &"a".repeat(64), 1);
    assert!(std::fs::read_to_string(first_log)?.contains("failure_class=transient_transport"));

    let report = build_apptainer_image(&request)?;

    assert_eq!(report.attempts.len(), 1);
    assert_eq!(report.attempts[0].attempt, 2);
    assert_eq!(report.attempts[0].failure_class, None);
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
        "curl: (18) transfer closed with 4096 bytes remaining to read",
        "curl: (56) Recv failure: Connection reset by peer",
        "HTTP 429 Too Many Requests",
        "HTTP status 500",
        "upstream returned status code 502",
        "curl: (22) The requested URL returned error: 503",
        "response status: 504 Gateway Timeout",
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
    for diagnostics in [
        "curl: (22) The requested URL returned error: 404",
        "curl: (22) request failed",
        "HTTP 401 Unauthorized",
    ] {
        assert_eq!(
            classify_apptainer_build_failure(diagnostics),
            ApptainerBuildFailureClass::Permanent
        );
    }
}
