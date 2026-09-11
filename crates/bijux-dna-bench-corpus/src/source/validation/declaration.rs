use super::evaluation::validate_corpus_evaluation_design;
use super::values::{
    validate_https, validate_identifier, validate_nonempty, validate_nonempty_list,
};
use super::{invalid, CorpusSourceError};
use crate::source::{BenchmarkCorpusClass, FastqCorpusSourceSpec, FASTQ_CORPUS_SOURCE_SCHEMA_V1};
use std::collections::BTreeSet;

/// Validate canonical corpus source semantics.
///
/// # Errors
/// Rejects missing, malformed or inconsistent source declarations.
pub fn validate_fastq_corpus_source_spec(
    spec: &FastqCorpusSourceSpec,
) -> Result<(), CorpusSourceError> {
    if spec.schema_version != FASTQ_CORPUS_SOURCE_SCHEMA_V1 {
        return Err(invalid("FASTQ corpus source schema mismatch"));
    }
    validate_identifier(&spec.corpus_id, "corpus")?;
    validate_identifier(&spec.portfolio_id, "portfolio")?;
    if !matches!(
        spec.policy.class,
        BenchmarkCorpusClass::ProductionReal | BenchmarkCorpusClass::QualificationReal
    ) {
        return Err(invalid("FASTQ corpus source class is not publishable real data"));
    }
    for (value, label) in [
        (&spec.species, "species"),
        (&spec.description, "description"),
        (&spec.source_authority, "source authority"),
        (&spec.policy.access_classification, "access classification"),
        (&spec.policy.redistribution_classification, "redistribution classification"),
        (&spec.policy.attribution, "attribution"),
        (&spec.policy.projection_basis, "projection basis"),
        (&spec.policy.truth_strategy, "truth strategy"),
    ] {
        validate_nonempty(value, label)?;
    }
    for (value, label) in [
        (spec.species_id.as_deref(), "species identifier"),
        (spec.preferred_root.as_deref(), "preferred root"),
    ] {
        if let Some(value) = value {
            validate_nonempty(value, label)?;
        }
    }
    validate_https(&spec.policy.access_terms_url, "access terms")?;
    if spec.policy.projected_storage_bytes == 0 || spec.policy.projected_cpu_hours == 0 {
        return Err(invalid("FASTQ corpus storage and compute projections must be positive"));
    }
    validate_nonempty_list(&spec.policy.truth_roles, "truth role")?;
    validate_nonempty_list(&spec.policy.known_limitations, "known limitation")?;
    validate_corpus_evaluation_design(&spec.evaluation)?;
    if spec.samples.is_empty() || spec.samples.len() != spec.target_total {
        return Err(invalid("FASTQ corpus target total disagrees with samples"));
    }
    let cohort_targets = [
        spec.target_ancient_se,
        spec.target_ancient_pe,
        spec.target_modern_se,
        spec.target_modern_pe,
    ];
    if cohort_targets.iter().any(Option::is_some)
        && (cohort_targets.iter().any(Option::is_none)
            || cohort_targets.into_iter().flatten().sum::<usize>() != spec.target_total)
    {
        return Err(invalid("FASTQ corpus cohort targets do not close over target total"));
    }
    let mut accessions = BTreeSet::new();
    for sample in &spec.samples {
        if !accessions.insert(sample.accession.as_str()) {
            return Err(invalid("FASTQ corpus source repeats a run accession"));
        }
        for (value, label) in [
            (&sample.accession, "run accession"),
            (&sample.study_accession, "study accession"),
            (&sample.era, "era"),
            (&sample.size_band, "size band"),
            (&sample.reason, "selection reason"),
        ] {
            validate_nonempty(value, label)?;
        }
        if !matches!(sample.era.as_str(), "ancient" | "modern") {
            return Err(invalid("FASTQ corpus sample era is unsupported"));
        }
    }
    Ok(())
}
