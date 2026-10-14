use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_containers::api::{
    apply_final_run_evidence_plan, derive_run_evidence_index, import_run_evidence,
    load_run_evidence_index, plan_final_run_evidence, publish_run_evidence_book,
    replay_run_evidence_book, run_evidence_index_schema, validate_run_evidence_index,
    validate_run_evidence_root, verify_run_evidence_publication, AttemptExecutionStatus,
    RunEvidenceImportRequest,
};

#[test]
fn checked_in_selected_runs_are_admitted_by_the_container_owner() -> Result<()> {
    let index =
        load_run_evidence_index(&repository_root().join("containers/reports/run-evidence.json"))?;

    validate_run_evidence_index(&index)?;
    assert!(!index.runs.is_empty());
    assert!(index
        .runs
        .iter()
        .all(|record| record.execution_status == AttemptExecutionStatus::Passed));
    assert_eq!(index.summary.run_count, index.runs.len());
    assert_eq!(index.summary.tool_count, 71);
    assert!(run_evidence_index_schema().as_value().is_object());
    Ok(())
}

#[test]
fn selected_run_summary_drift_is_rejected() -> Result<()> {
    let mut index =
        load_run_evidence_index(&repository_root().join("containers/reports/run-evidence.json"))?;
    index.summary.run_count += 1;

    assert!(validate_run_evidence_index(&index)
        .is_err_and(|error| error.to_string().contains("summary run count drifted")));
    Ok(())
}

#[test]
fn selected_run_index_is_derived_from_locks_and_attempt_history() -> Result<()> {
    let root = repository_root();
    let checked = load_run_evidence_index(&root.join("containers/reports/run-evidence.json"))?;

    assert_eq!(derive_run_evidence_index(&root.join("containers/runs"))?, checked);
    Ok(())
}

#[test]
fn import_and_final_selection_preserve_failed_attempt_history() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let results = temp.path().join("results");
    make_result(&results, "run-1001", "passed")?;
    make_result(&results, "run-1002", "failed")?;
    let runs = temp.path().join("runs");
    let ledger = runs.join("attempt-ledger.json");

    assert_eq!(
        import_run_evidence(&RunEvidenceImportRequest {
            result_root: results,
            output_root: runs.clone(),
            minimum_run: None,
            maximum_run: None,
        })?,
        2
    );
    let plan = plan_final_run_evidence(&runs, &ledger)?;
    assert_eq!(plan.selected_run_count, 1);
    assert_eq!(plan.superseded_run_directories.len(), 1);
    assert_eq!(plan.ledger.attempts[0].run_id, "run-1002");
    assert_eq!(plan.ledger.attempts[0].failures, ["governed failure"]);

    apply_final_run_evidence_plan(&runs, &ledger, &plan)?;
    assert!(runs.join("run-1001-adapterremoval").is_dir());
    assert!(!runs.join("run-1002-adapterremoval").exists());
    assert_eq!(validate_run_evidence_root(&runs)?.len(), 1);
    Ok(())
}

#[test]
fn copied_run_evidence_tampering_is_rejected() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let results = temp.path().join("results");
    make_result(&results, "run-1001", "passed")?;
    let runs = temp.path().join("runs");
    import_run_evidence(&RunEvidenceImportRequest {
        result_root: results,
        output_root: runs.clone(),
        minimum_run: None,
        maximum_run: None,
    })?;
    fs::write(runs.join("run-1001-adapterremoval/config-pipeline.yml"), "changed\n")?;

    assert!(validate_run_evidence_root(&runs)
        .is_err_and(|error| error.to_string().contains("size changed")));
    Ok(())
}

#[test]
fn evidence_book_publication_is_owned_and_verified_by_the_container_crate() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let results = temp.path().join("results");
    make_result(&results, "run-1001", "passed")?;
    let runs = temp.path().join("runs");
    import_run_evidence(&RunEvidenceImportRequest {
        result_root: results,
        output_root: runs.clone(),
        minimum_run: None,
        maximum_run: None,
    })?;
    let ledger = runs.join("attempt-ledger.json");
    let plan = plan_final_run_evidence(&runs, &ledger)?;
    apply_final_run_evidence_plan(&runs, &ledger, &plan)?;
    let json = temp.path().join("book/run-evidence.json");
    let markdown = temp.path().join("book/run-evidence.md");

    let index = publish_run_evidence_book(&runs, &json, &markdown)?;

    assert_eq!(index.summary.run_count, 1);
    verify_run_evidence_publication(&runs, &json, &markdown)?;
    assert!(fs::read_to_string(json)?.ends_with('\n'));
    Ok(())
}

#[test]
fn clean_room_replay_rejects_a_mixed_publication_root() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let results = temp.path().join("results");
    make_result(&results, "run-1001", "passed")?;
    let replay = temp.path().join("replay");
    fs::create_dir_all(&replay)?;
    fs::write(replay.join("retained.json"), b"{}\n")?;

    let result = replay_run_evidence_book(
        &results,
        &replay,
        &temp.path().join("book.json"),
        &temp.path().join("book.md"),
    );
    let Err(error) = result else {
        anyhow::bail!("mixed replay roots must be rejected");
    };

    assert!(error.to_string().contains("must be empty"));
    assert!(replay.join("retained.json").is_file());
    Ok(())
}

#[test]
fn clean_room_replay_selects_and_publishes_from_raw_results() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let results = temp.path().join("results");
    make_result(&results, "run-1001", "passed")?;
    make_result(&results, "run-1002", "failed")?;
    let replay = temp.path().join("replay");
    let json = temp.path().join("book.json");
    let markdown = temp.path().join("book.md");

    let index = replay_run_evidence_book(&results, &replay, &json, &markdown)?;

    assert_eq!(index.summary.run_count, 1);
    assert_eq!(index.runs[0].run_id, "run-1001");
    assert!(replay.join("attempt-ledger.json").is_file());
    assert!(!replay.join("run-1002-adapterremoval").exists());
    verify_run_evidence_publication(&replay, &json, &markdown)?;
    Ok(())
}

