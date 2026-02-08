use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::foundation::{ContractVersion, Result};
use bijux_infra::atomic_write_bytes;
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FastqLayout {
    SingleEnd,
    PairedEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastqSampleId {
    pub sample_name: String,
    pub layout: FastqLayout,
    pub r1_path: PathBuf,
    pub r2_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqFileAssessment {
    pub path: PathBuf,
    pub gzip: bool,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqSampleAssessment {
    pub id: FastqSampleId,
    pub r1: FastqFileAssessment,
    pub r2: Option<FastqFileAssessment>,
    pub naming_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputAssessmentV1 {
    pub schema_version: u32,
    #[serde(default = "ContractVersion::v1")]
    pub contract_version: ContractVersion,
    pub created_at: String,
    pub samples: Vec<FastqSampleAssessment>,
    pub unpaired_files: Vec<PathBuf>,
    pub issues: Vec<String>,
}

#[must_use]
pub fn discover_fastq_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| is_fastq_path(path))
        .collect()
}

#[must_use]
pub fn is_fastq_path(path: &Path) -> bool {
    let ext = path.extension().and_then(|value| value.to_str());
    if let Some(ext) = ext {
        if ext.eq_ignore_ascii_case("fastq") || ext.eq_ignore_ascii_case("fq") {
            return true;
        }
        if ext.eq_ignore_ascii_case("gz") {
            let stem = path.file_stem().and_then(|value| value.to_str());
            if let Some(stem) = stem {
                let stem_ext = Path::new(stem).extension().and_then(|value| value.to_str());
                return stem_ext.is_some_and(|value| {
                    value.eq_ignore_ascii_case("fastq") || value.eq_ignore_ascii_case("fq")
                });
            }
        }
    }
    false
}

#[must_use]
pub fn is_gzip_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
}

fn file_assessment(path: &Path) -> Result<FastqFileAssessment> {
    let size_bytes = std::fs::metadata(path)?.len();
    let sha256 = hash_file_sha256(path)?;
    Ok(FastqFileAssessment {
        path: path.to_path_buf(),
        gzip: is_gzip_path(path),
        size_bytes,
        sha256,
    })
}

fn infer_sample_key(re: &Regex, path: &Path) -> (String, Option<u8>) {
    let filename = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_string();
    if let Some(caps) = re.captures(&filename) {
        let base = caps
            .name("base")
            .map_or_else(|| filename.clone(), |value| value.as_str().to_string());
        let read = caps
            .name("read")
            .and_then(|value| value.as_str().parse::<u8>().ok());
        return (base, read);
    }
    (filename, None)
}

/// Assess FASTQ inputs under a directory.
///
/// # Errors
/// Returns an error if regex compilation or hashing fails.
pub fn assess_input_dir(root: &Path) -> Result<InputAssessmentV1> {
    let mut issues = Vec::new();
    let mut unpaired = Vec::new();
    let mut grouped: BTreeMap<String, Vec<(PathBuf, Option<u8>)>> = BTreeMap::new();
    let re =
        Regex::new(r"(?i)^(?P<base>.+?)(?:[._-](?:R)?(?P<read>[12]))?(?:\.f(?:ast)?q)?(?:\.gz)?$")?;
    for path in discover_fastq_files(root) {
        let (key, read) = infer_sample_key(&re, &path);
        grouped.entry(key).or_default().push((path, read));
    }

    let mut samples = Vec::new();
    for (sample_name, files) in grouped {
        let mut r1 = None;
        let mut r2 = None;
        let mut naming_warnings = Vec::new();
        for (path, read) in files {
            match read {
                Some(1) => {
                    if r1.is_some() {
                        naming_warnings.push(format!("multiple R1 candidates for {sample_name}"));
                    }
                    r1 = Some(path);
                }
                Some(2) => {
                    if r2.is_some() {
                        naming_warnings.push(format!("multiple R2 candidates for {sample_name}"));
                    }
                    r2 = Some(path);
                }
                _ => {
                    if r1.is_some() {
                        naming_warnings.push(format!("ambiguous FASTQ filename for {sample_name}"));
                    } else {
                        r1 = Some(path);
                    }
                }
            }
        }
        let Some(r1_path) = r1 else {
            issues.push(format!("sample {sample_name} missing R1"));
            continue;
        };
        let (layout, r2_path) = match r2 {
            Some(r2_path) => (FastqLayout::PairedEnd, Some(r2_path)),
            None => (FastqLayout::SingleEnd, None),
        };
        if layout == FastqLayout::PairedEnd && r2_path.is_none() {
            issues.push(format!("sample {sample_name} missing R2"));
        }
        let r1_assessment = file_assessment(&r1_path)?;
        let r2_assessment = match r2_path.as_ref() {
            Some(path) => Some(file_assessment(path)?),
            None => None,
        };
        let assessment = FastqSampleAssessment {
            id: FastqSampleId {
                sample_name: sample_name.clone(),
                layout,
                r1_path: r1_path.clone(),
                r2_path: r2_path.clone(),
            },
            r1: r1_assessment,
            r2: r2_assessment,
            naming_warnings,
        };
        samples.push(assessment);
    }

    for sample in &samples {
        if sample.id.layout == FastqLayout::SingleEnd {
            if let Some(r2) = sample.id.r2_path.clone() {
                unpaired.push(r2);
            }
        }
    }

    Ok(InputAssessmentV1 {
        schema_version: 1,
        contract_version: ContractVersion::v1(),
        created_at: Utc::now().to_rfc3339(),
        samples,
        unpaired_files: unpaired,
        issues,
    })
}

/// Write the input assessment to disk.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_input_assessment(path: &Path, assessment: &InputAssessmentV1) -> Result<()> {
    let payload = serde_json::to_string_pretty(assessment)?;
    atomic_write_bytes(path, payload.as_bytes()).map_err(|err| {
        crate::foundation::BijuxError::Io(format!("write input assessment: {err}"))
    })?;
    Ok(())
}

fn hash_file_sha256(path: &Path) -> Result<String> {
    use sha2::Digest;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = sha2::Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}
