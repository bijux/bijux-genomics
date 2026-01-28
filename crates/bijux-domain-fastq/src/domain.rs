use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FastqArtifactKind {
    SingleEnd,
    PairedEnd,
    Merged,
    StatsOnly,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FastqArtifact {
    pub path: PathBuf,
    pub kind: FastqArtifactKind,
}

impl FastqArtifact {
    pub fn single_end(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            kind: FastqArtifactKind::SingleEnd,
        }
    }

    pub fn merged(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            kind: FastqArtifactKind::Merged,
        }
    }

    pub fn paired_end(r1: impl Into<PathBuf>, r2: impl Into<PathBuf>) -> (Self, Self) {
        (
            Self {
                path: r1.into(),
                kind: FastqArtifactKind::PairedEnd,
            },
            Self {
                path: r2.into(),
                kind: FastqArtifactKind::PairedEnd,
            },
        )
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct FastqStageContract {
    pub input_kind: FastqArtifactKind,
    pub output_kind: FastqArtifactKind,
    pub may_drop_reads: bool,
    pub must_preserve_pairing: bool,
    pub emits_fastq: bool,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum QcClass {
    Structural,
    Statistical,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricDescriptor {
    pub value: serde_json::Value,
    pub meaning: &'static str,
    pub range: &'static str,
    pub notes: &'static str,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct IntegrityMetrics {
    pub reads_in: MetricDescriptor,
    pub reads_out: MetricDescriptor,
    pub bases_in: MetricDescriptor,
    pub bases_out: MetricDescriptor,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RetentionMetrics {
    pub read_retention: MetricDescriptor,
    pub base_retention: MetricDescriptor,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct QualityShiftMetrics {
    pub mean_q_before: MetricDescriptor,
    pub mean_q_after: MetricDescriptor,
    pub delta_mean_q: MetricDescriptor,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ContaminationMetrics {
    pub gc_before: MetricDescriptor,
    pub gc_after: MetricDescriptor,
    pub delta_gc: MetricDescriptor,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SemanticMetrics {
    pub integrity: IntegrityMetrics,
    pub retention: RetentionMetrics,
    pub quality_shift: Option<QualityShiftMetrics>,
    pub contamination: Option<ContaminationMetrics>,
    pub interpretation: &'static str,
}

fn metric_u64(
    value: u64,
    meaning: &'static str,
    range: &'static str,
    notes: &'static str,
) -> MetricDescriptor {
    MetricDescriptor {
        value: serde_json::json!(value),
        meaning,
        range,
        notes,
    }
}

fn metric_f64(
    value: f64,
    meaning: &'static str,
    range: &'static str,
    notes: &'static str,
) -> MetricDescriptor {
    MetricDescriptor {
        value: serde_json::json!(value),
        meaning,
        range,
        notes,
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FastqDelta {
    pub delta_mean_q: f64,
    pub delta_gc: f64,
    pub read_retention: f64,
    pub base_retention: f64,
}

impl FastqDelta {
    /// Validate delta metrics are sane.
    ///
    /// # Errors
    /// Returns an error if any delta values are invalid.
    pub fn validate(&self) -> Result<()> {
        if !self.delta_mean_q.is_finite() {
            return Err(anyhow!("delta_mean_q must be finite"));
        }
        if !self.delta_gc.is_finite() {
            return Err(anyhow!("delta_gc must be finite"));
        }
        if !(0.0..=1.0).contains(&self.read_retention) {
            return Err(anyhow!("read_retention must be within [0, 1]"));
        }
        if !(0.0..=1.0).contains(&self.base_retention) {
            return Err(anyhow!("base_retention must be within [0, 1]"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeaderInspection {
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NormalizedOutputs {
    pub r1: Option<PathBuf>,
    pub r2: Option<PathBuf>,
    pub merged: Option<PathBuf>,
}

#[must_use]
pub fn contract_for_stage(stage_id: &str) -> Option<FastqStageContract> {
    match stage_id {
        "fastq.trim" | "fastq.filter" => Some(FastqStageContract {
            input_kind: FastqArtifactKind::SingleEnd,
            output_kind: FastqArtifactKind::SingleEnd,
            may_drop_reads: true,
            must_preserve_pairing: true,
            emits_fastq: true,
        }),
        "fastq.merge" => Some(FastqStageContract {
            input_kind: FastqArtifactKind::PairedEnd,
            output_kind: FastqArtifactKind::Merged,
            may_drop_reads: true,
            must_preserve_pairing: false,
            emits_fastq: true,
        }),
        "fastq.correct" => Some(FastqStageContract {
            input_kind: FastqArtifactKind::PairedEnd,
            output_kind: FastqArtifactKind::PairedEnd,
            may_drop_reads: false,
            must_preserve_pairing: true,
            emits_fastq: true,
        }),
        "fastq.umi" | "fastq.preprocess" => Some(FastqStageContract {
            input_kind: FastqArtifactKind::PairedEnd,
            output_kind: FastqArtifactKind::PairedEnd,
            may_drop_reads: true,
            must_preserve_pairing: true,
            emits_fastq: true,
        }),
        "fastq.validate" | "fastq.stats" | "fastq.qc2" | "fastq.screen" => {
            Some(FastqStageContract {
                input_kind: FastqArtifactKind::SingleEnd,
                output_kind: FastqArtifactKind::StatsOnly,
                may_drop_reads: false,
                must_preserve_pairing: true,
                emits_fastq: false,
            })
        }
        _ => None,
    }
}

#[must_use]
pub fn qc_class_for_stage(stage_id: &str) -> Option<QcClass> {
    match stage_id {
        "fastq.validate" => Some(QcClass::Structural),
        "fastq.qc2" => Some(QcClass::Statistical),
        _ => None,
    }
}

#[must_use]
pub fn infer_input_kind(r2: Option<&Path>) -> FastqArtifactKind {
    if r2.is_some() {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    }
}

#[must_use]
pub fn compute_delta(
    before: bijux_engine::api::SeqkitMetrics,
    after: bijux_engine::api::SeqkitMetrics,
) -> FastqDelta {
    let read_retention = if before.reads > 0 {
        ratio_u64(after.reads, before.reads)
    } else {
        0.0
    };
    let base_retention = if before.bases > 0 {
        ratio_u64(after.bases, before.bases)
    } else {
        0.0
    };
    FastqDelta {
        delta_mean_q: after.mean_q - before.mean_q,
        delta_gc: after.gc_percent - before.gc_percent,
        read_retention,
        base_retention,
    }
}

#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn delta_from_counts(
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
    mean_q_before: f64,
    mean_q_after: f64,
    gc_before: f64,
    gc_after: f64,
) -> FastqDelta {
    let read_retention = if reads_in > 0 {
        ratio_u64(reads_out, reads_in)
    } else {
        0.0
    };
    let base_retention = if bases_in > 0 {
        ratio_u64(bases_out, bases_in)
    } else {
        0.0
    };
    FastqDelta {
        delta_mean_q: mean_q_after - mean_q_before,
        delta_gc: gc_after - gc_before,
        read_retention,
        base_retention,
    }
}

#[must_use]
pub fn semantic_trim(metrics: &bijux_analyze::FastqTrimMetrics) -> SemanticMetrics {
    let delta = delta_from_counts(
        metrics.reads_in,
        metrics.reads_out,
        metrics.bases_in,
        metrics.bases_out,
        metrics.mean_q_before,
        metrics.mean_q_after,
        0.0,
        0.0,
    );
    SemanticMetrics {
        integrity: IntegrityMetrics {
            reads_in: metric_u64(
                metrics.reads_in,
                "Input reads",
                ">= 0",
                "Raw input read count.",
            ),
            reads_out: metric_u64(
                metrics.reads_out,
                "Output reads",
                ">= 0",
                "Reads after trimming.",
            ),
            bases_in: metric_u64(metrics.bases_in, "Input bases", ">= 0", "Raw input bases."),
            bases_out: metric_u64(
                metrics.bases_out,
                "Output bases",
                ">= 0",
                "Bases after trimming.",
            ),
        },
        retention: RetentionMetrics {
            read_retention: metric_f64(
                delta.read_retention,
                "Reads retained",
                "[0,1]",
                "Fraction of reads retained.",
            ),
            base_retention: metric_f64(
                delta.base_retention,
                "Bases retained",
                "[0,1]",
                "Fraction of bases retained.",
            ),
        },
        quality_shift: Some(QualityShiftMetrics {
            mean_q_before: metric_f64(
                metrics.mean_q_before,
                "Mean Q before",
                "[0,45]",
                "Mean quality before trimming.",
            ),
            mean_q_after: metric_f64(
                metrics.mean_q_after,
                "Mean Q after",
                "[0,45]",
                "Mean quality after trimming.",
            ),
            delta_mean_q: metric_f64(
                delta.delta_mean_q,
                "Mean Q shift",
                "(-45,45)",
                "Mean quality change.",
            ),
        }),
        contamination: None,
        interpretation: "Trim should increase mean quality and may reduce read/base counts.",
    }
}

#[must_use]
pub fn semantic_filter(metrics: &bijux_analyze::FastqFilterMetrics) -> SemanticMetrics {
    let delta = delta_from_counts(
        metrics.reads_in,
        metrics.reads_out,
        metrics.reads_in,
        metrics.reads_out,
        metrics.mean_q_before,
        metrics.mean_q_after,
        0.0,
        0.0,
    );
    SemanticMetrics {
        integrity: IntegrityMetrics {
            reads_in: metric_u64(
                metrics.reads_in,
                "Input reads",
                ">= 0",
                "Raw input read count.",
            ),
            reads_out: metric_u64(
                metrics.reads_out,
                "Output reads",
                ">= 0",
                "Reads after filtering.",
            ),
            bases_in: metric_u64(
                metrics.reads_in,
                "Input bases",
                ">= 0",
                "Approx bases inferred from reads.",
            ),
            bases_out: metric_u64(
                metrics.reads_out,
                "Output bases",
                ">= 0",
                "Approx bases after filtering.",
            ),
        },
        retention: RetentionMetrics {
            read_retention: metric_f64(
                delta.read_retention,
                "Reads retained",
                "[0,1]",
                "Fraction of reads retained.",
            ),
            base_retention: metric_f64(
                delta.base_retention,
                "Bases retained",
                "[0,1]",
                "Fraction of bases retained.",
            ),
        },
        quality_shift: Some(QualityShiftMetrics {
            mean_q_before: metric_f64(
                metrics.mean_q_before,
                "Mean Q before",
                "[0,45]",
                "Mean quality before filtering.",
            ),
            mean_q_after: metric_f64(
                metrics.mean_q_after,
                "Mean Q after",
                "[0,45]",
                "Mean quality after filtering.",
            ),
            delta_mean_q: metric_f64(
                delta.delta_mean_q,
                "Mean Q shift",
                "(-45,45)",
                "Mean quality change.",
            ),
        }),
        contamination: None,
        interpretation: "Filter should drop low-quality reads and improve mean quality.",
    }
}

#[must_use]
pub fn semantic_validate(metrics: &bijux_analyze::FastqValidateMetrics) -> SemanticMetrics {
    let reads_in = metrics.reads_total;
    let reads_out = metrics.reads_valid;
    let delta = delta_from_counts(reads_in, reads_out, reads_in, reads_out, 0.0, 0.0, 0.0, 0.0);
    SemanticMetrics {
        integrity: IntegrityMetrics {
            reads_in: metric_u64(reads_in, "Total reads", ">= 0", "Total reads inspected."),
            reads_out: metric_u64(
                reads_out,
                "Valid reads",
                ">= 0",
                "Reads passing validation.",
            ),
            bases_in: metric_u64(0, "Input bases", ">= 0", "Not computed in validation."),
            bases_out: metric_u64(0, "Output bases", ">= 0", "Not computed in validation."),
        },
        retention: RetentionMetrics {
            read_retention: metric_f64(
                delta.read_retention,
                "Valid read ratio",
                "[0,1]",
                "Fraction of valid reads.",
            ),
            base_retention: metric_f64(
                delta.base_retention,
                "Base retention",
                "[0,1]",
                "Not computed in validation.",
            ),
        },
        quality_shift: None,
        contamination: None,
        interpretation: "Validation should report invalid reads; no data is modified.",
    }
}

#[must_use]
pub fn semantic_stats(metrics: &bijux_analyze::FastqStatsMetrics) -> SemanticMetrics {
    SemanticMetrics {
        integrity: IntegrityMetrics {
            reads_in: metric_u64(metrics.reads_total, "Input reads", ">= 0", "Total reads."),
            reads_out: metric_u64(
                metrics.reads_total,
                "Output reads",
                ">= 0",
                "Stats stage does not modify reads.",
            ),
            bases_in: metric_u64(metrics.bases_total, "Input bases", ">= 0", "Total bases."),
            bases_out: metric_u64(
                metrics.bases_total,
                "Output bases",
                ">= 0",
                "Stats stage does not modify bases.",
            ),
        },
        retention: RetentionMetrics {
            read_retention: metric_f64(
                1.0,
                "Reads retained",
                "[0,1]",
                "Stats is non-transforming.",
            ),
            base_retention: metric_f64(
                1.0,
                "Bases retained",
                "[0,1]",
                "Stats is non-transforming.",
            ),
        },
        quality_shift: Some(QualityShiftMetrics {
            mean_q_before: metric_f64(metrics.mean_q, "Mean Q", "[0,45]", "Mean quality."),
            mean_q_after: metric_f64(metrics.mean_q, "Mean Q", "[0,45]", "Mean quality."),
            delta_mean_q: metric_f64(
                0.0,
                "Mean Q shift",
                "0",
                "Stats stage does not alter reads.",
            ),
        }),
        contamination: Some(ContaminationMetrics {
            gc_before: metric_f64(metrics.gc_percent, "GC%", "[0,100]", "GC percentage."),
            gc_after: metric_f64(metrics.gc_percent, "GC%", "[0,100]", "GC percentage."),
            delta_gc: metric_f64(0.0, "GC shift", "0", "Stats stage does not alter reads."),
        }),
        interpretation: "Stats is observational; values describe the input only.",
    }
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn ratio_u64(num: u64, denom: u64) -> f64 {
    if denom == 0 {
        0.0
    } else {
        (num as f64) / (denom as f64)
    }
}

/// Validate that a stage can accept the provided input kind.
///
/// # Errors
/// Returns an error if the stage contract is violated.
pub fn preflight_stage(stage_id: &str, input_kind: FastqArtifactKind) -> Result<()> {
    let Some(contract) = contract_for_stage(stage_id) else {
        return Ok(());
    };
    match contract.input_kind {
        FastqArtifactKind::SingleEnd => {
            if input_kind == FastqArtifactKind::PairedEnd && stage_id != "fastq.stats" {
                return Err(anyhow!("stage {stage_id} does not accept paired-end input"));
            }
        }
        FastqArtifactKind::PairedEnd => {
            if input_kind != FastqArtifactKind::PairedEnd {
                return Err(anyhow!("stage {stage_id} requires paired-end input"));
            }
        }
        FastqArtifactKind::Merged => {
            if input_kind != FastqArtifactKind::Merged {
                return Err(anyhow!("stage {stage_id} requires merged input"));
            }
        }
        FastqArtifactKind::StatsOnly => {}
    }
    Ok(())
}

/// Inspect FASTQ headers for pairing and style drift.
///
/// # Errors
/// Returns an error in strict mode if mismatches are detected.
pub fn inspect_headers(r1: &Path, r2: Option<&Path>, strict: bool) -> Result<HeaderInspection> {
    let mut warnings = Vec::new();
    let r1_names = read_header_names(r1, 16)?;
    if let Some(r2) = r2 {
        let r2_names = read_header_names(r2, 16)?;
        for (idx, (left, right)) in r1_names.iter().zip(r2_names.iter()).enumerate() {
            if normalize_header(left) != normalize_header(right) {
                let msg = format!("pairing mismatch at record {idx}: {left} vs {right}");
                if strict {
                    return Err(anyhow!(msg));
                }
                warnings.push(msg);
                break;
            }
        }
    }
    if has_style_drift(&r1_names) {
        let msg = "header style drift detected".to_string();
        if strict {
            return Err(anyhow!(msg));
        }
        warnings.push(msg);
    }
    Ok(HeaderInspection { warnings })
}

pub fn log_header_warnings(stage_id: &str, inspection: &HeaderInspection) {
    for warning in &inspection.warnings {
        warn!(stage = stage_id, "{warning}");
    }
}

/// Normalize FASTQ outputs to canonical names for a stage.
///
/// # Errors
/// Returns an error if expected outputs are missing.
pub fn normalize_outputs(
    stage_id: &str,
    out_dir: &Path,
    output_kind: FastqArtifactKind,
) -> Result<NormalizedOutputs> {
    if let Some(contract) = contract_for_stage(stage_id) {
        if !contract.emits_fastq {
            return Ok(NormalizedOutputs {
                r1: None,
                r2: None,
                merged: None,
            });
        }
    }
    match output_kind {
        FastqArtifactKind::SingleEnd => {
            let src = find_first_fastq(out_dir)?;
            let dst = out_dir.join("reads.fastq.gz");
            let final_path = rename_if_needed(&src, &dst)?;
            Ok(NormalizedOutputs {
                r1: Some(final_path),
                r2: None,
                merged: None,
            })
        }
        FastqArtifactKind::PairedEnd => {
            let (r1, r2) = find_fastq_pair(out_dir)?;
            let r1_dst = out_dir.join("reads_r1.fastq.gz");
            let r2_dst = out_dir.join("reads_r2.fastq.gz");
            let r1_final = rename_if_needed(&r1, &r1_dst)?;
            let r2_final = rename_if_needed(&r2, &r2_dst)?;
            Ok(NormalizedOutputs {
                r1: Some(r1_final),
                r2: Some(r2_final),
                merged: None,
            })
        }
        FastqArtifactKind::Merged => {
            let src = find_first_fastq(out_dir)?;
            let dst = out_dir.join("merged.fastq.gz");
            let final_path = rename_if_needed(&src, &dst)?;
            Ok(NormalizedOutputs {
                r1: None,
                r2: None,
                merged: Some(final_path),
            })
        }
        FastqArtifactKind::StatsOnly => Ok(NormalizedOutputs {
            r1: None,
            r2: None,
            merged: None,
        }),
    }
}

/// Locate the first FASTQ-like output in a directory.
///
/// # Errors
/// Returns an error if no output FASTQ is found.
pub fn find_first_fastq(dir: &Path) -> Result<PathBuf> {
    let entries = std::fs::read_dir(dir)
        .map_err(|err| anyhow!("failed to read output directory {}: {err}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
            if ext.eq_ignore_ascii_case("fq")
                || ext.eq_ignore_ascii_case("fastq")
                || ext.eq_ignore_ascii_case("gz")
            {
                return Ok(path);
            }
        }
    }
    Err(anyhow!("no FASTQ output found in {}", dir.display()))
}

fn rename_if_needed(src: &Path, dst: &Path) -> Result<PathBuf> {
    if src == dst {
        return Ok(src.to_path_buf());
    }
    if dst.exists() {
        return Ok(dst.to_path_buf());
    }
    std::fs::rename(src, dst).map_err(|err| {
        anyhow!(
            "rename {} -> {} failed: {err}",
            src.display(),
            dst.display()
        )
    })?;
    Ok(dst.to_path_buf())
}

fn find_fastq_pair(dir: &Path) -> Result<(PathBuf, PathBuf)> {
    let mut candidates: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("gz") || ext.eq_ignore_ascii_case("fastq")
                })
        })
        .collect();
    candidates.sort();
    let mut r1 = None;
    let mut r2 = None;
    for path in &candidates {
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name.contains("R1") || name.contains("_1") || name.contains("r1") {
            r1 = Some(path.clone());
        } else if name.contains("R2") || name.contains("_2") || name.contains("r2") {
            r2 = Some(path.clone());
        }
    }
    if (r1.is_none() || r2.is_none()) && candidates.len() == 2 {
        r1 = Some(candidates[0].clone());
        r2 = Some(candidates[1].clone());
    }
    match (r1, r2) {
        (Some(a), Some(b)) => Ok((a, b)),
        _ => Err(anyhow!("failed to locate paired FASTQ outputs")),
    }
}

fn read_header_names(path: &Path, max_records: usize) -> Result<Vec<String>> {
    let data = std::fs::read_to_string(path)?;
    let mut names = Vec::new();
    for (idx, line) in data.lines().enumerate() {
        if idx % 4 == 0 {
            if let Some(name) = line.strip_prefix('@') {
                names.push(name.trim().to_string());
                if names.len() >= max_records {
                    break;
                }
            }
        }
    }
    Ok(names)
}

fn normalize_header(name: &str) -> String {
    let name = name.split_whitespace().next().unwrap_or(name);
    name.trim_end_matches("/1")
        .trim_end_matches("/2")
        .to_string()
}

fn has_style_drift(names: &[String]) -> bool {
    if names.is_empty() {
        return false;
    }
    let first = normalize_header(&names[0]);
    names.iter().any(|name| normalize_header(name) != first)
}
