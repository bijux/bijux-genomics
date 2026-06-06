use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};

use crate::taxonomy::VcfDomainStage;

const RAW_COMMAND_NAME: &str = "raw.command.json";
const RAW_IMPUTED_VCF_NAME: &str = "raw.imputed.vcf";
const RAW_IMPUTATION_QC_NAME: &str = "raw.imputation_qc.json";
const RAW_IMPUTATION_MANIFEST_NAME: &str = "raw.imputation_manifest.json";
const RAW_IMPUTATION_ACCEPT_NAME: &str = "raw.imputation_accept.json";
const RAW_ORCHESTRATION_MANIFEST_NAME: &str = "raw.orchestration_manifest.json";
const RAW_TRUTH_VCF_NAME: &str = "raw.truth.vcf";

#[derive(Debug, Clone)]
struct ParsedVcf {
    sample_ids: Vec<String>,
    variant_count: u64,
    genotypes_by_key: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct TruthComparison {
    masked_truth_site_count: u64,
    imputed_match_count: u64,
    unresolved_count: u64,
}

/// Normalize the governed raw imputation artifact set for a retained VCF backend.
///
/// # Errors
/// Returns an error when required raw artifacts are missing, malformed, or internally drifted.
pub fn parse_imputation_stage_metrics(
    tool_id: &str,
    stage: VcfDomainStage,
    artifact_root: &Path,
) -> Result<serde_json::Value> {
    match stage {
        VcfDomainStage::Imputation => parse_imputation_metrics(tool_id, artifact_root),
        VcfDomainStage::Impute => parse_impute_metrics(tool_id, artifact_root),
        other => bail!("unsupported imputation VCF parser stage `{}`", other.as_str()),
    }
}

fn parse_imputation_metrics(tool_id: &str, root: &Path) -> Result<serde_json::Value> {
    validate_command(tool_id, "vcf.imputation", &read_json(&root.join(RAW_COMMAND_NAME))?)?;

    let imputation_qc = read_json(&root.join(RAW_IMPUTATION_QC_NAME))?;
    validate_backend_field(tool_id, &imputation_qc, "imputation qc")?;
    let manifest = read_json(&root.join(RAW_IMPUTATION_MANIFEST_NAME))?;
    validate_manifest(tool_id, "vcf.imputation", &manifest)?;
    let orchestration_manifest = read_json(&root.join(RAW_ORCHESTRATION_MANIFEST_NAME))?;
    validate_orchestration_manifest(tool_id, &orchestration_manifest)?;
    validate_accept_report(&read_json(&root.join(RAW_IMPUTATION_ACCEPT_NAME))?)?;

    let output = parse_vcf(&root.join(RAW_IMPUTED_VCF_NAME))?;
    let truth = compare_truth_if_present(root, &output)?;
    validate_truth_concordance(&imputation_qc, truth.as_ref())?;

    let concordance = imputation_qc
        .pointer("/concordance/genotype_concordance")
        .and_then(serde_json::Value::as_f64);
    let dosage_r2 =
        imputation_qc.pointer("/concordance/dosage_r2").and_then(serde_json::Value::as_f64);
    let masked_truth_sites = imputation_qc
        .pointer("/concordance/masked_truth_site_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let maf_strata_present = imputation_qc
        .pointer("/concordance/maf_strata")
        .and_then(serde_json::Value::as_array)
        .is_some();
    let mean_info_score =
        json_f64(&imputation_qc, "/imputation_info_mean", "imputation_info_mean")?;
    let low_confidence_sites =
        json_u64(&imputation_qc, "/low_confidence_count", "low_confidence_count")?;

    let mut missing_quality_fields = Vec::<String>::new();
    if concordance.is_none() {
        missing_quality_fields.push("concordance".to_string());
    }
    if dosage_r2.is_none() {
        missing_quality_fields.push("dosage_r2".to_string());
    }
    if !maf_strata_present {
        missing_quality_fields.push("maf_strata".to_string());
    }
    if imputation_qc.pointer("/concordance/masked_truth_site_count").is_none() {
        missing_quality_fields.push("masked_truth_sites".to_string());
    }
    let status = if missing_quality_fields.is_empty() {
        "complete"
    } else {
        "explicit_missing_quality_fields"
    };

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.imputation.v1",
        "stage_id": "vcf.imputation",
        "tool_id": tool_id,
        "status": status,
        "mean_info_score": mean_info_score,
        "r2_available": dosage_r2.is_some(),
        "low_confidence_sites": low_confidence_sites,
        "masked_truth_sites": masked_truth_sites,
        "missing_quality_fields": missing_quality_fields,
        "concordance": concordance,
        "dosage_r2": dosage_r2,
        "variant_count": output.variant_count,
        "sample_count": output.sample_ids.len(),
        "sample_ids": output.sample_ids,
    }))
}

