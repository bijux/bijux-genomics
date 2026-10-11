use super::values::{validate_nonempty, validate_nonempty_list};
use super::{invalid, CorpusSourceError};
use crate::source::CorpusEvaluationDesign;

/// Validate canonical corpus source semantics.
///
/// # Errors
/// Rejects missing, malformed or inconsistent source declarations.
pub fn validate_corpus_evaluation_design(
    design: &CorpusEvaluationDesign,
) -> Result<(), CorpusSourceError> {
    for (value, label) in [
        (&design.sample_context, "sample context"),
        (&design.assay_context, "assay context"),
        (&design.held_out_split, "held-out split"),
        (&design.lineage, "lineage"),
    ] {
        validate_nonempty(value, label)?;
    }
    validate_ordered_nonempty_values(&design.strata, "stratum")?;
    validate_ordered_nonempty_values(&design.controls, "control")?;
    validate_ordered_nonempty_values(&design.allowed_uses, "allowed use")
}

fn validate_ordered_nonempty_values(
    values: &[String],
    label: &str,
) -> Result<(), CorpusSourceError> {
    validate_nonempty_list(values, label)?;
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(invalid("corpus evaluation values are not strictly ordered"));
    }
    Ok(())
}
