//! Owner: bijux-dna-bench
//! Picard command closure for stratified BAM cohort observations.

use anyhow::{bail, Context, Result};

use super::super::bam_observation::BamCohortCommandSample;

pub(super) struct PicardCohortCommandPolicy<'a> {
    pub(super) stage_id: &'a str,
    pub(super) result_stem: &'a str,
    pub(super) awk_source: &'a str,
    pub(super) consumer_stage_id: &'a str,
    pub(super) input_contract: &'a str,
    pub(super) scientific_metrics: &'a [&'a str],
    pub(super) reference_input_bytes: u64,
}

pub(super) fn warmup_command(
    policy: &PicardCohortCommandPolicy<'_>,
    samples: &[BamCohortCommandSample],
) -> Result<Vec<String>> {
    let sample = samples.first().context("Picard BAM cohort has no warmup sample")?;
    Ok(shell(picard_command(policy, sample, "/scratch/warmup")?))
}

pub(super) fn measured_command(
    policy: &PicardCohortCommandPolicy<'_>,
    samples: &[BamCohortCommandSample],
    cohort_input_bytes: u64,
) -> Result<Vec<String>> {
    if samples.is_empty() {
        bail!("Picard BAM cohort has no measured samples");
    }
    let mut command = format!(
        "set -eu\nmkdir -p /result/repetitions\nprefix=\"/result/repetitions/repetition-{{repetition}}\"\ncohort=\"${{prefix}}.{}.tsv\"\nprintf 'sample_id\\tmetric\\tvalue\\n' > \"$cohort\"\n",
        policy.result_stem
    );
    for sample in samples {
        let sample_stem = normalized_id(&sample.sample_id);
        let prefix = format!("${{prefix}}.{sample_stem}");
        command.push_str(&picard_command(policy, sample, &prefix)?);
        command.push('\n');
        let parser_input = parser_input(policy, &prefix);
        command.push_str(&format!(
            "awk -v mode=summary -f /repository/benchmarks/execution/{} {} > \"{}.{}.tsv\"\n",
            policy.awk_source, parser_input, prefix, policy.result_stem
        ));
        command.push_str(&format!(
            "awk -v mode=bundle -v sample_id={} -f /repository/benchmarks/execution/{} \"{}.{}.tsv\" > \"{}.{}.json\"\n",
            quote(&sample.sample_id), policy.awk_source, prefix, policy.result_stem, prefix, policy.result_stem
        ));
        command.push_str(&format!(
            "awk -v mode=probe -f /repository/benchmarks/execution/{} \"{}.{}.tsv\" > \"{}.report-qc.json\"\n",
            policy.awk_source, prefix, policy.result_stem, prefix
        ));
        command.push_str(&format!(
            "sample_output_bytes=$(wc -c \"{}.picard.metrics.txt\" {} \"{}.{}.tsv\" \"{}.{}.json\" \"{}.report-qc.json\" | awk 'END {{print $1}}')\n",
            prefix,
            auxiliary_outputs(policy, &prefix),
            prefix,
            policy.result_stem,
            prefix,
            policy.result_stem,
            prefix,
        ));
        let sample_input_bytes = sample
            .bam_input_bytes
            .checked_add(sample.bam_index_input_bytes)
            .and_then(|value| value.checked_add(policy.reference_input_bytes))
            .context("Picard cohort sample input size overflow")?;
        command.push_str(&format!(
            "awk -v mode=metrics -v bam_input_bytes={} -v bam_index_input_bytes={} -v reference_input_bytes={} -v input_bytes={} -v output_bytes=\"$sample_output_bytes\" -f /repository/benchmarks/execution/{} \"{}.{}.tsv\" > \"{}.metrics.json\"\n",
            sample.bam_input_bytes,
            sample.bam_index_input_bytes,
            policy.reference_input_bytes,
            sample_input_bytes,
            policy.awk_source,
            prefix,
            policy.result_stem,
            prefix,
        ));
        command.push_str(&format!(
            "awk -v sample_id={} 'NR > 1 {{print sample_id \"\\t\" $0}}' \"{}.{}.tsv\" >> \"$cohort\"\n",
            quote(&sample.sample_id), prefix, policy.result_stem
        ));
    }
    let total_input_bytes = cohort_input_bytes
        .checked_add(policy.reference_input_bytes)
        .context("Picard cohort total input size overflow")?;
    command.push_str(&format!(
        "output_bytes=$(wc -c \"${{prefix}}\".* | awk 'END {{print $1}}')\nawk -v mode=metrics -v expected_samples={} -v expected_metrics={} -v input_bytes={} -v output_bytes=\"$output_bytes\" -f /repository/benchmarks/execution/bam_cohort_summary.awk \"$cohort\" > \"${{prefix}}.metrics.json\"\nawk -v mode=probe -v expected_samples={} -v expected_metrics={} -v consumer_stage_id={} -v input_contract={} -f /repository/benchmarks/execution/bam_cohort_summary.awk \"$cohort\" > \"${{prefix}}.report-qc.json\"",
        samples.len(),
        policy.scientific_metrics.len(),
        total_input_bytes,
        samples.len(),
        policy.scientific_metrics.len(),
        quote(policy.consumer_stage_id),
        quote(policy.input_contract),
    ));
    Ok(shell(command))
}

