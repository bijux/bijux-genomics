use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};

use crate::taxonomy::VcfDomainStage;

use super::segments::parse_segment_stage_metrics;

const PLINK_TOOL_ID: &str = "plink";
const PLINK2_TOOL_ID: &str = "plink2";

#[derive(Debug, Clone)]
struct SampleMissingnessRow {
    sample_id: String,
    total_genotype_count: u64,
    missing_genotype_count: u64,
    missingness: f64,
}

#[derive(Debug, Clone)]
struct VariantMissingnessRow {
    variant_id: String,
    contig: String,
    position: u64,
    reference: String,
    alternate: String,
    total_sample_count: u64,
    missing_sample_count: u64,
    missingness: f64,
}

#[derive(Debug, Clone)]
struct FrequencySummary {
    allele_frequency_mean: f64,
    maf_bin_counts: BTreeMap<String, u64>,
    observed_variant_count: u64,
}

#[derive(Debug, Clone)]
struct HeterozygosityRow {
    sample_id: String,
    observed_homozygous_count: u64,
    nonmissing_variant_count: u64,
    heterozygous_call_count: u64,
    inbreeding_coefficient: f64,
}

#[derive(Debug, Clone)]
struct HweSummary {
    tested_variant_count: u64,
    pvalue_mean: Option<f64>,
    status: String,
}

#[derive(Debug, Clone)]
struct PcaRow {
    sample_id: String,
    components: BTreeMap<String, f64>,
}

/// Normalize the governed raw `plink` artifact set for a retained VCF stage.
///
/// # Errors
/// Returns an error when required raw artifacts are missing or malformed.
pub fn parse_plink_stage_metrics(
    stage: VcfDomainStage,
    artifact_root: &Path,
) -> Result<serde_json::Value> {
    match stage {
        VcfDomainStage::Qc => parse_plink_qc_metrics(artifact_root),
        VcfDomainStage::Admixture => parse_plink_admixture_prep_metrics(artifact_root),
        other => bail!("unsupported plink VCF parser stage `{}`", other.as_str()),
    }
}

/// Normalize the governed raw `plink2` artifact set for a retained VCF stage.
///
/// # Errors
/// Returns an error when required raw artifacts are missing or malformed.
pub fn parse_plink2_stage_metrics(
    stage: VcfDomainStage,
    artifact_root: &Path,
) -> Result<serde_json::Value> {
    match stage {
        VcfDomainStage::Qc => parse_plink2_qc_metrics(artifact_root),
        VcfDomainStage::Pca => parse_plink2_pca_metrics(artifact_root),
        VcfDomainStage::Admixture => parse_plink2_admixture_metrics(artifact_root),
        VcfDomainStage::PopulationStructure => {
            parse_plink2_population_structure_metrics(artifact_root)
        }
        VcfDomainStage::Roh => parse_plink2_roh_metrics(artifact_root),
        other => bail!("unsupported plink2 VCF parser stage `{}`", other.as_str()),
    }
}

fn parse_plink_qc_metrics(root: &Path) -> Result<serde_json::Value> {
    let thresholds = parse_thresholds(&root.join("raw.log"))?;
    let sample_missingness = parse_sample_missingness_table(&root.join("raw.imiss"))?;
    let variant_missingness = parse_variant_missingness_table(&root.join("raw.lmiss"))?;
    let maf_summary = parse_plink_frequency_table(&root.join("raw.frq"))?;
    let heterozygosity = parse_heterozygosity_table(&root.join("raw.het"))?;
    let hwe_summary = parse_hwe_summary(&root.join("raw.hwe"))?;

    Ok(render_qc_payload(
        "plink",
        &sample_missingness,
        &variant_missingness,
        &maf_summary,
        &heterozygosity,
        &hwe_summary,
        &thresholds,
    ))
}

fn parse_plink2_qc_metrics(root: &Path) -> Result<serde_json::Value> {
    let thresholds = parse_thresholds(&root.join("raw.log"))?;
    let sample_missingness = parse_sample_missingness_table(&root.join("raw.smiss"))?;
    let variant_missingness = parse_variant_missingness_table(&root.join("raw.vmiss"))?;
    let maf_summary = parse_plink2_frequency_table(&root.join("raw.afreq"))?;
    let heterozygosity = parse_heterozygosity_table(&root.join("raw.het"))?;
    let hwe_summary = parse_hwe_summary(&root.join("raw.hardy"))?;

    Ok(render_qc_payload(
        "plink2",
        &sample_missingness,
        &variant_missingness,
        &maf_summary,
        &heterozygosity,
        &hwe_summary,
        &thresholds,
    ))
}

