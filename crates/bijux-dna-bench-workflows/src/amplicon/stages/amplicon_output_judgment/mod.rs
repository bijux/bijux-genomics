//! Owner: bijux-dna-bench-workflows
//! Amplicon stages amplicon output judgment.
use super::super::{
    json, path_relative_to_repo, row, stage_root, AmpliconMicroPipelineRow, BTreeMap, Path, Result,
    DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH,
};
mod assessment;
use crate::execution::stage::StageIdentity;
use anyhow::Context;

pub(crate) fn run_amplicon_output_judgment_stage(
    repo_root: &Path,
    output_root: &Path,
    stages: &super::super::execution::AmpliconStageArtifacts,
) -> Result<AmpliconMicroPipelineRow> {
    let corpus_manifest_path = &stages.corpus.manifest_path;
    let truth_manifest_path = &stages.truth.manifest_path;

    let evidence = assessment::assess_outputs(repo_root, stages)?;

    let stage_root = stage_root(output_root, "benchmark.amplicon_output_judgment");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let evidence_path = stage_root.join("report.json");
    bijux_dna_infra::atomic_write_json(&evidence_path, &evidence)?;

    Ok(row(StageIdentity { stage_id: "benchmark.amplicon_output_judgment", domain: "benchmark", tool_id: "bijux", execution_mode: "truth_judgment" },
"validated amplicon primer, ASV, chimera, OTU, and abundance outputs against governed truth",
Some(path_relative_to_repo(repo_root, &evidence_path)),
Some(evidence.schema_version.to_string()),
BTreeMap::from([
            (
                "corpus_manifest_path".to_string(),
                path_relative_to_repo(repo_root, corpus_manifest_path),
            ),
            (
                "truth_manifest_path".to_string(),
                path_relative_to_repo(repo_root, truth_manifest_path),
            ),
            (
                "truth_bundle_path".to_string(),
                path_relative_to_repo(repo_root, &repo_root.join(DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH)),
            ),
            (
                "normalized_reads_r1".to_string(),
                path_relative_to_repo(repo_root, &stages.normalize.normalized_reads_r1),
            ),
            (
                "asv_table_tsv".to_string(),
                path_relative_to_repo(repo_root, &stages.infer.asv_table_tsv),
            ),
            (
                "asv_representatives".to_string(),
                path_relative_to_repo(repo_root, &stages.infer.representatives_fasta),
            ),
            (
                "chimeras_tsv".to_string(),
                path_relative_to_repo(repo_root, &stages.chimera.chimeras_tsv),
            ),
            (
                "non_chimeric_representatives".to_string(),
                path_relative_to_repo(repo_root, &stages.chimera.non_chimeric_fasta),
            ),
            (
                "otu_table_tsv".to_string(),
                path_relative_to_repo(repo_root, &stages.otu.otu_table_tsv),
            ),
            (
                "otu_representatives".to_string(),
                path_relative_to_repo(repo_root, &stages.otu.otu_representatives_fasta),
            ),
            (
                "otu_abundance_table".to_string(),
                path_relative_to_repo(repo_root, &stages.abundance.otu_abundance_table_tsv),
            ),
            (
                "normalized_abundance_tsv".to_string(),
                path_relative_to_repo(repo_root, &stages.abundance.normalized_abundance_tsv),
            ),
        ]),
BTreeMap::from([("judgment_report".to_string(), path_relative_to_repo(repo_root, &evidence_path))]),
BTreeMap::from([
            ("valid".to_string(), json!(evidence.valid)),
            (
                "checked_section_count".to_string(),
                json!(evidence.checked_section_count),
            ),
            ("checked_row_count".to_string(), json!(evidence.checked_row_count)),
            ("primer_case_valid".to_string(), json!(evidence.primer_case_valid)),
            ("asv_table_valid".to_string(), json!(evidence.asv_table_valid)),
            (
                "asv_representatives_valid".to_string(),
                json!(evidence.asv_representatives_valid),
            ),
            ("chimera_table_valid".to_string(), json!(evidence.chimera_table_valid)),
            (
                "non_chimeric_representatives_valid".to_string(),
                json!(evidence.non_chimeric_representatives_valid),
            ),
            ("otu_table_valid".to_string(), json!(evidence.otu_table_valid)),
            (
                "otu_representatives_valid".to_string(),
                json!(evidence.otu_representatives_valid),
            ),
            (
                "otu_abundance_table_valid".to_string(),
                json!(evidence.otu_abundance_table_valid),
            ),
            (
                "normalized_abundance_valid".to_string(),
                json!(evidence.normalized_abundance_valid),
            ),
        ])))
}
