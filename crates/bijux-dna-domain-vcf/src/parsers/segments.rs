use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};

use crate::taxonomy::VcfDomainStage;

const RAW_COMMAND_NAME: &str = "raw.command.json";
const RAW_LOG_NAME: &str = "raw.log";
const RAW_ROH_SEGMENTS_NAME: &str = "raw.hom";
const RAW_IBD_SEGMENTS_NAME: &str = "raw.ibd_filtered_segments.tsv";
const RAW_IBD_SUMMARY_NAME: &str = "raw.ibd_summary.json";
const RAW_IBD_METRICS_NAME: &str = "raw.ibd_metrics.json";
const RAW_NE_TRAJECTORY_NAME: &str = "raw.ne_trajectory.tsv";
const RAW_DEMOGRAPHY_CONTRACT_NAME: &str = "raw.demography.json";
const RAW_DEMOGRAPHY_METRICS_NAME: &str = "raw.demography_metrics.json";

#[derive(Debug, Clone)]
struct RohSegment {
    sample_id: String,
    contig: String,
    start: u64,
    end: u64,
    length: u64,
    variant_count: u64,
}

#[derive(Debug, Clone)]
struct IbdSegment {
    sample_a: String,
    sample_b: String,
    contig: String,
    start: u64,
    end: u64,
    length_cm: f64,
    marker_count: u64,
}

#[derive(Debug, Clone)]
struct DemographyPoint {
    generation: u64,
    ne: f64,
    ci_low: f64,
    ci_high: f64,
}

/// Normalize the governed raw descent artifact set for a retained VCF backend.
///
/// # Errors
/// Returns an error when required raw artifacts are missing, malformed, or drift away from the
/// governed VCF normalized schema family.
pub fn parse_segment_stage_metrics(
    tool_id: &str,
    stage: VcfDomainStage,
    artifact_root: &Path,
) -> Result<serde_json::Value> {
    match (tool_id, stage) {
        ("plink2", VcfDomainStage::Roh) => parse_roh_metrics(artifact_root),
        ("germline", VcfDomainStage::Ibd)
        | ("ibdseq", VcfDomainStage::Ibd)
        | ("ibdhap", VcfDomainStage::Ibd) => parse_ibd_metrics(tool_id, artifact_root),
        ("ibdne", VcfDomainStage::Demography) => parse_demography_metrics(artifact_root),
        _ => bail!("unsupported VCF segment parser row `{tool_id}` / `{}`", stage.as_str()),
    }
}

fn parse_roh_metrics(root: &Path) -> Result<serde_json::Value> {
    validate_command("plink2", "vcf.roh", &read_json(&root.join(RAW_COMMAND_NAME))?)?;
    validate_nonempty_text_file(&root.join(RAW_LOG_NAME), "ROH log")?;

    let segments = parse_roh_segments(&root.join(RAW_ROH_SEGMENTS_NAME))?;
    if segments.is_empty() {
        bail!("ROH parser requires at least one segment row");
    }

    let mut per_sample_counts = BTreeMap::<String, u64>::new();
    let mut per_sample_lengths = BTreeMap::<String, u64>::new();
    let mut total_length = 0_u64;
    for segment in &segments {
        total_length += segment.length;
        *per_sample_counts.entry(segment.sample_id.clone()).or_insert(0) += 1;
        *per_sample_lengths.entry(segment.sample_id.clone()).or_insert(0) += segment.length;
    }

    let per_sample_summary = per_sample_counts
        .iter()
        .map(|(sample_id, segment_count)| {
            let sample_total = per_sample_lengths.get(sample_id).copied().unwrap_or(0);
            serde_json::json!({
                "sample_id": sample_id,
                "segment_count": segment_count,
                "total_length": sample_total,
                "mean_length": if *segment_count == 0 {
                    0.0
                } else {
                    sample_total as f64 / *segment_count as f64
                },
            })
        })
        .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.roh.v1",
        "stage_id": "vcf.roh",
        "tool_id": "plink2",
        "status": "complete",
        "sample_count": per_sample_counts.len(),
        "segment_count": segments.len(),
        "total_length": total_length,
        "segments": segments
            .into_iter()
            .map(|segment| {
                serde_json::json!({
                    "sample_id": segment.sample_id,
                    "contig": segment.contig,
                    "start": segment.start,
                    "end": segment.end,
                    "length": segment.length,
                    "variant_count": segment.variant_count,
                })
            })
            .collect::<Vec<_>>(),
        "per_sample_summary": per_sample_summary,
    }))
}

