//! Owner: bijux-dna-bench
//! Container command and output closure for exact FASTQ read-length attempts.

use super::inputs::CommandSample;

const MANIFEST_HEADER: &str = "sample_id\tinput_layout\tfile_count\tread_count\tminimum_read_length\tmean_read_length\tmedian_read_length\tmaximum_read_length\tdistinct_read_lengths\thistogram_artifact\thistogram_sha256";

pub(super) fn warmup_command(tool: &str, samples: &[CommandSample]) -> Vec<String> {
    let mut commands = vec!["set -eu".to_string()];
    commands.extend(
        samples
            .iter()
            .map(|sample| format!("{} > /dev/null", tool_command(tool, &sample.mounted_fastq))),
    );
    shell(commands.join("; "))
}

pub(super) fn negative_command(tool: &str) -> Vec<String> {
    shell(tool_command(tool, &["/input/missing.fastq.gz".to_string()]))
}

pub(super) fn measured_command(
    tool: &str,
    samples: &[CommandSample],
    input_bytes: u64,
) -> Vec<String> {
    let mut lines = vec![
        "set -eu".to_string(),
        "mkdir -p /result/repetitions".to_string(),
        "prefix=\"/result/repetitions/repetition-{repetition}\"".to_string(),
        "manifest=\"${prefix}.manifest.tsv\"".to_string(),
        format!("printf '%s\\n' {} > \"$manifest\"", shell_quote(MANIFEST_HEADER)),
    ];
    let mut output_paths = vec!["\"$manifest\"".to_string()];
    for sample in samples {
        let raw = format!("\"${{prefix}}.{}.raw.tsv\"", sample.sample_id);
        let summary = format!("\"${{prefix}}.{}.summary.tsv\"", sample.sample_id);
        let histogram = format!("\"${{prefix}}.{}.histogram.tsv\"", sample.sample_id);
        let artifact =
            format!("repetitions/repetition-{{repetition}}.{}.histogram.tsv", sample.sample_id);
        lines.extend([
            format!("{} > {raw}", tool_command(tool, &sample.mounted_fastq)),
            format!("observed_exact_sha=$(sha256sum {raw} | awk '{{print $1}}')"),
            format!(
                "[ \"$observed_exact_sha\" = \"{}\" ]",
                sample.expected_exact_sha256
            ),
            format!(
                "awk -v mode=summary -v sample_id={} -v input_layout={} -f /repository/benchmarks/execution/fastq_read_lengths.awk {raw} > {summary}",
                shell_quote(&sample.sample_id),
                sample.input_layout,
            ),
            format!(
                "diff -u {} {summary}",
                shell_quote(&sample.expected_summary_mount)
            ),
            format!(
                "awk -v mode=histogram -v sample_id={} -v input_layout={} -f /repository/benchmarks/execution/fastq_read_lengths.awk {raw} > {histogram}",
                shell_quote(&sample.sample_id),
                sample.input_layout,
            ),
            format!(
                "diff -u {} {histogram}",
                shell_quote(&sample.expected_histogram_mount)
            ),
            format!("observed_histogram_sha=$(sha256sum {histogram} | awk '{{print $1}}')"),
            format!(
                "tail -n +2 {summary} | awk -v OFS='\\t' -v artifact={} -v sha=\"$observed_histogram_sha\" '{{print $0, artifact, sha}}' >> \"$manifest\"",
                shell_quote(&artifact),
            ),
        ]);
        output_paths.extend([raw, summary, histogram]);
    }
    lines.extend([
        format!(
            "awk -v tool={} -v mode=bundle -f /repository/benchmarks/execution/fastq_read_lengths.awk \"$manifest\" > \"${{prefix}}.read-lengths.json\"",
            shell_quote(tool),
        ),
        "awk -v mode=probe -f /repository/benchmarks/execution/fastq_read_lengths.awk \"$manifest\" > \"${prefix}.report-qc.json\"".to_string(),
        format!(
            "output_bytes=$(wc -c {} \"${{prefix}}.read-lengths.json\" \"${{prefix}}.report-qc.json\" | awk 'END {{print $1}}')",
            output_paths.join(" ")
        ),
        format!(
            "awk -v mode=metrics -v input_bytes={input_bytes} -v output_bytes=\"$output_bytes\" -f /repository/benchmarks/execution/fastq_read_lengths.awk \"$manifest\" > \"${{prefix}}.metrics.json\""
        ),
    ]);
    shell(lines.join("\n"))
}

