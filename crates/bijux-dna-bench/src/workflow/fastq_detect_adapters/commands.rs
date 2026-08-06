//! Owner: bijux-dna-bench
//! Reproducible commands and output closure for FASTQ adapter-inspection attempts.

use anyhow::{bail, Context, Result};

use super::super::fastq_profile_reads::FastqCampaignSample;

const WARMUP_ROOT: &str = "/scratch/warmup";
const NEGATIVE_ROOT: &str = "/scratch/negative";
const FASTQC_TOOL: &str = "fastqc";

pub(super) fn warmup_command(
    tool_id: &str,
    samples: &[FastqCampaignSample],
) -> Result<Vec<String>> {
    let sample = samples.first().context("FASTQ detect-adapters campaign has no samples")?;
    let command = sample_command(tool_id, sample, WARMUP_ROOT)?;
    Ok(shell(format!("set -eu; mkdir -p /scratch/warmup; {command}")))
}

pub(super) fn measured_command(
    tool_id: &str,
    samples: &[FastqCampaignSample],
    input_bytes: u64,
) -> Result<Vec<String>> {
    if samples.len() != 20 {
        bail!("FASTQ detect-adapters command requires the complete twenty-sample cohort");
    }
    let mut script = String::from(
        "set -eu\nmkdir -p /result/repetitions /scratch/detect-adapters\n\
         prefix=\"/result/repetitions/repetition-{repetition}\"\n\
         manifest=\"${prefix}.sample-manifest.tsv\"\n\
         sh /repository/benchmarks/execution/fastq_detect_adapters.sh header > \"$manifest\"\n",
    );
    for (index, sample) in samples.iter().enumerate() {
        let sample_root = format!("/scratch/detect-adapters/sample-{index:02}");
        let command = sample_command(tool_id, sample, &sample_root)?;
        script.push_str(&format!(
            "mkdir -p {}\n{}\ncat {} >> \"$manifest\"\n",
            shell_quote(&sample_root),
            command,
            shell_quote(&format!("{sample_root}/sample-summary.tsv")),
        ));
    }
    script.push_str(&format!(
        "sh /repository/benchmarks/execution/fastq_detect_adapters.sh normalize \"$manifest\" > \"${{prefix}}.adapter-summary.tsv\"\n\
         sh /repository/benchmarks/execution/fastq_detect_adapters.sh bundle \"${{prefix}}.adapter-summary.tsv\" > \"${{prefix}}.adapter-bundle.json\"\n\
         sh /repository/benchmarks/execution/fastq_detect_adapters.sh probe \"${{prefix}}.adapter-summary.tsv\" > \"${{prefix}}.report-qc.json\"\n\
         output_bytes=$(wc -c \"$manifest\" \"${{prefix}}.adapter-summary.tsv\" \"${{prefix}}.adapter-bundle.json\" \"${{prefix}}.report-qc.json\" | awk 'END {{print $1}}')\n\
         input_bytes={input_bytes} output_bytes=\"$output_bytes\" sh /repository/benchmarks/execution/fastq_detect_adapters.sh metrics \"${{prefix}}.adapter-summary.tsv\" > \"${{prefix}}.metrics.json\"",
    ));
    Ok(shell(script))
}

pub(super) fn negative_command(tool_id: &str) -> Result<Vec<String>> {
    let sample = FastqCampaignSample {
        sample_id: "missing-input".to_string(),
        input_layout: "single_end",
        mounted_fastq: vec!["/input/missing.fastq.gz".to_string()],
        expected_read_count: 1,
    };
    let command = sample_command(tool_id, &sample, NEGATIVE_ROOT)?;
    Ok(shell(format!("mkdir -p /scratch/negative; {command}")))
}

pub(super) fn required_files(repetitions: u32) -> Vec<String> {
    let mut files = Vec::new();
    for repetition in 1..=repetitions {
        let prefix = format!("repetitions/repetition-{repetition}");
        files.extend([
            format!("{prefix}.sample-manifest.tsv"),
            format!("{prefix}.adapter-summary.tsv"),
            format!("{prefix}.adapter-bundle.json"),
            format!("{prefix}.report-qc.json"),
            format!("{prefix}.metrics.json"),
            format!("{prefix}.resource.tsv"),
        ]);
    }
    files
}

fn sample_command(
    tool_id: &str,
    sample: &FastqCampaignSample,
    output_root: &str,
) -> Result<String> {
    if tool_id != FASTQC_TOOL {
        bail!("unsupported FASTQ detect-adapters tool");
    }
    if !matches!(sample.mounted_fastq.len(), 1 | 2) {
        bail!("FASTQ detect-adapters sample has an unsupported input layout");
    }
    let fastqc_root = format!("{output_root}/fastqc");
    let report_json = format!("{output_root}/adapter_report.json");
    let summary_tsv = format!("{output_root}/sample-summary.tsv");
    let mut command = format!(
        "mkdir -p {}\nfastqc --extract --outdir {} --threads 4 {}",
        shell_quote(&fastqc_root),
        shell_quote(&fastqc_root),
        shell_quote(&sample.mounted_fastq[0]),
    );
    if let Some(r2) = sample.mounted_fastq.get(1) {
        command.push(' ');
        command.push_str(&shell_quote(r2));
    }
    command.push('\n');
    command.push_str(&format!(
        "sh /repository/benchmarks/execution/fastq_detect_adapters.sh sample-report {} {} {} {} {} {} {} {}",
        shell_quote(&sample.sample_id),
        shell_quote(sample.input_layout),
        shell_quote(&report_json),
        shell_quote(&summary_tsv),
        shell_quote(&fastqc_root),
        shell_quote(tool_id),
        4,
        shell_quote(&sample.mounted_fastq[0]),
    ));
    if let Some(r2) = sample.mounted_fastq.get(1) {
        command.push(' ');
        command.push_str(&shell_quote(r2));
    }
    Ok(format!("( {command} )"))
}

