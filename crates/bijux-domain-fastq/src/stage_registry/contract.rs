use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use tracing::warn;

use crate::types::FastqArtifactKind;
use super::FastqStageContract;

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
        "fastq.validate_pre"
        | "fastq.detect_adapters"
        | "fastq.stats_neutral"
        | "fastq.qc_post"
        | "fastq.screen" => Some(FastqStageContract {
            input_kind: FastqArtifactKind::SingleEnd,
            output_kind: FastqArtifactKind::StatsOnly,
            may_drop_reads: false,
            must_preserve_pairing: true,
            emits_fastq: false,
        }),
        _ => None,
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
            if input_kind == FastqArtifactKind::PairedEnd && stage_id != "fastq.stats_neutral" {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MergeSuitability {
    pub suitable: bool,
    pub reason: String,
    pub r1_mean_len: Option<usize>,
    pub r2_mean_len: Option<usize>,
    pub predicted_merge_rate: Option<f64>,
    pub probe_pairs: Option<usize>,
}

/// Assess whether paired-end reads are suitable for merging.
///
/// # Errors
/// Returns an error if inputs cannot be read.
pub fn assess_merge_suitability(r1: &Path, r2: &Path) -> Result<MergeSuitability> {
    let (r1_mean, r2_mean) = read_sequence_length_means(r1, r2, 64)?;
    let (predicted_merge_rate, probe_pairs) = estimate_overlap_rate(r1, r2, 64)?;
    let (Some(r1_len), Some(r2_len)) = (r1_mean, r2_mean) else {
        return Ok(MergeSuitability {
            suitable: false,
            reason: "missing read length samples".to_string(),
            r1_mean_len: r1_mean,
            r2_mean_len: r2_mean,
            predicted_merge_rate,
            probe_pairs,
        });
    };
    if r1_len == 0 || r2_len == 0 {
        return Ok(MergeSuitability {
            suitable: false,
            reason: "zero-length reads detected".to_string(),
            r1_mean_len: r1_mean,
            r2_mean_len: r2_mean,
            predicted_merge_rate,
            probe_pairs,
        });
    }
    if r1_len != r2_len {
        return Ok(MergeSuitability {
            suitable: false,
            reason: "read lengths differ between R1 and R2".to_string(),
            r1_mean_len: r1_mean,
            r2_mean_len: r2_mean,
            predicted_merge_rate,
            probe_pairs,
        });
    }
    let overlap_threshold = 0.05;
    let suitable =
        r1_len <= 150 && predicted_merge_rate.is_none_or(|rate| rate >= overlap_threshold);
    let reason = if let Some(rate) = predicted_merge_rate {
        if rate < overlap_threshold {
            format!("overlap probe predicts merge rate {rate:.2} < {overlap_threshold:.2}")
        } else if r1_len <= 150 {
            "read length suggests overlap is likely".to_string()
        } else {
            "read length suggests overlap is unlikely".to_string()
        }
    } else if r1_len <= 150 {
        "read length suggests overlap is likely".to_string()
    } else {
        "read length suggests overlap is unlikely".to_string()
    };
    Ok(MergeSuitability {
        suitable,
        reason,
        r1_mean_len: r1_mean,
        r2_mean_len: r2_mean,
        predicted_merge_rate,
        probe_pairs,
    })
}

/// Ensure UMI headers are present before UMI stage execution.
///
/// # Errors
/// Returns an error if UMI markers are not detected and override is not set.
pub fn ensure_umi_headers(r1: &Path, r2: Option<&Path>) -> Result<()> {
    let mut names = read_header_names(r1, 32)?;
    if let Some(r2) = r2 {
        names.extend(read_header_names(r2, 32)?);
    }
    let markers = ["UMI", "RX:", "BX:", "UB:"];
    let has_marker = names
        .iter()
        .any(|name| markers.iter().any(|m| name.contains(m)));
    if has_marker {
        return Ok(());
    }
    if std::env::var("BIJUX_ALLOW_NO_UMI").is_ok() {
        warn!("UMI headers not detected; proceeding due to BIJUX_ALLOW_NO_UMI");
        return Ok(());
    }
    Err(anyhow!(
        "UMI headers not detected; set BIJUX_ALLOW_NO_UMI=1 to bypass"
    ))
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

fn read_sequence_length_means(
    r1: &Path,
    r2: &Path,
    max_records: usize,
) -> Result<(Option<usize>, Option<usize>)> {
    let r1_lengths = read_sequence_lengths(r1, max_records)?;
    let r2_lengths = read_sequence_lengths(r2, max_records)?;
    Ok((mean_length(&r1_lengths), mean_length(&r2_lengths)))
}

fn read_sequence_lengths(path: &Path, max_records: usize) -> Result<Vec<usize>> {
    let data = std::fs::read_to_string(path)?;
    let mut lengths = Vec::new();
    for (idx, line) in data.lines().enumerate() {
        if idx % 4 == 1 {
            lengths.push(line.trim().len());
            if lengths.len() >= max_records {
                break;
            }
        }
    }
    Ok(lengths)
}

fn read_sequences(path: &Path, max_records: usize) -> Result<Vec<String>> {
    let data = std::fs::read_to_string(path)?;
    let mut seqs = Vec::new();
    for (idx, line) in data.lines().enumerate() {
        if idx % 4 == 1 {
            seqs.push(line.trim().to_string());
            if seqs.len() >= max_records {
                break;
            }
        }
    }
    Ok(seqs)
}

fn estimate_overlap_rate(
    r1: &Path,
    r2: &Path,
    max_pairs: usize,
) -> Result<(Option<f64>, Option<usize>)> {
    let r1_seqs = read_sequences(r1, max_pairs)?;
    let r2_seqs = read_sequences(r2, max_pairs)?;
    let pairs = r1_seqs.len().min(r2_seqs.len());
    if pairs == 0 {
        return Ok((None, None));
    }
    let mut overlaps = 0_usize;
    for idx in 0..pairs {
        let a = &r1_seqs[idx];
        let b = reverse_complement(&r2_seqs[idx]);
        if has_overlap(a, &b, 10) {
            overlaps += 1;
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let rate = overlaps as f64 / pairs as f64;
    Ok((Some(rate), Some(pairs)))
}

fn reverse_complement(seq: &str) -> String {
    seq.chars()
        .rev()
        .map(|c| match c {
            'A' | 'a' => 'T',
            'C' | 'c' => 'G',
            'G' | 'g' => 'C',
            'T' | 't' => 'A',
            'N' | 'n' => 'N',
            other => other,
        })
        .collect()
}

fn has_overlap(a: &str, b: &str, min_len: usize) -> bool {
    let max_len = a.len().min(b.len());
    for len in (min_len..=max_len).rev() {
        if a.ends_with(&b[..len]) {
            return true;
        }
    }
    false
}
fn mean_length(lengths: &[usize]) -> Option<usize> {
    if lengths.is_empty() {
        return None;
    }
    let sum: usize = lengths.iter().sum();
    Some(sum / lengths.len())
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
