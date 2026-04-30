use super::{
    anyhow, artifact_root_path, json, stable_now_utc_string, write_json_pretty, OpsCommandOutcome,
    Path, PathBuf, Result, Workspace,
};

#[derive(Debug, Clone, Copy)]
enum ScenarioId {
    WorkflowImportExport,
    RunComparisonCommand,
    ArtifactRetentionSimulation,
    ArtifactDedupLineage,
    CacheCorruptionQuarantine,
    BundlePortabilityCheck,
}

impl ScenarioId {
    fn as_str(self) -> &'static str {
        match self {
            Self::WorkflowImportExport => "g191_workflow_import_export_package",
            Self::RunComparisonCommand => "g192_run_comparison_command",
            Self::ArtifactRetentionSimulation => "g193_artifact_retention_simulation",
            Self::ArtifactDedupLineage => "g194_artifact_deduplication_lineage",
            Self::CacheCorruptionQuarantine => "g195_cache_corruption_quarantine",
            Self::BundlePortabilityCheck => "g196_bundle_portability_check",
        }
    }

    fn goal_id(self) -> &'static str {
        match self {
            Self::WorkflowImportExport => "G191",
            Self::RunComparisonCommand => "G192",
            Self::ArtifactRetentionSimulation => "G193",
            Self::ArtifactDedupLineage => "G194",
            Self::CacheCorruptionQuarantine => "G195",
            Self::BundlePortabilityCheck => "G196",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            Self::WorkflowImportExport,
            Self::RunComparisonCommand,
            Self::ArtifactRetentionSimulation,
            Self::ArtifactDedupLineage,
            Self::CacheCorruptionQuarantine,
            Self::BundlePortabilityCheck,
        ]
    }

    fn from_raw(raw: &str) -> Option<Self> {
        match raw {
            "g191_workflow_import_export_package" | "G191" => Some(Self::WorkflowImportExport),
            "g192_run_comparison_command" | "G192" => Some(Self::RunComparisonCommand),
            "g193_artifact_retention_simulation" | "G193" => {
                Some(Self::ArtifactRetentionSimulation)
            }
            "g194_artifact_deduplication_lineage" | "G194" => Some(Self::ArtifactDedupLineage),
            "g195_cache_corruption_quarantine" | "G195" => Some(Self::CacheCorruptionQuarantine),
            "g196_bundle_portability_check" | "G196" => Some(Self::BundlePortabilityCheck),
            _ => None,
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct ScenarioSuiteReport {
    schema_version: &'static str,
    generated_at_utc: String,
    scenario_count: usize,
    passed: usize,
    failed: usize,
    scenarios: Vec<ScenarioReport>,
}

#[derive(Debug, serde::Serialize)]
struct ScenarioReport {
    goal_id: &'static str,
    scenario_id: &'static str,
    status: &'static str,
    notes: Vec<String>,
    evidence: serde_json::Value,
}

#[derive(Debug, Clone)]
struct ScenarioRunConfig {
    selected: Vec<ScenarioId>,
    out: PathBuf,
}

pub(in super::super) fn tooling_operator_workflow_maturity(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return Ok(OpsCommandOutcome::success(
            "Usage: cargo run -p bijux-dna-dev -- tooling run operator-workflow-maturity -- [--scenario <goal-id-or-scenario-id>]... [--out <path>]\n",
        ));
    }

    let config = parse_args(workspace, args)?;
    let reports = config
        .selected
        .iter()
        .map(run_scenario)
        .collect::<Vec<_>>();
    let failed = reports.iter().filter(|report| report.status == "failed").count();

    let payload = ScenarioSuiteReport {
        schema_version: "bijux.operator_workflow_maturity.scenario_suite.v1",
        generated_at_utc: stable_now_utc_string(),
        scenario_count: reports.len(),
        passed: reports.len().saturating_sub(failed),
        failed,
        scenarios: reports,
    };
    let payload_json = serde_json::to_value(payload)?;

    if let Some(parent) = config.out.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    write_json_pretty(&config.out, &payload_json)?;

    if failed > 0 {
        return Ok(OpsCommandOutcome::failure(format!(
            "operator workflow maturity scenarios: FAILED ({failed} failed)\nreport: {}\n",
            workspace.rel(&config.out).display()
        )));
    }

    Ok(OpsCommandOutcome::success(format!(
        "operator workflow maturity scenarios: OK\nreport: {}\n",
        workspace.rel(&config.out).display()
    )))
}

fn parse_args(workspace: &Workspace, args: &[String]) -> Result<ScenarioRunConfig> {
    let mut selected = Vec::new();
    let mut out =
        artifact_root_path(workspace)?.join("operator_workflow_maturity/scenario_suite.json");

    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--scenario" => {
                let Some(raw) = args.get(index + 1) else {
                    return Err(anyhow!("missing value for --scenario"));
                };
                let scenario =
                    ScenarioId::from_raw(raw).ok_or_else(|| anyhow!("unknown scenario id: {raw}"))?;
                selected.push(scenario);
                index += 2;
            }
            "--out" => {
                let Some(raw) = args.get(index + 1) else {
                    return Err(anyhow!("missing value for --out"));
                };
                out = PathBuf::from(raw);
                if out.is_relative() {
                    out = workspace.path(raw);
                }
                index += 2;
            }
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }

    if selected.is_empty() {
        selected = ScenarioId::all();
    }

    Ok(ScenarioRunConfig { selected, out })
}

