use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use flate2::read::MultiGzDecoder;

use crate::taxonomy::VcfDomainStage;

const BCFTOOLS_TOOL_ID: &str = "bcftools";

const RAW_CALL_VCF_NAME: &str = "raw.calls.vcf";
const RAW_DIPLOID_VCF_NAME: &str = "raw.diploid.vcf";
const RAW_GL_VCF_NAME: &str = "raw.gl.vcf";
const RAW_PSEUDOHAPLOID_VCF_NAME: &str = "raw.pseudohaploid.vcf";
const RAW_PSEUDOHAPLOID_COMMAND_NAME: &str = "raw.command.json";
const RAW_DAMAGE_FILTER_VCF_NAME: &str = "raw.damage_filtered.vcf";
const RAW_DAMAGE_FILTER_SUMMARY_NAME: &str = "raw.damage_filter_summary.json";
const RAW_DAMAGE_FILTER_COUNTS_NAME: &str = "raw.damage_filter_counts.json";
const RAW_FILTER_VCF_NAME: &str = "raw.filtered.vcf";
const RAW_FILTER_EXPLAIN_NAME: &str = "raw.filter_explain.json";
const RAW_GL_PROPAGATION_INPUT_NAME: &str = "raw.input.vcf";
const RAW_GL_PROPAGATION_OUTPUT_NAME: &str = "raw.propagated.vcf";
const RAW_POSTPROCESS_TBI_NAME: &str = "raw.postprocess.vcf.tbi";
const RAW_POSTPROCESS_VALIDATE_NAME: &str = "raw.validate_outputs.json";
const RAW_POSTPROCESS_MANIFEST_NAME: &str = "raw.final_manifest.json";
const RAW_PANEL_VCF_NAME: &str = "raw.panel.vcf";
const RAW_PANEL_TBI_NAME: &str = "raw.panel.vcf.tbi";
const RAW_PANEL_MANIFEST_NAME: &str = "raw.panel_manifest.json";
const RAW_STATS_NAME: &str = "raw.bcftools_stats.txt";

#[derive(Debug, Clone)]
struct ParsedVcfRecord {
    reference: String,
    alternates: Vec<String>,
    format: Option<String>,
    samples: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct ParsedVcfDocument {
    sample_ids: Vec<String>,
    records: Vec<ParsedVcfRecord>,
}

#[derive(Debug, Clone)]
struct LikelihoodSummary {
    likelihood_field: String,
    sites_with_likelihoods: u64,
    samples_with_likelihoods: u64,
    missing_likelihoods: u64,
}

#[derive(Debug, Clone)]
struct LikelihoodFieldSetSummary {
    fields: BTreeSet<String>,
    site_count: u64,
}

#[derive(Debug, Clone)]
struct FilteredOutputSummary {
    pass_variants: u64,
    failed_variants: u64,
    filter_ids: Vec<String>,
}

/// Normalize the governed raw `bcftools` artifact set for a retained VCF stage.
///
/// # Errors
/// Returns an error when required raw artifacts are missing or malformed.
pub fn parse_bcftools_stage_metrics(
    stage: VcfDomainStage,
    artifact_root: &Path,
) -> Result<serde_json::Value> {
    match stage {
        VcfDomainStage::Call => parse_call_metrics(artifact_root),
        VcfDomainStage::CallDiploid => parse_call_diploid_metrics(artifact_root),
        VcfDomainStage::CallGl => parse_call_gl_metrics(artifact_root),
        VcfDomainStage::CallPseudohaploid => parse_call_pseudohaploid_metrics(artifact_root),
        VcfDomainStage::DamageFilter => parse_damage_filter_metrics(artifact_root),
        VcfDomainStage::Filter => parse_filter_metrics(artifact_root),
        VcfDomainStage::GlPropagation => parse_gl_propagation_metrics(artifact_root),
        VcfDomainStage::Postprocess => parse_postprocess_metrics(artifact_root),
        VcfDomainStage::PrepareReferencePanel => {
            parse_prepare_reference_panel_metrics(artifact_root)
        }
        VcfDomainStage::Stats => parse_stats_metrics(artifact_root),
        other => bail!("unsupported bcftools VCF parser stage `{}`", other.as_str()),
    }
}

fn parse_call_metrics(root: &Path) -> Result<serde_json::Value> {
    let doc = parse_vcf_document(&root.join(RAW_CALL_VCF_NAME))?;
    let (snp_count, indel_count) = count_snp_and_indel_records(&doc);
    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.call.v1",
        "stage_id": "vcf.call",
        "tool_id": BCFTOOLS_TOOL_ID,
        "variant_count": doc.records.len(),
        "snp_count": snp_count,
        "indel_count": indel_count,
        "sample_count": doc.sample_ids.len(),
    }))
}