pub(super) fn required_files(samples: &[CommandSample], repetitions: u32) -> Vec<String> {
    let mut files = Vec::new();
    for repetition in 1..=repetitions {
        let prefix = format!("repetitions/repetition-{repetition}");
        files.push(format!("{prefix}.manifest.tsv"));
        for sample in samples {
            files.extend([
                format!("{prefix}.{}.raw.tsv", sample.sample_id),
                format!("{prefix}.{}.summary.tsv", sample.sample_id),
                format!("{prefix}.{}.histogram.tsv", sample.sample_id),
            ]);
        }
        files.extend([
            format!("{prefix}.read-lengths.json"),
            format!("{prefix}.report-qc.json"),
            format!("{prefix}.metrics.json"),
            format!("{prefix}.resource.tsv"),
        ]);
    }
    files
}

fn tool_command(tool: &str, mounted_fastq: &[String]) -> String {
    match tool {
        "seqfu" => {
            let commands = mounted_fastq.iter().map(|path| {
                let path = shell_quote(path);
                format!("test -s {path} && seqfu tab {path} | awk -F '\\t' '{{print $1 \"\\t\" length($3)}}'")
            });
            format!("( {} )", commands.collect::<Vec<_>>().join("; "))
        }
        "seqkit" => {
            let quoted = mounted_fastq.iter().map(|path| shell_quote(path)).collect::<Vec<_>>();
            format!(
                "test -s {} && seqkit fx2tab --name --only-id --length --threads 4 {} | awk -F '\\t' 'NF < 2 {{ exit 2 }} {{ print $1 \"\\t\" $NF }}'",
                quoted.join(" && test -s "),
                quoted.join(" "),
            )
        }
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
    use crate::workflow::fastq_profile_read_lengths::inputs::CommandSample;

    fn sample(id: &str, layout: &'static str, files: &[&str]) -> CommandSample {
        CommandSample {
            sample_id: id.to_string(),
            input_layout: layout,
            mounted_fastq: files.iter().map(|path| (*path).to_string()).collect(),
            expected_summary_mount: format!("/input/{id}.summary.tsv"),
            expected_histogram_mount: format!("/input/{id}.histogram.tsv"),
            expected_exact_sha256: "a".repeat(64),
        }
    }

    #[test]
    fn commands_bind_exact_truth_and_complete_repetition_outputs() {
        let samples = [
            sample("ERR1", "single_end", &["/input/ERR1.fastq.gz"]),
            sample("ERR2", "paired_end", &["/input/ERR2_1.fastq.gz", "/input/ERR2_2.fastq.gz"]),
        ];
        let warmup = warmup_command("seqkit", &samples);
        let measured = measured_command("seqkit", &samples, 42);
        assert!(warmup[2].contains("seqkit fx2tab") && warmup[2].contains("> /dev/null"));
        assert!(measured[2].contains("repetition-{repetition}"));
        assert!(measured[2].contains("observed_exact_sha"));
        assert!(measured[2].contains("diff -u '/input/ERR2.histogram.tsv'"));
        assert!(negative_command("seqfu")[2].contains("missing.fastq.gz"));
        let required = required_files(&samples, 3);
        assert_eq!(required.len(), 33);
        assert!(required.contains(&"repetitions/repetition-3.resource.tsv".to_string()));
    }
}
