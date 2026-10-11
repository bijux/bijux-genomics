//! Owner: bijux-dna-bench
//! One-shot Bowtie2/BWA execution over an admitted alignment-truth cohort.

use anyhow::{bail, Context, Result};

use super::inputs::{normalized_id, AlignmentCommandSample};

const REFERENCE: &str = "/input/reference/reference.fasta";
const NORMALIZER: &str = "/repository/benchmarks/execution/bam_align.awk";

pub(super) fn warmup_command(
    tool: &str,
    samples: &[AlignmentCommandSample],
) -> Result<Vec<String>> {
    let sample = samples.first().context("alignment cohort has no warmup sample")?;
    let index = "/scratch/alignment-warmup-index/reference";
    Ok(shell(format!(
        "set -eu\nmkdir -p /scratch/alignment-warmup-index\n{}\n{} > /scratch/alignment-warmup.sam",
        index_command(tool, index)?,
        align_command(tool, sample, index)?,
    )))
}

pub(super) fn measured_command(
    tool: &str,
    samples: &[AlignmentCommandSample],
    reference_input_bytes: u64,
    total_fastq_bytes: u64,
    total_truth_bytes: u64,
) -> Result<Vec<String>> {
    if samples.is_empty() {
        bail!("alignment cohort has no measured samples");
    }
    let total_input_bytes = total_fastq_bytes
        .checked_add(total_truth_bytes)
        .and_then(|value| value.checked_add(reference_input_bytes))
        .context("alignment input size overflow")?;
    let mut command = format!(
        "set -eu\nmkdir -p /result/repetitions\nprefix=\"/result/repetitions/repetition-{{repetition}}\"\nindex=\"/scratch/alignment-index-{{repetition}}/reference\"\nmkdir -p \"$(dirname \"$index\")\"\n{}\ncohort=\"${{prefix}}.alignment-truth-summary.tsv\"\nprintf 'sample_id\\tmetric\\tvalue\\n' > \"$cohort\"\n",
        index_command(tool, "$index")?,
    );
    for sample in samples {
        let stem = normalized_id(&sample.sample_id);
        let prefix = format!("${{prefix}}.{stem}");
        command.push_str(&format!(
            "{} > \"{}.sam\"\nawk -v mode=summary -v truth_path={} -f {NORMALIZER} \"{}.sam\" > \"{}.alignment-truth-summary.tsv\"\n",
            align_command(tool, sample, "$index")?,
            prefix,
            quote(&sample.truth_mount),
            prefix,
            prefix,
        ));
        command.push_str(&format!(
            "awk -v mode=bundle -v tool={} -v sample_id={} -f {NORMALIZER} \"{}.alignment-truth-summary.tsv\" > \"{}.alignment-truth-summary.json\"\nawk -v mode=probe -f {NORMALIZER} \"{}.alignment-truth-summary.tsv\" > \"{}.report-qc.json\"\n",
            quote(tool), quote(&sample.sample_id), prefix, prefix, prefix, prefix,
        ));
        let sample_input = sample
            .fastq_input_bytes
            .checked_add(sample.truth_input_bytes)
            .and_then(|value| value.checked_add(reference_input_bytes))
            .context("alignment sample input size overflow")?;
        command.push_str(&format!(
            "sample_output_bytes=$(wc -c \"{}.sam\" \"{}.alignment-truth-summary.tsv\" \"{}.alignment-truth-summary.json\" \"{}.report-qc.json\" | awk 'END {{print $1}}')\nawk -v mode=metrics -v fastq_input_bytes={} -v reference_input_bytes={} -v reference_index_input_bytes=0 -v truth_input_bytes={} -v input_bytes={} -v output_bytes=\"$sample_output_bytes\" -f {NORMALIZER} \"{}.alignment-truth-summary.tsv\" > \"{}.metrics.json\"\nawk -v sample_id={} 'NR > 1 {{print sample_id \"\\t\" $0}}' \"{}.alignment-truth-summary.tsv\" >> \"$cohort\"\n",
            prefix, prefix, prefix, prefix,
            sample.fastq_input_bytes, reference_input_bytes, sample.truth_input_bytes, sample_input,
            prefix, prefix, quote(&sample.sample_id), prefix,
        ));
    }
    command.push_str(&format!(
        "output_bytes=$(wc -c \"${{prefix}}\".* | awk 'END {{print $1}}')\nawk -v mode=metrics -v expected_samples={} -v expected_metrics=7 -v input_bytes={} -v output_bytes=\"$output_bytes\" -f /repository/benchmarks/execution/bam_cohort_summary.awk \"$cohort\" > \"${{prefix}}.metrics.json\"\nawk -v mode=probe -v expected_samples={} -v expected_metrics=7 -v consumer_stage_id='benchmark.bam.align.compare' -v input_contract='bijux.benchmark.bam.align.bundle.v1' -f /repository/benchmarks/execution/bam_cohort_summary.awk \"$cohort\" > \"${{prefix}}.report-qc.json\"",
        samples.len(), total_input_bytes, samples.len(),
    ));
    Ok(shell(command))
}

