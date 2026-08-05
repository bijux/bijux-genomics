//! Owner: bijux-dna-bench
//! Container commands and output closure for FASTQ read-profile attempts.

const PROFILE_HEADER: &str = "sample_id\tinput_layout\tfile_count\tread_count\tbase_count\tminimum_read_length\tmaximum_read_length";

pub(super) fn warmup_command(tool: &str, mounted_fastq: &[String]) -> Vec<String> {
    shell(format!(
        "set -eu; {} > /scratch/fastq-profile-warmup.tsv",
        tool_command(tool, mounted_fastq)
    ))
}

pub(super) fn negative_command(tool: &str) -> Vec<String> {
    shell(tool_command(tool, &["/input/missing.fastq.gz".to_string()]))
}

pub(super) fn measured_command(
    tool: &str,
    mounted_fastq: &[String],
    input_bytes: u64,
) -> Vec<String> {
    let command = tool_command(tool, mounted_fastq);
    let script = format!(
        "set -eu\n\
         mkdir -p /result/repetitions\n\
         prefix=\"/result/repetitions/repetition-{{repetition}}\"\n\
         raw=\"${{prefix}}.raw.tsv\"\n\
         profile=\"${{prefix}}.read-profile.tsv\"\n\
         {command} > \"$raw\"\n\
         printf '%s\\n' {} > \"$profile\"\n\
         awk -v mode=normalize -v tool={} -f /repository/benchmarks/execution/fastq_profile_reads.awk \"$raw\" | sort >> \"$profile\"\n\
         cut -f1-5 \"$profile\" > \"${{prefix}}.admission-observed.tsv\"\n\
         diff -u /input/expected-profile.tsv \"${{prefix}}.admission-observed.tsv\"\n\
         awk -v mode=bundle -v tool={} -f /repository/benchmarks/execution/fastq_profile_reads.awk \"$profile\" > \"${{prefix}}.read-profile.json\"\n\
         awk -v mode=probe -f /repository/benchmarks/execution/fastq_profile_reads.awk \"$profile\" > \"${{prefix}}.report-qc.json\"\n\
         output_bytes=$(wc -c \"$raw\" \"$profile\" \"${{prefix}}.read-profile.json\" \"${{prefix}}.report-qc.json\" | awk 'END {{print $1}}')\n\
         awk -v mode=metrics -v input_bytes={input_bytes} -v output_bytes=\"$output_bytes\" -f /repository/benchmarks/execution/fastq_profile_reads.awk \"$profile\" > \"${{prefix}}.metrics.json\"",
        shell_quote(PROFILE_HEADER),
        shell_quote(tool),
        shell_quote(tool),
    );
    shell(script)
}

pub(super) fn required_files(repetitions: u32) -> Vec<String> {
    let mut files = Vec::new();
    for repetition in 1..=repetitions {
        let prefix = format!("repetitions/repetition-{repetition}");
        files.extend([
            format!("{prefix}.raw.tsv"),
            format!("{prefix}.read-profile.tsv"),
            format!("{prefix}.read-profile.json"),
            format!("{prefix}.report-qc.json"),
            format!("{prefix}.admission-observed.tsv"),
            format!("{prefix}.metrics.json"),
            format!("{prefix}.resource.tsv"),
        ]);
    }
    files
}

fn tool_command(tool: &str, mounted_fastq: &[String]) -> String {
    let inputs = mounted_fastq.iter().map(|path| shell_quote(path)).collect::<Vec<_>>().join(" ");
    match tool {
        "seqfu" => format!("seqfu stats --threads 4 --gc {inputs}"),
        "seqkit_stats" => format!("seqkit_stats -a -T -j 4 {inputs}"),
        "seqkit" => format!("seqkit stats -a -T -j 4 {inputs}"),
        _ => "false".to_string(),
    }
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
    fn commands_bind_profile_truth_and_complete_repetition_outputs() {
        let inputs = vec!["/input/ERR1.fastq.gz".to_string()];
        assert!(warmup_command("seqfu", &inputs)[2].contains("seqfu stats"));
        let measured = measured_command("seqkit_stats", &inputs, 42);
        assert!(measured[2].contains("seqkit_stats -a -T -j 4"));
        assert!(measured[2].contains("diff -u /input/expected-profile.tsv"));
        assert!(negative_command("seqkit")[2].contains("missing.fastq.gz"));
        let required = required_files(3);
        assert_eq!(required.len(), 21);
        assert!(required.contains(&"repetitions/repetition-3.resource.tsv".to_string()));
    }
}
