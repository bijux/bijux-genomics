//! Publish terminal gate evidence after execution and owned cleanup, including failures.

use anyhow::Result;
use serde_json::{json, Value};
use std::{path::Path, time::Instant};

/// Keep the context (including exclusive leases) alive through cleanup and publication.
/// An operation may enrich the initial receipt before returning an error.
pub(crate) fn run_reported<T, C>(
    path: &Path,
    mut receipt: Value,
    started: Instant,
    context: &mut C,
    execute: impl FnOnce(&mut C, &mut Value) -> Result<T>,
    cleanup: impl FnOnce(&mut C, &mut Value) -> Result<()>,
) -> Result<T> {
    receipt["state"] = json!("running");
    write_receipt(path, &receipt)?;
    let result = execute(context, &mut receipt);
    if let Err(error) = &result {
        receipt["execution_error"] = json!(format!("{error:#}"));
    }
    let cleanup_result = cleanup(context, &mut receipt);
    if let Err(error) = &cleanup_result {
        receipt["cleanup_error"] = json!(format!("{error:#}"));
    }
    receipt["state"] = json!("complete");
    receipt["all_passed"] = json!(
        receipt["all_passed"].as_bool() == Some(true) && result.is_ok() && cleanup_result.is_ok()
    );
    receipt["wall_time_milliseconds"] = json!(started.elapsed().as_millis());
    let publication = write_receipt(path, &receipt);
    // Preserve all failure causes, including inability to replace the initial receipt.
    let mut errors = Vec::new();
    for (label, failure) in [
        ("execution", result.as_ref().err()),
        ("cleanup", cleanup_result.as_ref().err()),
        ("receipt publication", publication.as_ref().err()),
    ] {
        if let Some(error) = failure {
            errors.push(format!("{label}: {error:#}"));
        }
    }
    anyhow::ensure!(errors.is_empty(), "{}", errors.join("; "));
    result
}

fn write_receipt(path: &Path, receipt: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(receipt)?;
    bytes.push(b'\n');
    bijux_dna_infra::write_bytes(path, &bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_receipt_finalizes_early_failure_after_exactly_one_cleanup() -> Result<()> {
        let root = tempfile::tempdir()?;
        let path = root.path().join("receipt.json");
        let evidence = root.path().join("owned-output");
        std::fs::write(&evidence, b"partial")?;
        let mut cleanup_count = 0;
        let result = run_reported::<(), _>(
            &path,
            json!({"source_commit":"immutable-source","all_passed":false}),
            Instant::now(),
            &mut cleanup_count,
            |_, receipt| {
                receipt["failed_operation"] = json!("inventory");
                anyhow::bail!("malformed test inventory")
            },
            |count, _| {
                *count += 1;
                std::fs::remove_file(&evidence)?;
                Ok(())
            },
        );
        assert!(result.is_err());
        assert_eq!(cleanup_count, 1);
        assert!(!evidence.exists());
        let receipt: Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(receipt["state"], "complete");
        assert_eq!(receipt["source_commit"], "immutable-source");
        assert_eq!(receipt["failed_operation"], "inventory");
        assert_eq!(receipt["all_passed"], false);
        assert_eq!(receipt["execution_error"], "malformed test inventory");
        Ok(())
    }

    #[test]
    fn gate_receipt_retains_execution_cleanup_and_publication_failures() -> Result<()> {
        let root = tempfile::tempdir()?;
        let path = root.path().join("receipt.json");
        let result = run_reported::<(), _>(
            &path,
            json!({"all_passed":true}),
            Instant::now(),
            &mut (),
            |(), _| anyhow::bail!("cache authority refused"),
            |(), _| {
                std::fs::remove_file(&path)?;
                std::fs::create_dir(&path)?;
                anyhow::bail!("cleanup refused external link")
            },
        );
        let error = result.err().ok_or_else(|| anyhow::anyhow!("failure was lost"))?;
        let message = format!("{error:#}");
        assert!(message.contains("cache authority refused"));
        assert!(message.contains("cleanup refused external link"));
        assert!(message.contains("receipt publication"));
        Ok(())
    }
}
