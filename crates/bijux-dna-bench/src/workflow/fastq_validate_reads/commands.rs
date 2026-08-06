//! Owner: bijux-dna-bench
//! Planner-derived commands and output closure for FASTQ validation attempts.

use std::path::Path;

use anyhow::{bail, Context, Result};

use super::super::fastq_profile_reads::FastqCampaignSample;
use super::planner_contract::{load_validation_planner_contract, PlannerToolContract};

pub(super) fn warmup_command(
    repository_root: &Path,
    tool_id: &str,
    samples: &[FastqCampaignSample],
) -> Result<Vec<String>> {
    let sample = samples.first().context("FASTQ validation campaign has no samples")?;
    let contract = load_validation_planner_contract(repository_root)?;
    let command = planned_sample_command(contract.tool(tool_id)?, sample, "/scratch/warmup")?;
    Ok(shell(format!("set -eu; mkdir -p /scratch/warmup; {command}")))
}

pub(super) fn measured_command(
    repository_root: &Path,
    tool_id: &str,
    samples: &[FastqCampaignSample],
    input_bytes: u64,
) -> Result<Vec<String>> {
    if samples.len() != 20 {
        bail!("FASTQ validation command requires the complete twenty-sample cohort");
    }
    let contract = load_validation_planner_contract(repository_root)?;
    let tool = contract.tool(tool_id)?;
    let mut script = String::from(
        "set -eu\nmkdir -p /result/repetitions /scratch/validation\n\
         prefix=\"/result/repetitions/repetition-{repetition}\"\n\
         manifest=\"${prefix}.validation-manifest.tsv\"\n\
         : > \"$manifest\"\n",
    );
    for (index, sample) in samples.iter().enumerate() {
        let sample_root = format!("/scratch/validation/sample-{index:02}");
        let command = planned_sample_command(tool, sample, &sample_root)?;
        script.push_str(&format!(
            "mkdir -p {}\n{}\nprintf '%s\\t%s\\t%s\\t%s\\t%s\\n' {} {} {} {} {} >> \"$manifest\"\n",
            shell_quote(&sample_root),
            command,
            shell_quote(&sample.sample_id),
            shell_quote(sample.input_layout),
            sample.mounted_fastq.len(),
            sample.expected_read_count,
            shell_quote(&format!("{sample_root}/validation.json")),
        ));
    }
    script.push_str(&format!(
        "awk -v mode=normalize -f /repository/benchmarks/execution/fastq_validate_reads.awk \"$manifest\" > \"${{prefix}}.validation-summary.tsv\"\n\
         awk -v mode=bundle -v tool={} -f /repository/benchmarks/execution/fastq_validate_reads.awk \"${{prefix}}.validation-summary.tsv\" > \"${{prefix}}.validation-bundle.json\"\n\
         awk -v mode=probe -f /repository/benchmarks/execution/fastq_validate_reads.awk \"${{prefix}}.validation-summary.tsv\" > \"${{prefix}}.report-qc.json\"\n\
         output_bytes=$(wc -c \"$manifest\" \"${{prefix}}.validation-summary.tsv\" \"${{prefix}}.validation-bundle.json\" \"${{prefix}}.report-qc.json\" | awk 'END {{print $1}}')\n\
         awk -v mode=metrics -v input_bytes={input_bytes} -v output_bytes=\"$output_bytes\" -f /repository/benchmarks/execution/fastq_validate_reads.awk \"${{prefix}}.validation-summary.tsv\" > \"${{prefix}}.metrics.json\"",
        shell_quote(tool_id),
    ));
    Ok(shell(script))
}

pub(super) fn negative_command(repository_root: &Path, tool_id: &str) -> Result<Vec<String>> {
    let sample = FastqCampaignSample {
        sample_id: "missing-input".to_string(),
        input_layout: "single_end",
        mounted_fastq: vec!["/input/missing.fastq.gz".to_string()],
        expected_read_count: 1,
    };
    let contract = load_validation_planner_contract(repository_root)?;
    let command = planned_sample_command(contract.tool(tool_id)?, &sample, "/scratch/negative")?;
    Ok(shell(format!("mkdir -p /scratch/negative; {command}")))
}

pub(super) fn required_files(repetitions: u32) -> Vec<String> {
    let mut files = Vec::new();
    for repetition in 1..=repetitions {
        let prefix = format!("repetitions/repetition-{repetition}");
        files.extend([
            format!("{prefix}.validation-manifest.tsv"),
            format!("{prefix}.validation-summary.tsv"),
            format!("{prefix}.validation-bundle.json"),
            format!("{prefix}.report-qc.json"),
            format!("{prefix}.metrics.json"),
            format!("{prefix}.resource.tsv"),
        ]);
    }
    files
}

fn planned_sample_command(
    tool: &PlannerToolContract,
    sample: &FastqCampaignSample,
    output_root: &str,
) -> Result<String> {
    if !matches!(sample.mounted_fastq.len(), 1 | 2) {
        bail!("FASTQ validation sample has an unsupported input layout");
    }
    let r2 = sample.mounted_fastq.get(1).map(String::as_str);
    Ok(format!("( {} )", tool.materialize_command(&sample.mounted_fastq[0], r2, output_root)?))
}

fn shell(command: String) -> Vec<String> {
    vec!["sh".to_string(), "-c".to_string(), command]
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::{measured_command, negative_command, required_files, warmup_command};
    use crate::workflow::fastq_profile_reads::FastqCampaignSample;

    #[test]
    fn commands_are_derived_for_all_five_governed_backends() -> anyhow::Result<()> {
        let repository =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let samples = (0..20)
            .map(|index| FastqCampaignSample {
                sample_id: format!("sample-{index:02}"),
                input_layout: "single_end",
                mounted_fastq: vec![format!("/input/sample-{index:02}.fastq.gz")],
                expected_read_count: 10,
            })
            .collect::<Vec<_>>();
        for tool in ["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqtk"] {
            let warmup = warmup_command(&repository, tool, &samples)?;
            let measured = measured_command(&repository, tool, &samples, 42)?;
            let negative = negative_command(&repository, tool)?;
            assert!(warmup[2].contains("bijux.fastq.validate.report.v1"));
            assert!(measured[2].contains("fastq_validate_reads.awk"));
            assert!(measured[2].contains("sample-19"));
            assert!(negative[2].contains("missing.fastq.gz"));
        }
        assert_eq!(required_files(3).len(), 18);
        Ok(())
    }
}
