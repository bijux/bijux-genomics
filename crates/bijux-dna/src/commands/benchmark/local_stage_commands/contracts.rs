use std::path::PathBuf;

use super::{
    local_stage_plans, materialize_local_report_qc_smoke_report, materialize_local_stage,
    render_local_stage_commands,
};
#[cfg(feature = "bam_downstream")]
use super::{
    BenchLocalDomain, BENCHMARK_LOCAL_STAGE_COMMANDS_SCHEMA_V3,
    DEFAULT_RENDERED_STAGE_COMMANDS_PATH,
};
#[cfg(feature = "bam_downstream")]
use crate::commands::benchmark::local_stage_inventory::load_local_stage_inventory;

fn repo_root() -> PathBuf {
    crate::commands::benchmark::scratch_repository::materialize_read_only_repository(
        "local-stage-commands",
    )
}

#[cfg(feature = "bam_downstream")]
#[test]
fn rendered_local_stage_commands_cover_governed_51_stage_slice() {
    let root = repo_root();
    let rendered =
        render_local_stage_commands(&root, PathBuf::from(DEFAULT_RENDERED_STAGE_COMMANDS_PATH))
            .expect("render local stage commands");

    let fastq = load_local_stage_inventory(&root, BenchLocalDomain::Fastq)
        .expect("load FASTQ local stage inventory");
    let bam = load_local_stage_inventory(&root, BenchLocalDomain::Bam)
        .expect("load BAM local stage inventory");

    assert_eq!(rendered.command_count, fastq.stage_count + bam.stage_count);
    assert_eq!(rendered.commands.len(), 51);
    assert_eq!(rendered.schema_version, BENCHMARK_LOCAL_STAGE_COMMANDS_SCHEMA_V3);
    assert_eq!(
        rendered.manifest_output_path,
        "artifacts/benchmark-smoke/readiness/local-ready/rendered-stage-commands.json"
    );
    assert_eq!(
        rendered.argv_output_path,
        "artifacts/benchmark-smoke/readiness/local-ready/rendered-stage-commands.argv.jsonl"
    );
    assert!(rendered.commands.iter().any(|entry| entry.stage_id == "fastq.report_qc"));
    assert!(rendered.commands.iter().all(|entry| {
        !entry.tool_id.is_empty()
            && !entry.inputs.is_empty()
            && !entry.outputs.is_empty()
            && entry.threads >= 1
            && entry.memory_mb >= 1024
            && entry.argv.first().is_some_and(|arg| arg == "cargo")
            && entry.argv.iter().any(|arg| arg == "--stage-id")
            && entry.argv.last().is_some_and(|arg| arg == &entry.stage_id)
            && entry.command.contains("bench local materialize-stage")
            && entry.command.contains(&entry.stage_id)
    }));
}

#[cfg(feature = "bam_downstream")]
#[test]
fn rendered_local_stage_commands_write_governed_argv_jsonl() {
    let root = repo_root();
    let rendered =
        render_local_stage_commands(&root, PathBuf::from(DEFAULT_RENDERED_STAGE_COMMANDS_PATH))
            .expect("render local stage commands");

    let argv_path = root.join(&rendered.argv_output_path);
    let argv_jsonl =
        std::fs::read_to_string(&argv_path).expect("read rendered stage command argv jsonl");
    let rows = argv_jsonl.lines().collect::<Vec<_>>();

    assert_eq!(rows.len(), 51);
    let parsed = rows
        .iter()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("parse argv row"))
        .collect::<Vec<_>>();
    assert!(parsed.iter().all(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str).is_some()
            && row.get("tool_id").and_then(serde_json::Value::as_str).is_some()
            && row.get("argv").and_then(serde_json::Value::as_array).is_some_and(|argv| {
                argv.first().and_then(serde_json::Value::as_str) == Some("cargo")
                    && argv.iter().any(|arg| arg.as_str() == Some("--stage-id"))
            })
    }));
}

