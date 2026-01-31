use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct StageExecutionPlan {
    pub tool: String,
    pub container_args: Vec<String>,
    pub expected_outputs: Vec<PathBuf>,
    pub output_fastq: Option<PathBuf>,
    pub env: BTreeMap<String, String>,
}

#[allow(clippy::too_many_lines)]
pub fn plan_tool_execution(
    tool: &str,
    input_path: &str,
    out_dir: &Path,
) -> Result<StageExecutionPlan> {
    let (container_args, output) = match tool {
        "fastp" => {
            let out_name = "fastp.fastq.gz";
            (
                vec![
                    "fastp".to_string(),
                    "-i".to_string(),
                    input_path.to_string(),
                    "-o".to_string(),
                    format!("/data/output/{out_name}"),
                ],
                Some(out_dir.join(out_name)),
            )
        }
        "cutadapt" => {
            let out_name = "cutadapt.fastq.gz";
            (
                vec![
                    "cutadapt".to_string(),
                    "-o".to_string(),
                    format!("/data/output/{out_name}"),
                    input_path.to_string(),
                ],
                Some(out_dir.join(out_name)),
            )
        }
        "atropos" => {
            let out_name = "atropos.fastq.gz";
            (
                vec![
                    "atropos".to_string(),
                    "trim".to_string(),
                    "-a".to_string(),
                    "AGATCGGAAGAGC".to_string(),
                    "-se".to_string(),
                    input_path.to_string(),
                    "-o".to_string(),
                    format!("/data/output/{out_name}"),
                ],
                Some(out_dir.join(out_name)),
            )
        }
        "bbduk" => {
            let out_name = "bbduk.fastq.gz";
            (
                vec![
                    format!("in={input_path}"),
                    format!("out=/data/output/{out_name}"),
                    "ref=adapters".to_string(),
                ],
                Some(out_dir.join(out_name)),
            )
        }
        "adapterremoval" => {
            let out_name = "adapterremoval.fastq.gz";
            (
                vec![
                    "adapterremoval".to_string(),
                    "--file1".to_string(),
                    input_path.to_string(),
                    "--output1".to_string(),
                    format!("/data/output/{out_name}"),
                ],
                Some(out_dir.join(out_name)),
            )
        }
        "trimmomatic" => {
            let out_name = "trimmomatic.fastq.gz";
            (
                vec![
                    "trimmomatic".to_string(),
                    "SE".to_string(),
                    "-phred33".to_string(),
                    input_path.to_string(),
                    format!("/data/output/{out_name}"),
                    "SLIDINGWINDOW:4:20".to_string(),
                    "MINLEN:30".to_string(),
                ],
                Some(out_dir.join(out_name)),
            )
        }
        "trim_galore" => {
            let basename = "trimmed";
            (
                vec![
                    "trim_galore".to_string(),
                    "--gzip".to_string(),
                    "--output_dir".to_string(),
                    "/data/output".to_string(),
                    "--basename".to_string(),
                    basename.to_string(),
                    input_path.to_string(),
                ],
                Some(out_dir.join(format!("{basename}_trimmed.fq.gz"))),
            )
        }
        "rcorrector" => (
            vec![
                "rcorrector".to_string(),
                "-1".to_string(),
                input_path.to_string(),
            ],
            None,
        ),
        "umi_tools" => (
            vec![
                "umi_tools".to_string(),
                "extract".to_string(),
                "--stdin".to_string(),
                input_path.to_string(),
            ],
            None,
        ),
        "kraken2" => (
            vec![
                "kraken2".to_string(),
                "--report".to_string(),
                "/data/output/kraken2.report".to_string(),
                input_path.to_string(),
            ],
            None,
        ),
        "seqkit" => (
            vec![
                "seqkit".to_string(),
                "seq".to_string(),
                input_path.to_string(),
            ],
            None,
        ),
        "seqpurge" => {
            let out_name = "seqpurge.fastq.gz";
            (
                vec![
                    "seqpurge".to_string(),
                    "-in1".to_string(),
                    input_path.to_string(),
                    "-out1".to_string(),
                    format!("/data/output/{out_name}"),
                ],
                Some(out_dir.join(out_name)),
            )
        }
        "prinseq" => {
            let prefix = "prinseq_good";
            (
                vec![
                    "prinseq++".to_string(),
                    "-fastq".to_string(),
                    input_path.to_string(),
                    "-out_good".to_string(),
                    format!("/data/output/{prefix}"),
                    "-out_bad".to_string(),
                    "/data/output/prinseq_bad".to_string(),
                ],
                Some(out_dir.join(format!("{prefix}.fastq"))),
            )
        }
        _ => return Err(anyhow!("unsupported tool: {tool}")),
    };

    Ok(StageExecutionPlan {
        tool: tool.to_string(),
        container_args,
        expected_outputs: output.iter().cloned().collect(),
        output_fastq: output,
        env: BTreeMap::new(),
    })
}

