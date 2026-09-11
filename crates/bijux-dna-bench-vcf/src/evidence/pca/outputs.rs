use crate::adapters::{
    eigensoft::{
        VcfEigensoftAdapterArtifact, VcfEigensoftAdapterRow, DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH,
    },
    plink::{
        VcfPlinkFamilyAdapterArtifact, VcfPlinkFamilyAdapterRow, DEFAULT_VCF_PLINK2_ADAPTER_PATH,
    },
};
use bijux_dna_bench_vcf_evidence::model::pca::PcaAdapterOutputProof;
use std::collections::BTreeMap;
pub(super) fn plink_output_proof(row: &VcfPlinkFamilyAdapterRow) -> PcaAdapterOutputProof {
    let declared_by_id = row
        .declared_outputs
        .iter()
        .cloned()
        .map(|artifact| (artifact.artifact_id.clone(), artifact))
        .collect::<BTreeMap<_, _>>();
    PcaAdapterOutputProof {
        output_proof_path: DEFAULT_VCF_PLINK2_ADAPTER_PATH.to_string(),
        benchmark_status: row.benchmark_status.clone(),
        raw_outputs: row
            .raw_output_ids
            .iter()
            .filter_map(|artifact_id| declared_by_id.get(artifact_id))
            .filter(|artifact| !is_index_artifact(&artifact.path))
            .map(render_plink_artifact_entry)
            .collect(),
        normalized_metrics_outputs: row
            .stage_output_ids
            .iter()
            .filter_map(|artifact_id| declared_by_id.get(artifact_id))
            .map(render_plink_artifact_entry)
            .collect(),
        manifest_output: render_stage_result_manifest_path(&row.stage_id, &row.tool_id),
        index_outputs: row
            .declared_outputs
            .iter()
            .filter(|artifact| is_index_artifact(&artifact.path))
            .map(render_plink_artifact_entry)
            .collect(),
    }
}
pub(super) fn eigensoft_output_proof(row: &VcfEigensoftAdapterRow) -> PcaAdapterOutputProof {
    let declared_by_id = row
        .declared_outputs
        .iter()
        .cloned()
        .map(|artifact| (artifact.artifact_id.clone(), artifact))
        .collect::<BTreeMap<_, _>>();
    PcaAdapterOutputProof {
        output_proof_path: DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH.to_string(),
        benchmark_status: row.benchmark_status.clone(),
        raw_outputs: row
            .raw_output_ids
            .iter()
            .filter_map(|artifact_id| declared_by_id.get(artifact_id))
            .filter(|artifact| !is_index_artifact(&artifact.path))
            .map(render_eigensoft_artifact_entry)
            .collect(),
        normalized_metrics_outputs: row
            .stage_output_ids
            .iter()
            .filter_map(|artifact_id| declared_by_id.get(artifact_id))
            .map(render_eigensoft_artifact_entry)
            .collect(),
        manifest_output: render_stage_result_manifest_path(&row.stage_id, &row.tool_id),
        index_outputs: row
            .declared_outputs
            .iter()
            .filter(|artifact| is_index_artifact(&artifact.path))
            .map(render_eigensoft_artifact_entry)
            .collect(),
    }
}
fn render_plink_artifact_entry(artifact: &VcfPlinkFamilyAdapterArtifact) -> String {
    format!("{}={}", artifact.artifact_id, artifact.path)
}
fn render_eigensoft_artifact_entry(artifact: &VcfEigensoftAdapterArtifact) -> String {
    format!("{}={}", artifact.artifact_id, artifact.path)
}
fn render_stage_result_manifest_path(stage_id: &str, tool_id: &str) -> String {
    format!(
        "artifacts/runs/bench/slurm-dry-run/runs/{}/{}/{}/{}/{}/stage-result.json",
        "local-benchmark-dry-run", "{fixture_scope}", stage_id, "{sample_scope}", tool_id
    )
}
#[expect(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "Governed artifact suffixes are case-sensitive contract values."
)]
fn is_index_artifact(path: &str) -> bool {
    path.ends_with(".tbi") || path.ends_with(".csi") || path.ends_with(".fai")
}
