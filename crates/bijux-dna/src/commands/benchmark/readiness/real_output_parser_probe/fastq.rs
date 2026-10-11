use anyhow::{anyhow, Result};
use bijux_dna_bench_parser_proofs::ports::MaterializedProof;
use std::path::Path;

pub(super) fn generate_index_reference_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let plan = bijux_dna_planner_fastq::stage_api::local_index_reference_plan(repo_root)?;
    let proof = bijux_dna_bench_fastq::probes::generate_index_reference_probe(
        repo_root, probe_root, &plan,
    )?;
    Ok(MaterializedProof { proof_path: proof.proof_path, observed_tool_id: proof.observed_tool_id })
}

pub(super) fn generate_correct_errors_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let cases = bijux_dna_planner_fastq::stage_api::local_correct_errors_smoke_plans(repo_root)?;
    let case =
        cases.first().ok_or_else(|| anyhow!("missing governed local correct-errors smoke case"))?;
    let proof = bijux_dna_bench_fastq::probes::generate_correct_errors_probe(
        repo_root,
        probe_root,
        &bijux_dna_bench_fastq::probes::CorrectErrorsProbeInput {
            plan: &case.plan,
            r1: &case.r1,
            r2: case.r2.as_deref(),
            quality_encoding: case.quality_encoding.clone(),
            conservative_mode: case.conservative_mode,
        },
    )?;
    Ok(MaterializedProof { proof_path: proof.proof_path, observed_tool_id: proof.observed_tool_id })
}

pub(super) fn generate_deplete_host_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let plan = bijux_dna_planner_fastq::stage_api::local_deplete_host_plan(repo_root)?;
    let proof =
        bijux_dna_bench_fastq::probes::generate_deplete_host_probe(repo_root, probe_root, &plan)?;
    Ok(MaterializedProof { proof_path: proof.proof_path, observed_tool_id: proof.observed_tool_id })
}

pub(super) fn generate_deplete_rrna_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let plan = bijux_dna_planner_fastq::stage_api::local_deplete_rrna_plan(repo_root)?;
    let proof =
        bijux_dna_bench_fastq::probes::generate_deplete_rrna_probe(repo_root, probe_root, &plan)?;
    Ok(MaterializedProof { proof_path: proof.proof_path, observed_tool_id: proof.observed_tool_id })
}

pub(super) fn generate_screen_taxonomy_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let plan = bijux_dna_planner_fastq::stage_api::local_screen_taxonomy_plan(repo_root)?;
    let proof = bijux_dna_bench_fastq::probes::generate_screen_taxonomy_probe(
        repo_root, probe_root, &plan,
    )?;
    Ok(MaterializedProof { proof_path: proof.proof_path, observed_tool_id: proof.observed_tool_id })
}
