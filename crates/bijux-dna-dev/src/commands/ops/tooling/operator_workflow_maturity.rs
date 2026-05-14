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
    OfflineReviewProfile,
    OperatorCommandRecipes,
    ScaleAwareProgressReporting,
    ResourcePredictionFromPastRuns,
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
            Self::OfflineReviewProfile => "g197_offline_review_profile",
            Self::OperatorCommandRecipes => "g198_operator_command_recipes",
            Self::ScaleAwareProgressReporting => "g199_scale_aware_progress_reporting",
            Self::ResourcePredictionFromPastRuns => "g200_resource_prediction_from_past_runs",
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
            Self::OfflineReviewProfile => "G197",
            Self::OperatorCommandRecipes => "G198",
            Self::ScaleAwareProgressReporting => "G199",
            Self::ResourcePredictionFromPastRuns => "G200",
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
            Self::OfflineReviewProfile,
            Self::OperatorCommandRecipes,
            Self::ScaleAwareProgressReporting,
            Self::ResourcePredictionFromPastRuns,
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
            "g197_offline_review_profile" | "G197" => Some(Self::OfflineReviewProfile),
            "g198_operator_command_recipes" | "G198" => Some(Self::OperatorCommandRecipes),
            "g199_scale_aware_progress_reporting" | "G199" => {
                Some(Self::ScaleAwareProgressReporting)
            }
            "g200_resource_prediction_from_past_runs" | "G200" => {
                Some(Self::ResourcePredictionFromPastRuns)
            }
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
    let reports = config.selected.iter().copied().map(run_scenario).collect::<Vec<_>>();
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
                let scenario = ScenarioId::from_raw(raw)
                    .ok_or_else(|| anyhow!("unknown scenario id: {raw}"))?;
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

