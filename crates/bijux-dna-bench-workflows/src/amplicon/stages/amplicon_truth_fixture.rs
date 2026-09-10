//! Owner: bijux-dna-bench-workflows
//! Amplicon stages amplicon truth fixture.
use super::super::{
    json, path_relative_to_repo, row, stage_root, validate_amplicon_truth_manifest_path,
    AmpliconTruthStageArtifacts, BTreeMap, Context, Path, Result,
};
use crate::execution::stage::StageIdentity;

pub(crate) fn run_amplicon_truth_fixture_stage(
    repo_root: &Path,
    output_root: &Path,
    manifest_path: &Path,
) -> Result<AmpliconTruthStageArtifacts> {
    let report = validate_amplicon_truth_manifest_path(repo_root, manifest_path)?;
    let stage_root = stage_root(output_root, "benchmark.amplicon_truth_fixture");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let row = row(
        StageIdentity {
            stage_id: "benchmark.amplicon_truth_fixture",
            domain: "benchmark",
            tool_id: "bijux",
            execution_mode: "fixture_validation",
        },
        "validated governed amplicon truth bundle contract",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.to_string()),
        BTreeMap::from([(
            "manifest_path".to_string(),
            path_relative_to_repo(repo_root, manifest_path),
        )]),
        BTreeMap::from([("expected_path".to_string(), report.expected_path.clone())]),
        BTreeMap::from([
            ("validated_section_count".to_string(), json!(report.validated_section_count)),
            ("validated_row_count".to_string(), json!(report.validated_row_count)),
            ("valid".to_string(), json!(report.valid)),
        ]),
    );
    Ok(AmpliconTruthStageArtifacts { row, manifest_path: manifest_path.to_path_buf() })
}