fn parse_ibd_metrics(tool_id: &str, root: &Path) -> Result<serde_json::Value> {
    validate_command(tool_id, "vcf.ibd", &read_json(&root.join(RAW_COMMAND_NAME))?)?;
    validate_nonempty_text_file(&root.join(RAW_LOG_NAME), "IBD log")?;

    let summary = read_json(&root.join(RAW_IBD_SUMMARY_NAME))?;
    let metrics = read_json(&root.join(RAW_IBD_METRICS_NAME))?;
    let status = json_string(&summary, "/status", "IBD status")?;
    let metrics_status = json_string(&metrics, "/status", "IBD metrics status")?;
    if status != metrics_status {
        bail!("IBD status drifted between summary and metrics: `{status}` vs `{metrics_status}`");
    }
    let insufficient_reason = json_optional_string(
        &summary,
        "/insufficient_data_reason",
        "IBD insufficient_data_reason",
    )?;
    let metrics_reason = json_optional_string(
        &metrics,
        "/insufficient_data_reason",
        "IBD metrics insufficient_data_reason",
    )?;
    if insufficient_reason != metrics_reason {
        bail!("IBD insufficient reason drifted between summary and metrics");
    }

    let segments = parse_ibd_segments(&root.join(RAW_IBD_SEGMENTS_NAME))?;
    let filtered_segment_count =
        json_u64(&summary, "/segments_filtered", "IBD filtered segment count")?;
    if filtered_segment_count != segments.len() as u64 {
        bail!(
            "IBD summary reported {} filtered segments but fixture contains {} rows",
            filtered_segment_count,
            segments.len()
        );
    }
    let metrics_segment_count =
        json_u64(&metrics, "/ibd_segment_count", "IBD metrics segment count")?;
    if metrics_segment_count != filtered_segment_count {
        bail!(
            "IBD metrics segment count drifted from summary: {} vs {}",
            metrics_segment_count,
            filtered_segment_count
        );
    }

    let total_length = segments.iter().map(|segment| segment.length_cm).sum::<f64>();
    let summary_total_length = json_f64(&summary, "/total_length_cm", "IBD summary total length")?;
    if !approx_equal(total_length, summary_total_length) {
        bail!(
            "IBD summary total length drifted from segment rows: {} vs {}",
            summary_total_length,
            total_length
        );
    }
    let metrics_total_length =
        json_f64(&metrics, "/ibd_total_length_cM", "IBD metrics total length")?;
    if !approx_equal(metrics_total_length, summary_total_length) {
        bail!(
            "IBD metrics total length drifted from summary: {} vs {}",
            metrics_total_length,
            summary_total_length
        );
    }

    match status {
        "complete" => {
            if segments.is_empty() {
                bail!("IBD complete fixture must contain at least one filtered segment");
            }
            if insufficient_reason.is_some() {
                bail!("IBD complete fixture must not report an insufficient reason");
            }
        }
        "insufficient_marker_overlap" => {
            if !segments.is_empty() {
                bail!("IBD insufficient-overlap fixture must not retain filtered segments");
            }
            if insufficient_reason.is_none() {
                bail!("IBD insufficient-overlap fixture must report an insufficient reason");
            }
        }
        other => bail!("unsupported IBD status `{other}`"),
    }

    let mut pair_totals = BTreeMap::<(String, String), (u64, f64, u64)>::new();
    for segment in &segments {
        let key = ordered_pair(&segment.sample_a, &segment.sample_b);
        let entry = pair_totals.entry(key).or_insert((0, 0.0, 0));
        entry.0 += 1;
        entry.1 += segment.length_cm;
        entry.2 += segment.marker_count;
    }
    let rows = pair_totals
        .into_iter()
        .map(|((sample_a, sample_b), (segment_count, pair_total_length, marker_count_total))| {
            serde_json::json!({
                "sample_a": sample_a,
                "sample_b": sample_b,
                "segment_count": segment_count,
                "total_length": pair_total_length,
                "overlap_marker_count": marker_count_total,
                "status": "complete",
            })
        })
        .collect::<Vec<_>>();

    let insufficient_overlap_probe = if status == "insufficient_marker_overlap" {
        serde_json::json!({
            "status": status,
            "insufficient_reason": insufficient_reason,
            "filtered_segment_count": filtered_segment_count,
        })
    } else {
        serde_json::json!({
            "status": "not_run",
            "insufficient_reason": serde_json::Value::Null,
            "filtered_segment_count": 0,
        })
    };

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.ibd.v1",
        "stage_id": "vcf.ibd",
        "tool_id": tool_id,
        "status": status,
        "insufficient_reason": insufficient_reason,
        "pair_count": rows.len(),
        "rows": rows,
        "insufficient_overlap_probe": insufficient_overlap_probe,
    }))
}

