use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};

use crate::taxonomy::VcfDomainStage;

const EIGENSOFT_TOOL_ID: &str = "eigensoft";

const RAW_COMMAND_NAME: &str = "raw.command.json";
const RAW_GENO_NAME: &str = "raw.geno";
const RAW_SNP_NAME: &str = "raw.snp";
const RAW_IND_NAME: &str = "raw.ind";
const RAW_EVEC_NAME: &str = "raw.evec";
const RAW_EVAL_NAME: &str = "raw.eval";
const RAW_LOG_NAME: &str = "raw.smartpca.log";
const RAW_SAMPLE_METADATA_NAME: &str = "raw.sample_metadata.tsv";
const RAW_POPULATION_STRUCTURE_NAME: &str = "raw.population_structure.json";

#[derive(Debug, Clone)]
struct EigensoftIndividual {
    sample_id: String,
    population_label: String,
}

#[derive(Debug, Clone)]
struct SampleMetadataRow {
    sample_id: String,
    population_id: String,
    role: String,
}

#[derive(Debug, Clone)]
struct PcaRow {
    sample_id: String,
    components: BTreeMap<String, f64>,
}

/// Normalize the governed raw `eigensoft` artifact set for a retained VCF stage.
///
/// # Errors
/// Returns an error when required raw artifacts are missing or malformed.
pub fn parse_eigensoft_stage_metrics(
    stage: VcfDomainStage,
    artifact_root: &Path,
) -> Result<serde_json::Value> {
    match stage {
        VcfDomainStage::Pca => parse_pca_metrics(artifact_root),
        VcfDomainStage::PopulationStructure => parse_population_structure_metrics(artifact_root),
        other => bail!("unsupported eigensoft VCF parser stage `{}`", other.as_str()),
    }
}

fn parse_pca_metrics(root: &Path) -> Result<serde_json::Value> {
    let command = read_json(&root.join(RAW_COMMAND_NAME))?;
    validate_stage_command(&command, "vcf.pca")?;
    validate_nonempty_log(&root.join(RAW_LOG_NAME))?;

    let metadata_rows = parse_sample_metadata(&root.join(RAW_SAMPLE_METADATA_NAME))?;
    let expected_samples = cohort_sample_ids(&metadata_rows);
    let population_labels = sample_population_labels(&metadata_rows);
    let individual_rows = parse_individual_rows(&root.join(RAW_IND_NAME))?;
    let pca_rows = parse_evec_rows(&root.join(RAW_EVEC_NAME))?;
    let eigenvalues = parse_eval_rows(&root.join(RAW_EVAL_NAME))?;
    let variant_count = count_rows(&root.join(RAW_SNP_NAME))?;
    let geno_row_count = count_rows(&root.join(RAW_GENO_NAME))?;
    if geno_row_count != variant_count {
        bail!(
            "eigensoft PCA geno/snp row count mismatch: geno={geno_row_count}, snp={variant_count}"
        );
    }

    let row_ids = pca_rows.iter().map(|row| row.sample_id.clone()).collect::<BTreeSet<_>>();
    let ind_ids = individual_rows.iter().map(|row| row.sample_id.clone()).collect::<BTreeSet<_>>();
    if row_ids != ind_ids {
        bail!("eigensoft PCA sample drift between `.ind` and `.evec`: ind={ind_ids:?}, evec={row_ids:?}");
    }
    for individual in &individual_rows {
        let metadata_population = population_labels
            .get(&individual.sample_id)
            .ok_or_else(|| anyhow!("missing sample metadata for `{}`", individual.sample_id))?;
        if metadata_population != &individual.population_label {
            bail!(
                "eigensoft PCA sample `{}` population drifted: metadata=`{metadata_population}`, ind=`{}`",
                individual.sample_id,
                individual.population_label
            );
        }
    }

    let excluded_samples = expected_samples
        .iter()
        .filter(|sample_id| !row_ids.contains(sample_id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let unexpected_samples = row_ids
        .iter()
        .filter(|sample_id| !expected_samples.contains(*sample_id))
        .cloned()
        .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.pca.v1",
        "stage_id": "vcf.pca",
        "tool_id": EIGENSOFT_TOOL_ID,
        "variant_count": variant_count,
        "sample_count": pca_rows.len(),
        "excluded_samples": excluded_samples,
        "unexpected_samples": unexpected_samples,
        "eigenvalues": eigenvalues,
        "status": "complete",
        "rows": pca_rows.into_iter().map(|row| {
            let mut object = serde_json::Map::new();
            object.insert(
                "sample_id".to_string(),
                serde_json::Value::String(row.sample_id.clone()),
            );
            if let Some(population_id) = population_labels.get(&row.sample_id) {
                object.insert(
                    "population_id".to_string(),
                    serde_json::Value::String(population_id.clone()),
                );
            }
            for (component, value) in row.components {
                object.insert(component.to_ascii_lowercase(), serde_json::json!(value));
            }
            serde_json::Value::Object(object)
        }).collect::<Vec<_>>(),
    }))
}

