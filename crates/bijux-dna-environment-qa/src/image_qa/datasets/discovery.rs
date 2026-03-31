use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::image_qa::support::SeqkitMetrics;
use crate::image_qa::QaDataset;

fn fastq_corpus_root() -> PathBuf {
    std::env::var_os("BIJUX_FASTQ_CORPUS_ROOT")
        .map_or_else(|| PathBuf::from("artifacts/corpus/fastq"), PathBuf::from)
}

pub(crate) fn discover_qa_datasets() -> Result<Vec<QaDataset>> {
    let root = fastq_corpus_root();
    let canonical = root.join("canonical");
    if canonical.exists() {
        return discover_canonical_datasets(&canonical);
    }
    if !root.exists() {
        return Err(anyhow!(
            "FASTQ corpus root not found at {} (set BIJUX_FASTQ_CORPUS_ROOT to override)",
            root.display()
        ));
    }
    let mut datasets = Vec::new();
    for entry in std::fs::read_dir(&root)
        .with_context(|| format!("read FASTQ corpus root {}", root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow!("invalid dataset name"))?
                .to_string();
            let r1 = path.join(format!("{name}_1.fastq.gz"));
            let r2 = path.join(format!("{name}_2.fastq.gz"));
            if !r1.exists() {
                continue;
            }
            let r2 = if r2.exists() { Some(r2) } else { None };
            datasets.push(QaDataset {
                name,
                r1,
                r2,
                r1_dir: path.clone(),
                input_hash_r1: String::new(),
                input_hash_r2: None,
                input_stats_r1: SeqkitMetrics {
                    reads: 0,
                    bases: 0,
                    mean_q: 0.0,
                    gc_percent: 0.0,
                },
                input_stats_r2: None,
            });
        } else if path.is_file() && is_fastq_gz(&path) {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow!("invalid dataset name"))?
                .to_string();
            let r1_dir = path.parent().ok_or_else(|| anyhow!("missing parent"))?;
            datasets.push(QaDataset {
                name,
                r1: path.clone(),
                r2: None,
                r1_dir: r1_dir.to_path_buf(),
                input_hash_r1: String::new(),
                input_hash_r2: None,
                input_stats_r1: SeqkitMetrics {
                    reads: 0,
                    bases: 0,
                    mean_q: 0.0,
                    gc_percent: 0.0,
                },
                input_stats_r2: None,
            });
        }
    }

    if datasets.is_empty() {
        return Err(anyhow!("no FASTQ files found in {}", root.display()));
    }
    Ok(datasets)
}

fn discover_canonical_datasets(root: &Path) -> Result<Vec<QaDataset>> {
    let se = root.join("BIJUX_SE_R1.fastq.gz");
    let pe_r1 = root.join("BIJUX_PE_R1.fastq.gz");
    let pe_r2 = root.join("BIJUX_PE_R2.fastq.gz");
    if !se.exists() || !pe_r1.exists() || !pe_r2.exists() {
        return Err(anyhow!(
            "canonical FASTQ dataset missing in {}",
            root.display()
        ));
    }
    Ok(vec![
        QaDataset {
            name: "BIJUX_SE".to_string(),
            r1: se,
            r2: None,
            r1_dir: root.to_path_buf(),
            input_hash_r1: String::new(),
            input_hash_r2: None,
            input_stats_r1: SeqkitMetrics {
                reads: 0,
                bases: 0,
                mean_q: 0.0,
                gc_percent: 0.0,
            },
            input_stats_r2: None,
        },
        QaDataset {
            name: "BIJUX_PE".to_string(),
            r1: pe_r1,
            r2: Some(pe_r2),
            r1_dir: root.to_path_buf(),
            input_hash_r1: String::new(),
            input_hash_r2: None,
            input_stats_r1: SeqkitMetrics {
                reads: 0,
                bases: 0,
                mean_q: 0.0,
                gc_percent: 0.0,
            },
            input_stats_r2: None,
        },
    ])
}

fn is_fastq_gz(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
}
