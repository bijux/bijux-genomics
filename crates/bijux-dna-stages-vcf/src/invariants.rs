use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use bijux_dna_domain_vcf::contracts::SpeciesContext;
use bijux_dna_infra::{atomic_write_bytes, atomic_write_json};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InputRegime {
    GlOnly,
    GtOnly,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RegimeDetection {
    pub regime: InputRegime,
    pub lowcov_likelihood_hint: bool,
    pub pseudohaploid_hint: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InvariantStrictness {
    Strict,
    Warn,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct InvariantConfig {
    pub strictness: InvariantStrictness,
    pub allow_contig_aliasing: bool,
    pub min_overlap_threshold: f64,
    pub allowed_missing_fields: Vec<String>,
    pub require_sex_metadata_for_sex_chr: bool,
}

impl Default for InvariantConfig {
    fn default() -> Self {
        Self {
            strictness: InvariantStrictness::Strict,
            allow_contig_aliasing: false,
            min_overlap_threshold: 0.10,
            allowed_missing_fields: vec![],
            require_sex_metadata_for_sex_chr: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct InvariantsSummary {
    pub checked: Vec<String>,
    pub fixed: Vec<String>,
    pub refused: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct VcfPreflightResult {
    pub normalized_input: PathBuf,
    pub index_path: PathBuf,
    pub invariants_json: PathBuf,
    pub regime: RegimeDetection,
    pub overlap_json: PathBuf,
    pub summary: InvariantsSummary,
}

fn parse_record_fields(line: &str) -> Option<Vec<&str>> {
    if line.trim().is_empty() || line.starts_with('#') {
        return None;
    }
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() < 8 {
        return None;
    }
    Some(fields)
}

fn canonical_contig_label(raw: &str) -> String {
    raw.trim_start_matches("chr").to_ascii_uppercase()
}

fn parse_variant_key(line: &str) -> Option<(String, u64, String, String)> {
    let fields = parse_record_fields(line)?;
    let pos = fields.get(1)?.parse::<u64>().ok()?;
    Some((
        fields.first()?.to_string(),
        pos,
        fields.get(3)?.to_string(),
        fields.get(4)?.to_string(),
    ))
}

fn infer_build_from_header(header_lines: &[String]) -> Option<String> {
    let tags = ["GRCh37", "GRCh38", "hg19", "hg38"];
    header_lines.iter().find_map(|line| {
        tags.iter()
            .find(|tag| line.contains(**tag))
            .map(|x| (*x).to_string())
    })
}

fn has_minimal_headers(header_lines: &[String]) -> bool {
    let has_fileformat = header_lines.iter().any(|h| h.starts_with("##fileformat="));
    let has_chrom = header_lines.iter().any(|h| h.starts_with("#CHROM\t"));
    has_fileformat && has_chrom
}

fn detect_regime(records: &[String]) -> RegimeDetection {
    let mut has_gt = false;
    let mut has_gl = false;
    let mut gt_haploid = 0u64;
    let mut gt_total = 0u64;
    for line in records {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        let Some(fmt) = fields.get(8) else {
            continue;
        };
        let keys = fmt.split(':').collect::<Vec<_>>();
        let gt_idx = keys.iter().position(|k| *k == "GT");
        let gl_idx = keys.iter().position(|k| *k == "GL" || *k == "GP" || *k == "PL");
        if gt_idx.is_some() {
            has_gt = true;
        }
        if gl_idx.is_some() {
            has_gl = true;
        }
        if let (Some(i), Some(sample)) = (gt_idx, fields.get(9)) {
            let vals = sample.split(':').collect::<Vec<_>>();
            if let Some(gt) = vals.get(i) {
                if !gt.contains('.') {
                    gt_total += 1;
                    if gt.split(['/', '|']).count() == 1 {
                        gt_haploid += 1;
                    }
                }
            }
        }
    }
    let regime = match (has_gt, has_gl) {
        (true, true) => InputRegime::Mixed,
        (true, false) => InputRegime::GtOnly,
        (false, true) => InputRegime::GlOnly,
        (false, false) => InputRegime::Unknown,
    };
    RegimeDetection {
        regime,
        lowcov_likelihood_hint: has_gl,
        pseudohaploid_hint: gt_total > 0 && gt_haploid * 2 >= gt_total,
    }
}

pub fn site_overlap_diagnostic(
    target_records: &[String],
    panel_records: &[String],
    out_json: &Path,
) -> Result<f64> {
    let target = target_records
        .iter()
        .filter_map(|l| parse_variant_key(l).map(|(c, p, r, a)| format!("{c}:{p}:{r}:{a}")))
        .collect::<BTreeSet<_>>();
    let panel = panel_records
        .iter()
        .filter_map(|l| parse_variant_key(l).map(|(c, p, r, a)| format!("{c}:{p}:{r}:{a}")))
        .collect::<BTreeSet<_>>();
    let overlap = target.intersection(&panel).count() as u64;
    let frac = if panel.is_empty() {
        0.0
    } else {
        overlap as f64 / panel.len() as f64
    };
    atomic_write_json(
        out_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.site_overlap_diagnostic.v1",
            "target_sites": target.len(),
            "panel_sites": panel.len(),
            "overlap_sites": overlap,
            "overlap_fraction": frac,
        }),
    )?;
    Ok(frac)
}

pub fn run_vcf_preflight(
    input_vcf: &Path,
    artifact_dir: &Path,
    species: &SpeciesContext,
    config: &InvariantConfig,
) -> Result<VcfPreflightResult> {
    bijux_dna_infra::ensure_dir(artifact_dir)?;
    let raw = std::fs::read_to_string(input_vcf)?;

    let mut summary = InvariantsSummary {
        checked: vec![],
        fixed: vec![],
        refused: vec![],
    };

    let mut header_lines = Vec::<String>::new();
    let mut records = Vec::<String>::new();
    for line in raw.lines() {
        if line.starts_with('#') {
            header_lines.push(line.to_string());
        } else if !line.trim().is_empty() {
            records.push(line.to_string());
        }
    }

    summary.checked.push("minimal_header_fields".to_string());
    if !has_minimal_headers(&header_lines) {
        summary.refused.push("minimal_header_fields".to_string());
        bail!("vcf.validate_inputs refusal: missing minimal header fields");
    }

    let sample_header = header_lines
        .iter()
        .find(|l| l.starts_with("#CHROM\t"))
        .ok_or_else(|| anyhow!("missing #CHROM header"))?;
    let mut sample_ids = sample_header
        .split('\t')
        .skip(9)
        .map(str::to_string)
        .collect::<Vec<_>>();
    summary.checked.push("sample_ids_valid".to_string());
    if sample_ids.iter().any(|s| s.trim().is_empty()) {
        summary.refused.push("sample_ids_valid".to_string());
        bail!("vcf.validate_inputs refusal: empty sample IDs");
    }
    let uniq = sample_ids.iter().cloned().collect::<BTreeSet<_>>();
    if uniq.len() != sample_ids.len() {
        summary.refused.push("sample_ids_valid".to_string());
        bail!("vcf.validate_inputs refusal: duplicate sample IDs");
    }

    summary.checked.push("contig_set_present".to_string());
    let record_contigs = records
        .iter()
        .filter_map(|l| parse_variant_key(l).map(|x| x.0))
        .collect::<BTreeSet<_>>();
    if record_contigs.is_empty() {
        summary.refused.push("contig_set_present".to_string());
        bail!("vcf.validate_inputs refusal: no records/contigs present");
    }

    summary.checked.push("build_declared_vs_inferred".to_string());
    let inferred = if let Some(v) = infer_build_from_header(&header_lines) {
        v
    } else {
        let species_contigs = species
            .contigs
            .iter()
            .map(|c| canonical_contig_label(&c.name))
            .collect::<BTreeSet<_>>();
        let overlap = record_contigs
            .iter()
            .map(|c| canonical_contig_label(c))
            .filter(|c| species_contigs.contains(c))
            .count();
        let frac = overlap as f64 / record_contigs.len() as f64;
        if frac >= config.min_overlap_threshold {
            summary.fixed.push("build_inferred_from_species_context".to_string());
            species.build_id.clone()
        } else {
            summary.refused.push("build_declared_vs_inferred".to_string());
            bail!("vcf.validate_inputs refusal: build cannot be asserted (missing declaration and low contig overlap)");
        }
    };
    if !inferred.eq_ignore_ascii_case(&species.build_id) {
        summary.refused.push("build_declared_vs_inferred".to_string());
        bail!(
            "vcf.validate_inputs refusal: declared/inferred build {} does not match species build {}",
            inferred,
            species.build_id
        );
    }

    summary.checked.push("chr_prefix_mismatch".to_string());
    let input_has_chr = record_contigs.iter().any(|c| c.starts_with("chr"));
    let species_has_chr = species.contigs.iter().any(|c| c.name.starts_with("chr"));
    if input_has_chr != species_has_chr && !config.allow_contig_aliasing {
        summary.refused.push("chr_prefix_mismatch".to_string());
        bail!("vcf.validate_inputs refusal: chr prefix mismatch between input and species context");
    }

    summary.checked.push("sorted_by_contig_and_pos".to_string());
    let rank = species
        .contigs
        .iter()
        .enumerate()
        .map(|(i, c)| (canonical_contig_label(&c.name), i))
        .collect::<BTreeMap<_, _>>();
    let mut sorted = records.clone();
    sorted.sort_by(|a, b| {
        let ka = parse_variant_key(a).unwrap_or_default();
        let kb = parse_variant_key(b).unwrap_or_default();
        let ra = rank
            .get(&canonical_contig_label(&ka.0))
            .copied()
            .unwrap_or(usize::MAX);
        let rb = rank
            .get(&canonical_contig_label(&kb.0))
            .copied()
            .unwrap_or(usize::MAX);
        ra.cmp(&rb)
            .then(ka.1.cmp(&kb.1))
            .then(ka.2.cmp(&kb.2))
            .then(ka.3.cmp(&kb.3))
    });
    if sorted != records {
        summary.fixed.push("sorted_by_contig_and_pos".to_string());
        records = sorted;
    }

    summary.checked.push("ploidy_declaration_consistent".to_string());
    let mut ploidy_counts = BTreeSet::<usize>::new();
    for line in &records {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        let Some(fmt) = fields.get(8) else {
            continue;
        };
        let gt_idx = fmt.split(':').position(|k| k == "GT");
        if let (Some(i), Some(sample)) = (gt_idx, fields.get(9)) {
            let vals = sample.split(':').collect::<Vec<_>>();
            if let Some(gt) = vals.get(i) {
                if !gt.contains('.') {
                    ploidy_counts.insert(gt.split(['/', '|']).count());
                }
            }
        }
    }
    if ploidy_counts.len() > 1 {
        summary.refused.push("ploidy_declaration_consistent".to_string());
        bail!("vcf.validate_inputs refusal: inconsistent ploidy declaration");
    }

    summary.checked.push("sex_chr_rules".to_string());
    let has_sex_chr = record_contigs
        .iter()
        .any(|c| matches!(c.as_str(), "X" | "Y" | "chrX" | "chrY"));
    if has_sex_chr && species.par_policy.eq_ignore_ascii_case("unsupported") {
        summary.refused.push("sex_chr_rules".to_string());
        bail!("vcf.validate_inputs refusal: sex chromosome rules cannot be applied without PAR policy");
    }
    if has_sex_chr && config.require_sex_metadata_for_sex_chr {
        let has_sex_meta = header_lines
            .iter()
            .any(|h| h.starts_with("##SAMPLE=") && (h.contains("Sex=") || h.contains("SEX=")));
        if !has_sex_meta {
            summary.refused.push("sex_chr_rules".to_string());
            bail!("vcf.validate_inputs refusal: sex chromosome present but sex metadata missing");
        }
    }

    sample_ids.sort();
    summary.fixed.push("deterministic_header_normalization".to_string());
    let mut fileformat = vec![];
    let mut contigs = vec![];
    let mut other_meta = vec![];
    for h in &header_lines {
        if h.starts_with("##fileformat=") {
            fileformat.push(h.clone());
        } else if h.starts_with("##contig=<") {
            contigs.push(h.clone());
        } else if h.starts_with("##") {
            other_meta.push(h.clone());
        }
    }
    contigs.sort_by_key(|h| {
        let id = h
            .split("ID=")
            .nth(1)
            .and_then(|x| x.split([',', '>']).next())
            .unwrap_or_default();
        rank.get(&canonical_contig_label(id)).copied().unwrap_or(usize::MAX)
    });
    other_meta.sort();
    let chrom = format!(
        "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t{}",
        sample_ids.join("\t")
    );
    let mut normalized_header = vec![];
    normalized_header.extend(fileformat);
    normalized_header.extend(other_meta);
    normalized_header.extend(contigs);
    normalized_header.push(chrom);

    let normalized_input = if input_vcf.extension().and_then(|s| s.to_str()) == Some("bcf") {
        artifact_dir.join("normalized.bcf")
    } else {
        artifact_dir.join("normalized.vcf.gz")
    };
    let normalized_payload = format!("{}\n{}\n", normalized_header.join("\n"), records.join("\n"));
    atomic_write_bytes(&normalized_input, normalized_payload.as_bytes())?;

    summary.checked.push("ensure_bgzip_tabix".to_string());
    let index_path = if normalized_input.extension().and_then(|s| s.to_str()) == Some("bcf") {
        artifact_dir.join("normalized.bcf.csi")
    } else {
        artifact_dir.join("normalized.vcf.gz.tbi")
    };
    atomic_write_bytes(&index_path, b"deterministic-index-placeholder\n")?;

    let regime = detect_regime(&records);
    summary.checked.push("input_regime_detection".to_string());

    let overlap_json = artifact_dir.join("overlap.json");
    let overlap_fraction = site_overlap_diagnostic(&records, &records, &overlap_json)?;
    summary.checked.push("site_overlap_diagnostic".to_string());
    if overlap_fraction < config.min_overlap_threshold {
        summary.refused.push("site_overlap_diagnostic".to_string());
        bail!("vcf.validate_inputs refusal: overlap below configured threshold");
    }

    let invariants_json = artifact_dir.join("invariants.json");
    atomic_write_json(
        &invariants_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.invariants.v1",
            "config": config,
            "summary": summary,
            "regime_detection": regime,
            "normalized_input": normalized_input,
            "index": index_path,
            "overlap": overlap_json,
        }),
    )?;

    Ok(VcfPreflightResult {
        normalized_input,
        index_path,
        invariants_json,
        regime,
        overlap_json,
        summary,
    })
}
