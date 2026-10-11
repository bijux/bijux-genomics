//! Owner: bijux-dna-bench-population
//! Pca observations records.
use crate::pca::model::LocalVcfPcaSmokeRow;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_corpus::vcf::SampleMetadataRow;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(crate) fn parse_pca_rows(
    path: &Path,
    metadata_by_sample: &BTreeMap<String, SampleMetadataRow>,
    population_labels: &BTreeMap<String, String>,
) -> Result<Vec<LocalVcfPcaSmokeRow>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| anyhow!("PCA table is empty: {}", path.display()))?;
    if header.trim() != "sample\tPC1\tPC2" {
        bail!("PCA table header drifted in {}: `{header}`", path.display());
    }
    let mut rows = Vec::<LocalVcfPcaSmokeRow>::new();
    let mut seen = BTreeSet::<String>::new();
    for (index, line) in lines.enumerate() {
        let columns = line.split_whitespace().collect::<Vec<_>>();
        if columns.len() < 3 {
            bail!("PCA row {} in {} must provide sample, PC1, and PC2", index + 2, path.display());
        }
        let sample_id = columns[0].to_string();
        if !seen.insert(sample_id.clone()) {
            bail!("PCA table duplicated sample `{sample_id}` in {}", path.display());
        }
        let metadata = metadata_by_sample.get(&sample_id).ok_or_else(|| {
            anyhow!("PCA table sample `{sample_id}` is missing governed metadata")
        })?;
        let population_label = population_labels.get(&metadata.population_id).ok_or_else(|| {
            anyhow!("population metadata is missing label for `{}`", metadata.population_id)
        })?;
        let pc1 = columns[1]
            .parse::<f64>()
            .with_context(|| format!("parse PC1 for `{sample_id}` from {}", path.display()))?;
        let pc2 = columns[2]
            .parse::<f64>()
            .with_context(|| format!("parse PC2 for `{sample_id}` from {}", path.display()))?;
        rows.push(LocalVcfPcaSmokeRow {
            sample_id,
            population_id: metadata.population_id.clone(),
            population_label: population_label.clone(),
            sex: metadata.sex.clone(),
            pc1,
            pc2,
        });
    }
    if rows.is_empty() {
        bail!("PCA table contained no sample rows in {}", path.display());
    }
    Ok(rows)
}

pub(crate) fn parse_eigenvalues(path: &Path) -> Result<Vec<f64>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let header =
        lines.next().ok_or_else(|| anyhow!("PCA eigenvalue table is empty: {}", path.display()))?;
    if header.trim() != "component\teigenvalue" {
        bail!("PCA eigenvalue header drifted in {}: `{header}`", path.display());
    }
    let mut eigenvalues = Vec::<f64>::new();
    for (index, line) in lines.enumerate() {
        let columns = line.split_whitespace().collect::<Vec<_>>();
        if columns.len() != 2 {
            bail!(
                "PCA eigenvalue row {} in {} must provide exactly 2 columns",
                index + 2,
                path.display()
            );
        }
        eigenvalues.push(columns[1].parse::<f64>().with_context(|| {
            format!("parse eigenvalue row {} from {}", index + 2, path.display())
        })?);
    }
    Ok(eigenvalues)
}
