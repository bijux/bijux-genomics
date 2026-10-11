//! Owner: bijux-dna-bench
//! Bounded loading and semantic validation for finalization progress ledgers.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{validate_benchmark_progress_ledger, BenchmarkProgressLedger};

const MAX_PROGRESS_LEDGER_BYTES: usize = 64 * 1024 * 1024;

/// Load and validate one complete finalization progress ledger.
///
/// # Errors
/// Returns an error when the path is not a regular bounded file, JSON decoding fails, or the
/// ledger violates its typed cross-record and frontier invariants.
pub fn load_benchmark_progress_ledger(path: &Path) -> Result<BenchmarkProgressLedger> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect benchmark progress ledger {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("benchmark progress ledger is not a regular non-symlink file: {}", path.display());
    }
    if metadata.len() > MAX_PROGRESS_LEDGER_BYTES as u64 {
        bail!("benchmark progress ledger exceeds the bounded size limit");
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_PROGRESS_LEDGER_BYTES)
        .with_context(|| format!("read benchmark progress ledger {}", path.display()))?;
    let ledger = serde_json::from_slice::<BenchmarkProgressLedger>(&bytes)
        .with_context(|| format!("parse benchmark progress ledger {}", path.display()))?;
    validate_benchmark_progress_ledger(&ledger).context("validate benchmark progress ledger")?;
    Ok(ledger)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::load_benchmark_progress_ledger;

    const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
    const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn loads_a_semantically_valid_ledger() {
        let root = tempfile::tempdir().expect("temporary progress-ledger root");
        let path = root.path().join("ledger.json");
        let ledger = serde_json::json!({
            "schema_version": "bijux.finalization_v3.progress_ledger.v1",
            "cycle_id": "cycle-current-validation",
            "source": {
                "local_head": SHA,
                "remote_head": SHA,
                "catalog_sha256": DIGEST,
                "benchmark_registry_sha256": DIGEST,
                "local_clean": true,
                "remote_clean": true
            },
            "frontier": {
                "ready_task_ids": ["DATA-001"],
                "ready_unit_ids": [],
                "submitted_unit_ids": [],
                "collectible_unit_ids": []
            },
            "tasks": [{
                "task_id": "DATA-001",
                "status": "ready",
                "next_action": "materialize the independently ready corpus"
            }],
            "execution_units": [],
            "attempts": [],
            "commits": [],
            "checks": [],
            "external_waits": []
        });
        fs::write(&path, serde_json::to_vec_pretty(&ledger).expect("serialize ledger"))
            .expect("write ledger");

        let loaded = load_benchmark_progress_ledger(&path).expect("load valid ledger");
        assert_eq!(loaded.cycle_id, "cycle-current-validation");
    }

    #[test]
    fn rejects_a_commit_that_names_an_unknown_check() {
        let root = tempfile::tempdir().expect("temporary progress-ledger root");
        let path = root.path().join("ledger.json");
        let ledger = serde_json::json!({
            "schema_version": "bijux.finalization_v3.progress_ledger.v1",
            "cycle_id": "cycle-invalid-check-reference",
            "source": {
                "local_head": SHA,
                "remote_head": SHA,
                "catalog_sha256": DIGEST,
                "benchmark_registry_sha256": DIGEST
            },
            "frontier": {
                "ready_task_ids": [],
                "ready_unit_ids": [],
                "submitted_unit_ids": [],
                "collectible_unit_ids": []
            },
            "tasks": [],
            "execution_units": [],
            "attempts": [],
            "commits": [{"sha": SHA, "intent": "record evidence", "checks": ["missing"]}],
            "checks": [],
            "external_waits": []
        });
        fs::write(&path, serde_json::to_vec_pretty(&ledger).expect("serialize ledger"))
            .expect("write ledger");

        let error = load_benchmark_progress_ledger(&path).expect_err("reject unknown check");
        let diagnostic = format!("{error:#}");
        assert!(diagnostic.contains("validate benchmark progress ledger"));
        assert!(diagnostic.contains("unknown check"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_symlink_before_reading_ledger_bytes() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().expect("temporary progress-ledger root");
        let target = root.path().join("target.json");
        let path = root.path().join("ledger.json");
        fs::write(&target, b"{}").expect("write symlink target");
        symlink(&target, &path).expect("create ledger symlink");

        let error = load_benchmark_progress_ledger(&path).expect_err("reject ledger symlink");
        assert!(error.to_string().contains("regular non-symlink file"));
    }
}
