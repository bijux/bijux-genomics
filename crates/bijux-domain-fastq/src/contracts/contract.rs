use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use tracing::warn;

use super::model::FastqArtifactKind;
use super::stages::FastqStageContract;

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
        "fastq.validate_pre" | "fastq.stats_neutral" | "fastq.qc_post" | "fastq.screen" => {
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
