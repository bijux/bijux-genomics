use anyhow::{anyhow, Result};
use bijux_dna_api::v1::api::run::{execute_local_fastq_workflow, verify_run_bundle};
use std::path::Path;

#[test]
fn run_bundle_verifier_validates_original_and_copied_bundles() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let response = execute_local_fastq_workflow(&temp.path().join("source"))?;
    let run_dir = response
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("manifest missing parent"))?;
    let original = verify_run_bundle(run_dir)?;
    if original.get("ok").and_then(serde_json::Value::as_bool) != Some(true) {
        panic!("original bundle verification failed: {}", original);
    }
    assert!(original.get("log_files").is_some());

    let copied_dir = temp.path().join("copied-run");
    copy_tree(run_dir, &copied_dir)?;
    let copied = verify_run_bundle(&copied_dir)?;
    if copied.get("ok").and_then(serde_json::Value::as_bool) != Some(true) {
        panic!("copied bundle verification failed: {}", copied);
    }
    assert!(copied.get("trust_classes").is_some());
    Ok(())
}

fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    if !src.is_dir() {
        return Err(anyhow!("missing source directory {}", src.display()));
    }
    bijux_dna_infra::ensure_dir(dst)?;
    let mut stack = vec![(src.to_path_buf(), dst.to_path_buf())];
    while let Some((current_src, current_dst)) = stack.pop() {
        for entry in std::fs::read_dir(&current_src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = current_dst.join(entry.file_name());
            if src_path.is_dir() {
                bijux_dna_infra::ensure_dir(&dst_path)?;
                stack.push((src_path, dst_path));
                continue;
            }
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