fn parse_population_structure_metrics(root: &Path) -> Result<serde_json::Value> {
    let command = read_json(&root.join(RAW_COMMAND_NAME))?;
    validate_stage_command(&command, "vcf.population_structure")?;
    validate_nonempty_log(&root.join(RAW_LOG_NAME))?;

    let metadata_rows = parse_sample_metadata(&root.join(RAW_SAMPLE_METADATA_NAME))?;
    let expected_samples = cohort_sample_ids(&metadata_rows);
    let population_labels = sample_population_labels(&metadata_rows);
    let pca_rows = parse_evec_rows(&root.join(RAW_EVEC_NAME))?;
    let eigenvalues = parse_eval_rows(&root.join(RAW_EVAL_NAME))?;
    let individual_rows = parse_individual_rows(&root.join(RAW_IND_NAME))?;
    let variant_count = count_rows(&root.join(RAW_SNP_NAME))?;
    let source = read_json(&root.join(RAW_POPULATION_STRUCTURE_NAME))?;

    let row_ids = pca_rows.iter().map(|row| row.sample_id.clone()).collect::<BTreeSet<_>>();
    let ind_ids = individual_rows.iter().map(|row| row.sample_id.clone()).collect::<BTreeSet<_>>();
    if row_ids != ind_ids {
        bail!(
            "eigensoft population structure sample drift between `.ind` and `.evec`: ind={ind_ids:?}, evec={row_ids:?}"
        );
    }

    let sample_groups = source
        .get("sample_groups")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("population structure source is missing `sample_groups`"))?;
    let group_ids = sample_groups
        .iter()
        .map(|row| {
            row.get("sample_id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| anyhow!("population structure sample group is missing sample_id"))
        })
        .collect::<Result<BTreeSet<_>>>()?;
    if group_ids != row_ids {
        bail!(
            "eigensoft population structure sample drift between source and PCA rows: source={group_ids:?}, pca={row_ids:?}"
        );
    }

    let pca_by_sample =
        pca_rows.into_iter().map(|row| (row.sample_id.clone(), row)).collect::<BTreeMap<_, _>>();
    for sample_id in &group_ids {
        if !expected_samples.contains(sample_id) {
            bail!("population structure sample `{sample_id}` is not declared in cohort metadata");
        }
        if !population_labels.contains_key(sample_id) {
            bail!("population structure sample `{sample_id}` is missing metadata labels");
        }
        if !pca_by_sample.contains_key(sample_id) {
            bail!("population structure sample `{sample_id}` is missing PCA coordinates");
        }
    }

    let consumed_pca = source
        .get("consumed_pca")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow!("population structure source is missing `consumed_pca`"))?;
    let consumed_pca_sample_count =
        consumed_pca.get("sample_count").and_then(serde_json::Value::as_u64).ok_or_else(|| {
            anyhow!("population structure source consumed_pca is missing sample_count")
        })?;
    if consumed_pca_sample_count != u64::try_from(group_ids.len()).unwrap_or(0) {
        bail!(
            "population structure consumed_pca sample count drifted: source={consumed_pca_sample_count}, parsed={}",
            group_ids.len()
        );
    }
    let consumed_pca_variant_count =
        consumed_pca.get("variant_count").and_then(serde_json::Value::as_u64).ok_or_else(|| {
            anyhow!("population structure source consumed_pca is missing variant_count")
        })?;
    if consumed_pca_variant_count != variant_count {
        bail!(
            "population structure consumed_pca variant count drifted: source={consumed_pca_variant_count}, parsed={variant_count}"
        );
    }

    let mut normalized = source.clone();
    let object = normalized
        .as_object_mut()
        .ok_or_else(|| anyhow!("population structure source must be an object"))?;
    object.insert(
        "schema_version".to_string(),
        serde_json::Value::String("bijux.vcf.population_structure.v1".to_string()),
    );
    object.insert(
        "stage_id".to_string(),
        serde_json::Value::String("vcf.population_structure".to_string()),
    );
    object.insert("tool_id".to_string(), serde_json::Value::String(EIGENSOFT_TOOL_ID.to_string()));
    object.insert("status".to_string(), serde_json::Value::String("complete".to_string()));
    if let Some(consumed_pca_value) = object.get_mut("consumed_pca") {
        if let Some(consumed_pca_object) = consumed_pca_value.as_object_mut() {
            consumed_pca_object.insert("eigenvalues".to_string(), serde_json::json!(eigenvalues));
        }
    }
    Ok(normalized)
}