fn parse_call_diploid_metrics(root: &Path) -> Result<serde_json::Value> {
    let doc = parse_vcf_document(&root.join(RAW_DIPLOID_VCF_NAME))?;
    let mut called_genotypes = 0_u64;
    let mut heterozygous_count = 0_u64;
    let mut homozygous_ref_count = 0_u64;
    let mut homozygous_alt_count = 0_u64;
    let mut missing_count = 0_u64;

    for record in &doc.records {
        let format = record
            .format
            .as_deref()
            .ok_or_else(|| anyhow!("diploid call raw VCF row is missing FORMAT"))?;
        for sample in &record.samples {
            let gt = extract_sample_field(format, sample, "GT")?;
            if gt.contains('.') {
                missing_count += 1;
                continue;
            }
            let alleles = split_genotype(gt)?;
            if alleles.len() != 2 {
                bail!("diploid call genotype is not diploid-compatible: `{gt}`");
            }
            called_genotypes += 1;
            match (alleles[0], alleles[1]) {
                (0, 0) => homozygous_ref_count += 1,
                (left, right) if left == right => homozygous_alt_count += 1,
                _ => heterozygous_count += 1,
            }
        }
    }

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.call_diploid.v1",
        "stage_id": "vcf.call_diploid",
        "tool_id": BCFTOOLS_TOOL_ID,
        "ploidy": "diploid",
        "called_genotypes": called_genotypes,
        "heterozygous_count": heterozygous_count,
        "homozygous_ref_count": homozygous_ref_count,
        "homozygous_alt_count": homozygous_alt_count,
        "missing_count": missing_count,
        "sample_count": doc.sample_ids.len(),
    }))
}

fn parse_call_gl_metrics(root: &Path) -> Result<serde_json::Value> {
    let summary = summarize_likelihood_values(&root.join(RAW_GL_VCF_NAME))?;
    let sample_count = parse_vcf_document(&root.join(RAW_GL_VCF_NAME))?.sample_ids.len();
    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.call_gl.v1",
        "stage_id": "vcf.call_gl",
        "tool_id": BCFTOOLS_TOOL_ID,
        "likelihood_field": summary.likelihood_field,
        "sites_with_likelihoods": summary.sites_with_likelihoods,
        "samples_with_likelihoods": summary.samples_with_likelihoods,
        "missing_likelihoods": summary.missing_likelihoods,
        "sample_count": sample_count,
    }))
}

fn parse_call_pseudohaploid_metrics(root: &Path) -> Result<serde_json::Value> {
    let doc = parse_vcf_document(&root.join(RAW_PSEUDOHAPLOID_VCF_NAME))?;
    let command = read_json(&root.join(RAW_PSEUDOHAPLOID_COMMAND_NAME))?;
    let sampling_policy = json_string(&command, "/sampling_policy", "sampling_policy")?;
    let random_seed = json_u64(&command, "/random_seed", "random_seed")?;

    let mut called_sites = 0_u64;
    let mut missing_sites = 0_u64;
    for record in &doc.records {
        let format = record
            .format
            .as_deref()
            .ok_or_else(|| anyhow!("pseudohaploid raw VCF row is missing FORMAT"))?;
        for sample in &record.samples {
            let gt = extract_sample_field(format, sample, "GT")?;
            if gt == "." {
                missing_sites += 1;
                continue;
            }
            let alleles = split_genotype(gt)?;
            if alleles.len() != 1 {
                bail!("pseudohaploid genotype is not haploid-compatible: `{gt}`");
            }
            called_sites += 1;
        }
    }
    let target_sites = u64::try_from(doc.records.len())
        .map_err(|_| anyhow!("pseudohaploid target site overflow"))?;

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.call_pseudohaploid.v1",
        "stage_id": "vcf.call_pseudohaploid",
        "tool_id": BCFTOOLS_TOOL_ID,
        "target_sites": target_sites,
        "covered_sites": target_sites,
        "called_sites": called_sites,
        "missing_sites": missing_sites,
        "sampling_policy": sampling_policy,
        "random_seed": random_seed,
        "sample_count": doc.sample_ids.len(),
    }))
}

