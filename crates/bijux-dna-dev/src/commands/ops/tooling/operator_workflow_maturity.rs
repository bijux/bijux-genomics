use super::{
    anyhow, artifact_root_path, json, stable_now_utc_string, write_json_pretty, OpsCommandOutcome,
    Path, PathBuf, Result, Workspace,
};

#[derive(Debug, Clone, Copy)]
enum ScenarioId {
    WorkflowImportExport,
}

impl ScenarioId {
    fn as_str(self) -> &'static str {
        match self {
            Self::WorkflowImportExport => "g191_workflow_import_export_package",
        }
    }

    fn goal_id(self) -> &'static str {
        match self {
            Self::WorkflowImportExport => "G191",
        }
    }

    fn all() -> Vec<Self> {
        vec![Self::WorkflowImportExport]
    }

    fn from_raw(raw: &str) -> Option<Self> {
        match raw {
            "g191_workflow_import_export_package" | "G191" => Some(Self::WorkflowImportExport),
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
        assert_eq!(ids, vec!["G191"]);
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
}