fn shell(command: String) -> Vec<String> {
    vec!["sh".to_string(), "-c".to_string(), command]
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::{measured_command, negative_command, required_files, warmup_command};
    use crate::workflow::fastq_profile_reads::FastqCampaignSample;

    #[test]
    fn commands_close_the_governed_fastqc_campaign() -> anyhow::Result<()> {
        let samples = (0..20)
            .map(|index| FastqCampaignSample {
                sample_id: format!("sample-{index:02}"),
                input_layout: if index < 10 { "paired_end" } else { "single_end" },
                mounted_fastq: if index < 10 {
                    vec![
                        format!("/input/sample-{index:02}_R1.fastq.gz"),
                        format!("/input/sample-{index:02}_R2.fastq.gz"),
                    ]
                } else {
                    vec![format!("/input/sample-{index:02}.fastq.gz")]
                },
                expected_read_count: 10,
            })
            .collect::<Vec<_>>();
        let warmup = warmup_command("fastqc", &samples)?;
        let measured = measured_command("fastqc", &samples, 42)?;
        let negative = negative_command("fastqc")?;
        assert!(warmup[2].contains("sample-report"));
        assert!(warmup[2].contains("mkdir -p '/scratch/warmup/fastqc'"));
        assert!(measured[2].contains("fastq_detect_adapters.sh header"));
        assert!(measured[2].contains("fastq_detect_adapters.sh metrics"));
        assert!(measured[2].contains("sample-19"));
        assert!(measured[2].contains("mkdir -p '/scratch/detect-adapters/sample-19/fastqc'"));
        assert!(negative[2].contains("missing.fastq.gz"));
        assert!(negative[2].contains("mkdir -p '/scratch/negative/fastqc'"));
        assert_eq!(required_files(3).len(), 18);
        Ok(())
    }

    #[test]
    fn sample_report_normalizer_runs_with_posix_awk() -> anyhow::Result<()> {
        let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let temporary = bijux_dna_infra::temp_dir("fastq-detect-adapters-normalizer")?;
        let report = temporary.path().join("report.json");
        let summary = temporary.path().join("summary.tsv");
        let evidence = temporary.path().join("evidence");
        std::fs::create_dir(&evidence)?;
        let status = Command::new("sh")
            .arg(repository_root.join("benchmarks/execution/fastq_detect_adapters.sh"))
            .args(["sample-report", "smoke", "single_end"])
            .arg(&report)
            .arg(&summary)
            .arg(&evidence)
            .args(["fastqc", "1"])
            .arg(repository_root.join("assets/golden/smoke-inputs-v1/fastq/se/reads.fastq"))
            .env("BIJUX_BENCHMARK_REPOSITORY_ROOT", &repository_root)
            .status()?;
        anyhow::ensure!(status.success(), "FASTQ adapter sample report failed");

        let report = std::fs::read_to_string(report)?;
        anyhow::ensure!(
            report.contains("\"schema_version\": \"bijux.fastq.detect_adapters.report.v3\"")
        );
        let summary_row = std::fs::read(&summary)?;
        let rows =
            String::from_utf8_lossy(&summary_row).lines().map(str::to_owned).collect::<Vec<_>>();
        anyhow::ensure!(rows.len() == 1 && rows[0].split('\t').count() == 19);

        let script = repository_root.join("benchmarks/execution/fastq_detect_adapters.sh");
        let header = Command::new("sh").arg(&script).arg("header").output()?;
        anyhow::ensure!(header.status.success(), "FASTQ adapter header failed");
        let manifest = temporary.path().join("manifest.tsv");
        let mut manifest_bytes = header.stdout;
        manifest_bytes.extend(summary_row);
        std::fs::write(&manifest, manifest_bytes)?;
        let normalized = Command::new("sh").arg(script).arg("normalize").arg(manifest).output()?;
        anyhow::ensure!(normalized.status.success(), "FASTQ adapter normalization failed");
        let normalized_rows = String::from_utf8(normalized.stdout)?;
        let mut normalized_rows = normalized_rows.lines();
        anyhow::ensure!(
            normalized_rows.next().is_some_and(|row| row.starts_with("sample_id\t")),
            "FASTQ adapter normalization did not preserve the governed header"
        );
        anyhow::ensure!(normalized_rows.next().is_some_and(|row| row.starts_with("smoke\t")));
        anyhow::ensure!(normalized_rows.next().is_none());
        Ok(())
    }
}
