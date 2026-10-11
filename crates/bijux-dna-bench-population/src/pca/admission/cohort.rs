//! Owner: bijux-dna-bench-population
//! Pca admission cohort.
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_corpus::vcf::SampleMetadataRow;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(crate) fn build_metadata_by_sample(
    sample_metadata: &[SampleMetadataRow],
    expected_samples: &[String],
) -> Result<BTreeMap<String, SampleMetadataRow>> {
    let expected_sample_set = expected_samples.iter().cloned().collect::<BTreeSet<_>>();
    let mut metadata_by_sample = BTreeMap::<String, SampleMetadataRow>::new();
    for row in sample_metadata {
        if !expected_sample_set.contains(&row.sample_id) {
            continue;
        }
        if row.role != "cohort" {
            bail!(
                "governed VCF PCA smoke expected cohort role for `{}`, found `{}`",
                row.sample_id,
                row.role
            );
        }
        metadata_by_sample.insert(row.sample_id.clone(), row.clone());
    }
    for sample_id in expected_samples {
        if !metadata_by_sample.contains_key(sample_id) {
            bail!("governed VCF PCA smoke metadata is missing expected sample `{sample_id}`");
        }
    }
    Ok(metadata_by_sample)
}

pub(crate) fn load_population_labels(path: &Path) -> Result<BTreeMap<String, String>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut rows = raw.lines();
    let header = rows.next().ok_or_else(|| anyhow!("population metadata is empty"))?;
    if header.trim() != "population_id\tpopulation_label\tsuper_population\trole" {
        bail!("population metadata header drifted in {}", path.display());
    }
    let mut labels = BTreeMap::<String, String>::new();
    for (index, line) in rows.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 4 {
            bail!(
                "population metadata row {} in {} must have 4 columns",
                index + 2,
                path.display()
            );
        }
        let population_id = columns[0].trim();
        let population_label = columns[1].trim();
        if population_id.is_empty() || population_label.is_empty() {
            bail!(
                "population metadata row {} in {} must keep non-empty ids and labels",
                index + 2,
                path.display()
            );
        }
        labels.insert(population_id.to_string(), population_label.to_string());
    }
    Ok(labels)
}

pub(crate) fn write_population_labels_manifest(
    path: &Path,
    metadata_by_sample: &BTreeMap<String, SampleMetadataRow>,
    expected_samples: &[String],
) -> Result<()> {
    let payload = serde_json::json!({
        "samples": expected_samples.iter().map(|sample_id| {
            let row = metadata_by_sample
                .get(sample_id)
                .unwrap_or_else(|| panic!("missing metadata for sample `{sample_id}`"));
            serde_json::json!({
                "sample": row.sample_id,
                "population": row.population_id,
            })
        }).collect::<Vec<_>>()
    });
    bijux_dna_infra::atomic_write_json(path, &payload)?;
    Ok(())
}