fn render_qc_payload(
    tool_id: &str,
    sample_missingness: &[SampleMissingnessRow],
    variant_missingness: &[VariantMissingnessRow],
    maf_summary: &FrequencySummary,
    heterozygosity: &[HeterozygosityRow],
    hwe_summary: &HweSummary,
    thresholds: &BTreeMap<String, String>,
) -> serde_json::Value {
    let sample_threshold = thresholds
        .get("sample_missingness_exclusion_threshold")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.5);
    let variant_threshold = thresholds
        .get("variant_missingness_exclusion_threshold")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.5);

    let excluded_samples = sample_missingness
        .iter()
        .filter(|row| row.missingness > sample_threshold)
        .map(sample_missingness_json)
        .collect::<Vec<_>>();
    let excluded_variants = variant_missingness
        .iter()
        .filter(|row| row.missingness > variant_threshold)
        .map(variant_missingness_json)
        .collect::<Vec<_>>();

    let heterozygous_total =
        heterozygosity.iter().map(|row| row.heterozygous_call_count).sum::<u64>();
    let observed_homozygous_total =
        heterozygosity.iter().map(|row| row.observed_homozygous_count).sum::<u64>();
    let mean_inbreeding_coefficient = if heterozygosity.is_empty() {
        0.0
    } else {
        heterozygosity.iter().map(|row| row.inbreeding_coefficient).sum::<f64>()
            / usize_to_f64(heterozygosity.len())
    };

    serde_json::json!({
        "schema_version": "bijux.vcf.qc.v1",
        "stage_id": "vcf.qc",
        "tool_id": tool_id,
        "variant_count": maf_summary.observed_variant_count,
        "sample_missingness": sample_missingness
            .iter()
            .map(sample_missingness_json)
            .collect::<Vec<_>>(),
        "variant_missingness": variant_missingness
            .iter()
            .map(variant_missingness_json)
            .collect::<Vec<_>>(),
        "maf_summary": {
            "allele_frequency_mean": maf_summary.allele_frequency_mean,
            "maf_bin_counts": maf_summary.maf_bin_counts,
            "observed_variant_count": maf_summary.observed_variant_count,
        },
        "heterozygosity": {
            "sample_rows": heterozygosity
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "sample_id": row.sample_id,
                        "observed_homozygous_count": row.observed_homozygous_count,
                        "nonmissing_variant_count": row.nonmissing_variant_count,
                        "heterozygous_call_count": row.heterozygous_call_count,
                        "inbreeding_coefficient": row.inbreeding_coefficient,
                    })
                })
                .collect::<Vec<_>>(),
            "heterozygous_call_count": heterozygous_total,
            "observed_homozygous_count": observed_homozygous_total,
            "het_hom_ratio": if observed_homozygous_total == 0 {
                serde_json::Value::Null
            } else {
                serde_json::json!(
                    u64_to_f64(heterozygous_total) / u64_to_f64(observed_homozygous_total)
                )
            },
            "mean_inbreeding_coefficient": mean_inbreeding_coefficient,
        },
        "hwe_summary": {
            "tested_variant_count": hwe_summary.tested_variant_count,
            "pvalue_mean": hwe_summary.pvalue_mean,
            "status": hwe_summary.status,
        },
        "excluded_samples": excluded_samples,
        "excluded_variants": excluded_variants,
        "sample_missingness_exclusion_threshold": sample_threshold,
        "variant_missingness_exclusion_threshold": variant_threshold,
    })
}

fn parse_plink_admixture_prep_metrics(root: &Path) -> Result<serde_json::Value> {
    let sample_rows = parse_fam_samples(&root.join("raw.fam"))?;
    let variant_count = count_bim_rows(&root.join("raw.bim"))?;
    let maf_summary = parse_plink_frequency_table(&root.join("raw.frq"))?;
    let sample_missingness = parse_sample_missingness_table(&root.join("raw.imiss"))?;
    let variant_missingness = parse_variant_missingness_table(&root.join("raw.lmiss"))?;
    let _log = parse_key_value_lines(&root.join("raw.log"))?;

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.admixture.v1",
        "stage_id": "vcf.admixture",
        "tool_id": PLINK_TOOL_ID,
        "selected_k": 0,
        "status": "cohort_preparation_only",
        "sample_count": sample_rows.len(),
        "population_count": 0,
        "cluster_headers": Vec::<String>::new(),
        "variant_count": variant_count,
        "maf_summary": {
            "allele_frequency_mean": maf_summary.allele_frequency_mean,
            "maf_bin_counts": maf_summary.maf_bin_counts,
            "observed_variant_count": maf_summary.observed_variant_count,
        },
        "sample_missingness": sample_missingness
            .iter()
            .map(sample_missingness_json)
            .collect::<Vec<_>>(),
        "variant_missingness": variant_missingness
            .iter()
            .map(variant_missingness_json)
            .collect::<Vec<_>>(),
        "rows": sample_rows
            .into_iter()
            .map(|(sample_id, pedigree_group)| {
                serde_json::json!({
                    "sample_id": sample_id,
                    "pedigree_group": pedigree_group,
                    "status": "cohort_preparation_only",
                })
            })
            .collect::<Vec<_>>(),
    }))
}

