#![allow(clippy::expect_used, clippy::too_many_lines)]

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> (support::RepoSandbox, std::process::Output) {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let sandbox = support::RepoSandbox::new("output-contract-audit-").expect("repo sandbox");
    let home = tempfile::tempdir().expect("tempdir");

    let output = sandbox
        .bijux_dna_command()
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli");
    (sandbox, output)
}

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let (_sandbox, output) = run_cli(args);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn bench_readiness_output_contract_audit_report_governs_all_retained_bindings() {
    let payload = run_cli_json(&["bench", "readiness", "render-output-contract-tests", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.output_contract_audit.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/tools/output-contract-tests.json")
    );
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(
        payload.get("row_count").and_then(serde_json::Value::as_u64),
        Some(rows.len() as u64)
    );
    assert_eq!(
        payload.get("passed_row_count").and_then(serde_json::Value::as_u64),
        Some(rows.len() as u64 - 25)
    );
    assert_eq!(payload.get("blocked_row_count").and_then(serde_json::Value::as_u64), Some(25));
    assert_eq!(payload.get("failed_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("blocked_tool_ids").and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::Value::String("angsd".to_string()),
            serde_json::Value::String("bcftools".to_string()),
            serde_json::Value::String("beagle".to_string()),
            serde_json::Value::String("eagle".to_string()),
            serde_json::Value::String("eigensoft".to_string()),
            serde_json::Value::String("fastuniq".to_string()),
            serde_json::Value::String("germline".to_string()),
            serde_json::Value::String("glimpse".to_string()),
            serde_json::Value::String("ibdhap".to_string()),
            serde_json::Value::String("ibdne".to_string()),
            serde_json::Value::String("ibdseq".to_string()),
            serde_json::Value::String("impute5".to_string()),
            serde_json::Value::String("minimac4".to_string()),
            serde_json::Value::String("ngsbriggs".to_string()),
            serde_json::Value::String("plink".to_string()),
            serde_json::Value::String("shapeit".to_string()),
        ])
    );
    assert!(payload
        .get("failed_tool_ids")
        .and_then(serde_json::Value::as_array)
        .is_some_and(Vec::is_empty));

    let proof_surface_counts = payload
        .get("output_proof_surface_counts")
        .and_then(serde_json::Value::as_object)
        .expect("proof surface counts");
    let proof_count =
        proof_surface_counts.values().filter_map(serde_json::Value::as_u64).sum::<u64>();
    assert_eq!(proof_count, rows.len() as u64);
    for (tool_id, stage_id) in [
        ("ngsbriggs", "bam.damage"),
        ("angsd", "bam.kinship"),
        ("angsd", "bam.sex"),
        ("fastuniq", "fastq.remove_duplicates"),
        ("angsd", "vcf.call_gl"),
        ("angsd", "vcf.call_pseudohaploid"),
        ("impute5", "vcf.imputation_metrics"),
        ("impute5", "vcf.impute"),
    ] {
        assert!(rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                && row.get("passed").and_then(serde_json::Value::as_bool) == Some(false)
                && row.get("independent_execution_proven").and_then(serde_json::Value::as_bool)
                    == Some(false)
                && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                    == Some("external_artifact_admission")
                && row
                    .get("execution_blocker")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|reason| reason.starts_with("blocked_external_access:"))
        }));
    }

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.haplogroups")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("yleaf")
            && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                == Some("direct_declared_output_proof")
            && row
                .get("observed_normalized_metric_ids")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some("haplogroups")))
    }));
    for blocked_tool_id in ["germline", "ibdne"] {
        assert!(rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some(blocked_tool_id)
                && row.get("passed").and_then(serde_json::Value::as_bool) == Some(false)
                && row.get("independent_execution_proven").and_then(serde_json::Value::as_bool)
                    == Some(false)
                && row
                    .get("execution_blocker")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|reason| reason.contains("was not executed"))
        }));
    }
    for blocked_tool_id in ["ibdhap", "ibdseq", "shapeit"] {
        assert!(rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some(blocked_tool_id)
                && row.get("passed").and_then(serde_json::Value::as_bool) == Some(false)
                && row.get("independent_execution_proven").and_then(serde_json::Value::as_bool)
                    == Some(false)
                && row
                    .get("execution_blocker")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|reason| reason.starts_with("missing_direct_execution:"))
        }));
    }

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.postprocess")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                == Some("direct_tool_smoke")
            && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
    }));
    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.roh")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
            && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                == Some("direct_tool_smoke")
            && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
    }));
}

#[test]
fn bench_readiness_output_contract_audit_write_governed_json_file() {
    let (sandbox, output) = run_cli(&["bench", "readiness", "render-output-contract-tests"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        rendered_path.trim(),
        "artifacts/benchmark-smoke/readiness/tools/output-contract-tests.json"
    );

    let report_path = sandbox.path().join(rendered_path.trim());
    assert!(report_path.is_file(), "output contract report JSON must exist");

    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).expect("read report"))
            .expect("parse report json");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.output_contract_audit.v1")
    );
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(
        payload.get("row_count").and_then(serde_json::Value::as_u64),
        Some(rows.len() as u64)
    );
    assert_eq!(payload.get("failed_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("blocked_row_count").and_then(serde_json::Value::as_u64), Some(25));
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("mapdamage2")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("bam.bias_mitigation")
                && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                    == Some("direct_declared_output_proof")
        }),
        "report file must retain governed BAM downstream direct-proof coverage"
    );
}