fn validate_stage_command(command: &serde_json::Value, stage_id: &str) -> Result<()> {
    let tool_id = json_string(command, "/tool_id", "tool_id")?;
    if tool_id != EIGENSOFT_TOOL_ID {
        bail!("eigensoft parser expected tool_id `{EIGENSOFT_TOOL_ID}`, found `{tool_id}`");
    }
    let declared_stage_id = json_string(command, "/stage_id", "stage_id")?;
    if declared_stage_id != stage_id {
        bail!("eigensoft parser expected stage_id `{stage_id}`, found `{declared_stage_id}`");
    }
    let argv = json_string_array(command, "/argv", "argv")?;
    for required_token in ["convertf", "smartpca"] {
        if !argv.iter().any(|token| token.contains(required_token)) {
            bail!(
                "eigensoft parser command metadata for `{stage_id}` is missing `{required_token}`"
            );
        }
    }
    Ok(())
}

fn validate_nonempty_log(path: &Path) -> Result<()> {
    let log = read_text(path)?;
    if log.trim().is_empty() {
        bail!("eigensoft log {} is empty", path.display());
    }
    Ok(())
}

fn parse_individual_rows(path: &Path) -> Result<Vec<EigensoftIndividual>> {
    let rows = read_rows(path)?;
    let mut individuals = rows
        .into_iter()
        .map(|row| {
            if row.len() < 3 {
                bail!(
                    "eigensoft IND row in {} must provide id, sex code, and population label",
                    path.display()
                );
            }
            Ok(EigensoftIndividual { sample_id: row[0].clone(), population_label: row[2].clone() })
        })
        .collect::<Result<Vec<_>>>()?;
    individuals.sort_by(|left, right| left.sample_id.cmp(&right.sample_id));
    Ok(individuals)
}

fn parse_sample_metadata(path: &Path) -> Result<Vec<SampleMetadataRow>> {
    let (header, rows) = read_table(path)?;
    let sample_idx = index_for(&header, "sample_id")?;
    let population_idx = index_for(&header, "population_id")?;
    let role_idx = index_for(&header, "role")?;
    let mut parsed = rows
        .into_iter()
        .map(|row| {
            Ok(SampleMetadataRow {
                sample_id: field(&row, sample_idx, path)?.to_string(),
                population_id: field(&row, population_idx, path)?.to_string(),
                role: field(&row, role_idx, path)?.to_string(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    parsed.sort_by(|left, right| left.sample_id.cmp(&right.sample_id));
    Ok(parsed)
}

fn cohort_sample_ids(rows: &[SampleMetadataRow]) -> BTreeSet<String> {
    rows.iter().filter(|row| row.role == "cohort").map(|row| row.sample_id.clone()).collect()
}

fn sample_population_labels(rows: &[SampleMetadataRow]) -> BTreeMap<String, String> {
    rows.iter().map(|row| (row.sample_id.clone(), row.population_id.clone())).collect()
}

fn parse_evec_rows(path: &Path) -> Result<Vec<PcaRow>> {
    let rows = read_rows(path)?;
    let mut parsed = Vec::<PcaRow>::new();
    for row in rows {
        if row.is_empty() {
            continue;
        }
        let sample_id = row[0].clone();
        let mut components = BTreeMap::<String, f64>::new();
        let mut component_index = 1_usize;
        for value in row.iter().skip(1) {
            match value.parse::<f64>() {
                Ok(parsed_value) => {
                    components.insert(format!("PC{component_index}"), parsed_value);
                    component_index += 1;
                }
                Err(_) => break,
            }
        }
        if components.is_empty() {
            bail!("eigensoft EVEC row for `{sample_id}` does not contain any numeric components");
        }
        parsed.push(PcaRow { sample_id, components });
    }
    parsed.sort_by(|left, right| left.sample_id.cmp(&right.sample_id));
    Ok(parsed)
}

fn parse_eval_rows(path: &Path) -> Result<Vec<f64>> {
    read_rows(path)?
        .into_iter()
        .map(|row| {
            let value = row
                .first()
                .ok_or_else(|| anyhow!("eigensoft EVAL row in {} is empty", path.display()))?;
            value
                .parse::<f64>()
                .with_context(|| format!("parse eigenvalue `{value}` in {}", path.display()))
        })
        .collect()
}

fn count_rows(path: &Path) -> Result<u64> {
    Ok(u64::try_from(read_rows(path)?.len()).unwrap_or(0))
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
        if line.is_empty() || line.starts_with('#') || line.starts_with("##") {
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

fn index_for(header: &[String], field_name: &str) -> Result<usize> {
    header
        .iter()
        .position(|value| value == field_name)
        .ok_or_else(|| anyhow!("table is missing required column `{field_name}`"))
}

fn field<'a>(row: &'a [String], index: usize, path: &Path) -> Result<&'a str> {
    row.get(index)
        .map(String::as_str)
        .ok_or_else(|| anyhow!("row in {} is missing field index {}", path.display(), index))
}

fn json_string(value: &serde_json::Value, pointer: &str, field: &str) -> Result<String> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("json field `{field}` is missing or not a string"))
}

fn json_string_array(value: &serde_json::Value, pointer: &str, field: &str) -> Result<Vec<String>> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("json field `{field}` is missing or not an array"))?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| anyhow!("json field `{field}` contains a non-string entry"))
        })
        .collect()
}
