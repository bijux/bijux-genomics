//! Admit scientific output evidence and construct the persisted benchmark metrics.
use crate::postprocess::execution::artifacts::StageArtifacts;
use crate::postprocess::model::GovernedVcfPostprocessSmokeContract;
use crate::postprocess::model::LocalVcfPostprocessSmokeMetrics;
use crate::postprocess::model::LOCAL_VCF_POSTPROCESS_SMOKE_METRICS_SCHEMA_VERSION;
use crate::postprocess::observations::reports::read_json_value;
use crate::postprocess::observations::variants::json_bool;
use crate::postprocess::observations::variants::json_u64;
use crate::postprocess::observations::variants::summarize_postprocess_output;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::vcf_io::vcf_validate_input;
use bijux_dna_stages_vcf::vcf_io::VcfFieldRequirement;
use bijux_dna_stages_vcf::vcf_io::VcfValidationSummary;

pub(crate) struct StageEvidence {
    pub(crate) metrics: LocalVcfPostprocessSmokeMetrics,
    pub(crate) validation: VcfValidationSummary,
}

pub(crate) fn observe_stage(
    contract: &GovernedVcfPostprocessSmokeContract,
    artifacts: &StageArtifacts,
) -> Result<StageEvidence> {
    let output_vcf = &artifacts.output_vcf;
    let validate_outputs_path = &artifacts.validate_outputs_path;
    let final_manifest_path = &artifacts.final_manifest_path;
    let validation =
        vcf_validate_input(output_vcf, VcfFieldRequirement { require_gt: true, require_gl: false })
            .with_context(|| format!("validate {}", output_vcf.display()))?;
    let validate_outputs = read_json_value(validate_outputs_path)?;
    let final_manifest = read_json_value(final_manifest_path)?;
    let output_summary = summarize_postprocess_output(output_vcf)?;

    let metrics = LocalVcfPostprocessSmokeMetrics {
        schema_version: LOCAL_VCF_POSTPROCESS_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage.clone(),
        corpus_id: contract.corpus.clone(),
        input_fixture_id: contract.input_fixture.clone(),
        readable_vcf: json_bool(&validate_outputs, "/readable_vcf")?,
        tabix_present: json_bool(&validate_outputs, "/tabix_present")?,
        contigs_consistent_with_species_context: json_bool(
            &validate_outputs,
            "/contigs_consistent_with_species_context",
        )?,
        left_align_applied: json_bool(&final_manifest, "/normalization/left_align_applied")?,
        multiallelic_records_split: json_u64(
            &final_manifest,
            "/normalization/multiallelic_records_split",
        )?,
        indels_normalized: json_u64(&final_manifest, "/normalization/indels_normalized")?,
        variant_ids_normalized: json_u64(&final_manifest, "/normalization/variant_ids_normalized")?,
        invalid_records_removed: json_u64(
            &final_manifest,
            "/normalization/invalid_records_removed",
        )?,
        filter_standardized_to_pass: json_u64(
            &final_manifest,
            "/normalization/filter_standardized_to_pass",
        )?,
        record_count: output_summary.record_count,
        sample_count: output_summary.sample_count,
        sample_ids: output_summary.sample_ids.clone(),
        mq_removed: output_summary.mq_removed,
        multiallelic_records_remaining: output_summary.multiallelic_records_remaining,
        tool_id: contract.tool.clone(),
        exit_code: 0,
    };
    if !metrics.readable_vcf
        || !metrics.tabix_present
        || !metrics.contigs_consistent_with_species_context
    {
        bail!("governed VCF postprocess smoke produced invalid validation state");
    }
    if metrics.multiallelic_records_split == 0 {
        bail!("governed VCF postprocess smoke must prove multiallelic splitting");
    }
    if metrics.indels_normalized == 0 {
        bail!("governed VCF postprocess smoke must prove indel normalization");
    }
    if metrics.variant_ids_normalized == 0 {
        bail!("governed VCF postprocess smoke must prove variant-id normalization");
    }
    if metrics.filter_standardized_to_pass == 0 {
        bail!("governed VCF postprocess smoke must prove filter standardization");
    }
    if !metrics.mq_removed {
        bail!("governed VCF postprocess smoke must remove MQ from INFO");
    }
    if metrics.multiallelic_records_remaining != 0 {
        bail!("governed VCF postprocess smoke must emit only biallelic output rows");
    }

    Ok(StageEvidence { metrics, validation })
}
