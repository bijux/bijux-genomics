//! Owner: bijux-dna-bench
//! Container commands and output closure for BAM duplication-metrics attempts.

const BAM: &str = "/input/ERR2112767.sorted.bam";

pub(super) fn warmup_command(tool: &str) -> Vec<String> {
    shell(match tool {
        "picard" => format!(
            "picard MarkDuplicates I={BAM} O=/scratch/warmup.bam M=/scratch/warmup.metrics.txt ASSUME_SORTED=true VALIDATION_STRINGENCY=SILENT TMP_DIR=/scratch"
        ),
        "samtools" => format!(
            "samtools collate -u -O {BAM} | samtools fixmate -m -u - - | samtools sort -u -@ 4 -T /scratch/warmup-sort - | samtools markdup -s -@ 4 - /dev/null"
        ),
        _ => "false".to_string(),
    })
}

pub(super) fn negative_command(tool: &str) -> Vec<String> {
    match tool {
        "picard" => strings(&[
            "picard",
            "MarkDuplicates",
            "I=/input/missing.bam",
            "O=/scratch/missing.bam",
            "M=/scratch/missing.metrics.txt",
        ]),
        "samtools" => strings(&["samtools", "markdup", "/input/missing.bam", "/dev/null"]),
        _ => shell("false".to_string()),
    }
}

pub(super) fn measured_command(
    tool: &str,
    sample_id: &str,
    bam_bytes: u64,
    bam_index_bytes: u64,
    input_bytes: u64,
) -> Vec<String> {
    let (tool_command, raw_suffix) = match tool {
        "picard" => (
            format!(
                "picard MarkDuplicates I={BAM} O=\"/scratch/repetition-{{repetition}}.picard.bam\" M=\"${{prefix}}.picard.metrics.txt\" ASSUME_SORTED=true VALIDATION_STRINGENCY=SILENT TMP_DIR=/scratch"
            ),
            "picard.metrics.txt",
        ),
        "samtools" => (
            format!(
                "samtools collate -u -O {BAM} | samtools fixmate -m -u - - | samtools sort -u -@ 4 -T \"/scratch/repetition-{{repetition}}-sort\" - | samtools markdup -s -@ 4 - /dev/null 2> \"${{prefix}}.samtools.stats.txt\""
            ),
            "samtools.stats.txt",
        ),
        _ => ("false".to_string(), "invalid"),
    };
    shell(format!(
        "set -eu
mkdir -p /result/repetitions
prefix=\"/result/repetitions/repetition-{{repetition}}\"
raw=\"${{prefix}}.{raw_suffix}\"
summary=\"${{prefix}}.duplication-summary.tsv\"
{tool_command}
awk -v mode=summary -v source={} -f /repository/benchmarks/execution/bam_duplication_metrics.awk \"$raw\" > \"$summary\"
awk -v mode=bundle -v tool={} -v sample_id={} -f /repository/benchmarks/execution/bam_duplication_metrics.awk \"$summary\" > \"${{prefix}}.duplication-summary.json\"
awk -v mode=probe -f /repository/benchmarks/execution/bam_duplication_metrics.awk \"$summary\" > \"${{prefix}}.report-qc.json\"
output_bytes=$(wc -c \"$raw\" \"$summary\" \"${{prefix}}.duplication-summary.json\" \"${{prefix}}.report-qc.json\" | awk 'END {{print $1}}')
awk -v mode=metrics -v bam_input_bytes={bam_bytes} -v bam_index_input_bytes={bam_index_bytes} -v input_bytes={input_bytes} -v output_bytes=\"$output_bytes\" -f /repository/benchmarks/execution/bam_duplication_metrics.awk \"$summary\" > \"${{prefix}}.metrics.json\"",
        quote(tool),
        quote(tool),
        quote(sample_id),
    ))
}

pub(super) fn required_files(tool: &str, repetitions: u32) -> Vec<String> {
    let raw_suffix = match tool {
        "picard" => "picard.metrics.txt",
        "samtools" => "samtools.stats.txt",
        _ => "invalid",
    };
    let mut files = Vec::new();
    for repetition in 1..=repetitions {
        let prefix = format!("repetitions/repetition-{repetition}");
        files.extend([
            format!("{prefix}.{raw_suffix}"),
            format!("{prefix}.duplication-summary.tsv"),
            format!("{prefix}.duplication-summary.json"),
            format!("{prefix}.report-qc.json"),
            format!("{prefix}.metrics.json"),
            format!("{prefix}.resource.tsv"),
        ]);
    }
    files
}

fn shell(command: String) -> Vec<String> {
    vec!["sh".to_string(), "-c".to_string(), command]
}

fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::{measured_command, required_files};

    #[test]
    fn duplication_commands_stream_samtools_fixmate_preparation() {
        let command = measured_command("samtools", "ERR2112767", 23, 3, 26);
        assert!(command[2].contains("samtools collate"));
        assert!(command[2].contains("samtools fixmate -m"));
        assert!(command[2].contains("samtools markdup -s"));
        assert_eq!(required_files("samtools", 3).len(), 18);
    }
}
