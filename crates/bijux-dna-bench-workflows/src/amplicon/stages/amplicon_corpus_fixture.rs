//! Owner: bijux-dna-bench-workflows
//! Amplicon stages amplicon corpus fixture.
use super::super::{
    json, path_relative_to_repo, row, stage_root, validate_amplicon_corpus_fixture_manifest_path,
    AmpliconCorpusStageArtifacts, BTreeMap, Context, Path, Result,
};
use crate::execution::stage::StageIdentity;

pub(crate) fn run_amplicon_corpus_fixture_stage(
    repo_root: &Path,
    output_root: &Path,
    manifest_path: &Path,
) -> Result<AmpliconCorpusStageArtifacts> {
    let report = validate_amplicon_corpus_fixture_manifest_path(repo_root, manifest_path)?;
    let stage_root = stage_root(output_root, "benchmark.amplicon_corpus_fixture");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &report)?;

    let row = row(
        StageIdentity {
            stage_id: "benchmark.amplicon_corpus_fixture",
            domain: "benchmark",
            tool_id: "bijux",
            execution_mode: "fixture_validation",
        },
        "validated governed amplicon corpus fixture contract",
        Some(path_relative_to_repo(repo_root, &evidence_path)),
        Some(report.schema_version.to_string()),
        BTreeMap::from([(
            "manifest_path".to_string(),
            path_relative_to_repo(repo_root, manifest_path),
        )]),
        BTreeMap::from([
            ("primers_tsv_path".to_string(), report.primers_tsv_path.clone()),
            ("expected_asvs_path".to_string(), report.expected_asvs_path.clone()),
            ("chimera_expectations_path".to_string(), report.chimera_expectations_path.clone()),
            ("amplicon_abundance_table".to_string(), report.abundance_tables[0].table_path.clone()),
        ]),
        BTreeMap::from([
            ("sample_count".to_string(), json!(report.sample_count)),
            ("control_count".to_string(), json!(report.control_count)),
            ("primer_table_row_count".to_string(), json!(report.primer_table_row_count)),
            ("expected_asv_row_count".to_string(), json!(report.expected_asv_row_count)),
            (
                "chimera_expectation_row_count".to_string(),
                json!(report.chimera_expectation_row_count),
            ),
            ("abundance_table_count".to_string(), json!(report.abundance_table_count)),
            ("valid".to_string(), json!(report.valid)),
        ]),
    );
    Ok(AmpliconCorpusStageArtifacts { row, manifest_path: manifest_path.to_path_buf() })
}