fn parser_input(policy: &PicardCohortCommandPolicy<'_>, prefix: &str) -> String {
    match policy.stage_id {
        "bam.gc_bias" => format!("\"{prefix}.picard.summary.txt\""),
        _ => format!("\"{prefix}.picard.metrics.txt\""),
    }
}

pub(super) fn negative_command(policy: &PicardCohortCommandPolicy<'_>) -> Vec<String> {
    let command = match policy.stage_id {
        "bam.gc_bias" => "picard CollectGcBiasMetrics I=/input/missing.bam O=/scratch/missing.txt S=/scratch/missing.summary.txt CHART_OUTPUT=/scratch/missing.pdf R=/input/reference/reference.fasta",
        "bam.insert_size" => "picard CollectInsertSizeMetrics I=/input/missing.bam O=/scratch/missing.txt H=/scratch/missing.pdf",
        _ => "false",
    };
    shell(command.to_string())
}

pub(super) fn required_files(
    policy: &PicardCohortCommandPolicy<'_>,
    samples: &[BamCohortCommandSample],
    repetitions: u32,
) -> Vec<String> {
    let mut files = Vec::new();
    for repetition in 1..=repetitions {
        let repetition_prefix = format!("repetitions/repetition-{repetition}");
        for sample in samples {
            let prefix = format!("{repetition_prefix}.{}", normalized_id(&sample.sample_id));
            files.push(format!("{prefix}.picard.metrics.txt"));
            files.extend(auxiliary_relative_outputs(policy, &prefix));
            files.extend([
                format!("{prefix}.{}.tsv", policy.result_stem),
                format!("{prefix}.{}.json", policy.result_stem),
                format!("{prefix}.report-qc.json"),
                format!("{prefix}.metrics.json"),
            ]);
        }
        files.extend([
            format!("{repetition_prefix}.{}.tsv", policy.result_stem),
            format!("{repetition_prefix}.metrics.json"),
            format!("{repetition_prefix}.report-qc.json"),
            format!("{repetition_prefix}.resource.tsv"),
        ]);
    }
    files
}

fn picard_command(
    policy: &PicardCohortCommandPolicy<'_>,
    sample: &BamCohortCommandSample,
    prefix: &str,
) -> Result<String> {
    match policy.stage_id {
        "bam.gc_bias" => Ok(format!(
            "picard CollectGcBiasMetrics I={} O=\"{}.picard.metrics.txt\" S=\"{}.picard.summary.txt\" CHART_OUTPUT=\"{}.picard.chart.pdf\" R=/input/reference/reference.fasta VALIDATION_STRINGENCY=SILENT",
            quote(&sample.bam_mount), prefix, prefix, prefix
        )),
        "bam.insert_size" => Ok(format!(
            "picard CollectInsertSizeMetrics I={} O=\"{}.picard.metrics.txt\" H=\"{}.picard.histogram.pdf\" VALIDATION_STRINGENCY=SILENT",
            quote(&sample.bam_mount), prefix, prefix
        )),
        _ => bail!("unsupported Picard BAM cohort stage: {}", policy.stage_id),
    }
}

fn auxiliary_outputs(policy: &PicardCohortCommandPolicy<'_>, prefix: &str) -> String {
    match policy.stage_id {
        "bam.gc_bias" => {
            format!("\"{prefix}.picard.summary.txt\" \"{prefix}.picard.chart.pdf\"")
        }
        "bam.insert_size" => format!("\"{prefix}.picard.histogram.pdf\""),
        _ => String::new(),
    }
}

fn auxiliary_relative_outputs(policy: &PicardCohortCommandPolicy<'_>, prefix: &str) -> Vec<String> {
    match policy.stage_id {
        "bam.gc_bias" => {
            vec![format!("{prefix}.picard.summary.txt"), format!("{prefix}.picard.chart.pdf")]
        }
        "bam.insert_size" => vec![format!("{prefix}.picard.histogram.pdf")],
        _ => Vec::new(),
    }
}

fn normalized_id(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .map(|character| if character.is_ascii_alphanumeric() { character } else { '_' })
        .collect()
}

fn shell(command: String) -> Vec<String> {
    vec!["sh".to_string(), "-c".to_string(), command]
}

fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
