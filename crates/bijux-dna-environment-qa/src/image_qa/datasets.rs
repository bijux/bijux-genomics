use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use super::support::{hash_file_sha256, input_fastq_stats, SeqkitMetrics};

use super::{QaDataset, QaStage};

pub(crate) fn discover_qa_datasets() -> Result<Vec<QaDataset>> {
    let canonical = PathBuf::from("scripts/lab/corpus/fastq/canonical");
    if canonical.exists() {
        return discover_canonical_datasets(&canonical);
    }
    let root = PathBuf::from("scripts/lab/corpus/fastq");
    if !root.exists() {
        return Err(anyhow!("scripts/lab/corpus/fastq not found"));
    }
    let mut datasets = Vec::new();
    for entry in std::fs::read_dir(&root).context("read scripts/lab/corpus/fastq")? {
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
        return Err(anyhow!("no FASTQ files found in scripts/lab/corpus/fastq"));
    }
    Ok(datasets)
}

fn discover_canonical_datasets(root: &Path) -> Result<Vec<QaDataset>> {
    let se = root.join("BIJUX_SE_R1.fastq.gz");
    let pe_r1 = root.join("BIJUX_PE_R1.fastq.gz");
    let pe_r2 = root.join("BIJUX_PE_R2.fastq.gz");
    if !se.exists() || !pe_r1.exists() || !pe_r2.exists() {
        return Err(anyhow!(
            "canonical FASTQ dataset missing in scripts/lab/corpus/fastq/canonical"
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

pub(crate) fn hydrate_datasets(
    datasets: &mut [QaDataset],
    seqkit_image: &super::support::ResolvedImage,
) -> Result<()> {
    let qa_root = PathBuf::from("artifacts/image-qa/inputs");
    bijux_dna_infra::ensure_dir(&qa_root).context("create image qa inputs dir")?;
    for dataset in datasets {
        let r1 = dataset.r1.canonicalize().context("resolve r1 path")?;
        let subset_dir = qa_root.join(&dataset.name);
        bijux_dna_infra::ensure_dir(&subset_dir).context("create dataset qa dir")?;
        let subset_dir = subset_dir
            .canonicalize()
            .context("resolve dataset qa dir")?;
        let r1_subset = subset_fastq(seqkit_image, &r1, &subset_dir, "R1")?;
        dataset.r1.clone_from(&r1_subset);
        dataset.r1_dir.clone_from(&subset_dir);
        dataset.input_hash_r1 = hash_file_sha256(&dataset.r1)?;
        dataset.input_stats_r1 = input_fastq_stats(seqkit_image, &subset_dir, &dataset.r1)?;

        if let Some(r2) = dataset.r2.clone() {
            let r2 = r2.canonicalize().context("resolve r2 path")?;
            let r2_subset = subset_fastq(seqkit_image, &r2, &subset_dir, "R2")?;
            dataset.r2 = Some(r2_subset.clone());
            let stats = input_fastq_stats(seqkit_image, &subset_dir, &r2_subset)?;
            dataset.input_stats_r2 = Some(stats);
            let r2_hash = hash_file_sha256(&r2_subset)?;
            dataset.input_hash_r2 = Some(r2_hash);
        }
    }
    Ok(())
}

fn subset_fastq(
    seqkit_image: &super::support::ResolvedImage,
    input: &Path,
    out_dir: &Path,
    label: &str,
) -> Result<PathBuf> {
    const QA_READS: u64 = 5000;
    let input_dir = input
        .parent()
        .ok_or_else(|| anyhow!("input FASTQ has no parent"))?
        .canonicalize()
        .context("resolve input dir")?;
    let input_name = input
        .file_name()
        .ok_or_else(|| anyhow!("input FASTQ missing filename"))?
        .to_string_lossy()
        .to_string();
    let output_name = format!("qa_{label}.fastq.gz");
    let output_dir = out_dir.canonicalize().context("resolve output dir")?;
    let output_path = output_dir.join(&output_name);

    let status = std::process::Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(format!("{}:/in:ro", input_dir.display()))
        .arg("-v")
        .arg(format!("{}:/out", output_dir.display()))
        .arg(&seqkit_image.full_name)
        .arg("seqkit")
        .arg("head")
        .arg("-n")
        .arg(QA_READS.to_string())
        .arg(format!("/in/{input_name}"))
        .arg("-o")
        .arg(format!("/out/{output_name}"))
        .status()
        .context("run seqkit head for QA subset")?;

    if !status.success() {
        return Err(anyhow!("seqkit head failed"));
    }
    if !output_path.exists() {
        return Err(anyhow!(
            "QA subset output missing: {}",
            output_path.display()
        ));
    }
    Ok(output_path)
}

pub(crate) fn datasets_for_stage(stage: QaStage, datasets: &[QaDataset]) -> Vec<QaDataset> {
    match stage {
        QaStage::Merge => datasets
            .iter()
            .filter(|dataset| dataset.r2.is_some())
            .cloned()
            .collect(),
        QaStage::Trim => {
            let pe: Vec<QaDataset> = datasets
                .iter()
                .filter(|dataset| dataset.r2.is_some())
                .cloned()
                .collect();
            if pe.is_empty() {
                datasets.to_vec()
            } else {
                pe
            }
        }
        _ => datasets.to_vec(),
    }
}

pub(crate) fn dataset_input_hash(stage: QaStage, dataset: &QaDataset) -> String {
    match stage {
        QaStage::Merge => {
            let r1 = dataset.input_hash_r1.as_str();
            let r2 = dataset.input_hash_r2.as_deref().unwrap_or("missing");
            format!("{r1},{r2}")
        }
        _ => dataset.input_hash_r1.clone(),
    }
}

fn is_fastq_gz(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
}