fn parse_plink2_pca_metrics(root: &Path) -> Result<serde_json::Value> {
    let manifest = read_json(&root.join("raw.pca_manifest.json"))?;
    let log = parse_key_value_lines(&root.join("raw.log"))?;
    let rows = parse_pca_rows(&root.join("raw.eigenvec"))?;
    let eigenvalues = parse_eigenvalues(&root.join("raw.eigenval"))?;

    let declared_tool_id = json_string(&manifest, "/toolchain", "toolchain")
        .unwrap_or_else(|_| PLINK2_TOOL_ID.to_string());
    if declared_tool_id != PLINK2_TOOL_ID {
        bail!("plink2 pca manifest declared toolchain `{declared_tool_id}`");
    }

    let sample_ids = json_string_array(&manifest, "/sample_ids", "sample_ids")?;
    let labels = sample_population_labels(&manifest)?;
    let row_ids = rows.iter().map(|row| row.sample_id.clone()).collect::<BTreeSet<_>>();
    let excluded_samples = sample_ids
        .iter()
        .filter(|sample_id| !row_ids.contains(sample_id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let unexpected_samples = row_ids
        .iter()
        .filter(|sample_id| !sample_ids.iter().any(|declared| declared == *sample_id))
        .cloned()
        .collect::<Vec<_>>();
    let variant_count = manifest
        .pointer("/variants_passing")
        .and_then(serde_json::Value::as_u64)
        .or_else(|| log.get("variants_passing").and_then(|value| value.parse::<u64>().ok()))
        .unwrap_or(0);

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.pca.v1",
        "stage_id": "vcf.pca",
        "tool_id": PLINK2_TOOL_ID,
        "variant_count": variant_count,
        "sample_count": sample_ids.len(),
        "excluded_samples": excluded_samples,
        "unexpected_samples": unexpected_samples,
        "eigenvalues": eigenvalues,
        "execution_mode": manifest.pointer("/execution_mode").and_then(serde_json::Value::as_str),
        "tool_ok": manifest.pointer("/tool_attempts/pca/ok").and_then(serde_json::Value::as_bool),
        "rows": rows
            .into_iter()
            .map(|row| {
                let mut object = serde_json::Map::new();
                object.insert("sample_id".to_string(), serde_json::Value::String(row.sample_id.clone()));
                if let Some(population_id) = labels.get(&row.sample_id) {
                    object.insert(
                        "population_id".to_string(),
                        serde_json::Value::String(population_id.clone()),
                    );
                }
                for (component, value) in row.components {
                    object.insert(component.to_ascii_lowercase(), serde_json::json!(value));
                }
                serde_json::Value::Object(object)
            })
            .collect::<Vec<_>>(),
    }))
}

