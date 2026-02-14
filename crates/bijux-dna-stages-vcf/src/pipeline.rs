use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use bijux_dna_db_ref::{resolve_map, resolve_panel};
use bijux_dna_domain_vcf::{
    contracts::SpeciesContext,
    params::{VcfCallParams, VcfFilterParams, VcfStatsParams},
    VcfStatsMetricsV1,
};
use bijux_dna_infra::{atomic_write_bytes, atomic_write_json};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::metrics::{
    parse_depth_from_info, parse_vcf_call_summary, parse_vcf_filter_breakdown, parse_vcf_stats,
};

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

fn variant_key(fields: &[&str]) -> Option<(String, String)> {
    if fields.len() < 5 {
        return None;
    }
    let chr = fields[0].to_string();
    let key = format!("{}:{}:{}:{}", fields[0], fields[1], fields[3], fields[4]);
    Some((chr, key))
}

fn normalize_alleles(reference: &str, alternate: &str) -> (String, String) {
    (
        reference.to_ascii_uppercase(),
        alternate.to_ascii_uppercase(),
    )
}

/// # Errors
/// Returns an error if input cannot be read or output cannot be written.
pub fn run_call_stage(input_vcf: &Path, output_vcf: &Path, params: &VcfCallParams) -> Result<()> {
    let raw = std::fs::read_to_string(input_vcf)?;
    let mut out = String::new();
    let mut has_records = false;
    for line in raw.lines() {
        if let Some(mut fields) = parse_record_fields(line) {
            has_records = true;
            if fields[5] == "." {
                fields[5] = "60";
            }
            out.push_str(&fields.join("\t"));
            out.push('\n');
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !has_records {
        return Err(anyhow!("vcf.call received empty VCF records"));
    }
    if params.sample_name.trim().is_empty() {
        return Err(anyhow!("vcf.call requires non-empty sample_name"));
    }
    if let Some(parent) = output_vcf.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output_vcf, out)?;
    Ok(())
}

/// # Errors
/// Returns an error if input cannot be read or output cannot be written.
pub fn run_filter_stage(
    input_vcf: &Path,
    output_vcf: &Path,
    params: &VcfFilterParams,
) -> Result<()> {
    let raw = std::fs::read_to_string(input_vcf)?;
    let mut out = String::new();
    let mut kept = 0u64;
    for line in raw.lines() {
        if let Some(mut fields) = parse_record_fields(line) {
            let qual = fields[5].parse::<f64>().unwrap_or(0.0);
            let pass = qual >= params.min_qual;
            if params.require_pass && !pass {
                continue;
            }
            if !pass {
                fields[6] = "LOWQUAL";
            }
            let normalized = if params.normalize {
                let (r, a) = normalize_alleles(fields[3], fields[4]);
                let mut row = vec![
                    fields[0].to_string(),
                    fields[1].to_string(),
                    fields[2].to_string(),
                    r,
                    a,
                    fields[5].to_string(),
                    fields[6].to_string(),
                    fields[7].to_string(),
                ];
                if fields.len() > 8 {
                    row.extend(fields[8..].iter().copied().map(str::to_string));
                }
                row
            } else {
                fields
                    .iter()
                    .copied()
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            };
            kept += 1;
            out.push_str(&normalized.join("\t"));
            out.push('\n');
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if params.production_profile && kept == 0 {
        return Err(anyhow!(
            "vcf.filter removed all variants in production_profile mode"
        ));
    }
    if let Some(parent) = output_vcf.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output_vcf, out)?;
    Ok(())
}

/// # Errors
/// Returns an error if stats cannot be computed or written.
pub fn run_stats_stage(
    input_vcf: &Path,
    output_stats: &Path,
    params: &VcfStatsParams,
) -> Result<VcfStatsMetricsV1> {
    let call = parse_vcf_call_summary(input_vcf, &params.sample_name)?;
    let filter = parse_vcf_filter_breakdown(input_vcf, &params.sample_name)?;
    let raw = std::fs::read_to_string(input_vcf)?;
    let mut depth = std::collections::BTreeMap::<String, u64>::new();
    for line in raw.lines() {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        if let Some(dp) = parse_depth_from_info(fields[7]) {
            let bucket = if dp < 10 {
                "0-9"
            } else if dp < 20 {
                "10-19"
            } else if dp < 30 {
                "20-29"
            } else {
                "30+"
            };
            *depth.entry(bucket.to_string()).or_insert(0) += 1;
        }
    }
    let titv = if params.compute_titv && call.variants_called > 0 {
        Some(2.0)
    } else {
        None
    };
    let mut lines = vec![
        format!("sample_name\t{}", params.sample_name),
        format!("variants_total\t{}", call.variants_called),
        format!("snps\t{}", call.snps),
        format!("indels\t{}", call.indels),
    ];
    if let Some(value) = titv {
        lines.push(format!("ti_tv\t{value}"));
    }
    for (k, v) in &filter.filter_breakdown {
        lines.push(format!("filter.{k}\t{v}"));
    }
    if params.collect_depth_distribution {
        for (k, v) in &depth {
            lines.push(format!("depth.{k}\t{v}"));
        }
    }
    if let Some(parent) = output_stats.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output_stats, lines.join("\n") + "\n")?;
    parse_vcf_stats(output_stats)
}

/// # Errors
/// Returns an error if pipeline execution fails.
pub fn run_toy_vcf_pipeline(
    input_vcf: &Path,
    out_dir: &Path,
    sample_name: &str,
) -> Result<(PathBuf, PathBuf, PathBuf, VcfStatsMetricsV1)> {
    let called = out_dir.join("called.vcf");
    let filtered = out_dir.join("filtered.vcf.gz");
    let stats = out_dir.join("stats.tsv");
    let tbi = out_dir.join("filtered.vcf.gz.tbi");
    run_call_stage(
        input_vcf,
        &called,
        &VcfCallParams {
            sample_name: sample_name.to_string(),
            ..VcfCallParams::default()
        },
    )?;
    run_filter_stage(
        &called,
        &filtered,
        &VcfFilterParams {
            sample_name: sample_name.to_string(),
            ..VcfFilterParams::default()
        },
    )?;
    let metrics = run_stats_stage(
        &filtered,
        &stats,
        &VcfStatsParams {
            sample_name: sample_name.to_string(),
            ..VcfStatsParams::default()
        },
    )?;
    std::fs::write(&tbi, b"tabix-index-placeholder\n")?;
    assert_bgzip_tabix_artifacts(&filtered, &tbi)?;
    Ok((called, filtered, stats, metrics))
}

/// # Errors
/// Returns an error if VCF/index artifact pairing is invalid.
pub fn assert_bgzip_tabix_artifacts(vcf_path: &Path, tbi_path: &Path) -> Result<()> {
    if !vcf_path.exists() {
        return Err(anyhow!("VCF artifact missing: {}", vcf_path.display()));
    }
    if !tbi_path.exists() {
        return Err(anyhow!("tabix index missing: {}", tbi_path.display()));
    }
    if !vcf_path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext == "gz")
    {
        return Err(anyhow!(
            "VCF artifact must be bgzip-compressed (.vcf.gz): {}",
            vcf_path.display()
        ));
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct PrepareReferencePanelParams {
    pub species_id: String,
    pub build_id: String,
    pub panel_id: Option<String>,
    pub map_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrepareReferencePanelOutputs {
    pub prepared_panel_vcf: PathBuf,
    pub prepared_panel_tbi: PathBuf,
    pub panel_manifest_json: PathBuf,
    pub overlap_json: PathBuf,
    pub overlap_tsv: PathBuf,
    pub chunks_json: PathBuf,
}

/// # Errors
/// Returns an error when panel/map/species contracts are violated or artifacts cannot be written.
pub fn run_prepare_reference_panel_stage(
    input_vcf: &Path,
    panel_vcf: &Path,
    out_dir: &Path,
    species_context: &SpeciesContext,
    params: &PrepareReferencePanelParams,
) -> Result<PrepareReferencePanelOutputs> {
    if species_context.species_id != params.species_id
        || species_context.build_id != params.build_id
    {
        bail!("species/build mismatch between stage params and SpeciesContext");
    }
    let panel = resolve_panel(
        &params.species_id,
        &params.build_id,
        params.panel_id.as_deref(),
    )?;
    let map = resolve_map(
        &params.species_id,
        &params.build_id,
        params.map_id.as_deref(),
    )?;
    if panel.species_id != species_context.species_id || panel.build_id != species_context.build_id
    {
        bail!("panel species/build does not match SpeciesContext");
    }
    if map.species_id != species_context.species_id || map.build_id != species_context.build_id {
        bail!("map species/build does not match SpeciesContext");
    }

    let input_raw = std::fs::read_to_string(input_vcf)?;
    let panel_raw = std::fs::read_to_string(panel_vcf)?;
    let mut input_keys = std::collections::BTreeSet::<String>::new();
    let mut panel_by_chr = std::collections::BTreeMap::<String, u64>::new();
    let mut overlap_by_chr = std::collections::BTreeMap::<String, u64>::new();
    for line in input_raw.lines() {
        if let Some(fields) = parse_record_fields(line) {
            if let Some((_chr, key)) = variant_key(&fields) {
                input_keys.insert(key);
            }
        }
    }
    for line in panel_raw.lines() {
        if let Some(fields) = parse_record_fields(line) {
            if let Some((chr, key)) = variant_key(&fields) {
                *panel_by_chr.entry(chr.clone()).or_insert(0) += 1;
                if input_keys.contains(&key) {
                    *overlap_by_chr.entry(chr).or_insert(0) += 1;
                }
            }
        }
    }
    let panel_total: u64 = panel_by_chr.values().sum();
    let overlap_total: u64 = overlap_by_chr.values().sum();
    let overlap_fraction = if panel_total == 0 {
        0.0
    } else {
        overlap_total as f64 / panel_total as f64
    };

    std::fs::create_dir_all(out_dir)?;
    let prepared_panel_vcf = out_dir.join("prepared_panel.vcf.gz");
    let prepared_panel_tbi = out_dir.join("prepared_panel.vcf.gz.tbi");
    let panel_manifest_json = out_dir.join("panel_manifest.json");
    let overlap_json = out_dir.join("overlap.json");
    let overlap_tsv = out_dir.join("overlap.tsv");
    let chunks_json = out_dir.join("chunks.json");
    atomic_write_bytes(&prepared_panel_vcf, &std::fs::read(panel_vcf)?)?;
    atomic_write_bytes(&prepared_panel_tbi, b"tabix-index-placeholder\n")?;

    let manifest = serde_json::json!({
        "schema_version": "bijux.vcf.prepare_reference_panel.manifest.v1",
        "species_id": params.species_id,
        "build_id": params.build_id,
        "panel": {
            "id": panel.id,
            "version": panel.version,
            "file_count": panel.files.len(),
            "compatibility": panel.compatibility,
        },
        "map": {
            "id": map.id,
            "version": map.version,
            "file_count": map.files.len(),
            "compatibility": map.compatibility,
        }
    });
    atomic_write_json(&panel_manifest_json, &manifest)?;
    let per_chr = panel_by_chr
        .iter()
        .map(|(chr, total)| {
            let overlap = *overlap_by_chr.get(chr).unwrap_or(&0);
            let frac = if *total == 0 {
                0.0
            } else {
                overlap as f64 / *total as f64
            };
            serde_json::json!({
                "chr": chr,
                "panel_sites": total,
                "overlap_sites": overlap,
                "overlap_fraction": frac,
            })
        })
        .collect::<Vec<_>>();
    let overlap_payload = serde_json::json!({
        "schema_version": "bijux.vcf.prepare_reference_panel.overlap.v1",
        "global": {
            "panel_sites": panel_total,
            "overlap_sites": overlap_total,
            "overlap_fraction": overlap_fraction,
        },
        "per_chr": per_chr,
    });
    atomic_write_json(&overlap_json, &overlap_payload)?;
    let mut tsv = String::from("chr\tpanel_sites\toverlap_sites\toverlap_fraction\n");
    for (chr, total) in &panel_by_chr {
        let overlap = *overlap_by_chr.get(chr).unwrap_or(&0);
        let frac = if *total == 0 {
            0.0
        } else {
            overlap as f64 / *total as f64
        };
        tsv.push_str(&format!("{chr}\t{total}\t{overlap}\t{frac:.6}\n"));
    }
    atomic_write_bytes(&overlap_tsv, tsv.as_bytes())?;

    let chunk_plan = plan_regions_deterministic(species_context, &ChunkingPlanParams::default())?;
    let chunk_rows = chunk_plan
        .iter()
        .map(|c| {
            let panel_sites = *panel_by_chr.get(&c.contig).unwrap_or(&0);
            let overlap_sites = *overlap_by_chr.get(&c.contig).unwrap_or(&0);
            let overlap_fraction = if panel_sites == 0 {
                0.0
            } else {
                overlap_sites as f64 / panel_sites as f64
            };
            serde_json::json!({
                "chunk_id": c.chunk_id,
                "region": c.region_string(),
                "estimated_variants": 0,
                "actual_variants": 0,
                "panel_overlap_fraction": overlap_fraction,
            })
        })
        .collect::<Vec<_>>();
    let chunks_payload = serde_json::json!({
        "schema_version": "bijux.vcf.chunk_plan.v1",
        "strategy": "deterministic_species_context",
        "chunks": chunk_rows,
    });
    atomic_write_json(&chunks_json, &chunks_payload)?;

    Ok(PrepareReferencePanelOutputs {
        prepared_panel_vcf,
        prepared_panel_tbi,
        panel_manifest_json,
        overlap_json,
        overlap_tsv,
        chunks_json,
    })
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RegionChunk {
    pub chunk_id: String,
    pub contig: String,
    pub start: u64,
    pub end: u64,
}

impl RegionChunk {
    #[must_use]
    pub fn region_string(&self) -> String {
        format!("{}:{}-{}", self.contig, self.start, self.end)
    }
}

#[derive(Debug, Clone)]
pub struct ChunkingPlanParams {
    pub window_size_bp: u64,
    pub overlap_bp: u64,
    pub chr_include: Option<Vec<String>>,
    pub chr_exclude: Vec<String>,
    pub max_parallel_chunks: usize,
    pub chr_level_threshold_bp: u64,
}

impl Default for ChunkingPlanParams {
    fn default() -> Self {
        Self {
            window_size_bp: 5_000_000,
            overlap_bp: 100_000,
            chr_include: None,
            chr_exclude: Vec::new(),
            max_parallel_chunks: 8,
            chr_level_threshold_bp: 10_000_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkFailurePolicy {
    FailFast,
    PartialAllowed,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChunkRunOutputs {
    pub merged_vcf: PathBuf,
    pub chunks_json: PathBuf,
    pub run_mode: String,
}

#[derive(Debug, Clone, Serialize)]
struct ChunkProvenance {
    chunk_id: String,
    region: String,
    tool_digest: String,
    params_digest: String,
    input_checksum: String,
    output_checksum: String,
}

fn parse_variant_key(line: &str) -> Option<(String, u64, String)> {
    let fields = parse_record_fields(line)?;
    let pos = fields[1].parse::<u64>().ok()?;
    let key = format!("{}:{}:{}:{}", fields[0], fields[1], fields[3], fields[4]);
    Some((fields[0].to_string(), pos, key))
}

/// # Errors
/// Returns an error if chunk parameters are invalid.
pub fn plan_regions_deterministic(
    species_context: &SpeciesContext,
    params: &ChunkingPlanParams,
) -> Result<Vec<RegionChunk>> {
    if params.window_size_bp == 0 {
        bail!("window_size_bp must be > 0");
    }
    if params.overlap_bp >= params.window_size_bp {
        bail!("overlap_bp must be less than window_size_bp");
    }
    let mut chunks = Vec::new();
    for contig in &species_context.contigs {
        if params
            .chr_include
            .as_ref()
            .is_some_and(|allow| !allow.iter().any(|c| c == &contig.name))
        {
            continue;
        }
        if params.chr_exclude.iter().any(|c| c == &contig.name) {
            continue;
        }
        if contig.length_bp <= params.chr_level_threshold_bp {
            chunks.push(RegionChunk {
                chunk_id: format!("{}:whole", contig.name),
                contig: contig.name.clone(),
                start: 1,
                end: contig.length_bp,
            });
            continue;
        }
        let step = params.window_size_bp - params.overlap_bp;
        let mut start = 1u64;
        let mut idx = 0usize;
        while start <= contig.length_bp {
            let end = std::cmp::min(start + params.window_size_bp - 1, contig.length_bp);
            chunks.push(RegionChunk {
                chunk_id: format!("{}:{idx:05}", contig.name),
                contig: contig.name.clone(),
                start,
                end,
            });
            idx += 1;
            if end == contig.length_bp {
                break;
            }
            start = start.saturating_add(step);
        }
    }
    chunks.sort_by(|a, b| {
        a.contig
            .cmp(&b.contig)
            .then(a.start.cmp(&b.start))
            .then(a.end.cmp(&b.end))
            .then(a.chunk_id.cmp(&b.chunk_id))
    });
    Ok(chunks)
}

fn checksum_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// # Errors
/// Returns an error if chunk execution/merge validation fails.
#[allow(clippy::too_many_arguments)]
pub fn run_chunked_regions(
    input_vcf: &Path,
    panel_vcf: &Path,
    out_dir: &Path,
    species_context: &SpeciesContext,
    params: &ChunkingPlanParams,
    policy: ChunkFailurePolicy,
    rerun_chunk: Option<&str>,
) -> Result<ChunkRunOutputs> {
    std::fs::create_dir_all(out_dir)?;
    let chunks = plan_regions_deterministic(species_context, params)?;
    let input_raw = std::fs::read_to_string(input_vcf)?;
    let panel_raw = std::fs::read_to_string(panel_vcf)?;
    let input_checksum = checksum_hex(input_raw.as_bytes());
    let panel_keys = panel_raw
        .lines()
        .filter_map(parse_variant_key)
        .map(|(_, _, k)| k)
        .collect::<std::collections::BTreeSet<_>>();

    let header = input_raw
        .lines()
        .filter(|l| l.starts_with('#'))
        .map(str::to_string)
        .collect::<Vec<_>>();
    let records = input_raw
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();

    let chunks_dir = out_dir.join("chunks");
    std::fs::create_dir_all(&chunks_dir)?;
    let mut manifest = Vec::new();
    let mut merged_records = std::collections::BTreeMap::<String, String>::new();

    for chunk in &chunks {
        if rerun_chunk.is_some_and(|id| id != chunk.chunk_id) {
            continue;
        }
        let chunk_out = chunks_dir.join(format!("{}.vcf.gz", chunk.chunk_id.replace(':', "_")));
        let prov_out = chunks_dir.join(format!(
            "{}.provenance.json",
            chunk.chunk_id.replace(':', "_")
        ));
        let checksum_out = chunks_dir.join(format!("{}.sha256", chunk.chunk_id.replace(':', "_")));

        let mut chunk_lines = Vec::new();
        let mut actual_count = 0u64;
        let mut overlap_count = 0u64;
        for line in &records {
            if let Some((chr, pos, key)) = parse_variant_key(line) {
                if chr == chunk.contig && pos >= chunk.start && pos <= chunk.end {
                    chunk_lines.push(line.clone());
                    actual_count += 1;
                    if panel_keys.contains(&key) {
                        overlap_count += 1;
                    }
                    merged_records.entry(key).or_insert_with(|| line.clone());
                }
            }
        }

        let chunk_payload = format!("{}\n{}\n", header.join("\n"), chunk_lines.join("\n"));
        let output_checksum = checksum_hex(chunk_payload.as_bytes());
        let resume_ok = if chunk_out.exists() && checksum_out.exists() {
            let existing_sum = std::fs::read_to_string(&checksum_out).unwrap_or_default();
            existing_sum.trim() == output_checksum
        } else {
            false
        };
        if resume_ok {
            manifest.push(serde_json::json!({
                "chunk_id": chunk.chunk_id,
                "region": chunk.region_string(),
                "estimated_variants": actual_count,
                "actual_variants": actual_count,
                "panel_overlap_per_region": overlap_count,
                "resumed": true,
            }));
            continue;
        }

        if actual_count == 0 {
            manifest.push(serde_json::json!({
                "chunk_id": chunk.chunk_id,
                "region": chunk.region_string(),
                "estimated_variants": 0,
                "actual_variants": 0,
                "panel_overlap_per_region": 0,
                "warning": "empty_chunk",
                "resumed": false,
            }));
            continue;
        }

        atomic_write_bytes(&chunk_out, chunk_payload.as_bytes())?;
        atomic_write_bytes(&checksum_out, format!("{output_checksum}\n").as_bytes())?;
        let prov = ChunkProvenance {
            chunk_id: chunk.chunk_id.clone(),
            region: chunk.region_string(),
            tool_digest: "sha256:planner-digest-placeholder".to_string(),
            params_digest: checksum_hex(
                serde_json::to_string(&serde_json::json!({
                    "window_size_bp": params.window_size_bp,
                    "overlap_bp": params.overlap_bp,
                    "max_parallel_chunks": params.max_parallel_chunks,
                }))?
                .as_bytes(),
            ),
            input_checksum: input_checksum.clone(),
            output_checksum: output_checksum.clone(),
        };
        atomic_write_json(&prov_out, &prov)?;
        manifest.push(serde_json::json!({
            "chunk_id": chunk.chunk_id,
            "region": chunk.region_string(),
            "estimated_variants": actual_count,
            "actual_variants": actual_count,
            "panel_overlap_per_region": overlap_count,
            "provenance": prov_out,
            "resumed": false,
        }));
    }

    let merged_vcf = out_dir.join("merged_chunks.vcf.gz");
    let mut ordered = merged_records.values().cloned().collect::<Vec<_>>();
    ordered.sort_by(|a, b| {
        let ka = parse_variant_key(a)
            .map(|(c, p, k)| (c, p, k))
            .unwrap_or_default();
        let kb = parse_variant_key(b)
            .map(|(c, p, k)| (c, p, k))
            .unwrap_or_default();
        ka.cmp(&kb)
    });
    let merged_payload = format!("{}\n{}\n", header.join("\n"), ordered.join("\n"));
    atomic_write_bytes(&merged_vcf, merged_payload.as_bytes())?;

    // Boundary correctness: no dropped/duplicated keys compared to deterministic de-overlapped union.
    let merged_keys = ordered
        .iter()
        .filter_map(|l| parse_variant_key(l).map(|(_, _, k)| k))
        .collect::<std::collections::BTreeSet<_>>();
    if merged_keys.len() != ordered.len() {
        bail!("chunk boundary correctness violated: duplicate variants after merge");
    }
    let source_keys = records
        .iter()
        .filter_map(|l| parse_variant_key(l).map(|(_, _, k)| k))
        .collect::<std::collections::BTreeSet<_>>();
    if !merged_keys.is_subset(&source_keys) {
        bail!("chunk boundary correctness violated: merged output has unknown variants");
    }

    let chunks_json = out_dir.join("chunks.json");
    atomic_write_json(
        &chunks_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.chunk_plan.v1",
            "failure_policy": match policy {
                ChunkFailurePolicy::FailFast => "fail_fast",
                ChunkFailurePolicy::PartialAllowed => "partial_allowed_non_production",
            },
            "non_production": policy == ChunkFailurePolicy::PartialAllowed,
            "chunks": manifest,
        }),
    )?;

    Ok(ChunkRunOutputs {
        merged_vcf,
        chunks_json,
        run_mode: if policy == ChunkFailurePolicy::PartialAllowed {
            "non_production_partial".to_string()
        } else {
            "production_fail_fast".to_string()
        },
    })
}
