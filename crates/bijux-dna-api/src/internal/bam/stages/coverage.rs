use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

const LOCAL_COVERAGE_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.coverage.local_smoke.metrics.v1";

/// Materialize the governed local-smoke `bam.coverage` artifacts and TSV summary.
///
/// The written summary artifact lives at `runs/bench/local-smoke/bam.coverage/coverage.tsv`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_coverage_smoke_summary() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_coverage_smoke_plans(&repo_root)?;
    let output_root = repo_root.join("runs/bench/local-smoke/bam.coverage");
    bijux_dna_infra::ensure_dir(&output_root)?;

    let mut body = String::from(
        "sample_id\tregion_id\tcontig\tstart\tend\tlength\tmean_depth\tbreadth_1x\tcovered_bases\tcoverage_regime\trow_expectation_matched\tcase_expectation_matched\tregions_bed\tcoverage_tsv\tcoverage_summary_json\tcoverage_depth\tcoverage_mosdepth_summary\tstage_metrics\n",
    );
    for case in &cases {
        for row in materialize_local_coverage_smoke_case(&repo_root, case)? {
            writeln!(
                body,
                "{sample_id}\t{region_id}\t{contig}\t{start}\t{end}\t{length}\t{mean_depth:.6}\t{breadth_1x:.6}\t{covered_bases}\t{coverage_regime}\t{row_expectation_matched}\t{case_expectation_matched}\t{regions_bed}\t{coverage_tsv}\t{coverage_summary_json}\t{coverage_depth}\t{coverage_mosdepth_summary}\t{stage_metrics}",
                sample_id = row.sample_id,
                region_id = row.region_id,
                contig = row.contig,
                start = row.start,
                end = row.end,
                length = row.length,
                mean_depth = row.mean_depth,
                breadth_1x = row.breadth_1x,
                covered_bases = row.covered_bases,
                coverage_regime = row.coverage_regime,
                row_expectation_matched = row.row_expectation_matched,
                case_expectation_matched = row.case_expectation_matched,
                regions_bed = row.regions_bed,
                coverage_tsv = row.coverage_tsv,
                coverage_summary_json = row.coverage_summary_json,
                coverage_depth = row.coverage_depth,
                coverage_mosdepth_summary = row.coverage_mosdepth_summary,
                stage_metrics = row.stage_metrics,
            )
            .map_err(|error| anyhow!("write bam.coverage local-smoke TSV row: {error}"))?;
        }
    }

    let summary_path = output_root.join("coverage.tsv");
    bijux_dna_infra::atomic_write_bytes(&summary_path, body.as_bytes())?;
    Ok(summary_path)
}

struct LocalCoverageSmokeRow {
    sample_id: String,
    region_id: String,
    contig: String,
    start: u64,
    end: u64,
    length: u64,
    mean_depth: f64,
    breadth_1x: f64,
    covered_bases: u64,
    coverage_regime: String,
    row_expectation_matched: bool,
    case_expectation_matched: bool,
    regions_bed: String,
    coverage_tsv: String,
    coverage_summary_json: String,
    coverage_depth: String,
    coverage_mosdepth_summary: String,
    stage_metrics: String,
}