pub(super) fn negative_command(tool: &str) -> Vec<String> {
    shell(match tool {
        "bowtie2" => "bowtie2 -x /input/missing-reference -U /input/missing.fastq.gz".to_string(),
        "bwa" => "bwa mem /input/missing-reference /input/missing.fastq.gz".to_string(),
        _ => "false".to_string(),
    })
}

pub(super) fn required_files(samples: &[AlignmentCommandSample], repetitions: u32) -> Vec<String> {
    let mut files = Vec::new();
    for repetition in 1..=repetitions {
        let repetition_prefix = format!("repetitions/repetition-{repetition}");
        for sample in samples {
            let prefix = format!("{repetition_prefix}.{}", normalized_id(&sample.sample_id));
            files.extend([
                format!("{prefix}.sam"),
                format!("{prefix}.alignment-truth-summary.tsv"),
                format!("{prefix}.alignment-truth-summary.json"),
                format!("{prefix}.report-qc.json"),
                format!("{prefix}.metrics.json"),
            ]);
        }
        files.extend([
            format!("{repetition_prefix}.alignment-truth-summary.tsv"),
            format!("{repetition_prefix}.metrics.json"),
            format!("{repetition_prefix}.report-qc.json"),
            format!("{repetition_prefix}.resource.tsv"),
        ]);
    }
    files
}

fn index_command(tool: &str, index: &str) -> Result<String> {
    match tool {
        "bowtie2" => Ok(format!("bowtie2-build --threads 8 {REFERENCE} {index}")),
        "bwa" => Ok(format!("bwa index -p {index} {REFERENCE}")),
        _ => bail!("unsupported alignment tool: {tool}"),
    }
}

fn align_command(tool: &str, sample: &AlignmentCommandSample, index: &str) -> Result<String> {
    let reads = sample.fastq_mounts.iter().map(|path| quote(path)).collect::<Vec<_>>();
    match (tool, sample.layout, reads.as_slice()) {
        ("bowtie2", bijux_dna_bench_model::BenchmarkFastqLayout::SingleEnd, [reads]) => {
            Ok(format!("bowtie2 --threads 8 -x {index} -U {reads}"))
        }
        ("bowtie2", bijux_dna_bench_model::BenchmarkFastqLayout::PairedEnd, [left, right]) => {
            Ok(format!("bowtie2 --threads 8 -x {index} -1 {left} -2 {right}"))
        }
        ("bwa", bijux_dna_bench_model::BenchmarkFastqLayout::SingleEnd, [reads]) => {
            Ok(format!("bwa mem -t 8 -R {} {index} {reads}", quote(&read_group(&sample.sample_id))))
        }
        ("bwa", bijux_dna_bench_model::BenchmarkFastqLayout::PairedEnd, [left, right]) => {
            Ok(format!(
                "bwa mem -t 8 -R {} {index} {left} {right}",
                quote(&read_group(&sample.sample_id))
            ))
        }
        _ => bail!("alignment sample FASTQ layout does not close over mounted files"),
    }
}

fn read_group(sample_id: &str) -> String {
    format!("@RG\\tID:{sample_id}\\tSM:{sample_id}\\tPL:ILLUMINA")
}

fn shell(command: String) -> Vec<String> {
    vec!["sh".to_string(), "-c".to_string(), command]
}

fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use bijux_dna_bench_model::BenchmarkFastqLayout;

    use super::{measured_command, required_files, AlignmentCommandSample};

    #[test]
    fn both_aligners_bind_truth_and_complete_every_sample_output() -> anyhow::Result<()> {
        let samples = vec![AlignmentCommandSample {
            sample_id: "sample-a".to_string(),
            layout: BenchmarkFastqLayout::PairedEnd,
            fastq_mounts: vec![
                "/input/a_R1.fastq.gz".to_string(),
                "/input/a_R2.fastq.gz".to_string(),
            ],
            fastq_input_bytes: 20,
            truth_mount: "/input/a.truth.tsv".to_string(),
            truth_input_bytes: 3,
        }];
        for tool in ["bowtie2", "bwa"] {
            let command = measured_command(tool, &samples, 100, 20, 3)?;
            assert!(command[2].contains("truth_path='/input/a.truth.tsv'"));
            assert!(command[2].contains("bam_cohort_summary.awk"));
            assert!(command[2].contains("reference_index_input_bytes=0"));
        }
        assert_eq!(required_files(&samples, 3).len(), 27);
        Ok(())
    }
}
