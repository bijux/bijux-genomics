//! Supported tag fallback must be recorded and queried under the image actually resolved.
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::build_qa_record;
use crate::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use crate::image_qa::records::{qa_already_passed, QaRecordStore};
use crate::image_qa::support::resolve_image_for_run;
use crate::image_qa::{ensure_image_qa_passed, QaStage};
use anyhow::{ensure, Result};
use bijux_dna_analyze::ImageQaOutcome;

const ROOT: &str = "BIJUX_IMAGE_QA_IDENTITY_FIXTURE";
const CASE: &str =
    "image_qa::records::builder::image_identity::fallback_records_and_queries_use_resolved_image";

#[test]
fn fallback_records_and_queries_use_resolved_image() -> Result<()> {
    if let Some(root) = std::env::var_os(ROOT) {
        ensure!(
            std::env::args().skip(1).collect::<Vec<_>>() == [CASE, "--exact", "--nocapture"],
            "exact identity child"
        );
        let root = PathBuf::from(root);
        ensure!(
            root.is_absolute() && root.canonicalize()? == root && std::env::current_dir()? == root,
            "owned child context"
        );
        return exercise_identity(&root);
    }
    let temporary = bijux_dna_testkit::tempdir_for("image-qa-resolved-identity");
    let root = temporary.path().canonicalize()?;
    let providers = root.join("providers");
    std::fs::create_dir(&providers)?;
    let docker = providers.join("docker");
    std::fs::write(&docker, "#!/bin/sh\n[ \"$1\" = image ] && [ \"$2\" = inspect ] || exit 9\nprintf '%s\\n' \"$3\" >> \"$BIJUX_IMAGE_QA_LOOKUPS\"\ncase \"$3\" in *@*) [ -f \"$BIJUX_IMAGE_QA_PIN_CONTROL\" ] ;; *) exit 0 ;; esac\n")?;
    std::fs::set_permissions(&docker, std::fs::Permissions::from_mode(0o700))?;
    let executable = std::env::current_exe()?.canonicalize()?;
    let digest = bijux_dna_infra::hash_file_sha256(&executable)?;
    let mut command = Command::new(&executable);
    command
        .args([CASE, "--exact", "--nocapture"])
        .current_dir(&root)
        .env(ROOT, &root)
        .env("PATH", &providers)
        .env_remove("BIJUX_SKIP_QA")
        .env("BIJUX_IMAGE_QA_LOOKUPS", root.join("docker-lookups.log"))
        .env("BIJUX_IMAGE_QA_PIN_CONTROL", root.join("pinned-image-present"))
        .env("HOME", &root)
        .env("TEST_TMP_DIR", &root)
        .env("TMPDIR", &root)
        .env("TMP", &root)
        .env("TEMP", &root);
    let result = bijux_dna_process::capture(
        &mut command,
        bijux_dna_process::CaptureLimits {
            timeout: Duration::from_secs(20),
            max_output_bytes: 65_536,
        },
        "isolated resolved-image identity",
    )?;
    ensure!(
        !result.timed_out && !result.output_limit_exceeded,
        "bounded identity child: {result:?}"
    );
    ensure!(bijux_dna_infra::hash_file_sha256(&executable)? == digest, "test executable drift");
    ensure!(
        result.status.success(),
        "identity child failed: stdout={} stderr={}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    ensure!(
        String::from_utf8_lossy(&result.stdout).contains("running 1 test"),
        "exact child executes"
    );
    Ok(())
}

