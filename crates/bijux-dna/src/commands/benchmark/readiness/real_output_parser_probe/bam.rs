use super::inputs::{
    optional_input_path, read_toml_document, required_input_path, string_json, u64_json,
};
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_bam_proofs::parser_probes as owner;
use bijux_dna_bench_parser_proofs::ports::MaterializedProof;
pub(super) use owner::{LocalBamGenotypingConfig, LocalBamKinshipConfig, LocalBamSexConfig};
use std::path::Path;
pub(super) const LOCAL_BAM_GENOTYPING_CONFIG_PATH: &str = "configs/bench/local/bam-genotyping.toml";
pub(super) const LOCAL_BAM_KINSHIP_CONFIG_PATH: &str = "configs/bench/local/bam-kinship.toml";
pub(super) const LOCAL_BAM_SEX_CONFIG_PATH: &str = "configs/bench/local/bam-sex.toml";

pub(super) fn generate_bam_align_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let plan = bijux_dna_planner_bam::stage_api::local_align_plan(repo_root)?;
    let reference_fasta = required_input_path(&plan, "reference_fasta")
        .or_else(|_| required_input_path(&plan, "reference"))?;
    let input_r1 = required_input_path(&plan, "fastq_r1")
        .or_else(|_| required_input_path(&plan, "reads_r1"))
        .or_else(|_| required_input_path(&plan, "input_r1"))?;
    let input_r2 = optional_input_path(&plan, "fastq_r2")
        .or_else(|| optional_input_path(&plan, "reads_r2"))
        .or_else(|| optional_input_path(&plan, "input_r2"));
    let sample_id =
        string_json(&plan.params, "sample_id").unwrap_or_else(|| "local_smoke".to_string());
    let read_group: bijux_dna_domain_bam::params::ReadGroupSpec =
        serde_json::from_value(plan.params.get("read_group").cloned().unwrap_or_else(|| {
            serde_json::json!({
                "id": sample_id,
                "sample": sample_id,
                "platform": "ILLUMINA",
                "library": "local_smoke",
                "platform_unit": "local_smoke",
            })
        }))
        .context("decode bam.align read_group")?;
    let proof = owner::generate_bam_align_probe(
        repo_root,
        probe_root,
        &owner::AlignmentProbe {
            tool_id: plan.tool_id.as_str(),
            reference_fasta: &reference_fasta,
            input_r1: &input_r1,
            input_r2: input_r2.as_deref(),
            sample_id: &sample_id,
            read_group: &read_group,
            sensitivity_profile: string_json(&plan.params, "sensitivity_profile").as_deref(),
            preset: string_json(&plan.params, "preset").as_deref(),
            seed_length: u64_json(&plan.params, "seed_length")
                .and_then(|value| u32::try_from(value).ok()),
        },
    )?;
    Ok(adapt(proof))
}

pub(super) fn generate_bam_contamination_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let plan = bijux_dna_planner_bam::stage_api::local_contamination_plan(repo_root)?;
    let proof = owner::generate_bam_contamination_probe(
        probe_root,
        &owner::ContaminationProbe {
            tool_id: plan.tool_id.as_str(),
            scope: serde_json::from_value(
                plan.params.get("scope").cloned().unwrap_or_else(|| serde_json::json!("both")),
            )
            .context("decode bam.contamination scope")?,
            assumptions: string_json(&plan.params, "assumptions")
                .as_deref()
                .unwrap_or("governed tiny contamination parser smoke"),
        },
    )?;
    Ok(adapt(proof))
}

pub(super) fn generate_bam_damage_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let cases = bijux_dna_planner_bam::stage_api::local_damage_smoke_plans(repo_root)?;
    let case = cases.first().ok_or_else(|| anyhow!("missing governed local damage smoke case"))?;
    let proof = owner::generate_bam_damage_probe(
        repo_root,
        probe_root,
        &owner::DamageProbe {
            tool_id: case.plan.tool_id.as_str(),
            bam: &case.bam,
            c_to_t_5p: case.expected_terminal_c_to_t_5p,
            g_to_a_3p: case.expected_terminal_g_to_a_3p,
            strict_profile_upgraded: case.expected_strict_profile_upgraded,
        },
    )?;
    Ok(adapt(proof))
}

pub(super) fn generate_bam_authenticity_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let cases = bijux_dna_planner_bam::stage_api::local_authenticity_smoke_plans(repo_root)?;
    let case =
        cases.first().ok_or_else(|| anyhow!("missing governed local authenticity smoke case"))?;
    let proof = owner::generate_bam_authenticity_probe(
        repo_root,
        probe_root,
        &owner::DamageProbe {
            tool_id: case.plan.tool_id.as_str(),
            bam: &case.bam,
            c_to_t_5p: case.damage_terminal_c_to_t_5p,
            g_to_a_3p: case.damage_terminal_g_to_a_3p,
            strict_profile_upgraded: false,
        },
    )?;
    Ok(adapt(proof))
}

pub(super) fn generate_bam_genotyping_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let config: LocalBamGenotypingConfig =
        read_toml_document(&repo_root.join(LOCAL_BAM_GENOTYPING_CONFIG_PATH))?;
    Ok(adapt(owner::generate_bam_genotyping_probe(repo_root, probe_root, &config)?))
}

pub(super) fn generate_bam_kinship_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let config: LocalBamKinshipConfig =
        read_toml_document(&repo_root.join(LOCAL_BAM_KINSHIP_CONFIG_PATH))?;
    Ok(adapt(owner::generate_bam_kinship_probe(repo_root, probe_root, &config)?))
}

pub(super) fn generate_bam_sex_probe(
    repo_root: &Path,
    probe_root: &Path,
) -> Result<MaterializedProof> {
    let config: LocalBamSexConfig = read_toml_document(&repo_root.join(LOCAL_BAM_SEX_CONFIG_PATH))?;
    Ok(adapt(owner::generate_bam_sex_probe(repo_root, probe_root, &config)?))
}

fn adapt(proof: owner::MaterializedProof) -> MaterializedProof {
    MaterializedProof { proof_path: proof.proof_path, observed_tool_id: proof.observed_tool_id }
}