fn parse_plink2_admixture_metrics(root: &Path) -> Result<serde_json::Value> {
    let log = parse_key_value_lines(&root.join("raw.log"))?;
    let manifest = read_json(&root.join("raw.k_selection.json"))?;
    let q_rows = parse_q_matrix_rows(&root.join("raw.q_matrix.tsv"))?;
    let cluster_headers = json_string_array(&manifest, "/cluster_headers", "cluster_headers")?;
    let selected_k = json_u64(&manifest, "/selected_k", "selected_k")?;
    let status = json_string(&manifest, "/status", "status")?;
    let population_labels = sample_population_labels(&manifest)?;

    for row in &q_rows {
        let observed_headers = row
            .iter()
            .filter_map(|(key, _)| key.strip_prefix("cluster_").map(|_| key.as_str()))
            .collect::<Vec<_>>();
        if observed_headers.len() != cluster_headers.len() {
            bail!(
                "plink2 admixture q-matrix cluster count drifted for sample `{}`",
                row.get("sample").cloned().unwrap_or_default()
            );
        }
    }

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.admixture.v1",
        "stage_id": "vcf.admixture",
        "tool_id": PLINK2_TOOL_ID,
        "selected_k": selected_k,
        "status": status,
        "execution_mode": log.get("execution_mode").cloned().or_else(|| manifest.pointer("/execution_mode").and_then(serde_json::Value::as_str).map(str::to_string)),
        "tool_ok": manifest.pointer("/tool_ok").and_then(serde_json::Value::as_bool),
        "sample_count": json_u64(&manifest, "/sample_count", "sample_count")?,
        "population_count": json_u64(&manifest, "/population_count", "population_count")?,
        "cluster_headers": cluster_headers,
        "rows": q_rows
            .into_iter()
            .map(|row| {
                let sample_id = row
                    .get("sample")
                    .cloned()
                    .ok_or_else(|| anyhow!("plink2 admixture row is missing `sample`"))
                    .and_then(|sample_id| {
                        let mut object = serde_json::Map::new();
                        object.insert("sample_id".to_string(), serde_json::Value::String(sample_id.clone()));
                        if let Some(population_id) = population_labels.get(&sample_id) {
                            object.insert(
                                "population_id".to_string(),
                                serde_json::Value::String(population_id.clone()),
                            );
                        }
                        object.insert("K".to_string(), serde_json::json!(selected_k));
                        object.insert("status".to_string(), serde_json::Value::String(status.clone()));
                        for (key, value) in row {
                            if key == "sample" {
                                continue;
                            }
                            object.insert(key, serde_json::json!(parse_f64(&value, "cluster value")?));
                        }
                        Ok::<serde_json::Value, anyhow::Error>(serde_json::Value::Object(object))
                    });
                sample_id
            })
            .collect::<Result<Vec<_>>>()?,
    }))
}

fn parse_plink2_population_structure_metrics(root: &Path) -> Result<serde_json::Value> {
    let pca = read_json(&root.join("raw.pca.json"))?;
    let admixture = read_json(&root.join("raw.admixture.json"))?;
    let source = read_json(&root.join("raw.population_structure.json"))?;
    let _prune_log = parse_key_value_lines(&root.join("raw.prune.log"))?;
    let _pca_log = parse_key_value_lines(&root.join("raw.pca.log"))?;
    let pruned_variants = count_data_rows(&root.join("raw.prune.in"))?;
    let _pruned_out = count_data_rows(&root.join("raw.prune.out"))?;

    let pca_rows = pca
        .pointer("/rows")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("plink2 population structure fixture is missing pca.rows"))?;
    let admixture_rows = admixture
        .pointer("/rows")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("plink2 population structure fixture is missing admixture.rows"))?;

    let admixture_by_sample = admixture_rows
        .iter()
        .map(|row| {
            let sample_id = row
                .get("sample_id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("admixture row is missing sample_id"))?;
            Ok((sample_id.to_string(), row))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;

    let sample_groups = pca_rows
        .iter()
        .map(|pca_row| build_population_structure_row(pca_row, &admixture_by_sample))
        .collect::<Result<Vec<_>>>()?;
    let distance_summary = build_distance_summary(&sample_groups)?;
    let source_status = json_string(&source, "/status", "status")?;
    let source_variant_count = source
        .pointer("/metrics/variants_passing_after_pruning")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    if source_variant_count != pruned_variants {
        bail!(
            "plink2 population structure pruned variant drifted: source declared {source_variant_count}, prune.in contained {pruned_variants}"
        );
    }

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.population_structure.v1",
        "stage_id": "vcf.population_structure",
        "tool_id": PLINK2_TOOL_ID,
        "consumed_pca": {
            "sample_count": pca.pointer("/sample_count").and_then(serde_json::Value::as_u64).unwrap_or(0),
            "variant_count": pca.pointer("/variant_count").and_then(serde_json::Value::as_u64).unwrap_or(0),
            "execution_mode": pca.pointer("/execution_mode").and_then(serde_json::Value::as_str),
            "tool_ok": pca.pointer("/tool_ok").and_then(serde_json::Value::as_bool),
        },
        "consumed_admixture": {
            "sample_count": admixture.pointer("/sample_count").and_then(serde_json::Value::as_u64).unwrap_or(0),
            "selected_k": admixture.pointer("/selected_k").and_then(serde_json::Value::as_u64).unwrap_or(0),
            "execution_mode": admixture.pointer("/execution_mode").and_then(serde_json::Value::as_str),
            "tool_ok": admixture.pointer("/tool_ok").and_then(serde_json::Value::as_bool),
            "status": admixture.pointer("/status").and_then(serde_json::Value::as_str),
        },
        "sample_groups": sample_groups,
        "distance_summary": distance_summary,
        "variant_count": pruned_variants,
        "status": source_status,
    }))
}