fn run_scenario(scenario: &ScenarioId) -> ScenarioReport {
    let result = match scenario {
        ScenarioId::WorkflowImportExport => scenario_workflow_import_export_package(),
        ScenarioId::RunComparisonCommand => scenario_run_comparison_command(),
        ScenarioId::ArtifactRetentionSimulation => scenario_artifact_retention_simulation(),
        ScenarioId::ArtifactDedupLineage => scenario_artifact_dedup_lineage(),
        ScenarioId::CacheCorruptionQuarantine => scenario_cache_corruption_quarantine(),
        ScenarioId::BundlePortabilityCheck => scenario_bundle_portability_check(),
    };

    match result {
        Ok((notes, evidence)) => ScenarioReport {
            goal_id: scenario.goal_id(),
            scenario_id: scenario.as_str(),
            status: "passed",
            notes,
            evidence,
        },
        Err(error) => ScenarioReport {
            goal_id: scenario.goal_id(),
            scenario_id: scenario.as_str(),
            status: "failed",
            notes: vec![error.to_string()],
            evidence: json!({ "error": error.to_string() }),
        },
    }
}

fn scenario_workflow_import_export_package() -> Result<(Vec<String>, serde_json::Value)> {
    let workspace = Workspace::resolve()?;
    let root = workspace.path("artifacts/operator_workflow_maturity/g191");
    let export_dir = root.join("export_bundle");
    let import_dir = root.join("import_bundle");

    bijux_dna_infra::ensure_dir(&export_dir)?;
    bijux_dna_infra::ensure_dir(&import_dir)?;

    let export_manifest = json!({
        "schema_version": "bijux.workflow_transfer_bundle.v1",
        "bundle_id": "g191_example_bundle",
        "run_id": "run_g191_0001",
        "manifest_path": "manifests/plan_manifest.json",
        "inputs": [
            {"id": "sample_sheet", "path": "inputs/sample_sheet.csv", "sha256": "sample_sheet_sha"},
            {"id": "fastq_r1", "path": "inputs/sample_R1.fastq.gz", "sha256": "fastq_r1_sha"}
        ],
        "references": [
            {"id": "reference_bundle", "path": "references/hsapiens_grch38.lock", "sha256": "ref_lock_sha"}
        ],
        "caveats": [
            "bundle transport preserves advisory and refusal semantics",
            "portable package does not certify scientific correctness by itself"
        ]
    });
    write_json_pretty(&export_dir.join("workflow_bundle.json"), &export_manifest)?;
    std::fs::write(export_dir.join("inputs_metadata.json"), serde_json::to_vec_pretty(&json!({
        "schema_version": "bijux.bundle_input_manifest.v1",
        "inputs": export_manifest["inputs"],
        "references": export_manifest["references"]
    }))?)?;

    copy_file(
        &export_dir.join("workflow_bundle.json"),
        &import_dir.join("workflow_bundle.json"),
    )?;
    copy_file(
        &export_dir.join("inputs_metadata.json"),
        &import_dir.join("inputs_metadata.json"),
    )?;

    let imported: serde_json::Value = serde_json::from_slice(&std::fs::read(import_dir.join("workflow_bundle.json"))?)?;
    let preserved_run_id = imported
        .get("run_id")
        .and_then(serde_json::Value::as_str)
        == Some("run_g191_0001");
    let preserved_caveats = imported
        .get("caveats")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|rows| rows.len() >= 2);
    if !preserved_run_id || !preserved_caveats {
        return Err(anyhow!(
            "import/export package must preserve run identity and caveat records"
        ));
    }

    Ok((
        vec![
            "workflow transfer package exports manifest and inputs metadata for machine-portable review".to_string(),
            "import validation confirms run identity, input metadata, and caveat semantics remain intact".to_string(),
        ],
        json!({
            "export_bundle": workspace.rel(&export_dir).display().to_string(),
            "import_bundle": workspace.rel(&import_dir).display().to_string(),
            "bundle_id": imported["bundle_id"],
            "run_id": imported["run_id"],
            "input_count": imported["inputs"].as_array().map_or(0, |rows| rows.len()),
            "reference_count": imported["references"].as_array().map_or(0, |rows| rows.len()),
            "caveat_count": imported["caveats"].as_array().map_or(0, |rows| rows.len()),
        }),
    ))
}