fn parse_demography_metrics(root: &Path) -> Result<serde_json::Value> {
    validate_command("ibdne", "vcf.demography", &read_json(&root.join(RAW_COMMAND_NAME))?)?;
    validate_nonempty_text_file(&root.join(RAW_LOG_NAME), "demography log")?;

    let contract = read_json(&root.join(RAW_DEMOGRAPHY_CONTRACT_NAME))?;
    let metrics = read_json(&root.join(RAW_DEMOGRAPHY_METRICS_NAME))?;
    let method = json_string(&contract, "/method", "demography method")?;
    let metrics_method = json_string(&metrics, "/method", "demography metrics method")?;
    if method != metrics_method {
        bail!("demography method drifted between contract and metrics");
    }
    let inference_status =
        json_string(&contract, "/inference_status", "demography inference_status")?;
    let metrics_inference_status =
        json_string(&metrics, "/inference_status", "demography metrics inference_status")?;
    if inference_status != metrics_inference_status {
        bail!("demography inference_status drifted between contract and metrics");
    }
    let status = json_string(&contract, "/status", "demography status")?;
    let metrics_status = json_string(&metrics, "/status", "demography metrics status")?;
    if status != metrics_status {
        bail!("demography status drifted between contract and metrics");
    }
    let insufficient_reason = json_optional_string(
        &contract,
        "/insufficient_data_reason",
        "demography insufficient_data_reason",
    )?;
    let metrics_reason = json_optional_string(
        &metrics,
        "/insufficient_data_reason",
        "demography metrics insufficient_data_reason",
    )?;
    if insufficient_reason != metrics_reason {
        bail!("demography insufficient reason drifted between contract and metrics");
    }

    let time_bins = json_u64_array(&contract, "/time_bins", "demography time_bins")?;
    if time_bins != json_u64_array(&metrics, "/time_bins", "demography metrics time_bins")? {
        bail!("demography time bins drifted between contract and metrics");
    }
    let ne_estimates = json_array(&contract, "/ne_estimates", "demography ne_estimates")?;
    if ne_estimates != json_array(&metrics, "/ne_estimates", "demography metrics ne_estimates")? {
        bail!("demography ne_estimates drifted between contract and metrics");
    }

    let trajectory = parse_demography_trajectory(&root.join(RAW_NE_TRAJECTORY_NAME))?;
    if trajectory.len() != time_bins.len() {
        bail!(
            "demography trajectory length {} drifted from time_bins {}",
            trajectory.len(),
            time_bins.len()
        );
    }
    if trajectory.len() != ne_estimates.len() {
        bail!(
            "demography trajectory length {} drifted from ne_estimates {}",
            trajectory.len(),
            ne_estimates.len()
        );
    }
    validate_demography_points(&trajectory, &ne_estimates)?;

    match status {
        "complete" => {
            if trajectory.is_empty() {
                bail!("demography complete fixture must contain trajectory rows");
            }
            if insufficient_reason.is_some() {
                bail!("demography complete fixture must not report an insufficient reason");
            }
        }
        "insufficient_data" => {
            if !trajectory.is_empty() || !time_bins.is_empty() || !ne_estimates.is_empty() {
                bail!("demography insufficient-data fixture must keep trajectory outputs empty");
            }
            if insufficient_reason.is_none() {
                bail!("demography insufficient-data fixture must report an insufficient reason");
            }
        }
        other => bail!("unsupported demography status `{other}`"),
    }

    let insufficient_data_probe = if status == "insufficient_data" {
        serde_json::json!({
            "status": status,
            "insufficient_reason": insufficient_reason,
            "time_bins": time_bins,
            "ne_estimates": ne_estimates,
        })
    } else {
        serde_json::json!({
            "status": "not_run",
            "insufficient_reason": serde_json::Value::Null,
            "time_bins": Vec::<u64>::new(),
            "ne_estimates": Vec::<serde_json::Value>::new(),
        })
    };

    Ok(serde_json::json!({
        "schema_version": "bijux.vcf.demography.v1",
        "stage_id": "vcf.demography",
        "tool_id": "ibdne",
        "method": method,
        "inference_status": inference_status,
        "status": status,
        "insufficient_reason": insufficient_reason,
        "time_bins": time_bins,
        "ne_estimates": ne_estimates,
        "insufficient_data_probe": insufficient_data_probe,
    }))
}