fn parse_plink2_roh_metrics(root: &Path) -> Result<serde_json::Value> {
    parse_segment_stage_metrics(PLINK2_TOOL_ID, VcfDomainStage::Roh, root)
}

fn build_population_structure_row(
    pca_row: &serde_json::Value,
    admixture_by_sample: &BTreeMap<String, &serde_json::Value>,
) -> Result<serde_json::Value> {
    let sample_id = pca_row
        .get("sample_id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("population structure pca row is missing sample_id"))?;
    let admixture_row = admixture_by_sample.get(sample_id).ok_or_else(|| {
        anyhow!("population structure admixture row is missing sample `{sample_id}`")
    })?;

    let mut dominant_cluster = None::<String>;
    let mut dominant_fraction = None::<f64>;
    for (key, value) in
        admixture_row.as_object().ok_or_else(|| anyhow!("admixture row must be an object"))?
    {
        if !key.starts_with("cluster_") {
            continue;
        }
        let fraction = value
            .as_f64()
            .ok_or_else(|| anyhow!("cluster fraction for `{sample_id}` is not numeric"))?;
        let replace = dominant_fraction.is_none_or(|current| fraction > current);
        if replace {
            dominant_cluster = Some(key.clone());
            dominant_fraction = Some(fraction);
        }
    }

    Ok(serde_json::json!({
        "sample_id": sample_id,
        "population_id": pca_row.get("population_id").cloned().unwrap_or(serde_json::Value::Null),
        "dominant_cluster": dominant_cluster,
        "dominant_fraction": dominant_fraction,
        "pc1": pca_row.get("pc1").cloned().unwrap_or(serde_json::Value::Null),
        "pc2": pca_row.get("pc2").cloned().unwrap_or(serde_json::Value::Null),
        "status": admixture_row.get("status").cloned().unwrap_or_else(|| serde_json::Value::String("complete".to_string())),
    }))
}

fn build_distance_summary(sample_groups: &[serde_json::Value]) -> Result<serde_json::Value> {
    let mut pair_count = 0_u64;
    let mut within_population_pair_count = 0_u64;
    let mut cross_population_pair_count = 0_u64;
    let mut distances = Vec::<f64>::new();

    for (index, left) in sample_groups.iter().enumerate() {
        let left_population = left.get("population_id").and_then(serde_json::Value::as_str);
        let left_pc1 = left
            .get("pc1")
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| anyhow!("population structure sample row is missing pc1"))?;
        let left_pc2 = left
            .get("pc2")
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| anyhow!("population structure sample row is missing pc2"))?;
        for right in &sample_groups[index + 1..] {
            let right_population = right.get("population_id").and_then(serde_json::Value::as_str);
            let right_pc1 = right
                .get("pc1")
                .and_then(serde_json::Value::as_f64)
                .ok_or_else(|| anyhow!("population structure sample row is missing pc1"))?;
            let right_pc2 = right
                .get("pc2")
                .and_then(serde_json::Value::as_f64)
                .ok_or_else(|| anyhow!("population structure sample row is missing pc2"))?;
            pair_count += 1;
            if left_population == right_population {
                within_population_pair_count += 1;
            } else {
                cross_population_pair_count += 1;
            }
            distances
                .push(((left_pc1 - right_pc1).powi(2) + (left_pc2 - right_pc2).powi(2)).sqrt());
        }
    }

    let min_pc_distance = distances.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let max_pc_distance = distances.iter().copied().reduce(f64::max).unwrap_or(0.0);
    let mean_pc_distance = if distances.is_empty() {
        0.0
    } else {
        distances.iter().sum::<f64>() / usize_to_f64(distances.len())
    };

    Ok(serde_json::json!({
        "sample_count": sample_groups.len(),
        "pair_count": pair_count,
        "within_population_pair_count": within_population_pair_count,
        "cross_population_pair_count": cross_population_pair_count,
        "min_pc_distance": min_pc_distance,
        "max_pc_distance": max_pc_distance,
        "mean_pc_distance": mean_pc_distance,
    }))
}

fn parse_q_matrix_rows(path: &Path) -> Result<Vec<BTreeMap<String, String>>> {
    let (header, rows) = read_table(path)?;
    rows.into_iter()
        .map(|row| {
            if row.len() != header.len() {
                bail!(
                    "q-matrix row in {} expected {} columns, found {}",
                    path.display(),
                    header.len(),
                    row.len()
                );
            }
            Ok(header.iter().cloned().zip(row).collect::<BTreeMap<_, _>>())
        })
        .collect()
}

