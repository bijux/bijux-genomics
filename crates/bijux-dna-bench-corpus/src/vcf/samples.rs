//! Owner: bijux-dna-bench-corpus
//! VCF corpus samples ownership.
use super::{anyhow, fs, BTreeSet, Path, Result, SampleMetadataRow, VcfVariantSetValidationReport};
use anyhow::Context;

/// Load the typed corpus document from its explicit local path.
///
/// # Errors
/// Rejects unreadable documents, malformed syntax and incompatible typed fields.
pub fn load_sample_metadata(sample_metadata_path: &Path) -> Result<Vec<SampleMetadataRow>> {
    if !sample_metadata_path.is_file() {
        return Err(anyhow!(
            "VCF corpus fixture sample metadata is missing: {}",
            sample_metadata_path.display()
        ));
    }
    let raw = fs::read_to_string(sample_metadata_path)
        .with_context(|| format!("read {}", sample_metadata_path.display()))?;
    let mut lines = raw.lines();
    let Some(header) = lines.next() else {
        return Err(anyhow!("VCF corpus fixture sample metadata must not be empty"));
    };
    if header != "sample_id\tpopulation_id\tsex\trole\tdescription" {
        return Err(anyhow!(
            "VCF corpus fixture sample metadata header must be `sample_id\\tpopulation_id\\tsex\\trole\\tdescription`"
        ));
    }
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for (row_index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 5 {
            return Err(anyhow!(
                "VCF corpus fixture sample metadata row {} must contain exactly 5 tab-delimited fields",
                row_index + 2
            ));
        }
        let sample_id = fields[0].trim();
        let population_id = fields[1].trim();
        let sex = fields[2].trim();
        let role = fields[3].trim();
        let description = fields[4].trim();
        if sample_id.is_empty() || population_id.is_empty() || sex.is_empty() || role.is_empty() {
            return Err(anyhow!(
                "VCF corpus fixture sample metadata row {} must declare non-empty sample_id, population_id, sex, and role",
                row_index + 2
            ));
        }
        if description.is_empty() {
            return Err(anyhow!(
                "VCF corpus fixture sample metadata row {} must declare a non-empty description",
                row_index + 2
            ));
        }
        if !matches!(sex, "female" | "male" | "unknown") {
            return Err(anyhow!(
                "VCF corpus fixture sample metadata row {} has unsupported sex `{sex}`",
                row_index + 2
            ));
        }
        if !matches!(role, "cohort" | "panel") {
            return Err(anyhow!(
                "VCF corpus fixture sample metadata row {} has unsupported role `{role}`",
                row_index + 2
            ));
        }
        if !seen.insert(sample_id.to_string()) {
            return Err(anyhow!(
                "VCF corpus fixture sample metadata repeats sample_id `{sample_id}`"
            ));
        }
        rows.push(SampleMetadataRow {
            sample_id: sample_id.to_string(),
            population_id: population_id.to_string(),
            sex: sex.to_string(),
            role: role.to_string(),
            description: description.to_string(),
        });
    }
    if rows.is_empty() {
        return Err(anyhow!(
            "VCF corpus fixture sample metadata must declare at least one sample row"
        ));
    }
    Ok(rows)
}

pub(super) fn load_population_metadata(
    population_metadata_path: &Path,
) -> Result<BTreeSet<String>> {
    if !population_metadata_path.is_file() {
        return Err(anyhow!(
            "VCF corpus fixture population metadata is missing: {}",
            population_metadata_path.display()
        ));
    }
    let raw = fs::read_to_string(population_metadata_path)
        .with_context(|| format!("read {}", population_metadata_path.display()))?;
    let mut lines = raw.lines();
    let Some(header) = lines.next() else {
        return Err(anyhow!("VCF corpus fixture population metadata must not be empty"));
    };
    if header != "population_id\tpopulation_label\tsuper_population\trole" {
        return Err(anyhow!(
            "VCF corpus fixture population metadata header must be `population_id\\tpopulation_label\\tsuper_population\\trole`"
        ));
    }
    let mut population_ids = BTreeSet::new();
    for (row_index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 4 {
            return Err(anyhow!(
                "VCF corpus fixture population metadata row {} must contain exactly 4 tab-delimited fields",
                row_index + 2
            ));
        }
        let population_id = fields[0].trim();
        if population_id.is_empty() {
            return Err(anyhow!(
                "VCF corpus fixture population metadata row {} must declare a non-empty population_id",
                row_index + 2
            ));
        }
        if !population_ids.insert(population_id.to_string()) {
            return Err(anyhow!(
                "VCF corpus fixture population metadata repeats population_id `{population_id}`"
            ));
        }
    }
    if population_ids.is_empty() {
        return Err(anyhow!(
            "VCF corpus fixture population metadata must declare at least one population row"
        ));
    }
    Ok(population_ids)
}

pub(super) fn validate_sample_metadata_population_links(
    sample_metadata: &[SampleMetadataRow],
    population_ids: &BTreeSet<String>,
) -> Result<()> {
    for row in sample_metadata {
        if !population_ids.contains(&row.population_id) {
            return Err(anyhow!(
                "VCF corpus fixture sample `{}` references unknown population `{}`",
                row.sample_id,
                row.population_id
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_manifest_sample_coverage(
    sample_metadata: &[SampleMetadataRow],
    variant_sets: &[(VcfVariantSetValidationReport, BTreeSet<String>)],
) -> Result<()> {
    let declared_sample_ids =
        sample_metadata.iter().map(|row| row.sample_id.clone()).collect::<BTreeSet<_>>();
    let observed_sample_ids = variant_sets
        .iter()
        .flat_map(|(_, sample_ids)| sample_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    if declared_sample_ids != observed_sample_ids {
        let missing_metadata =
            observed_sample_ids.difference(&declared_sample_ids).cloned().collect::<Vec<_>>();
        let extra_metadata =
            declared_sample_ids.difference(&observed_sample_ids).cloned().collect::<Vec<_>>();
        return Err(anyhow!(
            "VCF corpus fixture sample metadata does not match VCF sample ids (missing_metadata={missing_metadata:?}, extra_metadata={extra_metadata:?})"
        ));
    }
    Ok(())
}