fn parse_damage_filter_metrics(root: &Path) -> Result<serde_json::Value> {
    let retained_doc = parse_vcf_document(&root.join(RAW_DAMAGE_FILTER_VCF_NAME))?;
    let summary = read_json(&root.join(RAW_DAMAGE_FILTER_SUMMARY_NAME))?;
    let counts = read_json(&root.join(RAW_DAMAGE_FILTER_COUNTS_NAME))?;

    let input_variants = json_u64(&summary, "/prefilter/records_total", "prefilter.records_total")?;
    let retained_variants = u64::try_from(retained_doc.records.len())
        .map_err(|_| anyhow!("damage_filter retained variant overflow"))?;
    let removed_variants = input_variants.saturating_sub(retained_variants);
    let low_quality_filtered_variants = json_u64(&counts, "/counts/low_qual", "counts.low_qual")?;
    let damage_ratio_filtered_variants =
        json_u64(&counts, "/counts/damage_ratio_exceeded", "counts.damage_ratio_exceeded")?;
    let terminal_damage_filtered_variants =
        json_u64(&counts, "/counts/terminal_damage_filtered", "counts.terminal_damage_filtered")?;
    let terminal_context_count = summary
        .pointer("/prefilter/read_position_signal/ct_five_prime_high")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
        + summary
            .pointer("/prefilter/read_position_signal/ga_three_prime_high")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.damage_filter.v1",
        "stage_id": "vcf.damage_filter",
        "tool_id": BCFTOOLS_TOOL_ID,
        "input_variants": input_variants,
        "removed_variants": removed_variants,
        "retained_variants": retained_variants,
        "low_quality_filtered_variants": low_quality_filtered_variants,
        "damage_ratio_filtered_variants": damage_ratio_filtered_variants,
        "terminal_damage_filtered_variants": terminal_damage_filtered_variants,
        "damage_context_rule": format_damage_context_rule(&summary)?,
        "terminal_context_count": terminal_context_count,
        "sample_count": retained_doc.sample_ids.len(),
    }))
}

fn parse_filter_metrics(root: &Path) -> Result<serde_json::Value> {
    let output = summarize_filtered_output(&root.join(RAW_FILTER_VCF_NAME))?;
    let doc = parse_vcf_document(&root.join(RAW_FILTER_VCF_NAME))?;
    let report = read_json(&root.join(RAW_FILTER_EXPLAIN_NAME))?;
    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.filter.v1",
        "stage_id": "vcf.filter",
        "tool_id": BCFTOOLS_TOOL_ID,
        "input_variants": output.pass_variants + output.failed_variants,
        "pass_variants": output.pass_variants,
        "failed_variants": output.failed_variants,
        "filter_ids": output.filter_ids,
        "depth_threshold": json_f64(&report, "/thresholds/min_depth", "thresholds.min_depth")?,
        "quality_threshold": json_f64(&report, "/thresholds/min_qual", "thresholds.min_qual")?,
        "missingness_threshold": json_f64(
            &report,
            "/thresholds/sample_missingness_max",
            "thresholds.sample_missingness_max",
        )?,
        "sample_count": doc.sample_ids.len(),
    }))
}

fn parse_gl_propagation_metrics(root: &Path) -> Result<serde_json::Value> {
    let input = summarize_likelihood_field_set(&root.join(RAW_GL_PROPAGATION_INPUT_NAME))?;
    let output = summarize_likelihood_field_set(&root.join(RAW_GL_PROPAGATION_OUTPUT_NAME))?;
    let lost_fields = input.fields.difference(&output.fields).cloned().collect::<Vec<_>>();
    let sample_count =
        parse_vcf_document(&root.join(RAW_GL_PROPAGATION_OUTPUT_NAME))?.sample_ids.len();
    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.gl_propagation.v1",
        "stage_id": "vcf.gl_propagation",
        "tool_id": BCFTOOLS_TOOL_ID,
        "input_likelihood_fields": input.fields.into_iter().collect::<Vec<_>>(),
        "output_likelihood_fields": output.fields.into_iter().collect::<Vec<_>>(),
        "lost_fields": lost_fields,
        "site_count_before": input.site_count,
        "site_count_after": output.site_count,
        "sample_count": sample_count,
    }))
}