fn parse_pca_rows(path: &Path) -> Result<Vec<PcaRow>> {
    let (header, rows) = read_table(path)?;
    let sample_idx = index_for(&header, &["sample", "iid"])?;
    let component_indexes = header
        .iter()
        .enumerate()
        .filter_map(|(index, name)| {
            let normalized = normalize_header(name);
            normalized
                .strip_prefix("pc")
                .and_then(|suffix| suffix.parse::<usize>().ok())
                .map(|_| (index, normalized.to_uppercase()))
        })
        .collect::<Vec<_>>();
    if component_indexes.is_empty() {
        bail!("pca eigenvector table in {} does not declare PC columns", path.display());
    }

    let mut rows = rows
        .into_iter()
        .map(|row| {
            let sample_id = field(&row, sample_idx, path)?.to_string();
            let mut components = BTreeMap::<String, f64>::new();
            for (index, name) in &component_indexes {
                components.insert(name.clone(), parse_f64(field(&row, *index, path)?, name)?);
            }
            Ok(PcaRow { sample_id, components })
        })
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| left.sample_id.cmp(&right.sample_id));
    Ok(rows)
}

fn parse_eigenvalues(path: &Path) -> Result<Vec<f64>> {
    let (header, rows) = read_table(path)?;
    let value_idx = index_for(&header, &["eigenvalue"])?;
    rows.into_iter().map(|row| parse_f64(field(&row, value_idx, path)?, "eigenvalue")).collect()
}