fn validate_command(tool_id: &str, stage_id: &str, command: &serde_json::Value) -> Result<()> {
    let declared_tool_id = json_string(command, "/tool_id", "tool_id")?;
    if declared_tool_id != tool_id {
        bail!("segment parser expected tool_id `{tool_id}`, found `{declared_tool_id}`");
    }
    let declared_stage_id = json_string(command, "/stage_id", "stage_id")?;
    if declared_stage_id != stage_id {
        bail!("segment parser expected stage_id `{stage_id}`, found `{declared_stage_id}`");
    }
    let argv = json_string_array(command, "/argv", "argv")?;
    let joined = argv.join(" ");
    for token in required_command_tokens(tool_id, stage_id)? {
        if !joined.contains(token) {
            bail!("segment command for `{tool_id}` / `{stage_id}` is missing `{token}`");
        }
    }
    Ok(())
}

fn required_command_tokens(tool_id: &str, stage_id: &str) -> Result<&'static [&'static str]> {
    match (tool_id, stage_id) {
        ("plink2", "vcf.roh") => Ok(&["plink2", "--homozyg"]),
        ("germline", "vcf.ibd") => Ok(&["germline", "-output"]),
        ("ibdseq", "vcf.ibd") => Ok(&["ibdseq", "--vcf", "--out"]),
        ("ibdhap", "vcf.ibd") => Ok(&["ibdhap", "--vcf", "--out"]),
        ("ibdne", "vcf.demography") => Ok(&["ibdne", "--ibd", "--out"]),
        _ => bail!("unsupported segment parser command row `{tool_id}` / `{stage_id}`"),
    }
}