fn parse_impute_metrics(tool_id: &str, root: &Path) -> Result<serde_json::Value> {
    validate_command(tool_id, "vcf.impute", &read_json(&root.join(RAW_COMMAND_NAME))?)?;

    let imputation_qc = read_json(&root.join(RAW_IMPUTATION_QC_NAME))?;
    validate_backend_field(tool_id, &imputation_qc, "imputation qc")?;
    let manifest = read_json(&root.join(RAW_IMPUTATION_MANIFEST_NAME))?;
    validate_manifest(tool_id, "vcf.impute", &manifest)?;
    validate_accept_report(&read_json(&root.join(RAW_IMPUTATION_ACCEPT_NAME))?)?;

    let output = parse_vcf(&root.join(RAW_IMPUTED_VCF_NAME))?;
    let truth = compare_truth_if_present(root, &output)?;
    validate_truth_concordance(&imputation_qc, truth.as_ref())?;

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.impute.v1",
        "stage_id": "vcf.impute",
        "tool_id": tool_id,
        "variant_count": output.variant_count,
        "missing_before": json_u64(&imputation_qc, "/missing_genotypes_before", "missing_genotypes_before")?,
        "missing_after": json_u64(&imputation_qc, "/missing_genotypes_after", "missing_genotypes_after")?,
        "imputed_genotypes": json_u64(&imputation_qc, "/imputed_genotypes", "imputed_genotypes")?,
        "low_confidence_count": json_u64(&imputation_qc, "/low_confidence_count", "low_confidence_count")?,
        "masked_truth_site_count": json_u64(&imputation_qc, "/concordance/masked_truth_site_count", "masked_truth_site_count")?,
        "masked_truth_match_count": json_u64(&imputation_qc, "/concordance/imputed_match_count", "imputed_match_count")?,
        "unresolved_count": json_u64(&imputation_qc, "/concordance/unresolved_count", "unresolved_count")?,
        "not_imputable_reasons": json_object(&imputation_qc, "/not_imputable_reasons", "not_imputable_reasons")?,
        "sample_count": output.sample_ids.len(),
        "sample_ids": output.sample_ids,
    }))
}

fn validate_command(tool_id: &str, stage_id: &str, command: &serde_json::Value) -> Result<()> {
    let declared_tool_id = json_string(command, "/tool_id", "tool_id")?;
    if declared_tool_id != tool_id {
        bail!("imputation parser expected tool_id `{tool_id}`, found `{declared_tool_id}`");
    }
    let declared_stage_id = json_string(command, "/stage_id", "stage_id")?;
    if declared_stage_id != stage_id {
        bail!("imputation parser expected stage_id `{stage_id}`, found `{declared_stage_id}`");
    }
    let argv = json_string_array(command, "/argv", "argv")?;
    let joined = argv.join(" ");
    for token in required_command_tokens(tool_id)? {
        if !joined.contains(token) {
            bail!("imputation command for `{tool_id}` is missing `{token}`");
        }
    }
    Ok(())
}

fn validate_backend_field(tool_id: &str, value: &serde_json::Value, surface: &str) -> Result<()> {
    let backend = json_string(value, "/backend", "backend")?;
    if backend != tool_id {
        bail!("{surface} drifted away from backend `{tool_id}`: found `{backend}`");
    }
    Ok(())
}

fn validate_manifest(tool_id: &str, stage_id: &str, manifest: &serde_json::Value) -> Result<()> {
    validate_backend_field(tool_id, manifest, "imputation manifest")?;
    let declared_stage_id = json_string(manifest, "/stage_id", "stage_id")?;
    if declared_stage_id != stage_id {
        bail!("imputation manifest expected stage_id `{stage_id}`, found `{declared_stage_id}`");
    }
    let argv = json_string_array(manifest, "/command_argv", "command_argv")?;
    let joined = argv.join(" ");
    for token in required_command_tokens(tool_id)? {
        if !joined.contains(token) {
            bail!("imputation manifest for `{tool_id}` is missing `{token}`");
        }
    }
    Ok(())
}