#[cfg(not(feature = "bam_downstream"))]
#[test]
fn rendered_local_stage_commands_explain_downstream_feature_requirement() {
    let root = repo_root();
    let error = render_local_stage_commands(
        &root,
        std::path::Path::new(
            "artifacts/benchmark-smoke/readiness/local-ready/rendered-stage-commands.sh",
        )
        .to_path_buf(),
    )
    .expect_err("render without bam_downstream should explain the missing feature");

    assert!(
        error.to_string().contains("requires the `bam_downstream` feature"),
        "missing-feature error should stay explicit: {error:#}"
    );
}

#[test]
fn local_deplete_rrna_materialization_creates_declared_plan_outputs() {
    let root = repo_root();
    let proof_path = materialize_local_stage(&root, "fastq.deplete_rrna")
        .expect("materialize local fastq.deplete_rrna outputs");
    assert!(proof_path.exists(), "proof path should exist");

    let plans = local_stage_plans(&root, "fastq.deplete_rrna")
        .expect("collect local fastq.deplete_rrna plans");
    assert!(!plans.is_empty(), "deplete_rrna should expose governed plans");
    for plan in &plans {
        for artifact in &plan.io.outputs {
            let resolved = root.join(&artifact.path);
            assert!(
                resolved.exists(),
                "declared output should be materialized: {}",
                resolved.display()
            );
        }
    }
}

#[test]
fn local_align_materialization_creates_declared_plan_outputs() {
    let root = repo_root();
    let proof_path =
        materialize_local_stage(&root, "bam.align").expect("materialize local bam.align outputs");
    assert!(proof_path.exists(), "proof path should exist");

    let plans = local_stage_plans(&root, "bam.align").expect("collect local bam.align plans");
    assert_eq!(plans.len(), 1, "bam.align should expose one governed plan");
    for artifact in &plans[0].io.outputs {
        let resolved = root.join(&artifact.path);
        assert!(
            resolved.exists(),
            "declared output should be materialized: {}",
            resolved.display()
        );
    }
}

#[test]
fn local_report_qc_smoke_materialization_writes_governed_bundle() {
    let root = repo_root();
    let report_path = materialize_local_report_qc_smoke_report(&root)
        .expect("materialize local report_qc smoke report");

    assert_eq!(
        report_path.strip_prefix(&root).expect("relative report path").display().to_string(),
        "artifacts/runs/bench/local-smoke/fastq.report_qc/report.json"
    );
    assert!(root
        .join("artifacts/runs/bench/local-smoke/fastq.report_qc/report_qc_report.json")
        .is_file());
    assert!(root
        .join("artifacts/runs/bench/local-smoke/fastq.report_qc/governed_qc_inputs_manifest.json")
        .is_file());
    assert!(root
        .join("artifacts/runs/bench/local-smoke/fastq.report_qc/multiqc_report.html")
        .is_file());
    assert!(root
        .join("artifacts/runs/bench/local-smoke/fastq.report_qc/multiqc_data/multiqc_data.json")
        .is_file());
}

#[cfg(unix)]
#[test]
fn local_report_qc_smoke_materialization_overwrites_readonly_governed_outputs() {
    use std::os::unix::fs::PermissionsExt;

    let root = repo_root();
    let contributors_dir =
        root.join("artifacts/runs/bench/local-smoke/fastq.report_qc/contributors");
    std::fs::create_dir_all(&contributors_dir).expect("create contributors dir");
    let contributor = contributors_dir.join("detect_adapters.report.json");
    bijux_dna_infra::write_payload(&contributor, br#"{"stale":true}"#)
        .expect("seed stale contributor");
    std::fs::set_permissions(&contributor, std::fs::Permissions::from_mode(0o444))
        .expect("set readonly contributor");

    let report_path = materialize_local_report_qc_smoke_report(&root)
        .expect("materialize local report_qc smoke report over readonly outputs");
    assert!(report_path.is_file(), "report should be rewritten");

    let raw = std::fs::read_to_string(&contributor).expect("read rewritten contributor");
    assert!(
        raw.contains("detect_adapters"),
        "rewritten contributor should come from governed fixture content"
    );
    assert!(
        root.join("artifacts/runs/bench/local-smoke/fastq.report_qc/report_qc_report.json")
            .is_file(),
        "canonical stage-contract report should be regenerated"
    );
}