fn parse_postprocess_metrics(root: &Path) -> Result<serde_json::Value> {
    let validate = read_json(&root.join(RAW_POSTPROCESS_VALIDATE_NAME))?;
    let manifest = read_json(&root.join(RAW_POSTPROCESS_MANIFEST_NAME))?;
    let readable_vcf = json_bool(&validate, "/readable_vcf", "readable_vcf")?;
    let tabix_present = root.join(RAW_POSTPROCESS_TBI_NAME).exists()
        && json_bool(&validate, "/tabix_present", "tabix_present")?;
    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.postprocess.v1",
        "stage_id": "vcf.postprocess",
        "tool_id": BCFTOOLS_TOOL_ID,
        "readable_vcf": readable_vcf,
        "tabix_present": tabix_present,
        "contigs_consistent_with_species_context": json_bool(
            &validate,
            "/contigs_consistent_with_species_context",
            "contigs_consistent_with_species_context",
        )?,
        "left_align_applied": json_bool(
            &manifest,
            "/normalization/left_align_applied",
            "normalization.left_align_applied",
        )?,
        "multiallelic_records_split": json_u64(
            &manifest,
            "/normalization/multiallelic_records_split",
            "normalization.multiallelic_records_split",
        )?,
        "indels_normalized": json_u64(
            &manifest,
            "/normalization/indels_normalized",
            "normalization.indels_normalized",
        )?,
        "variant_ids_normalized": json_u64(
            &manifest,
            "/normalization/variant_ids_normalized",
            "normalization.variant_ids_normalized",
        )?,
        "invalid_records_removed": json_u64(
            &manifest,
            "/normalization/invalid_records_removed",
            "normalization.invalid_records_removed",
        )?,
        "filter_standardized_to_pass": json_u64(
            &manifest,
            "/normalization/filter_standardized_to_pass",
            "normalization.filter_standardized_to_pass",
        )?,
    }))
}

fn parse_prepare_reference_panel_metrics(root: &Path) -> Result<serde_json::Value> {
    let manifest = read_json(&root.join(RAW_PANEL_MANIFEST_NAME))?;
    let sample_ids = parse_vcf_document(&root.join(RAW_PANEL_VCF_NAME))?.sample_ids;
    let manifest_sample_ids =
        json_string_array(&manifest, "/normalization/sample_ids", "normalization.sample_ids")?;
    let manifest_sample_count =
        json_u64(&manifest, "/normalization/sample_count", "normalization.sample_count")?;
    let observed_sample_count = u64::try_from(sample_ids.len())
        .map_err(|_| anyhow!("prepared panel sample count overflow"))?;
    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.prepare_reference_panel.v1",
        "stage_id": "vcf.prepare_reference_panel",
        "tool_id": BCFTOOLS_TOOL_ID,
        "input_variants": json_u64(
            &manifest,
            "/normalization/input_variant_count",
            "normalization.input_variant_count",
        )?,
        "output_variants": json_u64(
            &manifest,
            "/normalization/output_variant_count",
            "normalization.output_variant_count",
        )?,
        "sample_count": observed_sample_count,
        "sample_ids": sample_ids,
        "sample_consistent": manifest_sample_count == observed_sample_count
            && manifest_sample_ids == parse_vcf_document(&root.join(RAW_PANEL_VCF_NAME))?.sample_ids,
        "duplicate_sites_removed": json_u64(
            &manifest,
            "/normalization/duplicate_sites_removed",
            "normalization.duplicate_sites_removed",
        )?,
        "normalization_status": json_string(
            &manifest,
            "/normalization/status",
            "normalization.status",
        )?,
        "parseable": root.join(RAW_PANEL_TBI_NAME).exists(),
    }))
}

fn parse_stats_metrics(root: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(root.join(RAW_STATS_NAME))
        .with_context(|| format!("read {}", root.join(RAW_STATS_NAME).display()))?;
    let mut variant_count = None::<u64>;
    let mut snp_count = None::<u64>;
    let mut indel_count = None::<u64>;
    let mut transition_count = None::<u64>;
    let mut transversion_count = None::<u64>;
    let mut ti_tv = None::<f64>;
    let mut sample_count = None::<u64>;

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('\t') else {
            continue;
        };
        match key {
            "variants_total" | "variant_count" => variant_count = value.parse::<u64>().ok(),
            "snps" | "snp_count" => snp_count = value.parse::<u64>().ok(),
            "indels" | "indel_count" => indel_count = value.parse::<u64>().ok(),
            "transitions" | "transition_count" => transition_count = value.parse::<u64>().ok(),
            "transversions" | "transversion_count" => {
                transversion_count = value.parse::<u64>().ok()
            }
            "ti_tv" => ti_tv = value.parse::<f64>().ok(),
            "sample_count" => sample_count = value.parse::<u64>().ok(),
            _ => {}
        }
    }

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.stats.v1",
        "stage_id": "vcf.stats",
        "tool_id": BCFTOOLS_TOOL_ID,
        "variant_count": variant_count.ok_or_else(|| anyhow!("stats raw output is missing variant_count"))?,
        "snp_count": snp_count.ok_or_else(|| anyhow!("stats raw output is missing snp_count"))?,
        "indel_count": indel_count.ok_or_else(|| anyhow!("stats raw output is missing indel_count"))?,
        "transition_count": transition_count.ok_or_else(|| anyhow!("stats raw output is missing transition_count"))?,
        "transversion_count": transversion_count.ok_or_else(|| anyhow!("stats raw output is missing transversion_count"))?,
        "ti_tv": ti_tv.ok_or_else(|| anyhow!("stats raw output is missing ti_tv"))?,
        "sample_count": sample_count.ok_or_else(|| anyhow!("stats raw output is missing sample_count"))?,
    }))
}