fn parse_sample_missingness_table(path: &Path) -> Result<Vec<SampleMissingnessRow>> {
    let (header, rows) = read_table(path)?;
    let sample_idx = index_for(&header, &["iid", "sample"])?;
    let missing_idx = index_for(&header, &["n_miss", "miss_ct"])?;
    let total_idx = index_for(&header, &["n_geno", "obs_ct"])?;
    let missingness_idx = index_for(&header, &["f_miss"])?;

    let mut parsed = rows
        .into_iter()
        .map(|row| {
            Ok(SampleMissingnessRow {
                sample_id: field(&row, sample_idx, path)?.to_string(),
                total_genotype_count: parse_u64(
                    field(&row, total_idx, path)?,
                    "total genotype count",
                )?,
                missing_genotype_count: parse_u64(
                    field(&row, missing_idx, path)?,
                    "missing genotype count",
                )?,
                missingness: parse_f64(field(&row, missingness_idx, path)?, "sample missingness")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    parsed.sort_by(|left, right| left.sample_id.cmp(&right.sample_id));
    Ok(parsed)
}

fn parse_variant_missingness_table(path: &Path) -> Result<Vec<VariantMissingnessRow>> {
    let (header, rows) = read_table(path)?;
    let missing_idx = index_for(&header, &["n_miss", "miss_ct"])?;
    let total_idx = index_for(&header, &["n_geno", "obs_ct"])?;
    let missingness_idx = index_for(&header, &["f_miss"])?;
    let id_idx = index_for(&header, &["snp", "id", "variant_id"])?;
    let chr_idx = find_index(&header, &["chr", "chrom"]);
    let pos_idx = find_index(&header, &["pos", "position"]);
    let ref_idx = find_index(&header, &["ref", "reference", "a2"]);
    let alt_idx = find_index(&header, &["alt", "alternate", "a1"]);

    let mut parsed = rows
        .into_iter()
        .map(|row| {
            let variant_id = field(&row, id_idx, path)?.to_string();
            let (contig, position, reference, alternate) =
                if let (Some(chr_idx), Some(pos_idx), Some(ref_idx), Some(alt_idx)) =
                    (chr_idx, pos_idx, ref_idx, alt_idx)
                {
                    (
                        field(&row, chr_idx, path)?.to_string(),
                        parse_u64(field(&row, pos_idx, path)?, "variant position")?,
                        field(&row, ref_idx, path)?.to_string(),
                        field(&row, alt_idx, path)?.to_string(),
                    )
                } else {
                    parse_variant_id(&variant_id)?
                };
            Ok(VariantMissingnessRow {
                variant_id,
                contig,
                position,
                reference,
                alternate,
                total_sample_count: parse_u64(
                    field(&row, total_idx, path)?,
                    "variant total sample count",
                )?,
                missing_sample_count: parse_u64(
                    field(&row, missing_idx, path)?,
                    "variant missing sample count",
                )?,
                missingness: parse_f64(field(&row, missingness_idx, path)?, "variant missingness")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    parsed.sort_by(|left, right| left.variant_id.cmp(&right.variant_id));
    Ok(parsed)
}

fn parse_plink_frequency_table(path: &Path) -> Result<FrequencySummary> {
    let (header, rows) = read_table(path)?;
    let frequency_idx = index_for(&header, &["maf", "alt_freqs"])?;
    let mut values = Vec::<f64>::new();
    let mut bins = BTreeMap::<String, u64>::new();
    for row in rows {
        let value = parse_f64(field(&row, frequency_idx, path)?, "frequency")?;
        values.push(value);
        if let Some(bin) = maf_bin_label(value) {
            *bins.entry(bin.to_string()).or_insert(0) += 1;
        }
    }
    Ok(FrequencySummary {
        allele_frequency_mean: mean(&values),
        maf_bin_counts: bins,
        observed_variant_count: u64::try_from(values.len()).unwrap_or(0),
    })
}

fn parse_plink2_frequency_table(path: &Path) -> Result<FrequencySummary> {
    parse_plink_frequency_table(path)
}

fn parse_heterozygosity_table(path: &Path) -> Result<Vec<HeterozygosityRow>> {
    let (header, rows) = read_table(path)?;
    let sample_idx = index_for(&header, &["iid", "sample"])?;
    let hom_idx = index_for(&header, &["ohom", "o_hom"])?;
    let total_idx = index_for(&header, &["nnm", "n_nm"])?;
    let f_idx = index_for(&header, &["f"])?;

    let mut parsed = rows
        .into_iter()
        .map(|row| {
            let observed_homozygous_count =
                parse_u64(field(&row, hom_idx, path)?, "observed homozygous count")?;
            let nonmissing_variant_count =
                parse_u64(field(&row, total_idx, path)?, "nonmissing variant count")?;
            Ok(HeterozygosityRow {
                sample_id: field(&row, sample_idx, path)?.to_string(),
                observed_homozygous_count,
                nonmissing_variant_count,
                heterozygous_call_count: nonmissing_variant_count
                    .saturating_sub(observed_homozygous_count),
                inbreeding_coefficient: parse_f64(
                    field(&row, f_idx, path)?,
                    "inbreeding coefficient",
                )?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    parsed.sort_by(|left, right| left.sample_id.cmp(&right.sample_id));
    Ok(parsed)
}

fn parse_hwe_summary(path: &Path) -> Result<HweSummary> {
    let (header, rows) = read_table(path)?;
    let pvalue_idx = index_for(&header, &["p"])?;
    let mut pvalues = Vec::<f64>::new();
    for row in rows {
        pvalues.push(parse_f64(field(&row, pvalue_idx, path)?, "hwe p-value")?);
    }
    if pvalues.is_empty() {
        bail!("required HWE table {} is empty", path.display());
    }
    Ok(HweSummary {
        tested_variant_count: u64::try_from(pvalues.len()).unwrap_or(0),
        pvalue_mean: Some(round_f64(mean(&pvalues), 6)),
        status: "computed_modern".to_string(),
    })
}

fn parse_fam_samples(path: &Path) -> Result<Vec<(String, String)>> {
    let rows = read_rows(path)?;
    rows.into_iter()
        .map(|row| {
            if row.len() < 2 {
                bail!("FAM row in {} must provide at least FID and IID", path.display());
            }
            Ok((row[1].clone(), row[0].clone()))
        })
        .collect()
}

fn count_bim_rows(path: &Path) -> Result<u64> {
    Ok(u64::try_from(read_rows(path)?.len()).unwrap_or(0))
}

fn parse_thresholds(path: &Path) -> Result<BTreeMap<String, String>> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    parse_key_value_lines(path)
}

fn parse_key_value_lines(path: &Path) -> Result<BTreeMap<String, String>> {
    let raw = read_text(path)?;
    let mut values = BTreeMap::<String, String>::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        values.insert(key.trim().to_string(), value.trim().to_string());
    }
    Ok(values)
}

fn sample_population_labels(source: &serde_json::Value) -> Result<BTreeMap<String, String>> {
    let mut labels = BTreeMap::<String, String>::new();
    if let Some(rows) =
        source.pointer("/sample_population_labels").and_then(serde_json::Value::as_array)
    {
        for row in rows {
            let sample_id = row
                .get("sample_id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("sample_population_labels row is missing sample_id"))?;
            let population_id = row
                .get("population_id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("sample_population_labels row is missing population_id"))?;
            labels.insert(sample_id.to_string(), population_id.to_string());
        }
    }
    Ok(labels)
}

fn count_data_rows(path: &Path) -> Result<u64> {
    let (_header, rows) = read_table(path)?;
    Ok(u64::try_from(rows.len()).unwrap_or(0))
}

fn round_f64(value: f64, scale: u32) -> f64 {
    let factor = 10_f64.powi(i32::try_from(scale).unwrap_or(0));
    (value * factor).round() / factor
}

fn read_table(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    let rows = read_rows(path)?;
    let (header, rows) =
        rows.split_first().ok_or_else(|| anyhow!("table {} is empty", path.display()))?;
    Ok((header.clone(), rows.to_vec()))
}

fn read_rows(path: &Path) -> Result<Vec<Vec<String>>> {
    let raw = read_text(path)?;
    let mut rows = Vec::<Vec<String>>::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("##") {
            continue;
        }
        rows.push(line.split_whitespace().map(str::to_string).collect());
    }
    Ok(rows)
}

fn read_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = read_text(path)?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn json_string(value: &serde_json::Value, pointer: &str, field: &str) -> Result<String> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("json field `{field}` is missing or not a string"))
}

fn json_u64(value: &serde_json::Value, pointer: &str, field: &str) -> Result<u64> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("json field `{field}` is missing or not a u64"))
}

fn json_string_array(value: &serde_json::Value, pointer: &str, field: &str) -> Result<Vec<String>> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("json field `{field}` is missing or not an array"))?
        .iter()
        .map(|row| {
            row.as_str()
                .map(str::to_string)
                .ok_or_else(|| anyhow!("json field `{field}` must contain only strings"))
        })
        .collect()
}

fn field<'a>(row: &'a [String], index: usize, path: &Path) -> Result<&'a str> {
    row.get(index)
        .map(String::as_str)
        .ok_or_else(|| anyhow!("row in {} is missing column {}", path.display(), index))
}

fn normalize_header(value: &str) -> String {
    value
        .trim_start_matches('#')
        .chars()
        .filter_map(|character| match character {
            '(' | ')' | '/' => None,
            '-' => Some('_'),
            _ => Some(character.to_ascii_lowercase()),
        })
        .collect()
}

fn index_for(header: &[String], aliases: &[&str]) -> Result<usize> {
    find_index(header, aliases)
        .ok_or_else(|| anyhow!("table header {header:?} is missing one of {aliases:?}"))
}

fn find_index(header: &[String], aliases: &[&str]) -> Option<usize> {
    header.iter().position(|column| {
        let normalized = normalize_header(column);
        aliases.iter().any(|alias| normalized == normalize_header(alias))
    })
}

fn parse_variant_id(value: &str) -> Result<(String, u64, String, String)> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 4 {
        bail!("variant id `{value}` must follow contig:position:reference:alternate");
    }
    Ok((
        parts[0].to_string(),
        parse_u64(parts[1], "variant position")?,
        parts[2].to_string(),
        parts[3].to_string(),
    ))
}

fn sample_missingness_json(row: &SampleMissingnessRow) -> serde_json::Value {
    serde_json::json!({
        "sample_id": row.sample_id,
        "total_genotype_count": row.total_genotype_count,
        "missing_genotype_count": row.missing_genotype_count,
        "missingness": row.missingness,
    })
}

fn variant_missingness_json(row: &VariantMissingnessRow) -> serde_json::Value {
    serde_json::json!({
        "variant_id": row.variant_id,
        "contig": row.contig,
        "position": row.position,
        "reference": row.reference,
        "alternate": row.alternate,
        "total_sample_count": row.total_sample_count,
        "missing_sample_count": row.missing_sample_count,
        "missingness": row.missingness,
    })
}

fn parse_u64(value: &str, field: &str) -> Result<u64> {
    value.parse::<u64>().with_context(|| format!("parse `{field}` from `{value}`"))
}

fn parse_f64(value: &str, field: &str) -> Result<f64> {
    value.parse::<f64>().with_context(|| format!("parse `{field}` from `{value}`"))
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / usize_to_f64(values.len())
    }
}

fn usize_to_f64(value: usize) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

fn maf_bin_label(value: f64) -> Option<&'static str> {
    if (0.05..0.1).contains(&value) {
        Some("0.05-0.1")
    } else if (0.1..0.2).contains(&value) {
        Some("0.1-0.2")
    } else if (0.2..=0.5).contains(&value) {
        Some("0.2-0.5")
    } else {
        None
    }
}