pub fn plan_validate_execution(
    tool: &str,
    input_path: &str,
    out_dir: &Path,
) -> Result<StageExecutionPlan> {
    let container_args = match tool {
        "seqtk" => vec![
            "seqtk".to_string(),
            "fqchk".to_string(),
            input_path.to_string(),
        ],
        "fastqc" => vec![
            "fastqc".to_string(),
            "--extract".to_string(),
            "-f".to_string(),
            "fastq".to_string(),
            "-o".to_string(),
            "/data/output".to_string(),
            input_path.to_string(),
        ],
        "fastqvalidator" | "fastqvalidator_official" => vec![
            "fastq-validator".to_string(),
            "--file".to_string(),
            input_path.to_string(),
            "--printCount".to_string(),
        ],
        "fqtools" => vec![
            "fqtools".to_string(),
            "count".to_string(),
            input_path.to_string(),
        ],
        "seqkit_stats" => vec![
            "seqkit".to_string(),
            "stats".to_string(),
            "-a".to_string(),
            "-T".to_string(),
            input_path.to_string(),
        ],
        "multiqc" => vec![
            "-o".to_string(),
            "/data/output".to_string(),
            "/data/input".to_string(),
        ],
        _ => return Err(anyhow!("unsupported tool: {tool}")),
    };

    let expected_outputs = match tool {
        "fastqc" => vec![out_dir.join("fastqc_data.txt")],
        "multiqc" => vec![out_dir.join("multiqc_report.html")],
        _ => Vec::new(),
    };

    Ok(StageExecutionPlan {
        tool: tool.to_string(),
        container_args,
        expected_outputs,
        output_fastq: None,
        env: BTreeMap::new(),
    })
}

pub fn plan_merge_execution(
    tool: &str,
    r1_path: &str,
    r2_path: &str,
    out_dir: &Path,
) -> Result<StageExecutionPlan> {
    let (container_args, outputs) = match tool {
        "pear" => (
            vec![
                "pear".to_string(),
                "-f".to_string(),
                r1_path.to_string(),
                "-r".to_string(),
                r2_path.to_string(),
                "-o".to_string(),
                "/data/output/pear".to_string(),
            ],
            vec![
                out_dir.join("pear.assembled.fastq"),
                out_dir.join("pear.unassembled.forward.fastq"),
                out_dir.join("pear.unassembled.reverse.fastq"),
            ],
        ),
        "vsearch" => (
            vec![
                "vsearch".to_string(),
                "--fastq_mergepairs".to_string(),
                r1_path.to_string(),
                "--reverse".to_string(),
                r2_path.to_string(),
                "--fastqout".to_string(),
                "/data/output/vsearch.merged.fastq".to_string(),
                "--fastqout_notmerged_fwd".to_string(),
                "/data/output/vsearch.unmerged_r1.fastq".to_string(),
                "--fastqout_notmerged_rev".to_string(),
                "/data/output/vsearch.unmerged_r2.fastq".to_string(),
            ],
            vec![
                out_dir.join("vsearch.merged.fastq"),
                out_dir.join("vsearch.unmerged_r1.fastq"),
                out_dir.join("vsearch.unmerged_r2.fastq"),
            ],
        ),
        "bbmerge" => (
            vec![
                "bbmerge.sh".to_string(),
                format!("in1={r1_path}"),
                format!("in2={r2_path}"),
                "out=/data/output/bbmerge.merged.fastq".to_string(),
                "outu1=/data/output/bbmerge.unmerged_r1.fastq".to_string(),
                "outu2=/data/output/bbmerge.unmerged_r2.fastq".to_string(),
            ],
            vec![
                out_dir.join("bbmerge.merged.fastq"),
                out_dir.join("bbmerge.unmerged_r1.fastq"),
                out_dir.join("bbmerge.unmerged_r2.fastq"),
            ],
        ),
        _ => return Err(anyhow!("unsupported tool: {tool}")),
    };

    Ok(StageExecutionPlan {
        tool: tool.to_string(),
        container_args,
        expected_outputs: outputs.clone(),
        output_fastq: outputs.first().cloned(),
        env: BTreeMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::{plan_merge_execution, plan_tool_execution, plan_validate_execution};
    use std::path::Path;

    #[test]
    fn plan_tool_execution_fastp() -> anyhow::Result<()> {
        let plan = plan_tool_execution("fastp", "/data/input/reads.fq", Path::new("/tmp"))?;
        assert!(plan.container_args.iter().any(|arg| arg == "fastp"));
        assert!(plan.output_fastq.is_some());
        Ok(())
    }

    #[test]
    fn plan_validate_execution_fastqc() -> anyhow::Result<()> {
        let plan = plan_validate_execution("fastqc", "/data/input/reads.fq", Path::new("/tmp"))?;
        assert!(plan.container_args.iter().any(|arg| arg == "fastqc"));
        Ok(())
    }

    #[test]
    fn plan_merge_execution_pear() -> anyhow::Result<()> {
        let plan = plan_merge_execution(
            "pear",
            "/data/input/r1.fq",
            "/data/input/r2.fq",
            Path::new("/tmp"),
        )?;
        assert!(plan.container_args.iter().any(|arg| arg == "pear"));
        assert_eq!(plan.expected_outputs.len(), 3);
        Ok(())
    }
}