fn parse_vcf_document(path: &Path) -> Result<ParsedVcfDocument> {
    let raw = read_text(path)?;
    let mut doc = ParsedVcfDocument::default();
    let mut saw_header = false;
    for (line_index, raw_line) in raw.lines().enumerate() {
        let line = raw_line.trim_end();
        if line.is_empty() || line.starts_with("##") {
            continue;
        }
        if let Some(header) = line.strip_prefix("#CHROM\t") {
            saw_header = true;
            let fields = header.split('\t').collect::<Vec<_>>();
            if fields.len() >= 9 {
                doc.sample_ids = fields[8..].iter().map(|value| (*value).to_string()).collect();
            }
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 8 {
            bail!(
                "malformed raw VCF record at line {}: expected at least 8 fields",
                line_index + 1
            );
        }
        let alternates = fields[4]
            .split(',')
            .filter(|allele| !allele.trim().is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        let format =
            if fields.len() >= 9 && fields[8] != "." { Some(fields[8].to_string()) } else { None };
        let samples = if fields.len() >= 10 {
            fields[9..].iter().map(|value| (*value).to_string()).collect()
        } else {
            Vec::new()
        };
        doc.records.push(ParsedVcfRecord {
            reference: fields[3].to_string(),
            alternates,
            format,
            samples,
        });
    }
    if !saw_header {
        bail!("raw VCF is missing #CHROM header");
    }
    Ok(doc)
}

fn count_snp_and_indel_records(doc: &ParsedVcfDocument) -> (u64, u64) {
    let mut snp_count = 0_u64;
    let mut indel_count = 0_u64;
    for record in &doc.records {
        let is_snp =
            record.alternates.iter().all(|alt| record.reference.len() == 1 && alt.len() == 1);
        if is_snp {
            snp_count += 1;
        } else {
            indel_count += 1;
        }
    }
    (snp_count, indel_count)
}

fn summarize_likelihood_values(path: &Path) -> Result<LikelihoodSummary> {
    let raw = read_text(path)?;
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
            bail!("GL raw VCF row is missing FORMAT/sample fields: {line}");
        }
        let format_tokens = fields[8].split(':').collect::<Vec<_>>();
        let field_name = ["GL", "GP", "PL"]
            .into_iter()
            .find(|candidate| format_tokens.iter().any(|token| token == candidate))
            .ok_or_else(|| anyhow!("GL raw VCF row is missing GL/GP/PL in FORMAT: {line}"))?;
        let field_index = format_tokens
            .iter()
            .position(|token| *token == field_name)
            .ok_or_else(|| anyhow!("GL raw VCF row lost {field_name} in FORMAT: {line}"))?;

        if let Some(previous) = &likelihood_field {
            if previous != field_name {
                bail!("GL likelihood field drifted across rows: `{previous}` vs `{field_name}`");
            }
        } else {
            likelihood_field = Some(field_name.to_string());
        }

        let mut row_has_likelihood = false;
        for (sample_index, sample_field) in fields[9..].iter().enumerate() {
            let sample_value = sample_field.split(':').nth(field_index).ok_or_else(|| {
                anyhow!("GL raw sample field is missing {field_name} value: {line}")
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
            .ok_or_else(|| anyhow!("GL raw VCF did not contain any GL/GP/PL fields"))?,
        sites_with_likelihoods,
        samples_with_likelihoods: u64::try_from(samples_with_likelihoods.len())
            .map_err(|_| anyhow!("GL sample-with-likelihood count overflowed u64"))?,
        missing_likelihoods,
    })
}

fn summarize_likelihood_field_set(path: &Path) -> Result<LikelihoodFieldSetSummary> {
    let raw = read_text(path)?;
    let mut fields = BTreeSet::<String>::new();
    let mut site_count = 0_u64;
    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 10 {
            bail!("gl_propagation raw VCF row is missing FORMAT/sample fields: {line}");
        }
        site_count += 1;
        for token in parts[8].split(':') {
            if ["GL", "GP", "PL"].contains(&token) {
                fields.insert(token.to_string());
            }
        }
    }
    Ok(LikelihoodFieldSetSummary { fields, site_count })
}

fn summarize_filtered_output(path: &Path) -> Result<FilteredOutputSummary> {
    let raw = read_text(path)?;
    let mut pass_variants = 0_u64;
    let mut failed_variants = 0_u64;
    let mut filter_ids = BTreeSet::<String>::new();
    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 8 {
            bail!("filter raw VCF row is missing FILTER field: {line}");
        }
        let filter_field = parts[6];
        if filter_field == "PASS" || filter_field == "." {
            pass_variants += 1;
            continue;
        }
        failed_variants += 1;
        for tag in filter_field.split(';') {
            if !tag.is_empty() && tag != "PASS" && tag != "." {
                filter_ids.insert(tag.to_string());
            }
        }
    }
    Ok(FilteredOutputSummary {
        pass_variants,
        failed_variants,
        filter_ids: filter_ids.into_iter().collect(),
    })
}