fn validate_orchestration_manifest(tool_id: &str, manifest: &serde_json::Value) -> Result<()> {
    validate_backend_field(tool_id, manifest, "orchestration manifest")?;
    let stage_id = json_string(manifest, "/stage_id", "stage_id")?;
    if stage_id != "vcf.imputation" {
        bail!("orchestration manifest stage drifted: found `{stage_id}`");
    }
    let status = json_string(manifest, "/status", "status")?;
    if status.trim().is_empty() {
        bail!("orchestration manifest status must not be empty");
    }
    Ok(())
}

fn validate_accept_report(report: &serde_json::Value) -> Result<()> {
    if report.get("accepted").and_then(serde_json::Value::as_bool).is_none() {
        bail!("imputation accept report is missing boolean `accepted`");
    }
    if report.get("status").and_then(serde_json::Value::as_str).is_none() {
        bail!("imputation accept report is missing string `status`");
    }
    Ok(())
}

fn required_command_tokens(tool_id: &str) -> Result<&'static [&'static str]> {
    match tool_id {
        "beagle" => Ok(&["beagle", "gt=", "ref=", "map=", "impute=true"]),
        "glimpse" => Ok(&[
            "GLIMPSE_phase",
            "--reference",
            "--map",
            "--input-region",
            "--output-region",
            "--output",
        ]),
        "impute5" => Ok(&["impute5", "--g", "--h", "--m", "--r", "--o"]),
        "minimac4" => Ok(&["minimac4", "--refHaps", "--haps", "--prefix"]),
        _ => bail!("unsupported imputation tool `{tool_id}`"),
    }
}

fn compare_truth_if_present(root: &Path, output: &ParsedVcf) -> Result<Option<TruthComparison>> {
    let truth_path = root.join(RAW_TRUTH_VCF_NAME);
    if !truth_path.exists() {
        return Ok(None);
    }
    let truth = parse_vcf(&truth_path)?;
    let output_sample_index = output
        .sample_ids
        .iter()
        .enumerate()
        .map(|(idx, sample_id)| (sample_id.clone(), idx))
        .collect::<BTreeMap<_, _>>();
    let truth_sample_index = truth
        .sample_ids
        .iter()
        .enumerate()
        .map(|(idx, sample_id)| (sample_id.clone(), idx))
        .collect::<BTreeMap<_, _>>();

    let mut compared = 0_u64;
    let mut matches = 0_u64;
    let mut unresolved = 0_u64;

    for (variant_key, truth_genotypes) in &truth.genotypes_by_key {
        let Some(output_genotypes) = output.genotypes_by_key.get(variant_key) else {
            unresolved +=
                truth_genotypes.iter().filter(|genotype| !is_missing_gt(genotype)).count() as u64;
            continue;
        };
        for (sample_id, truth_idx) in &truth_sample_index {
            let truth_gt = truth_genotypes.get(*truth_idx).ok_or_else(|| {
                anyhow!("truth VCF row `{variant_key}` is missing genotype for `{sample_id}`")
            })?;
            if is_missing_gt(truth_gt) {
                continue;
            }
            compared += 1;
            let Some(output_idx) = output_sample_index.get(sample_id) else {
                unresolved += 1;
                continue;
            };
            let output_gt = output_genotypes.get(*output_idx).ok_or_else(|| {
                anyhow!("imputed VCF row `{variant_key}` is missing genotype for `{sample_id}`")
            })?;
            if is_missing_gt(output_gt) {
                unresolved += 1;
                continue;
            }
            if canonicalize_gt(output_gt) == canonicalize_gt(truth_gt) {
                matches += 1;
            }
        }
    }

    Ok(Some(TruthComparison {
        masked_truth_site_count: compared,
        imputed_match_count: matches,
        unresolved_count: unresolved,
    }))
}

