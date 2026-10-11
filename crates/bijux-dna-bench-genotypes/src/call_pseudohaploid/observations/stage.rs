//! Validate observed genotype semantics and construct persisted scientific metrics.
use crate::bam::execution::native::CalledArtifacts;
use crate::bam::model::GovernedVcfCallBamSmokeContract;
use crate::call_pseudohaploid::model::LocalVcfCallPseudohaploidSmokeMetrics;
use crate::call_pseudohaploid::model::GOVERNED_PSEUDOHAPLOID_DETERMINISM_SCOPE;
use crate::call_pseudohaploid::model::GOVERNED_PSEUDOHAPLOID_RANDOM_SEED;
use crate::call_pseudohaploid::model::GOVERNED_PSEUDOHAPLOID_SAMPLING_POLICY;
use crate::call_pseudohaploid::model::GOVERNED_PSEUDOHAPLOID_SEED_USAGE;
use crate::call_pseudohaploid::model::LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_METRICS_SCHEMA_VERSION;
use crate::call_pseudohaploid::observations::variants::canonical_vcf_replay_sha256;
use crate::call_pseudohaploid::observations::variants::summarize_pseudohaploid_sites;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_infra::hash_file_sha256;
use bijux_dna_stages_vcf::pipeline::CallStageOutputs;
use bijux_dna_stages_vcf::vcf_io::vcf_sample_count as parse_output_sample_count;
use bijux_dna_stages_vcf::vcf_io::vcf_validate_input;
use bijux_dna_stages_vcf::vcf_io::VcfFieldRequirement;
use bijux_dna_stages_vcf::vcf_io::VcfValidationSummary;

pub(crate) struct StageEvidence {
    pub(crate) metrics: LocalVcfCallPseudohaploidSmokeMetrics,
    pub(crate) validation: VcfValidationSummary,
}

pub(crate) fn observe(
    contract: &GovernedVcfCallBamSmokeContract,
    artifacts: &CalledArtifacts,
    replay_outputs: &CallStageOutputs,
) -> Result<StageEvidence> {
    let output_vcf = &artifacts.vcf;
    let validation =
        vcf_validate_input(output_vcf, VcfFieldRequirement { require_gt: true, require_gl: false })
            .with_context(|| format!("validate {}", output_vcf.display()))?;
    let pseudohaploid_summary = summarize_pseudohaploid_sites(output_vcf)
        .with_context(|| format!("summarize pseudohaploid sites in {}", output_vcf.display()))?;
    let sample_count = parse_output_sample_count(output_vcf)
        .with_context(|| format!("count samples in {}", output_vcf.display()))?;
    if sample_count != 1 {
        bail!("governed pseudohaploid smoke expects exactly one sample, found {sample_count}");
    }
    let raw_output_sha256 = hash_file_sha256(output_vcf).map_err(|err| anyhow!(err.to_string()))?;
    let raw_replay_output_sha256 =
        hash_file_sha256(&replay_outputs.called_vcf).map_err(|err| anyhow!(err.to_string()))?;
    let canonical_output_sha256 = canonical_vcf_replay_sha256(output_vcf)?;
    let canonical_replay_output_sha256 = canonical_vcf_replay_sha256(&replay_outputs.called_vcf)?;
    let raw_replay_match = raw_output_sha256 == raw_replay_output_sha256;
    let deterministic_replay_match = canonical_output_sha256 == canonical_replay_output_sha256;
    if !deterministic_replay_match {
        bail!(
            "pseudohaploid replay drifted for `{}` after canonicalization: {} != {}",
            contract.tool_id,
            canonical_output_sha256,
            canonical_replay_output_sha256
        );
    }

    let metrics = LocalVcfCallPseudohaploidSmokeMetrics {
        schema_version: LOCAL_VCF_CALL_PSEUDOHAPLOID_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        sample_id: contract.sample_id.clone(),
        input_bam: contract.input_bam.clone(),
        reference: contract.reference.clone(),
        target_sites: pseudohaploid_summary.target,
        covered_sites: pseudohaploid_summary.covered,
        called_sites: pseudohaploid_summary.called,
        missing_sites: pseudohaploid_summary.missing,
        sampling_policy: GOVERNED_PSEUDOHAPLOID_SAMPLING_POLICY,
        seed_usage: GOVERNED_PSEUDOHAPLOID_SEED_USAGE,
        random_seed: GOVERNED_PSEUDOHAPLOID_RANDOM_SEED,
        sample_count,
        tool_id: contract.tool_id.clone(),
        raw_output_sha256: raw_output_sha256.clone(),
        raw_replay_output_sha256: raw_replay_output_sha256.clone(),
        canonical_output_sha256: canonical_output_sha256.clone(),
        canonical_replay_output_sha256: canonical_replay_output_sha256.clone(),
        determinism_scope: GOVERNED_PSEUDOHAPLOID_DETERMINISM_SCOPE,
        raw_replay_match,
        deterministic_replay_match,
        exit_code: 0,
    };
    Ok(StageEvidence { metrics, validation })
}