fn exercise_identity(root: &Path) -> Result<()> {
    let platform = PlatformSpec {
        name: "identity-fixture".to_string(),
        runner: RuntimeKind::Docker,
        container_dir: root.join("containers"),
        image_prefix: "fixture".to_string(),
        arch: "amd64".to_string(),
    };
    let spec = ToolImageSpec {
        tool: "seqkit".to_string(),
        version: "1.0".to_string(),
        digest: Some(format!("sha256:{}", "a".repeat(64))),
        enabled: Some(true),
        shipping_policy: None,
    };
    let pinned = spec.digest.as_ref().ok_or_else(|| anyhow::anyhow!("pinned control"))?.clone();
    let stage = QaStage::Validate;
    let stage_id = stage.stage_id();
    let input_hash = "e".repeat(64);
    let catalog = HashMap::from([(spec.tool.clone(), spec.clone())]);
    let marker = root.join("pinned-image-present");
    std::fs::write(&marker, b"present")?;
    let pinned_image = resolve_image_for_run(&spec, &platform)?;
    ensure!(pinned_image.full_name.contains('@'), "present pinned image resolution control");
    let pinned_record =
        build_qa_record(stage, &spec.tool, &platform, &catalog, &input_hash, ImageQaOutcome::Pass)?;
    ensure!(pinned_record.image_digest == pinned, "legacy pinned wire identity remains supported");
    std::fs::remove_file(&marker)?;
    let fallback = resolve_image_for_run(&spec, &platform)?;
    ensure!(!fallback.full_name.contains('@'), "documented fallback must be admitted");
    let mut unpinned = spec.clone();
    unpinned.digest = None;
    let unpinned_catalog = HashMap::from([(spec.tool.clone(), unpinned)]);
    let tag_record = build_qa_record(
        stage,
        &spec.tool,
        &platform,
        &unpinned_catalog,
        &input_hash,
        ImageQaOutcome::Pass,
    )?;
    ensure!(tag_record.image_digest == fallback.full_name, "actual unpinned record control");
    let store = QaRecordStore::prepare(root, &platform)?;
    store.insert_input(stage_id.as_str(), &input_hash)?;
    store.append_record(&tag_record)?;
    ensure!(
        qa_already_passed(
            store.conn(),
            stage,
            &spec.tool,
            &platform,
            &unpinned_catalog,
            &input_hash
        )?,
        "real tag-row query control"
    );
    ensure_image_qa_passed(
        stage_id.as_str(),
        std::slice::from_ref(&spec.tool),
        &platform,
        &unpinned_catalog,
    )?;
    let record =
        build_qa_record(stage, &spec.tool, &platform, &catalog, &input_hash, ImageQaOutcome::Pass)?;
    let cached_tag =
        qa_already_passed(store.conn(), stage, &spec.tool, &platform, &catalog, &input_hash)?;
    let gate_tag = ensure_image_qa_passed(
        stage_id.as_str(),
        std::slice::from_ref(&spec.tool),
        &platform,
        &catalog,
    );
    store.conn().execute("DELETE FROM image_qa_v1", [])?;
    store.append_record(&pinned_record)?;
    let cached_missing_pin =
        qa_already_passed(store.conn(), stage, &spec.tool, &platform, &catalog, &input_hash)?;
    let gate_missing_pin = ensure_image_qa_passed(
        stage_id.as_str(),
        std::slice::from_ref(&spec.tool),
        &platform,
        &catalog,
    );
    ensure!(
        resolve_image_for_run(&spec, &platform)?.full_name == fallback.full_name,
        "fallback physical provider state remains unchanged"
    );
    eprintln!("IMAGE_QA_RESOLVED_IDENTITY_CONTROLS_PASS: present pin, admitted tag fallback, actual tag store/query/gate; record={record:?}; cached_tag={cached_tag}; gate_tag={gate_tag:?}; cached_missing_pin={cached_missing_pin}; gate_missing_pin={gate_missing_pin:?}");
    let mut failures = Vec::new();
    if record.image_digest != fallback.full_name {
        failures.push("record bound absent configured pin");
    }
    if !cached_tag || gate_tag.is_err() {
        failures.push("query/gate rejected actual resolved tag evidence");
    }
    if cached_missing_pin || gate_missing_pin.is_ok() {
        failures.push("query/gate certified absent pinned evidence for fallback execution");
    }
    ensure!(failures.is_empty(), "image QA identity diverged from actual resolution: {failures:?}");
    Ok(())
}