fn extract_sample_field<'a>(format: &'a str, sample: &'a str, field_name: &str) -> Result<&'a str> {
    let keys = format.split(':').collect::<Vec<_>>();
    let index = keys
        .iter()
        .position(|token| *token == field_name)
        .ok_or_else(|| anyhow!("FORMAT field is missing `{field_name}`"))?;
    sample
        .split(':')
        .nth(index)
        .ok_or_else(|| anyhow!("sample payload is missing `{field_name}` value"))
}

fn split_genotype(genotype: &str) -> Result<Vec<u32>> {
    genotype
        .split(['/', '|'])
        .map(|allele| {
            allele
                .parse::<u32>()
                .with_context(|| format!("genotype allele is not numeric: `{genotype}`"))
        })
        .collect()
}

fn likelihood_value_is_missing(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed.split(',').all(|token| matches!(token.trim(), "." | ""))
}

fn format_damage_context_rule(summary: &serde_json::Value) -> Result<String> {
    let mode = json_string(&summary, "/masking_strategy/mode", "masking_strategy.mode")?;
    let max_damage_ratio =
        json_f64(summary, "/thresholds/max_damage_ratio", "thresholds.max_damage_ratio")?;
    let terminal_threshold = json_f64(
        summary,
        "/thresholds/terminal_damage_threshold",
        "thresholds.terminal_damage_threshold",
    )?;
    let pmd_min = json_f64(summary, "/thresholds/pmd_min", "thresholds.pmd_min")?;
    Ok(format!(
        "{mode}_ct_ga_with_ratio_gt_{max_damage_ratio:.2}_or_terminal_signal_ge_{terminal_threshold:.2}_or_pmd_lt_{pmd_min:.1}"
    ))
}

fn read_text(path: &Path) -> Result<String> {
    let mut buffer = String::new();
    if path.extension().and_then(|value| value.to_str()) == Some("gz") {
        let file = fs::File::open(path).with_context(|| format!("read {}", path.display()))?;
        let mut decoder = MultiGzDecoder::new(file);
        decoder
            .read_to_string(&mut buffer)
            .with_context(|| format!("decode {}", path.display()))?;
        return Ok(buffer);
    }
    fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn json_string(value: &serde_json::Value, pointer: &str, name: &str) -> Result<String> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("raw artifact is missing `{name}`"))
}

fn json_u64(value: &serde_json::Value, pointer: &str, name: &str) -> Result<u64> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("raw artifact is missing `{name}`"))
}

fn json_f64(value: &serde_json::Value, pointer: &str, name: &str) -> Result<f64> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("raw artifact is missing `{name}`"))
}

fn json_bool(value: &serde_json::Value, pointer: &str, name: &str) -> Result<bool> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("raw artifact is missing `{name}`"))
}

fn json_string_array(value: &serde_json::Value, pointer: &str, name: &str) -> Result<Vec<String>> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("raw artifact is missing `{name}`"))?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| anyhow!("raw artifact `{name}` contains a non-string entry"))
        })
        .collect()
}