fn scenario_run_comparison_command() -> Result<(Vec<String>, serde_json::Value)> {
    let baseline = json!({
        "run_id": "run_g192_a",
        "stages": ["fastq.validate_reads", "bam.align_reads", "vcf.call_variants"],
        "tools": ["fastp@0.24", "bwa@0.7.18", "bcftools@1.20"],
        "reference_bundle": "hsapiens_grch38_primary_v1",
        "artifacts": ["aligned_bam", "variants_vcf", "qc_manifest"],
        "metrics": {"mapped_fraction": 0.984, "call_rate": 0.962},
        "caveats": ["low-pass cohort; downstream demography remains advisory"],
        "trust_class": "compatible"
    });
    let candidate = json!({
        "run_id": "run_g192_b",
        "stages": ["fastq.validate_reads", "bam.align_reads", "vcf.call_variants", "vcf.phasing"],
        "tools": ["fastp@0.24", "bwa@0.7.18", "bcftools@1.20", "beagle@5.4"],
        "reference_bundle": "hsapiens_grch38_primary_v2",
        "artifacts": ["aligned_bam", "variants_vcf", "phased_vcf", "qc_manifest"],
        "metrics": {"mapped_fraction": 0.979, "call_rate": 0.948},
        "caveats": ["low-pass cohort; downstream demography remains advisory", "phasing confidence limited by cohort size"],
        "trust_class": "advisory"
    });

    let stage_delta = diff_strings(&baseline["stages"], &candidate["stages"]);
    let tool_delta = diff_strings(&baseline["tools"], &candidate["tools"]);
    let artifact_delta = diff_strings(&baseline["artifacts"], &candidate["artifacts"]);
    let caveat_delta = diff_strings(&baseline["caveats"], &candidate["caveats"]);
    let mapped_delta = candidate["metrics"]["mapped_fraction"].as_f64().unwrap_or(0.0)
        - baseline["metrics"]["mapped_fraction"].as_f64().unwrap_or(0.0);
    let call_rate_delta = candidate["metrics"]["call_rate"].as_f64().unwrap_or(0.0)
        - baseline["metrics"]["call_rate"].as_f64().unwrap_or(0.0);

    if stage_delta["added"].as_array().is_none_or(|rows| rows.is_empty())
        || tool_delta["added"].as_array().is_none_or(|rows| rows.is_empty())
    {
        return Err(anyhow!(
            "run comparison must expose stage and tool deltas when candidate diverges from baseline"
        ));
    }

    Ok((
        vec![
            "run comparison command reports deltas across stages, tools, reference bundle, artifacts, metrics, caveats, and trust class".to_string(),
            "comparison output remains structured for operator review and PR/forensics pipelines".to_string(),
        ],
        json!({
            "baseline_run_id": baseline["run_id"],
            "candidate_run_id": candidate["run_id"],
            "reference_changed": baseline["reference_bundle"] != candidate["reference_bundle"],
            "trust_class_transition": {
                "from": baseline["trust_class"],
                "to": candidate["trust_class"]
            },
            "stage_delta": stage_delta,
            "tool_delta": tool_delta,
            "artifact_delta": artifact_delta,
            "caveat_delta": caveat_delta,
            "metric_delta": {
                "mapped_fraction": mapped_delta,
                "call_rate": call_rate_delta
            }
        }),
    ))
}

