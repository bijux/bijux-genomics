//! Owner: bijux-dna-bench-population
//! Admixture observations records.
use crate::admixture::model::{LocalVcfAdmixtureSmokeRow, ANCESTRY_SUM_TOLERANCE};
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_corpus::vcf::SampleMetadataRow;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(crate) fn parse_admixture_rows(
    path: &Path,
    metadata_by_sample: &BTreeMap<String, SampleMetadataRow>,
    population_labels: &BTreeMap<String, String>,
    cluster_headers: &[String],
    selected_k: usize,
    status: &str,
) -> Result<Vec<LocalVcfAdmixtureSmokeRow>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let header =
        lines.next().ok_or_else(|| anyhow!("admixture table is empty: {}", path.display()))?;
    let expected_header = format!("sample\t{}", cluster_headers.join("\t"));
    if header.trim() != expected_header {
        bail!("admixture table header drifted in {}: `{header}`", path.display());
    }
    let mut rows = Vec::<LocalVcfAdmixtureSmokeRow>::new();
    let mut seen = BTreeSet::<String>::new();
    for (index, line) in lines.enumerate() {
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != selected_k + 1 {
            bail!(
                "admixture row {} in {} must provide sample plus {} cluster columns",
                index + 2,
                path.display(),
                selected_k
            );
        }
        let sample_id = columns[0].trim().to_string();
        if sample_id.is_empty() {
            bail!("admixture row {} in {} is missing sample id", index + 2, path.display());
        }
        if !seen.insert(sample_id.clone()) {
            bail!("admixture table duplicated sample `{sample_id}` in {}", path.display());
        }
        let metadata = metadata_by_sample.get(&sample_id).ok_or_else(|| {
            anyhow!("admixture table sample `{sample_id}` is missing governed metadata")
        })?;
        let population_label = population_labels.get(&metadata.population_id).ok_or_else(|| {
            anyhow!("population metadata is missing label for `{}`", metadata.population_id)
        })?;
        let mut clusters = BTreeMap::<String, f64>::new();
        let mut ancestry_sum = 0.0_f64;
        for (column_index, cluster_header) in cluster_headers.iter().enumerate() {
            let value = columns[column_index + 1].parse::<f64>().with_context(|| {
                format!("parse {} for `{sample_id}` from {}", cluster_header, path.display())
            })?;
            clusters.insert(cluster_header.clone(), value);
            ancestry_sum += value;
        }
        if (ancestry_sum - 1.0).abs() > ANCESTRY_SUM_TOLERANCE {
            bail!(
                "admixture row `{sample_id}` in {} sums to {:.6}, expected 1.0 ± {}",
                path.display(),
                ancestry_sum,
                ANCESTRY_SUM_TOLERANCE
            );
        }
        rows.push(LocalVcfAdmixtureSmokeRow {
            sample_id,
            population_id: metadata.population_id.clone(),
            population_label: population_label.clone(),
            sex: metadata.sex.clone(),
            k: selected_k,
            status: status.to_string(),
            clusters,
        });
    }
    if rows.is_empty() {
        bail!("admixture table contained no sample rows in {}", path.display());
    }
    Ok(rows)
}
