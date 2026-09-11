//! call gl observations variants.
use crate::call_gl::model::LikelihoodSummary;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use bijux_dna_stages_vcf::vcf_io::read_vcf_text;
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn summarize_likelihood_fields(vcf_path: &Path) -> Result<LikelihoodSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let mut likelihood_field = None::<String>;
    let mut sites_with_likelihoods = 0_u64;
    let mut missing_likelihoods = 0_u64;
    let mut samples_with_likelihoods = BTreeSet::<usize>::new();

    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 10 {
            bail!("GL smoke output row is missing FORMAT/sample fields: {line}");
        }
        let format_tokens = fields[8].split(':').collect::<Vec<_>>();
        let field_name = ["GL", "GP", "PL"]
            .into_iter()
            .find(|candidate| format_tokens.iter().any(|token| token == candidate))
            .ok_or_else(|| anyhow!("GL smoke output row is missing GL/GP/PL in FORMAT: {line}"))?;
        let field_index = format_tokens
            .iter()
            .position(|token| *token == field_name)
            .ok_or_else(|| anyhow!("GL smoke output row lost {field_name} in FORMAT: {line}"))?;

        if let Some(previous) = &likelihood_field {
            if previous != field_name {
                bail!(
                    "GL smoke likelihood field drifted across rows: `{previous}` vs `{field_name}`"
                );
            }
        } else {
            likelihood_field = Some(field_name.to_string());
        }

        let mut row_has_likelihood = false;
        for (sample_index, sample_field) in fields[9..].iter().enumerate() {
            let sample_value = sample_field.split(':').nth(field_index).ok_or_else(|| {
                anyhow!("GL smoke sample field is missing {field_name} value: {line}")
            })?;
            if likelihood_value_is_missing(sample_value) {
                missing_likelihoods += 1;
                continue;
            }
            row_has_likelihood = true;
            samples_with_likelihoods.insert(sample_index);
        }
        if row_has_likelihood {
            sites_with_likelihoods += 1;
        }
    }

    Ok(LikelihoodSummary {
        likelihood_field: likelihood_field
            .ok_or_else(|| anyhow!("GL smoke output did not contain any GL/GP/PL fields"))?,
        sites_with_likelihoods,
        samples_with_likelihoods: u64::try_from(samples_with_likelihoods.len())
            .map_err(|_| anyhow!("GL smoke sample-with-likelihood count overflowed u64"))?,
        missing_likelihoods,
    })
}

pub(crate) fn likelihood_value_is_missing(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed.split(',').all(|token| matches!(token.trim(), "." | ""))
}