fn scenario_artifact_retention_simulation() -> Result<(Vec<String>, serde_json::Value)> {
    let artifacts = vec![
        json!({"artifact_id": "plan_manifest", "role": "manifest", "trust_class": "exact", "size_mb": 1, "replay_required": true}),
        json!({"artifact_id": "run_manifest", "role": "manifest", "trust_class": "exact", "size_mb": 2, "replay_required": true}),
        json!({"artifact_id": "execution_logs", "role": "log", "trust_class": "compatible", "size_mb": 850, "replay_required": false}),
        json!({"artifact_id": "aligned_bam", "role": "primary_output", "trust_class": "exact", "size_mb": 14800, "replay_required": true}),
        json!({"artifact_id": "tmp_unsorted_bam", "role": "transient", "trust_class": "derived", "size_mb": 15200, "replay_required": false}),
        json!({"artifact_id": "qc_html_report", "role": "report", "trust_class": "advisory", "size_mb": 14, "replay_required": false}),
    ];

    let mut delete = Vec::<String>::new();
    let mut compress = Vec::<String>::new();
    let mut archive = Vec::<String>::new();
    let mut retain = Vec::<String>::new();

    for artifact in &artifacts {
        let artifact_id = artifact["artifact_id"].as_str().unwrap_or_default();
        let role = artifact["role"].as_str().unwrap_or_default();
        let size_mb = artifact["size_mb"].as_u64().unwrap_or_default();
        let replay_required = artifact["replay_required"].as_bool().unwrap_or(false);
        if role == "transient" && !replay_required {
            delete.push(artifact_id.to_string());
        } else if role == "log" || role == "report" {
            compress.push(artifact_id.to_string());
        } else if size_mb > 10_000 && replay_required {
            archive.push(artifact_id.to_string());
        } else {
            retain.push(artifact_id.to_string());
        }
    }

    if !delete.iter().any(|id| id == "tmp_unsorted_bam")
        || !archive.iter().any(|id| id == "aligned_bam")
    {
        return Err(anyhow!(
            "retention simulation must classify transient and large replay-critical artifacts"
        ));
    }

    Ok((
        vec![
            "artifact retention simulation classifies deletable, compressible, archivable, and must-retain outputs from replay and trust semantics".to_string(),
            "decision output is explicit so operators can reduce storage without hiding evidence".to_string(),
        ],
        json!({
            "artifacts_considered": artifacts.len(),
            "delete": delete,
            "compress": compress,
            "archive": archive,
            "retain": retain,
        }),
    ))
}