fn materialize_local_coverage_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalCoverageSmokeCasePlan,
) -> Result<Vec<LocalCoverageSmokeRow>> {
    let case_out_dir = resolve_plan_path(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let coverage_summary_artifact = resolve_output_path(repo_root, &case.plan, "coverage_summary")?;
    let coverage_depth = resolve_output_path(repo_root, &case.plan, "coverage_depth")?;
    let stage_metrics = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;
    let coverage_tsv = case_out_dir.join("coverage.tsv");
    let coverage_summary_json = case_out_dir.join("coverage.summary.json");
    let coverage_regime_json = case_out_dir.join("coverage.regime.json");

    let input_bam = repo_root.join(&case.bam);
    let regions_bed = repo_root.join(&case.regions);
    let (summary, region_rows) = bijux_dna_domain_bam::summarize_tiny_bam_coverage_regions(
        &input_bam,
        Some(&regions_bed),
        &case.depth_thresholds,
    )?;
    let coverage_regime = summary.coverage_regime.clone().unwrap_or_else(|| "unknown".to_string());

    bijux_dna_infra::atomic_write_bytes(
        &coverage_depth,
        render_depth_lines(&input_bam, &regions_bed)?.as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &coverage_summary_artifact,
        render_coverage_summary_artifact(&summary, &region_rows).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &coverage_tsv,
        render_case_coverage_tsv(&region_rows).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_json(&coverage_summary_json, &summary)?;
    if let Some(regime) = summary.regime.as_ref() {
        bijux_dna_infra::atomic_write_json(&coverage_regime_json, regime)?;
    }

    let expected_by_region = case
        .expected_rows
        .iter()
        .map(|row| (row.region_id.clone(), row))
        .collect::<HashMap<_, _>>();
    let row_expectation_matched = region_rows.iter().all(|row| {
        expected_by_region
            .get(&row.region_id)
            .is_some_and(|expected| coverage_row_matches(row, expected))
    });
    let case_expectation_matched =
        row_expectation_matched && coverage_regime == case.expected_coverage_regime;

    bijux_dna_infra::atomic_write_json(
        &stage_metrics,
        &serde_json::json!({
            "schema_version": LOCAL_COVERAGE_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.coverage",
            "sample_id": case.sample_id,
            "depth_thresholds": case.depth_thresholds,
            "expected_coverage_regime": case.expected_coverage_regime,
            "observed_coverage_regime": coverage_regime,
            "mean_depth": summary.mean_depth,
            "expected_region_count": case.expected_rows.len(),
            "observed_region_count": region_rows.len(),
            "region_ids": region_rows.iter().map(|row| row.region_id.clone()).collect::<Vec<_>>(),
            "row_expectation_matched": row_expectation_matched,
            "case_expectation_matched": case_expectation_matched,
        }),
    )?;

    Ok(region_rows
        .into_iter()
        .map(|row| {
            let row_match = expected_by_region
                .get(&row.region_id)
                .is_some_and(|expected| coverage_row_matches(&row, expected));
            LocalCoverageSmokeRow {
                sample_id: case.sample_id.clone(),
                region_id: row.region_id,
                contig: row.contig,
                start: row.start,
                end: row.end,
                length: row.length,
                mean_depth: row.mean_depth,
                breadth_1x: row.breadth_1x,
                covered_bases: row.covered_bases,
                coverage_regime: summary
                    .coverage_regime
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                row_expectation_matched: row_match,
                case_expectation_matched,
                regions_bed: path_relative_to_repo(repo_root, &regions_bed),
                coverage_tsv: path_relative_to_repo(repo_root, &coverage_tsv),
                coverage_summary_json: path_relative_to_repo(repo_root, &coverage_summary_json),
                coverage_depth: path_relative_to_repo(repo_root, &coverage_depth),
                coverage_mosdepth_summary: path_relative_to_repo(
                    repo_root,
                    &coverage_summary_artifact,
                ),
                stage_metrics: path_relative_to_repo(repo_root, &stage_metrics),
            }
        })
        .collect())
}

fn coverage_row_matches(
    observed: &bijux_dna_domain_bam::BamCoverageRegionSummaryV1,
    expected: &bijux_dna_planner_bam::stage_api::LocalCoverageSmokeExpectedRow,
) -> bool {
    observed.region_id == expected.region_id
        && observed.contig == expected.contig
        && observed.start == expected.start
        && observed.end == expected.end
        && observed.length == expected.length
        && observed.covered_bases == expected.covered_bases
        && (observed.mean_depth - expected.mean_depth).abs() <= 1e-9
        && (observed.breadth_1x - expected.breadth_1x).abs() <= 1e-9
}

fn render_case_coverage_tsv(rows: &[bijux_dna_domain_bam::BamCoverageRegionSummaryV1]) -> String {
    let mut body = String::from(
        "region_id\tcontig\tstart\tend\tlength\tmean_depth\tbreadth_1x\tcovered_bases\n",
    );
    for row in rows {
        let _ = writeln!(
            body,
            "{region_id}\t{contig}\t{start}\t{end}\t{length}\t{mean_depth:.6}\t{breadth_1x:.6}\t{covered_bases}",
            region_id = row.region_id,
            contig = row.contig,
            start = row.start,
            end = row.end,
            length = row.length,
            mean_depth = row.mean_depth,
            breadth_1x = row.breadth_1x,
            covered_bases = row.covered_bases,
        );
    }
    body
}

fn render_coverage_summary_artifact(
    summary: &bijux_dna_domain_bam::BamCoverageSummaryV1,
    rows: &[bijux_dna_domain_bam::BamCoverageRegionSummaryV1],
) -> String {
    let total_positions = rows.iter().map(|row| row.length).sum::<u64>();
    let total_covered = rows.iter().map(|row| row.covered_bases).sum::<u64>();
    let mean_depth = summary.mean_depth.unwrap_or(0.0);
    format!("total\t{total_positions}\t{total_covered}\t{mean_depth:.6}\n")
}

fn resolve_output_path(
    repo_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    output_id: &str,
) -> Result<PathBuf> {
    let path = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            anyhow!("bam.coverage local-smoke plan is missing governed output `{output_id}`")
        })?;
    Ok(resolve_plan_path(repo_root, &path))
}

fn resolve_plan_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    relative_path(repo_root, path).display().to_string()
}

#[derive(Clone)]
struct TinySamDepthRecord {
    contig: String,
    pos: u64,
    seq_len: usize,
}

#[derive(Clone)]
struct TinyDepthRegion {
    contig: String,
    start: u64,
    end: u64,
}