fn make_result(root: &Path, run_id: &str, status: &str) -> Result<()> {
    let result = root.join("adapterremoval").join(run_id);
    fs::create_dir_all(&result)?;
    fs::write(
        result.join("config-pipeline.yml"),
        format!(
            "schema_version: bijux.container.pipeline.v1\nrun:\n  id: {run_id}\n  campaign: public-read-trimming\n  corpus: ena-public-reads\n  sample: ERR000001\n  domain: fastq\n  stage: fastq.trim_reads\n  tool: adapterremoval\n  operator: test\nrepository:\n  path: /repository\n  commit: {}\n  clean: true\nimage:\n  path: /images/adapterremoval.sif\n  sha256: {}\ndata:\n  inputs:\n    - id: reads\n      kind: data\n      path: /data/reads.fastq.gz\n      sha256: {}\ncommands:\n  version: [AdapterRemoval, --version]\n  help: [AdapterRemoval, --help]\n  real: [AdapterRemoval, --file1, /data/reads.fastq.gz]\n  negative: [AdapterRemoval, --invalid]\noutputs:\n  result_root: /results/adapterremoval/{run_id}\n  required_files: [real.stdout.log]\nresources:\n  account: science\n  partition: compute\n  cpus: 1\n  memory_mb: 1024\n  walltime: '00:05:00'\n  scratch_mb: 1024\nexecution:\n  locale: C\n  timezone: UTC\n  runtime_network: false\nacceptance:\n  version_regex: '^2[.]3[.]3$'\n  negative_exit_nonzero: true\n",
            "a".repeat(40),
            "b".repeat(64),
            "c".repeat(64)
        ),
    )?;
    fs::write(
        result.join("config-credential.yml"),
        "schema_version: bijux.container.credential_reference.v1\ncredentials_required: false\nreferences: []\nredaction:\n  publish_values: false\n",
    )?;
    let failures: Vec<&str> =
        if status == "failed" { vec!["governed failure"] } else { Vec::new() };
    fs::write(
        result.join("evidence.json"),
        serde_json::to_vec(&serde_json::json!({
            "run_id": run_id,
            "status": status,
            "repository_commit": "a".repeat(40),
            "runtime": {"slurm_job_id": "42"},
            "image": {"sha256": "b".repeat(64)},
            "failures": failures,
        }))?,
    )?;
    Ok(())
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("container crate must live under repository crates/"))
        .to_path_buf()
}

#[test]
fn existing_attempt_history_rejects_duplicate_run_ids_and_preserves_refresh() -> Result<()> {
    for conflicting in [false, true] {
        let root = tempfile::tempdir()?;
        let results = root.path().join("results");
        make_result(&results, "run-1001", "passed")?;
        make_result(&results, "run-1002", "failed")?;
        let runs = root.path().join("runs");
        let ledger_path = runs.join("attempt-ledger.json");
        assert_eq!(
            import_run_evidence(&RunEvidenceImportRequest {
                result_root: results,
                output_root: runs.clone(),
                minimum_run: None,
                maximum_run: None,
            })?,
            2
        );
        let original = plan_final_run_evidence(&runs, &ledger_path)?;
        apply_final_run_evidence_plan(&runs, &ledger_path, &original)?;
        let healthy_ledger = fs::read(&ledger_path)?;
        let lock_path = runs.join("run-1001-adapterremoval/evidence.lock.json");
        let original_lock = fs::read(&lock_path)?;
        let mut ledger: serde_json::Value = serde_json::from_slice(&healthy_ledger)?;
        let retained = ledger["attempts"][0].clone();
        let mut earlier = retained.clone();
        if conflicting {
            earlier["tool"] = serde_json::json!("contradictory-tool");
        }
        ledger["attempts"] = serde_json::json!([earlier, retained]);
        ledger["summary"]["superseded_attempt_count"] = serde_json::json!(2);
        ledger["summary"]["execution_status"]["failed"] = serde_json::json!(2);
        fs::write(&ledger_path, serde_json::to_vec(&ledger)?)?;
        let duplicate_bytes = fs::read(&ledger_path)?;
        let error = plan_final_run_evidence(&runs, &ledger_path)
            .err()
            .ok_or_else(|| anyhow::anyhow!("duplicate persisted history was admitted"))?;
        assert!(error.to_string().contains("run identity is duplicated"));
        assert_eq!(fs::read(&ledger_path)?, duplicate_bytes);
        assert_eq!(fs::read(&lock_path)?, original_lock);
        assert!(runs.join("run-1001-adapterremoval").is_dir());
        fs::write(&ledger_path, &healthy_ledger)?;
        let next = root.path().join("next-results");
        make_result(&next, "run-1003", "passed")?;
        assert_eq!(
            import_run_evidence(&RunEvidenceImportRequest {
                result_root: next,
                output_root: runs.clone(),
                minimum_run: None,
                maximum_run: None,
            })?,
            1
        );
        let refreshed = plan_final_run_evidence(&runs, &ledger_path)?;
        assert_eq!(refreshed.selected_run_count, 1);
        assert_eq!(refreshed.ledger.attempts.len(), 2);
        assert!(refreshed.ledger.attempts.iter().all(|row| row.superseded_by == "run-1003"));
        assert!(refreshed
            .ledger
            .attempts
            .iter()
            .any(|row| row.run_id == "run-1002" && row.failures == ["governed failure"]));
        assert_eq!(fs::read(&ledger_path)?, healthy_ledger);
    }
    Ok(())
}
