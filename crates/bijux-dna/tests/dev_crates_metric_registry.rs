#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn dev_crates_metric_registry_writes_the_governed_stage_metric_table() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let temp = tempfile::tempdir().expect("tempdir");
    let out = temp.path().join("metric-registry.tsv");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .arg("dev")
        .arg("crates")
        .arg("metric-registry")
        .arg("--output")
        .arg(&out)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout json payload");
    let expected_output_path = out.display().to_string();
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.crates.metric_registry.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some(expected_output_path.as_str())
    );
    assert!(
        payload.get("row_count").and_then(serde_json::Value::as_u64).is_some_and(|count| count > 0),
        "metric registry must report at least one governed stage metric row"
    );

    let rendered = std::fs::read_to_string(&out).expect("read metric registry TSV");
    let mut lines = rendered.lines();
    assert_eq!(
        lines.next(),
        Some(
            "domain\tstage_id\tmetric_id\tmeaning\tcontract_kind\tstage_contract_surface\tdomain_registry_surface"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert!(rows.iter().any(|row| {
        row == &"fastq\tfastq.validate_reads\treads_in\tNumber of input reads\tyaml_stage_metrics\tdomain/fastq/stages/validate_reads.yaml\tdomain/fastq/metrics.yaml"
    }));
    assert!(rows.iter().any(|row| {
        row == &"bam\tbam.align\talignment_rate\tMapped fraction derived from governed alignment summaries\tyaml_stage_metrics\tdomain/bam/stages/align.yaml\tdomain/bam/metrics.yaml"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf\tvcf.stats\tvariant_count\t\trust_stage_metrics_contract\tcrates/bijux-dna-domain-vcf/src/contracts/stage_metrics.rs\tdomain/vcf/metrics.yaml"
    }));
    assert!(out.is_file(), "metric registry must write the governed TSV report");
}