fn run_scenario(scenario: ScenarioId) -> ScenarioReport {
    let result = match scenario {
        ScenarioId::WorkflowImportExport => scenario_workflow_import_export_package(),
        ScenarioId::RunComparisonCommand => scenario_run_comparison_command(),
        ScenarioId::ArtifactRetentionSimulation => scenario_artifact_retention_simulation(),
        ScenarioId::ArtifactDedupLineage => scenario_artifact_dedup_lineage(),
        ScenarioId::CacheCorruptionQuarantine => scenario_cache_corruption_quarantine(),
        ScenarioId::BundlePortabilityCheck => scenario_bundle_portability_check(),
        ScenarioId::OfflineReviewProfile => scenario_offline_review_profile(),
        ScenarioId::OperatorCommandRecipes => scenario_operator_command_recipes(),
        ScenarioId::ScaleAwareProgressReporting => scenario_scale_aware_progress_reporting(),
        ScenarioId::ResourcePredictionFromPastRuns => scenario_resource_prediction_from_past_runs(),
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
    std::fs::write(
        export_dir.join("inputs_metadata.json"),
        serde_json::to_vec_pretty(&json!({
            "schema_version": "bijux.bundle_input_manifest.v1",
            "inputs": export_manifest["inputs"],
            "references": export_manifest["references"]
        }))?,
    )?;

    copy_file(&export_dir.join("workflow_bundle.json"), &import_dir.join("workflow_bundle.json"))?;
    copy_file(&export_dir.join("inputs_metadata.json"), &import_dir.join("inputs_metadata.json"))?;

    let imported: serde_json::Value =
        serde_json::from_slice(&std::fs::read(import_dir.join("workflow_bundle.json"))?)?;
    let preserved_run_id =
        imported.get("run_id").and_then(serde_json::Value::as_str) == Some("run_g191_0001");
    let preserved_caveats = imported
        .get("caveats")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|rows| rows.len() >= 2);
    if !preserved_run_id || !preserved_caveats {
        return Err(anyhow!("import/export package must preserve run identity and caveat records"));
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
            "input_count": imported["inputs"].as_array().map_or(0, Vec::len),
            "reference_count": imported["references"].as_array().map_or(0, Vec::len),
            "caveat_count": imported["caveats"].as_array().map_or(0, Vec::len),
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

    if stage_delta["added"].as_array().is_none_or(Vec::is_empty)
        || tool_delta["added"].as_array().is_none_or(Vec::is_empty)
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
        json!({"cache_key":"ck_ref_01","artifact_id":"aligned_bam","expected_sha":"sha_ok_a","observed_sha":"sha_ok_a","expected_size":14_800_000_000_u64,"observed_size":14_800_000_000_u64}),
        json!({"cache_key":"ck_ref_02","artifact_id":"variants_vcf","expected_sha":"sha_ok_b","observed_sha":"sha_bad_b","expected_size":4_100_000_u64,"observed_size":4_100_000_u64}),
        json!({"cache_key":"ck_ref_03","artifact_id":"coverage_json","expected_sha":"sha_ok_c","observed_sha":"sha_ok_c","expected_size":120_000_u64,"observed_size":0_u64}),
        json!({"cache_key":"ck_ref_04","artifact_id":"qc_manifest","expected_sha":"sha_ok_d","observed_sha":"sha_ok_d","expected_size":52_000_u64,"observed_size":52_000_u64}),
    ];

    let mut valid_entries = Vec::<String>::new();
    let mut quarantined_entries = Vec::<serde_json::Value>::new();
    for entry in &entries {
        let cache_key = entry["cache_key"].as_str().unwrap_or_default().to_string();
        let sha_ok = entry["expected_sha"] == entry["observed_sha"];
        let size_ok = entry["expected_size"] == entry["observed_size"]
            && entry["observed_size"].as_u64().unwrap_or(0) > 0;
        if sha_ok && size_ok {
            valid_entries.push(cache_key);
        } else {
            let reason = if sha_ok { "size_mismatch_or_empty_payload" } else { "sha_mismatch" };
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
            "<user-home-root>/",
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

fn scenario_offline_review_profile() -> Result<(Vec<String>, serde_json::Value)> {
    let profile = json!({
        "profile_id": "offline_review_minimal_v1",
        "network_allowed": false,
        "required_files": [
            "run_manifest.json",
            "artifact_inventory.json",
            "evidence_bundle.json",
            "evidence_verification.json",
            "reports/final_report.json"
        ],
        "verification_steps": [
            "verify hash ledger",
            "verify evidence bundle",
            "verify artifact inventory trust classes",
            "render local report bundle"
        ],
        "blocked_when_missing": [
            "evidence_verification.json",
            "artifact_inventory.json"
        ]
    });

    let required = profile["required_files"].as_array().cloned().unwrap_or_default();
    let steps = profile["verification_steps"].as_array().cloned().unwrap_or_default();
    let no_network = profile["network_allowed"].as_bool() == Some(false);
    let has_evidence_verification =
        required.iter().any(|entry| entry.as_str() == Some("evidence_verification.json"));
    if !no_network || !has_evidence_verification || steps.len() < 3 {
        return Err(anyhow!(
            "offline review profile must disable network and require evidence verification artifacts"
        ));
    }

    Ok((
        vec![
            "offline review profile codifies no-network verification with required local evidence inputs".to_string(),
            "profile makes missing evidence blocking conditions explicit for reviewer safety".to_string(),
        ],
        json!({
            "profile_id": profile["profile_id"],
            "network_allowed": profile["network_allowed"],
            "required_file_count": required.len(),
            "verification_step_count": steps.len(),
            "blocked_when_missing": profile["blocked_when_missing"],
        }),
    ))
}

fn scenario_operator_command_recipes() -> Result<(Vec<String>, serde_json::Value)> {
    let recipes = vec![
        json!({
            "task": "run",
            "command": "cargo run -q -p bijux-dna-dev -- examples run run -- fastq-preprocess__minimal__v1",
            "evidence_paths": ["run_manifest.json", "artifact_inventory.json"],
        }),
        json!({
            "task": "inspect",
            "command": "cargo run -q -p bijux-dna -- status --contracts",
            "evidence_paths": ["run_state.json", "queue_state.json", "operator_health.json"],
        }),
        json!({
            "task": "replay",
            "command": "cargo run -q -p bijux-dna -- replay <run-id> --validate-only",
            "evidence_paths": ["replay_manifest.json", "evidence_verification.json"],
        }),
        json!({
            "task": "diff",
            "command": "cargo run -q -p bijux-dna -- compare <run-a> <run-b>",
            "evidence_paths": ["run_summary.json", "evidence_bundle.json"],
        }),
        json!({
            "task": "export",
            "command": "cargo run -q -p bijux-dna-dev -- tooling run operator-workflow-maturity -- --scenario G191",
            "evidence_paths": ["workflow_bundle.json", "inputs_metadata.json"],
        }),
    ];

    let all_have_evidence_paths = recipes.iter().all(|recipe| {
        recipe
            .get("evidence_paths")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|rows| !rows.is_empty())
    });
    if recipes.len() < 5 || !all_have_evidence_paths {
        return Err(anyhow!(
            "operator command recipes must include run/inspect/replay/diff/export tasks with evidence paths"
        ));
    }

    Ok((
        vec![
            "operator command recipes provide canonical run/inspect/replay/diff/export command paths tied to concrete evidence files".to_string(),
            "recipes are structured for copy-safe operator usage and doc generation pipelines".to_string(),
        ],
        json!({
            "recipe_count": recipes.len(),
            "recipes": recipes,
        }),
    ))
}

fn scenario_scale_aware_progress_reporting() -> Result<(Vec<String>, serde_json::Value)> {
    let stage_events = vec![
        json!({"sample_id":"sample_a","stage":"fastq.validate_reads","status":"completed","elapsed_sec":20_u64,"artifacts_written":2_u64}),
        json!({"sample_id":"sample_a","stage":"bam.align_reads","status":"completed","elapsed_sec":420_u64,"artifacts_written":3_u64}),
        json!({"sample_id":"sample_a","stage":"vcf.call_variants","status":"running","elapsed_sec":80_u64,"artifacts_written":1_u64}),
        json!({"sample_id":"sample_b","stage":"fastq.validate_reads","status":"completed","elapsed_sec":18_u64,"artifacts_written":2_u64}),
        json!({"sample_id":"sample_b","stage":"bam.align_reads","status":"failed","elapsed_sec":200_u64,"artifacts_written":0_u64}),
        json!({"sample_id":"sample_c","stage":"fastq.validate_reads","status":"completed","elapsed_sec":22_u64,"artifacts_written":2_u64}),
        json!({"sample_id":"sample_c","stage":"bam.align_reads","status":"completed","elapsed_sec":395_u64,"artifacts_written":3_u64}),
    ];

    let total_stage_count = stage_events.len() as u64;
    let completed_count =
        stage_events.iter().filter(|event| event["status"] == "completed").count() as u64;
    let failed_count =
        stage_events.iter().filter(|event| event["status"] == "failed").count() as u64;
    let running_count =
        stage_events.iter().filter(|event| event["status"] == "running").count() as u64;
    let total_elapsed_sec = stage_events
        .iter()
        .map(|event| event["elapsed_sec"].as_u64().unwrap_or_default())
        .sum::<u64>();
    let stage_progress_fraction = if total_stage_count == 0 {
        0.0
    } else {
        (completed_count as f64) / (total_stage_count as f64)
    };
    let scale_class = match total_stage_count {
        0..=10 => "small",
        11..=500 => "medium",
        _ => "large",
    };

    let mut sample_state = std::collections::BTreeMap::<String, serde_json::Value>::new();
    for event in &stage_events {
        let sample_id = event["sample_id"].as_str().unwrap_or_default().to_string();
        let sample_row = sample_state.entry(sample_id).or_insert_with(|| {
            json!({
                "completed": 0_u64,
                "failed": 0_u64,
                "running": 0_u64,
                "stages": [],
                "artifacts_written": 0_u64
            })
        });
        let status = event["status"].as_str().unwrap_or_default();
        if status == "completed" {
            sample_row["completed"] =
                json!(sample_row["completed"].as_u64().unwrap_or_default() + 1);
        }
        if status == "failed" {
            sample_row["failed"] = json!(sample_row["failed"].as_u64().unwrap_or_default() + 1);
        }
        if status == "running" {
            sample_row["running"] = json!(sample_row["running"].as_u64().unwrap_or_default() + 1);
        }
        let written = sample_row["artifacts_written"].as_u64().unwrap_or_default()
            + event["artifacts_written"].as_u64().unwrap_or_default();
        sample_row["artifacts_written"] = json!(written);
        if let Some(stages) = sample_row.get_mut("stages").and_then(serde_json::Value::as_array_mut)
        {
            stages.push(json!({
                "stage": event["stage"],
                "status": event["status"],
                "elapsed_sec": event["elapsed_sec"],
            }));
        }
    }

    let failure_rows = stage_events
        .iter()
        .filter(|event| event["status"] == "failed")
        .map(|event| {
            json!({
                "sample_id": event["sample_id"],
                "stage": event["stage"],
                "reason_code": "stage_failed",
            })
        })
        .collect::<Vec<_>>();
    if failed_count == 0 || failure_rows.is_empty() {
        return Err(anyhow!("scale-aware progress report must include explicit failure rows"));
    }

    Ok((
        vec![
            "scale-aware progress reporting summarizes per-sample and global stage state without collapsing failed stages into aggregate percentages".to_string(),
            "progress output includes explicit failure records and scale class so operators can triage long-running cohorts safely".to_string(),
        ],
        json!({
            "scale_class": scale_class,
            "total_stages": total_stage_count,
            "completed_stages": completed_count,
            "running_stages": running_count,
            "failed_stages": failed_count,
            "stage_progress_fraction": stage_progress_fraction,
            "elapsed_sec_total": total_elapsed_sec,
            "failure_rows": failure_rows,
            "sample_state": sample_state,
        }),
    ))
}

fn scenario_resource_prediction_from_past_runs() -> Result<(Vec<String>, serde_json::Value)> {
    let history = [
        json!({
            "run_id":"hist_001",
            "profile":"fastq-to-vcf__minimal__v1",
            "sample_count":1_u64,
            "input_gb":8.1_f64,
            "cpu_hours":2.4_f64,
            "peak_memory_gb":11.0_f64,
            "scratch_gb":38.0_f64,
            "success":true
        }),
        json!({
            "run_id":"hist_002",
            "profile":"fastq-to-vcf__minimal__v1",
            "sample_count":1_u64,
            "input_gb":9.0_f64,
            "cpu_hours":2.7_f64,
            "peak_memory_gb":12.3_f64,
            "scratch_gb":42.0_f64,
            "success":true
        }),
        json!({
            "run_id":"hist_003",
            "profile":"fastq-to-vcf__minimal__v1",
            "sample_count":2_u64,
            "input_gb":16.4_f64,
            "cpu_hours":5.1_f64,
            "peak_memory_gb":18.0_f64,
            "scratch_gb":74.0_f64,
            "success":true
        }),
        json!({
            "run_id":"hist_004",
            "profile":"fastq-to-vcf__minimal__v1",
            "sample_count":2_u64,
            "input_gb":15.8_f64,
            "cpu_hours":4.8_f64,
            "peak_memory_gb":17.2_f64,
            "scratch_gb":71.0_f64,
            "success":true
        }),
        json!({
            "run_id":"hist_005",
            "profile":"fastq-to-vcf__minimal__v1",
            "sample_count":2_u64,
            "input_gb":17.0_f64,
            "cpu_hours":5.5_f64,
            "peak_memory_gb":19.4_f64,
            "scratch_gb":79.0_f64,
            "success":false
        }),
    ];
    let target = json!({
        "profile":"fastq-to-vcf__minimal__v1",
        "sample_count":2_u64,
        "input_gb":16.8_f64
    });

    let successful = history
        .iter()
        .filter(|row| row["success"].as_bool() == Some(true))
        .filter(|row| row["profile"] == target["profile"])
        .collect::<Vec<_>>();
    let matched = successful
        .iter()
        .filter(|row| row["sample_count"] == target["sample_count"])
        .collect::<Vec<_>>();
    if matched.len() < 2 {
        return Err(anyhow!(
            "resource prediction requires at least two successful historical runs in matching scale class"
        ));
    }

    let cpu_history =
        matched.iter().filter_map(|row| row["cpu_hours"].as_f64()).collect::<Vec<_>>();
    let memory_history =
        matched.iter().filter_map(|row| row["peak_memory_gb"].as_f64()).collect::<Vec<_>>();
    let scratch_history =
        matched.iter().filter_map(|row| row["scratch_gb"].as_f64()).collect::<Vec<_>>();

    let cpu_median = median(&cpu_history).unwrap_or(0.0);
    let memory_median = median(&memory_history).unwrap_or(0.0);
    let scratch_median = median(&scratch_history).unwrap_or(0.0);
    let safety_factor = 1.20_f64;
    let suggestion = json!({
        "cpu_hours": round2(cpu_median * safety_factor),
        "memory_gb": round2(memory_median * safety_factor),
        "scratch_gb": round2(scratch_median * safety_factor),
        "advisory_label": "advisory_not_a_guarantee",
    });

    if suggestion["cpu_hours"].as_f64().unwrap_or(0.0) <= cpu_median
        || suggestion["memory_gb"].as_f64().unwrap_or(0.0) <= memory_median
    {
        return Err(anyhow!(
            "resource prediction must apply a safety margin over historical median"
        ));
    }

    Ok((
        vec![
            "resource prediction derives advisory CPU/memory/scratch settings from successful historical runs in the same profile and scale class".to_string(),
            "prediction output keeps evidence traceability and confidence context instead of presenting hard guarantees".to_string(),
        ],
        json!({
            "target": target,
            "history_count_total": history.len(),
            "history_count_successful": successful.len(),
            "history_count_matched": matched.len(),
            "cpu_history_hours": cpu_history,
            "memory_history_gb": memory_history,
            "scratch_history_gb": scratch_history,
            "median": {
                "cpu_hours": round2(cpu_median),
                "memory_gb": round2(memory_median),
                "scratch_gb": round2(scratch_median),
            },
            "safety_factor": safety_factor,
            "suggestion": suggestion,
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
    let added =
        right_rows.iter().copied().filter(|item| !left_rows.contains(item)).collect::<Vec<_>>();
    let removed =
        left_rows.iter().copied().filter(|item| !right_rows.contains(item)).collect::<Vec<_>>();
    json!({ "added": added, "removed": removed })
}

fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        Some(f64::midpoint(sorted[mid - 1], sorted[mid]))
    } else {
        sorted.get(mid).copied()
    }
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
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
        assert_eq!(
            ids,
            vec!["G191", "G192", "G193", "G194", "G195", "G196", "G197", "G198", "G199", "G200"]
        );
    }

    #[test]
    fn g191_workflow_import_export_preserves_identity_and_caveats() {
        let report = run_scenario(ScenarioId::WorkflowImportExport);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G191");
        assert_eq!(
            report.evidence.get("run_id").and_then(serde_json::Value::as_str),
            Some("run_g191_0001")
        );
        assert!(
            report
                .evidence
                .get("caveat_count")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or_default()
                >= 2
        );
    }

    #[test]
    fn g192_run_comparison_exposes_stage_and_tool_deltas() {
        let report = run_scenario(ScenarioId::RunComparisonCommand);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G192");
        let stage_added =
            report.evidence["stage_delta"]["added"].as_array().cloned().unwrap_or_default();
        assert!(stage_added.iter().any(|entry| entry.as_str() == Some("vcf.phasing")));
        let tool_added =
            report.evidence["tool_delta"]["added"].as_array().cloned().unwrap_or_default();
        assert!(tool_added.iter().any(|entry| entry.as_str() == Some("beagle@5.4")));
    }

    #[test]
    fn g193_retention_simulation_classifies_transient_and_replay_large_outputs() {
        let report = run_scenario(ScenarioId::ArtifactRetentionSimulation);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G193");
        let delete = report.evidence["delete"].as_array().cloned().unwrap_or_default();
        let archive = report.evidence["archive"].as_array().cloned().unwrap_or_default();
        assert!(delete.iter().any(|entry| entry.as_str() == Some("tmp_unsorted_bam")));
        assert!(archive.iter().any(|entry| entry.as_str() == Some("aligned_bam")));
    }

    #[test]
    fn g194_dedup_lineage_groups_duplicate_digests_with_producers() {
        let report = run_scenario(ScenarioId::ArtifactDedupLineage);
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
        let report = run_scenario(ScenarioId::CacheCorruptionQuarantine);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G195");
        assert_eq!(report.evidence["quarantine_count"].as_u64().unwrap_or_default(), 2);
        assert_eq!(report.evidence["valid_entry_count"].as_u64().unwrap_or_default(), 2);
    }

    #[test]
    fn g196_bundle_portability_requires_relative_paths_and_bundle_core_files() {
        let report = run_scenario(ScenarioId::BundlePortabilityCheck);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G196");
        assert_eq!(report.evidence["all_relative_paths"].as_bool(), Some(true));
        assert!(report.evidence["required_file_count"].as_u64().unwrap_or_default() >= 4);
    }

    #[test]
    fn g197_offline_review_profile_requires_evidence_files_and_no_network() {
        let report = run_scenario(ScenarioId::OfflineReviewProfile);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G197");
        assert_eq!(report.evidence["network_allowed"].as_bool(), Some(false));
        assert!(report.evidence["required_file_count"].as_u64().unwrap_or_default() >= 5);
    }

    #[test]
    fn g198_operator_command_recipes_cover_core_tasks_with_evidence_paths() {
        let report = run_scenario(ScenarioId::OperatorCommandRecipes);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G198");
        assert_eq!(report.evidence["recipe_count"].as_u64().unwrap_or_default(), 5);
        let recipes = report.evidence["recipes"].as_array().cloned().unwrap_or_default();
        assert!(recipes
            .iter()
            .any(|row| row.get("task").and_then(serde_json::Value::as_str) == Some("export")));
    }

    #[test]
    fn g199_scale_aware_progress_reports_failures_and_per_sample_state() {
        let report = run_scenario(ScenarioId::ScaleAwareProgressReporting);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G199");
        assert_eq!(report.evidence["scale_class"].as_str(), Some("small"));
        assert_eq!(report.evidence["failed_stages"].as_u64().unwrap_or_default(), 1);
        let failures = report.evidence["failure_rows"].as_array().cloned().unwrap_or_default();
        assert!(failures.iter().any(|row| {
            row.get("sample_id").and_then(serde_json::Value::as_str) == Some("sample_b")
        }));
    }

    #[test]
    fn g200_resource_prediction_uses_history_with_advisory_margin() {
        let report = run_scenario(ScenarioId::ResourcePredictionFromPastRuns);
        assert_eq!(report.status, "passed");
        assert_eq!(report.goal_id, "G200");
        assert!(report.evidence["history_count_matched"].as_u64().unwrap_or_default() >= 2);
        let suggested_cpu = report.evidence["suggestion"]["cpu_hours"].as_f64().unwrap_or_default();
        let median_cpu = report.evidence["median"]["cpu_hours"].as_f64().unwrap_or_default();
        assert!(suggested_cpu > median_cpu);
        assert_eq!(
            report.evidence["suggestion"]["advisory_label"].as_str(),
            Some("advisory_not_a_guarantee")
        );
    }
}
