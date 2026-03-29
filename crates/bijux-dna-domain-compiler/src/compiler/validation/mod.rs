use super::*;

mod catalog_validation;
mod index_rules;
mod stage_files;
mod tool_files;

use self::catalog_validation::{
    validate_domain_vocabularies, validate_reference_catalogs, DomainVocabularies,
};
use self::index_rules::validate_domain_indexes_and_pipelines;
use self::stage_files::validate_stage_files;
use self::tool_files::validate_tool_files;

/// Validate authored domain files and cross-domain invariants.
///
/// # Errors
///
/// Returns an error when required files are missing, schemas/invariants are
/// violated, or domain catalogs are inconsistent.
#[allow(clippy::too_many_lines)]
pub fn validate_domain(options: &ValidateOptions) -> Result<()> {
    for rel in [
        "fastq/stages/_schema.yaml",
        "bam/stages/_schema.yaml",
        "vcf/stages/_schema.yaml",
        "fastq/tools/_schema.yaml",
        "bam/tools/_schema.yaml",
        "vcf/tools/_schema.yaml",
        "fastq/artifacts.yaml",
        "bam/artifacts.yaml",
        "vcf/artifacts.yaml",
        "fastq/metrics.yaml",
        "bam/metrics.yaml",
        "vcf/metrics.yaml",
        "fastq/index.yaml",
        "bam/index.yaml",
        "vcf/index.yaml",
    ] {
        super::compile::require_exists(&options.domain_dir.join(rel))?;
    }
    let workspace_root = options.domain_dir.parent().unwrap_or(&options.domain_dir);
    validate_reference_catalogs(workspace_root)?;

    let mut tool_ids = BTreeMap::<String, String>::new();
    let mut stage_ids = BTreeMap::<String, String>::new();
    let mut tool_capabilities = BTreeMap::<String, BTreeSet<String>>::new();
    let mut tool_statuses = BTreeMap::<String, String>::new();
    let mut tool_metrics_schemas = BTreeMap::<String, String>::new();
    let DomainVocabularies {
        artifact_vocab,
        metric_vocab,
    } = validate_domain_vocabularies(&options.domain_dir)?;

    for dom in ["fastq", "bam", "vcf"] {
        validate_stage_files(options, dom, &artifact_vocab, &metric_vocab, &mut stage_ids)?;
        validate_tool_files(
            options,
            dom,
            &artifact_vocab,
            &mut tool_ids,
            &mut tool_capabilities,
            &mut tool_statuses,
            &mut tool_metrics_schemas,
        )?;
    }

    let fastq_canonical = bijux_dna_domain_fastq::stages::ids::STAGES
        .iter()
        .map(|id| id.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let bam_canonical = bijux_dna_domain_bam::stage_specs::BamStage::all()
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<BTreeSet<_>>();
    // Accept additional domain-declared stages so domain specs can evolve ahead
    // of canonical stage catalogs; still enforce that canonical stages are present.
    for stage_id in &fastq_canonical {
        if !stage_ids.contains_key(stage_id) {
            bail!("fastq stage catalog contains {stage_id} but domain yaml is missing it");
        }
    }
    for stage_id in &bam_canonical {
        if !stage_ids.contains_key(stage_id) {
            bail!("bam stage catalog contains {stage_id} but domain yaml is missing it");
        }
    }

    validate_domain_indexes_and_pipelines(
        options,
        &stage_ids,
        &tool_ids,
        &tool_capabilities,
        &tool_statuses,
        &tool_metrics_schemas,
    )?;

    println!("domain-validate: OK");
    Ok(())
}
