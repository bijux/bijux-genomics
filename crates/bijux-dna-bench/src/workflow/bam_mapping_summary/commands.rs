//! Owner: bijux-dna-bench
//! Container commands and output closure for BAM mapping-summary attempts.

const BAM: &str = "/input/ERR2112767.sorted.bam";

pub(super) fn warmup_command(tool: &str) -> Vec<String> {
    match tool {
        "picard" => shell(format!(
            "set -eu; mkdir -p /scratch/picard-warmup; picard CollectAlignmentSummaryMetrics I={BAM} O=/scratch/warmup.tsv VALIDATION_STRINGENCY=SILENT TMP_DIR=/scratch/picard-warmup"
        )),
        "samtools" => shell(format!("samtools flagstat {BAM} > /scratch/warmup.txt")),
        _ => shell("false".to_string()),
    }
}

pub(super) fn negative_command(tool: &str) -> Vec<String> {
    match tool {
        "picard" => vec![
            "picard".to_string(),
            "CollectAlignmentSummaryMetrics".to_string(),
            "I=/input/missing.bam".to_string(),
            "O=/scratch/missing.tsv".to_string(),
        ],
        "samtools" => {
            vec!["samtools".to_string(), "flagstat".to_string(), "/input/missing.bam".to_string()]
        }
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
    let raw_suffix = if tool == "picard" { "picard.tsv" } else { "flagstat.txt" };
    let tool_command = match tool {
        "picard" => format!(
            "mkdir -p /scratch/picard-{{repetition}}; picard CollectAlignmentSummaryMetrics I={BAM} O=\"$raw\" VALIDATION_STRINGENCY=SILENT TMP_DIR=/scratch/picard-{{repetition}}"
        ),
        "samtools" => format!("samtools flagstat {BAM} > \"$raw\""),
        _ => "false".to_string(),
    };
    let script = format!(
        "set -eu
mkdir -p /result/repetitions
prefix=\"/result/repetitions/repetition-{{repetition}}\"
raw=\"${{prefix}}.{raw_suffix}\"
summary=\"${{prefix}}.mapping-summary.tsv\"
{tool_command}
awk -v mode=summary -v source={} -f /repository/benchmarks/execution/bam_mapping_summary.awk \"$raw\" > \"$summary\"
awk -v mode=bundle -v tool={} -v sample_id={} -f /repository/benchmarks/execution/bam_mapping_summary.awk \"$summary\" > \"${{prefix}}.mapping-summary.json\"
awk -v mode=probe -f /repository/benchmarks/execution/bam_mapping_summary.awk \"$summary\" > \"${{prefix}}.report-qc.json\"
output_bytes=$(wc -c \"$raw\" \"$summary\" \"${{prefix}}.mapping-summary.json\" \"${{prefix}}.report-qc.json\" | awk 'END {{print $1}}')
awk -v mode=metrics -v bam_input_bytes={bam_bytes} -v bam_index_input_bytes={bam_index_bytes} -v input_bytes={} -v output_bytes=\"$output_bytes\" -f /repository/benchmarks/execution/bam_mapping_summary.awk \"$summary\" > \"${{prefix}}.metrics.json\"",
        shell_quote(tool),
        shell_quote(tool),
        shell_quote(sample_id),
        input_bytes,
    );
    shell(script)
}

pub(super) fn required_files(tool: &str, repetitions: u32) -> Vec<String> {
    let raw_suffix = if tool == "picard" { "picard.tsv" } else { "flagstat.txt" };
    let mut files = Vec::new();
    for repetition in 1..=repetitions {
        let prefix = format!("repetitions/repetition-{repetition}");
        files.extend([
            format!("{prefix}.{raw_suffix}"),
            format!("{prefix}.mapping-summary.tsv"),
            format!("{prefix}.mapping-summary.json"),
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

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::{measured_command, negative_command, required_files, warmup_command};

    #[test]
    fn commands_close_both_mapping_summary_tools() {
        assert!(warmup_command("picard")[2].contains("CollectAlignmentSummaryMetrics"));
        assert!(warmup_command("samtools")[2].contains("samtools flagstat"));
        let measured = measured_command("picard", "ERR2112767", 23, 3, 26);
        assert!(measured[2].contains("bam_mapping_summary.awk"));
        assert!(measured[2].contains("raw=\"${prefix}.picard.tsv\""));
        assert!(negative_command("samtools").contains(&"/input/missing.bam".to_string()));
        let required = required_files("samtools", 3);
        assert_eq!(required.len(), 18);
        assert!(required.contains(&"repetitions/repetition-3.resource.tsv".to_string()));
    }
}
