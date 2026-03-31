use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::image_qa::support::{hash_file_sha256, input_fastq_stats};
use crate::image_qa::QaDataset;

pub(crate) fn hydrate_datasets(
    datasets: &mut [QaDataset],
    seqkit_image: &super::super::support::ResolvedImage,
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
    seqkit_image: &super::super::support::ResolvedImage,
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