fn validate_truth_concordance(
    imputation_qc: &serde_json::Value,
    truth: Option<&TruthComparison>,
) -> Result<()> {
    let truth_provided = imputation_qc
        .pointer("/concordance/truth_provided")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    match truth {
        Some(truth) => {
            if !truth_provided {
                bail!("imputation QC drifted away from provided truth evidence");
            }
            let qc_truth_sites = json_u64(
                imputation_qc,
                "/concordance/masked_truth_site_count",
                "masked_truth_site_count",
            )?;
            if qc_truth_sites != truth.masked_truth_site_count {
                bail!(
                    "masked truth site count drifted: qc={qc_truth_sites}, parsed={}",
                    truth.masked_truth_site_count
                );
            }
            let qc_matches =
                json_u64(imputation_qc, "/concordance/imputed_match_count", "imputed_match_count")?;
            if qc_matches != truth.imputed_match_count {
                bail!(
                    "masked truth match count drifted: qc={qc_matches}, parsed={}",
                    truth.imputed_match_count
                );
            }
            let qc_unresolved =
                json_u64(imputation_qc, "/concordance/unresolved_count", "unresolved_count")?;
            if qc_unresolved != truth.unresolved_count {
                bail!(
                    "masked truth unresolved count drifted: qc={qc_unresolved}, parsed={}",
                    truth.unresolved_count
                );
            }
            if let Some(qc_concordance) = imputation_qc
                .pointer("/concordance/genotype_concordance")
                .and_then(serde_json::Value::as_f64)
            {
                let parsed_concordance = if truth.masked_truth_site_count == 0 {
                    0.0
                } else {
                    truth.imputed_match_count as f64 / truth.masked_truth_site_count as f64
                };
                if (qc_concordance - parsed_concordance).abs() > 1e-9 {
                    bail!(
                        "masked truth concordance drifted: qc={qc_concordance}, parsed={parsed_concordance}"
                    );
                }
            }
        }
        None => {
            if truth_provided {
                bail!("imputation QC declares truth evidence but fixture does not provide raw.truth.vcf");
            }
        }
    }
    Ok(())
}

fn parse_vcf(path: &Path) -> Result<ParsedVcf> {
    let raw = read_text(path)?;
    let header = raw
        .lines()
        .find(|line| line.starts_with("#CHROM\t"))
        .ok_or_else(|| anyhow!("VCF {} is missing the #CHROM header", path.display()))?;
    let sample_ids = header.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>();
    if sample_ids.is_empty() {
        bail!("VCF {} does not declare any samples", path.display());
    }

    let mut variant_count = 0_u64;
    let mut genotypes_by_key = BTreeMap::<String, Vec<String>>::new();
    for line in raw.lines().filter(|line| !line.starts_with('#') && !line.trim().is_empty()) {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 10 {
            bail!("VCF row in {} is missing sample columns", path.display());
        }
        variant_count += 1;
        let key = format!("{}:{}:{}:{}", fields[0], fields[1], fields[3], fields[4]);
        if genotypes_by_key.contains_key(&key) {
            bail!("VCF {} contains duplicated variant key `{key}`", path.display());
        }
        let format_keys = fields[8].split(':').collect::<Vec<_>>();
        let gt_index = format_keys.iter().position(|field| *field == "GT").unwrap_or(0);
        let genotypes = fields
            .iter()
            .skip(9)
            .map(|sample_field| {
                let values = sample_field.split(':').collect::<Vec<_>>();
                values.get(gt_index).copied().unwrap_or(".").to_string()
            })
            .collect::<Vec<_>>();
        genotypes_by_key.insert(key, genotypes);
    }

    Ok(ParsedVcf { sample_ids, variant_count, genotypes_by_key })
}

fn canonicalize_gt(genotype: &str) -> String {
    let mut alleles = genotype.replace('|', "/").split('/').map(str::to_string).collect::<Vec<_>>();
    alleles.sort();
    alleles.join("/")
}

fn is_missing_gt(genotype: &str) -> bool {
    matches!(genotype, "." | "./." | ".|.")
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = read_text(path)?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn read_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}

fn json_string(value: &serde_json::Value, pointer: &str, field: &str) -> Result<String> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("missing string `{field}`"))
}

fn json_string_array(value: &serde_json::Value, pointer: &str, field: &str) -> Result<Vec<String>> {
    let rows = value
        .pointer(pointer)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("missing string array `{field}`"))?;
    rows.iter()
        .map(|row| {
            row.as_str()
                .map(str::to_string)
                .ok_or_else(|| anyhow!("string array `{field}` contains non-string rows"))
        })
        .collect()
}

fn json_object(
    value: &serde_json::Value,
    pointer: &str,
    field: &str,
) -> Result<serde_json::Map<String, serde_json::Value>> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_object)
        .cloned()
        .ok_or_else(|| anyhow!("missing object `{field}`"))
}

fn json_u64(value: &serde_json::Value, pointer: &str, field: &str) -> Result<u64> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("missing integer `{field}`"))
}

fn json_f64(value: &serde_json::Value, pointer: &str, field: &str) -> Result<f64> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("missing number `{field}`"))
}