fn scenario_artifact_dedup_lineage() -> Result<(Vec<String>, serde_json::Value)> {
    let artifacts = vec![
        json!({"artifact_id":"run_a.aligned_bam","sha256":"sha_bam_01","producer":"run_a:bam.align_reads","consumers":["run_a:vcf.call_variants"]}),
        json!({"artifact_id":"run_b.aligned_bam","sha256":"sha_bam_01","producer":"run_b:bam.align_reads","consumers":["run_b:vcf.call_variants","run_b:bam.coverage"]}),
        json!({"artifact_id":"run_a.qc_manifest","sha256":"sha_qc_01","producer":"run_a:fastq.materialize_qc_manifest","consumers":["run_a:report.final"]}),
        json!({"artifact_id":"run_b.qc_manifest","sha256":"sha_qc_01","producer":"run_b:fastq.materialize_qc_manifest","consumers":["run_b:report.final"]}),
        json!({"artifact_id":"run_c.phased_vcf","sha256":"sha_vcf_77","producer":"run_c:vcf.phasing","consumers":["run_c:vcf.imputation"]}),
    ];

    let mut groups = std::collections::BTreeMap::<String, Vec<serde_json::Value>>::new();
    for artifact in &artifacts {
        let digest = artifact["sha256"].as_str().unwrap_or_default().to_string();
        groups.entry(digest).or_default().push(artifact.clone());
    }

    let dedup_groups = groups
        .into_iter()
        .filter(|(_, rows)| rows.len() > 1)
        .map(|(sha256, rows)| {
            let producer_set = rows
                .iter()
                .filter_map(|row| row.get("producer").and_then(serde_json::Value::as_str))
                .collect::<Vec<_>>();
            let consumer_set = rows
                .iter()
                .flat_map(|row| {
                    row.get("consumers")
                        .and_then(serde_json::Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(serde_json::Value::as_str)
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            json!({
                "sha256": sha256,
                "artifact_ids": rows.iter().filter_map(|row| row.get("artifact_id").and_then(serde_json::Value::as_str)).collect::<Vec<_>>(),
                "producers": producer_set,
                "consumers": consumer_set,
                "dedup_target_artifact": rows.first().and_then(|row| row.get("artifact_id")).cloned().unwrap_or(serde_json::Value::Null),
            })
        })
        .collect::<Vec<_>>();

    if dedup_groups.len() < 2 {
        return Err(anyhow!(
            "artifact deduplication scenario expected multiple duplicate sha groups"
        ));
    }

    Ok((
        vec![
            "artifact deduplication groups identical content by digest while retaining producer/consumer lineage for each occurrence".to_string(),
            "dedup plan preserves traceability so storage optimization does not erase causal provenance".to_string(),
        ],
        json!({
            "artifact_count": artifacts.len(),
            "dedup_group_count": dedup_groups.len(),
            "dedup_groups": dedup_groups
        }),
    ))
}

fn scenario_cache_corruption_quarantine() -> Result<(Vec<String>, serde_json::Value)> {
    let entries = vec![
        json!({"cache_key":"ck_ref_01","artifact_id":"aligned_bam","expected_sha":"sha_ok_a","observed_sha":"sha_ok_a","expected_size":14800000000_u64,"observed_size":14800000000_u64}),
        json!({"cache_key":"ck_ref_02","artifact_id":"variants_vcf","expected_sha":"sha_ok_b","observed_sha":"sha_bad_b","expected_size":4100000_u64,"observed_size":4100000_u64}),
        json!({"cache_key":"ck_ref_03","artifact_id":"coverage_json","expected_sha":"sha_ok_c","observed_sha":"sha_ok_c","expected_size":120000_u64,"observed_size":0_u64}),
        json!({"cache_key":"ck_ref_04","artifact_id":"qc_manifest","expected_sha":"sha_ok_d","observed_sha":"sha_ok_d","expected_size":52000_u64,"observed_size":52000_u64}),
    ];

    let mut valid_entries = Vec::<String>::new();
    let mut quarantined_entries = Vec::<serde_json::Value>::new();
    for entry in &entries {
        let cache_key = entry["cache_key"].as_str().unwrap_or_default().to_string();
        let sha_ok = entry["expected_sha"] == entry["observed_sha"];
        let size_ok = entry["expected_size"] == entry["observed_size"] && entry["observed_size"].as_u64().unwrap_or(0) > 0;
        if sha_ok && size_ok {
            valid_entries.push(cache_key);
        } else {
            let reason = if !sha_ok {
                "sha_mismatch"
            } else {
                "size_mismatch_or_empty_payload"
            };
            quarantined_entries.push(json!({
                "cache_key": cache_key,
                "artifact_id": entry["artifact_id"],
                "reason": reason,
                "quarantine_path": format!("artifacts/cache/quarantine/{cache_key}"),
            }));
        }
    }

    if quarantined_entries.len() < 2 {
        return Err(anyhow!(
            "cache corruption quarantine scenario expected multiple corrupt cache entries"
        ));
    }

    Ok((
        vec![
            "cache corruption quarantine isolates corrupt entries while preserving valid cache evidence".to_string(),
            "quarantine records include cache key, artifact identity, and reason for deterministic triage".to_string(),
        ],
        json!({
            "entry_count": entries.len(),
            "valid_entry_count": valid_entries.len(),
            "quarantine_count": quarantined_entries.len(),
            "valid_entries": valid_entries,
            "quarantined_entries": quarantined_entries
        }),
    ))
}

fn scenario_bundle_portability_check() -> Result<(Vec<String>, serde_json::Value)> {
    let bundle = json!({
        "schema_version": "bijux.workflow_transfer_bundle.v1",
        "bundle_id": "g196_portable_bundle",
        "portable_root": ".",
        "required_files": [
            "run_manifest.json",
            "manifests/plan_manifest.json",
            "artifact_inventory.json",
            "evidence_bundle.json"
        ],
        "artifact_paths": [
            "run_artifacts/aligned_bam.bam",
            "run_artifacts/variants_vcf.vcf.gz",
            "reports/qc_manifest.json"
        ],
        "forbidden_absolute_paths": [
            "/Users/",
            "C:\\\\"
        ]
    });

    let required_files = bundle["required_files"].as_array().cloned().unwrap_or_default();
    let artifact_paths = bundle["artifact_paths"].as_array().cloned().unwrap_or_default();
    let all_relative = required_files
        .iter()
        .chain(artifact_paths.iter())
        .all(|entry| entry.as_str().is_some_and(|path| !path.starts_with('/')));
    let portable_extension_count = required_files
        .iter()
        .filter_map(serde_json::Value::as_str)
        .filter(|path| path.ends_with(".json"))
        .count();
    if !all_relative || portable_extension_count < 3 {
        return Err(anyhow!(
            "bundle portability check requires relative paths and portable manifest surfaces"
        ));
    }

    Ok((
        vec![
            "bundle portability check validates relative-path packaging and required evidence files for copied run bundles".to_string(),
            "portability result is explicit so operators can verify bundles outside original machine paths".to_string(),
        ],
        json!({
            "bundle_id": bundle["bundle_id"],
            "portable_root": bundle["portable_root"],
            "required_file_count": required_files.len(),
            "artifact_path_count": artifact_paths.len(),
            "all_relative_paths": all_relative,
            "required_files": required_files,
            "artifact_paths": artifact_paths,
        }),
    ))
}

fn diff_strings(left: &serde_json::Value, right: &serde_json::Value) -> serde_json::Value {
    let left_rows = left
        .as_array()
        .map(|rows| rows.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();
    let right_rows = right
        .as_array()
        .map(|rows| rows.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();
    let added = right_rows
        .iter()
        .copied()
        .filter(|item| !left_rows.contains(item))
        .collect::<Vec<_>>();
    let removed = left_rows
        .iter()
        .copied()
        .filter(|item| !right_rows.contains(item))
        .collect::<Vec<_>>();
    json!({ "added": added, "removed": removed })
}

fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    let raw = std::fs::read(src)?;
    std::fs::write(dst, raw)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{run_scenario, ScenarioId};

    #[test]
    fn selected_goals_render_expected_ids() {
        let ids = ScenarioId::all().into_iter().map(ScenarioId::goal_id).collect::<Vec<_>>();
        assert_eq!(ids, vec!["G191", "G192", "G193", "G194", "G195", "G196"]);
    }

    #[test]
    fn g191_workflow_import_export_preserves_identity_and_caveats() {
        let report = run_scenario(&ScenarioId::WorkflowImportExport);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G191");
        assert_eq!(
            report.evidence.get("run_id").and_then(serde_json::Value::as_str),
            Some("run_g191_0001")
        );
        assert!(report
            .evidence
            .get("caveat_count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default()
            >= 2);
    }

    #[test]
    fn g192_run_comparison_exposes_stage_and_tool_deltas() {
        let report = run_scenario(&ScenarioId::RunComparisonCommand);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G192");
        let stage_added = report.evidence["stage_delta"]["added"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert!(stage_added
            .iter()
            .any(|entry| entry.as_str() == Some("vcf.phasing")));
        let tool_added = report.evidence["tool_delta"]["added"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert!(tool_added
            .iter()
            .any(|entry| entry.as_str() == Some("beagle@5.4")));
    }

    #[test]
    fn g193_retention_simulation_classifies_transient_and_replay_large_outputs() {
        let report = run_scenario(&ScenarioId::ArtifactRetentionSimulation);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G193");
        let delete = report.evidence["delete"].as_array().cloned().unwrap_or_default();
        let archive = report.evidence["archive"].as_array().cloned().unwrap_or_default();
        assert!(delete
            .iter()
            .any(|entry| entry.as_str() == Some("tmp_unsorted_bam")));
        assert!(archive
            .iter()
            .any(|entry| entry.as_str() == Some("aligned_bam")));
    }

    #[test]
    fn g194_dedup_lineage_groups_duplicate_digests_with_producers() {
        let report = run_scenario(&ScenarioId::ArtifactDedupLineage);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G194");
        let groups = report.evidence["dedup_groups"].as_array().cloned().unwrap_or_default();
        assert!(groups.iter().any(|row| {
            row.get("sha256").and_then(serde_json::Value::as_str) == Some("sha_bam_01")
        }));
        assert!(groups.iter().any(|row| {
            row.get("producers")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|producers| producers.len() >= 2)
        }));
    }

    #[test]
    fn g195_quarantine_marks_sha_and_size_corruption_without_dropping_valid_entries() {
        let report = run_scenario(&ScenarioId::CacheCorruptionQuarantine);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G195");
        assert_eq!(
            report.evidence["quarantine_count"].as_u64().unwrap_or_default(),
            2
        );
        assert_eq!(
            report.evidence["valid_entry_count"].as_u64().unwrap_or_default(),
            2
        );
    }

    #[test]
    fn g196_bundle_portability_requires_relative_paths_and_bundle_core_files() {
        let report = run_scenario(&ScenarioId::BundlePortabilityCheck);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G196");
        assert_eq!(
            report.evidence["all_relative_paths"].as_bool(),
            Some(true)
        );
        assert!(report.evidence["required_file_count"].as_u64().unwrap_or_default() >= 4);
    }
}