fn parse_roh_segments(path: &Path) -> Result<Vec<RohSegment>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines().filter(|line| !line.trim().is_empty());
    let header = lines
        .next()
        .ok_or_else(|| anyhow!("ROH segment table is empty: {}", path.display()))?
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();

    let sample_idx = index_for(&header, &["sample", "iid"])?;
    let contig_idx = index_for(&header, &["contig", "chr"])?;
    let start_idx = index_for(&header, &["start", "pos1"])?;
    let end_idx = index_for(&header, &["end", "pos2"])?;
    let length_idx = index_for(&header, &["length_bp", "length", "kb"])?;
    let variant_idx = index_for(&header, &["n_sites", "nsnp", "variant_count"])?;

    let mut segments = Vec::<RohSegment>::new();
    for (line_index, line) in lines.enumerate() {
        let columns = line.split_whitespace().collect::<Vec<_>>();
        segments.push(RohSegment {
            sample_id: field(&columns, sample_idx, path)?.to_string(),
            contig: field(&columns, contig_idx, path)?.to_string(),
            start: parse_u64(field(&columns, start_idx, path)?, "ROH start")?,
            end: parse_u64(field(&columns, end_idx, path)?, "ROH end")?,
            length: parse_u64(field(&columns, length_idx, path)?, "ROH length")?,
            variant_count: parse_u64(field(&columns, variant_idx, path)?, "ROH variant count")?,
        });
        if columns.len() != header.len() {
            bail!(
                "ROH segment row {} in {} drifted away from header width",
                line_index + 2,
                path.display()
            );
        }
    }
    segments.sort_by(|left, right| {
        left.sample_id
            .cmp(&right.sample_id)
            .then_with(|| left.contig.cmp(&right.contig))
            .then_with(|| left.start.cmp(&right.start))
            .then_with(|| left.end.cmp(&right.end))
    });
    Ok(segments)
}

fn parse_ibd_segments(path: &Path) -> Result<Vec<IbdSegment>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow!("IBD filtered segment table is empty: {}", path.display()))?;
    if header.trim() != "sample_a\tsample_b\tcontig\tstart\tend\tlength_cm\tmarker_count" {
        bail!("IBD filtered segment header drifted in {}: `{header}`", path.display());
    }

    let mut segments = Vec::<IbdSegment>::new();
    for (line_index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 7 {
            bail!(
                "IBD filtered segment row {} in {} must have 7 columns",
                line_index + 2,
                path.display()
            );
        }
        segments.push(IbdSegment {
            sample_a: columns[0].trim().to_string(),
            sample_b: columns[1].trim().to_string(),
            contig: columns[2].trim().to_string(),
            start: parse_u64(columns[3].trim(), "IBD start")?,
            end: parse_u64(columns[4].trim(), "IBD end")?,
            length_cm: parse_f64(columns[5].trim(), "IBD length_cm")?,
            marker_count: parse_u64(columns[6].trim(), "IBD marker_count")?,
        });
    }
    for segment in &segments {
        if segment.sample_a == segment.sample_b {
            bail!("IBD segment pair must contain two distinct sample IDs");
        }
        if segment.end < segment.start {
            bail!(
                "IBD segment `{}` / `{}` ends before it starts",
                segment.sample_a,
                segment.sample_b
            );
        }
        if segment.contig.trim().is_empty() {
            bail!("IBD segment contig must not be empty");
        }
    }
    Ok(segments)
}

fn parse_demography_trajectory(path: &Path) -> Result<Vec<DemographyPoint>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow!("demography trajectory is empty: {}", path.display()))?;
    if header.trim() != "generation\tne\tci_low\tci_high" {
        bail!("demography trajectory header drifted in {}: `{header}`", path.display());
    }

    let mut points = Vec::<DemographyPoint>::new();
    for (line_index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 4 {
            bail!(
                "demography trajectory row {} in {} must have 4 columns",
                line_index + 2,
                path.display()
            );
        }
        points.push(DemographyPoint {
            generation: parse_u64(columns[0].trim(), "generation")?,
            ne: parse_f64(columns[1].trim(), "ne")?,
            ci_low: parse_f64(columns[2].trim(), "ci_low")?,
            ci_high: parse_f64(columns[3].trim(), "ci_high")?,
        });
    }
    Ok(points)
}

fn validate_demography_points(
    trajectory: &[DemographyPoint],
    ne_estimates: &[serde_json::Value],
) -> Result<()> {
    for (index, (trajectory_point, estimate)) in trajectory.iter().zip(ne_estimates).enumerate() {
        let generation = json_value_u64(estimate, "/generation", "demography estimate generation")?;
        let ne = json_value_f64(estimate, "/ne", "demography estimate ne")?;
        let ci_low = json_value_f64(estimate, "/ci_low", "demography estimate ci_low")?;
        let ci_high = json_value_f64(estimate, "/ci_high", "demography estimate ci_high")?;
        if generation != trajectory_point.generation
            || !approx_equal(ne, trajectory_point.ne)
            || !approx_equal(ci_low, trajectory_point.ci_low)
            || !approx_equal(ci_high, trajectory_point.ci_high)
        {
            bail!("demography estimate row {} drifted from trajectory row", index + 1);
        }
    }
    Ok(())
}