fn render_depth_lines(input_bam: &Path, regions_bed: &Path) -> Result<String> {
    let (reference_lengths, records) = parse_tiny_depth_records(input_bam)?;
    let regions = parse_tiny_depth_regions(regions_bed)?;
    let coverage = build_coverage_vectors(&reference_lengths, &records);

    let mut body = String::new();
    for region in regions {
        let Some(depths) = coverage.get(&region.contig) else {
            return Err(anyhow!(
                "bam.coverage local-smoke depth render missing contig `{}`",
                region.contig
            ));
        };
        let region_end = usize::try_from(region.end)
            .map_err(|_| anyhow!("bam.coverage local-smoke region end exceeds platform limits"))?;
        if region_end > depths.len() {
            return Err(anyhow!(
                "bam.coverage local-smoke region extends beyond contig `{}` length {}",
                region.contig,
                depths.len()
            ));
        }
        for pos in region.start..=region.end {
            let depth_index = usize::try_from(pos.saturating_sub(1)).map_err(|_| {
                anyhow!("bam.coverage local-smoke position exceeds platform limits")
            })?;
            let depth = depths[depth_index];
            let _ = writeln!(body, "{}\t{}\t{}", region.contig, pos, depth);
        }
    }
    Ok(body)
}

fn parse_tiny_depth_records(
    input_bam: &Path,
) -> Result<(HashMap<String, u64>, Vec<TinySamDepthRecord>)> {
    let raw = std::fs::read_to_string(input_bam)?;
    let mut reference_lengths = HashMap::new();
    let mut records = Vec::new();

    for line in raw.lines() {
        if line.starts_with("@SQ") {
            let mut contig = None;
            let mut length = None;
            for field in line.split('\t').skip(1) {
                if let Some(value) = field.strip_prefix("SN:") {
                    contig = Some(value.to_string());
                } else if let Some(value) = field.strip_prefix("LN:") {
                    length = value.parse::<u64>().ok();
                }
            }
            if let (Some(contig), Some(length)) = (contig, length) {
                reference_lengths.insert(contig, length);
            }
            continue;
        }
        if line.starts_with('@') || line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 11 {
            continue;
        }
        let flag = fields[1].parse::<u16>()?;
        let contig = fields[2];
        if contig == "*" || (flag & 0x4) != 0 {
            continue;
        }
        let pos = fields[3].parse::<u64>()?;
        let seq_len = fields[9].len();
        records.push(TinySamDepthRecord { contig: contig.to_string(), pos, seq_len });
    }

    Ok((reference_lengths, records))
}

fn parse_tiny_depth_regions(path: &Path) -> Result<Vec<TinyDepthRegion>> {
    let raw = std::fs::read_to_string(path)?;
    let mut regions = Vec::new();
    for (index, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let fields = trimmed.split('\t').collect::<Vec<_>>();
        if fields.len() < 3 {
            return Err(anyhow!(
                "bam.coverage local-smoke BED line {} must declare contig, start, and end",
                index + 1
            ));
        }
        let start_zero_based = fields[1].parse::<u64>()?;
        let end_exclusive = fields[2].parse::<u64>()?;
        if end_exclusive <= start_zero_based {
            return Err(anyhow!(
                "bam.coverage local-smoke BED line {} must keep end greater than start",
                index + 1
            ));
        }
        regions.push(TinyDepthRegion {
            contig: fields[0].to_string(),
            start: start_zero_based + 1,
            end: end_exclusive,
        });
    }
    Ok(regions)
}

#[allow(clippy::cast_possible_truncation)]
fn build_coverage_vectors(
    reference_lengths: &HashMap<String, u64>,
    records: &[TinySamDepthRecord],
) -> HashMap<String, Vec<u32>> {
    let mut inferred_lengths = HashMap::<String, u64>::new();
    for record in records {
        let end = record.pos + u64::max(record.seq_len as u64, 1) - 1;
        let current = inferred_lengths.entry(record.contig.clone()).or_insert(0);
        *current = (*current).max(end);
    }

    let mut coverage = HashMap::<String, Vec<u32>>::new();
    for (contig, declared_length) in reference_lengths {
        let inferred_length = inferred_lengths.get(contig).copied().unwrap_or(0);
        let length = (*declared_length).max(inferred_length).max(1) as usize;
        coverage.insert(contig.clone(), vec![0; length]);
    }
    for (contig, inferred_length) in &inferred_lengths {
        coverage
            .entry(contig.clone())
            .or_insert_with(|| vec![0; (*inferred_length).max(1) as usize]);
    }

    for record in records {
        let Some(depths) = coverage.get_mut(&record.contig) else {
            continue;
        };
        let start = record.pos.saturating_sub(1) as usize;
        let end = usize::min(start.saturating_add(usize::max(record.seq_len, 1)), depths.len());
        for depth in depths.iter_mut().take(end).skip(start) {
            *depth += 1;
        }
    }

    coverage
}