fn validate_nonempty_text_file(path: &Path, label: &str) -> Result<()> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    if raw.trim().is_empty() {
        bail!("{label} must not be empty");
    }
    Ok(())
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn index_for(header: &[String], accepted: &[&str]) -> Result<usize> {
    header
        .iter()
        .position(|column| {
            accepted.iter().any(|accepted_name| normalize_header(column) == *accepted_name)
        })
        .ok_or_else(|| anyhow!("missing required header column from {:?}", accepted))
}

fn normalize_header(header: &str) -> String {
    header.trim().to_ascii_lowercase()
}

fn field<'a>(row: &'a [&str], index: usize, path: &Path) -> Result<&'a str> {
    row.get(index)
        .copied()
        .ok_or_else(|| anyhow!("row in {} is missing column {}", path.display(), index))
}

fn ordered_pair(sample_a: &str, sample_b: &str) -> (String, String) {
    if sample_a <= sample_b {
        (sample_a.to_string(), sample_b.to_string())
    } else {
        (sample_b.to_string(), sample_a.to_string())
    }
}

fn parse_u64(raw: &str, label: &str) -> Result<u64> {
    raw.parse::<u64>().with_context(|| format!("parse `{label}` from `{raw}`"))
}

fn parse_f64(raw: &str, label: &str) -> Result<f64> {
    raw.parse::<f64>().with_context(|| format!("parse `{label}` from `{raw}`"))
}

fn json_string<'a>(value: &'a serde_json::Value, pointer: &str, label: &str) -> Result<&'a str> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("{label} is missing at `{pointer}`"))
}

fn json_optional_string(
    value: &serde_json::Value,
    pointer: &str,
    label: &str,
) -> Result<Option<String>> {
    match value.pointer(pointer) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(raw)) => Ok(Some(raw.clone())),
        Some(_) => bail!("{label} at `{pointer}` must be a string or null"),
    }
}

fn json_u64(value: &serde_json::Value, pointer: &str, label: &str) -> Result<u64> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("{label} is missing at `{pointer}`"))
}

fn json_f64(value: &serde_json::Value, pointer: &str, label: &str) -> Result<f64> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("{label} is missing at `{pointer}`"))
}

fn json_u64_array(value: &serde_json::Value, pointer: &str, label: &str) -> Result<Vec<u64>> {
    let array = value
        .pointer(pointer)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("{label} is missing at `{pointer}`"))?;
    array
        .iter()
        .map(|item| item.as_u64().ok_or_else(|| anyhow!("{label} must contain integers")))
        .collect()
}

fn json_array(
    value: &serde_json::Value,
    pointer: &str,
    label: &str,
) -> Result<Vec<serde_json::Value>> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_array)
        .cloned()
        .ok_or_else(|| anyhow!("{label} is missing at `{pointer}`"))
}

fn json_string_array(value: &serde_json::Value, pointer: &str, label: &str) -> Result<Vec<String>> {
    let array = value
        .pointer(pointer)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("{label} is missing at `{pointer}`"))?;
    array
        .iter()
        .map(|item| {
            item.as_str().map(str::to_string).ok_or_else(|| anyhow!("{label} must contain strings"))
        })
        .collect()
}

fn json_value_u64(value: &serde_json::Value, pointer: &str, label: &str) -> Result<u64> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("{label} is missing at `{pointer}`"))
}

fn json_value_f64(value: &serde_json::Value, pointer: &str, label: &str) -> Result<f64> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("{label} is missing at `{pointer}`"))
}

fn approx_equal(left: f64, right: f64) -> bool {
    (left - right).abs() <= 0.000_001
}
