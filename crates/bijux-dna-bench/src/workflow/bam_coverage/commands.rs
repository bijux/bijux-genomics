//! Owner: bijux-dna-bench
//! Container commands and output closure for BAM coverage attempts.

const BAM: &str = "/input/ERR2112767.sorted.bam";

pub(super) fn warmup_command(tool: &str) -> Vec<String> {
    shell(match tool {
        "bedtools" => format!("bedtools genomecov -ibam {BAM} -bga > /scratch/warmup.bedgraph"),
        "mosdepth" => format!("mosdepth --threads 4 --no-per-base /scratch/warmup {BAM}"),
        "samtools" => format!("samtools depth {BAM} > /scratch/warmup.depth.tsv"),
        _ => "false".to_string(),
    })
}

pub(super) fn negative_command(tool: &str) -> Vec<String> {
    match tool {
        "bedtools" => strings(&["bedtools", "genomecov", "-ibam", "/input/missing.bam"]),
        "mosdepth" => strings(&["mosdepth", "/scratch/missing", "/input/missing.bam"]),
        "samtools" => strings(&["samtools", "depth", "/input/missing.bam"]),
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
    let (tool_command, raw_inputs, raw_files) = match tool {
        "bedtools" => (
            format!("bedtools genomecov -ibam {BAM} -bga > \"${{prefix}}.bedgraph\""),
            "\"${prefix}.bedgraph\"".to_string(),
            vec!["bedgraph"],
        ),
        "mosdepth" => (
            format!(
                "mosdepth --threads 4 --no-per-base --quantize 0:1: \"${{prefix}}\" {BAM}\ngzip -dc \"${{prefix}}.quantized.bed.gz\" > \"${{prefix}}.quantized.bed\""
            ),
            "\"${prefix}.mosdepth.summary.txt\" \"${prefix}.quantized.bed\" /input/reference-span.tsv"
                .to_string(),
            vec![
                "mosdepth.summary.txt",
                "mosdepth.global.dist.txt",
                "quantized.bed.gz",
                "quantized.bed.gz.csi",
                "quantized.bed",
            ],
        ),
        "samtools" => (
            format!("samtools depth {BAM} > \"${{prefix}}.depth.tsv\""),
            "\"${prefix}.depth.tsv\" /input/reference-span.tsv".to_string(),
            vec!["depth.tsv"],
        ),
        _ => ("false".to_string(), String::new(), Vec::new()),
    };
    let raw_wc = raw_files
        .iter()
        .map(|suffix| format!("\"${{prefix}}.{suffix}\""))
        .collect::<Vec<_>>()
        .join(" ");
    shell(format!(
        "set -eu
mkdir -p /result/repetitions
prefix=\"/result/repetitions/repetition-{{repetition}}\"
summary=\"${{prefix}}.coverage-summary.tsv\"
{tool_command}
awk -v mode=summary -v source={} -f /repository/benchmarks/execution/bam_coverage.awk {raw_inputs} > \"$summary\"
awk -v mode=bundle -v tool={} -v sample_id={} -f /repository/benchmarks/execution/bam_coverage.awk \"$summary\" > \"${{prefix}}.coverage-summary.json\"
awk -v mode=probe -f /repository/benchmarks/execution/bam_coverage.awk \"$summary\" > \"${{prefix}}.report-qc.json\"
output_bytes=$(wc -c {raw_wc} \"$summary\" \"${{prefix}}.coverage-summary.json\" \"${{prefix}}.report-qc.json\" | awk 'END {{print $1}}')
awk -v mode=metrics -v bam_input_bytes={bam_bytes} -v bam_index_input_bytes={bam_index_bytes} -v input_bytes={input_bytes} -v output_bytes=\"$output_bytes\" -f /repository/benchmarks/execution/bam_coverage.awk \"$summary\" > \"${{prefix}}.metrics.json\"",
        quote(tool),
        quote(tool),
        quote(sample_id),
    ))
}

pub(super) fn required_files(tool: &str, repetitions: u32) -> Vec<String> {
    let raw = match tool {
        "bedtools" => vec!["bedgraph"],
        "mosdepth" => vec![
            "mosdepth.summary.txt",
            "mosdepth.global.dist.txt",
            "quantized.bed.gz",
            "quantized.bed.gz.csi",
            "quantized.bed",
        ],
        "samtools" => vec!["depth.tsv"],
        _ => Vec::new(),
    };
    let mut files = Vec::new();
    for repetition in 1..=repetitions {
        let prefix = format!("repetitions/repetition-{repetition}");
        files.extend(raw.iter().map(|suffix| format!("{prefix}.{suffix}")));
        files.extend([
            format!("{prefix}.coverage-summary.tsv"),
            format!("{prefix}.coverage-summary.json"),
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
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use super::{measured_command, required_files};

    #[test]
    fn coverage_normalizer_closes_all_backend_shapes() -> anyhow::Result<()> {
        let temporary = bijux_dna_infra::temp_dir("bam-coverage-normalizer")?;
        let bedgraph = temporary.path().join("coverage.bedgraph");
        let depth = temporary.path().join("coverage.depth.tsv");
        let summary = temporary.path().join("coverage.mosdepth.summary.txt");
        let quantized = temporary.path().join("coverage.quantized.bed");
        std::fs::write(&bedgraph, b"chr1\t0\t4\t0\nchr1\t4\t10\t2\n")?;
        std::fs::write(
            &depth,
            b"chr1\t5\t2\nchr1\t6\t2\nchr1\t7\t2\nchr1\t8\t2\nchr1\t9\t2\nchr1\t10\t2\n",
        )?;
        let reference_span = temporary.path().join("coverage.reference-span.tsv");
        std::fs::write(
            &reference_span,
            b"schema_version\tbijux.benchmark.bam.reference-span.v1\nsource_bam_sha256\tabc\nreference_bases\t10\n",
        )?;
        std::fs::write(
            &summary,
            b"chrom\tlength\tbases\tmean\tmin\tmax\ntotal\t8\t12\t1.5\t0\t2\n",
        )?;
        std::fs::write(&quantized, b"chr1\t0\t4\t0:1\nchr1\t4\t10\t1:inf\n")?;

        let expected = "metric\tvalue\nreference_bases\t10\ncovered_bases\t6\ncoverage_fraction\t0.600000000000\nmean_depth\t1.200000000000\n";
        assert_eq!(normalize("bedtools", &[bedgraph, reference_span.clone()])?, expected);
        assert_eq!(normalize("samtools", &[depth, reference_span])?, expected);
        let reference_span = temporary.path().join("coverage.reference-span.tsv");
        assert_eq!(normalize("mosdepth", &[summary, quantized, reference_span])?, expected);
        Ok(())
    }

    #[test]
    fn coverage_commands_declare_every_backend_output() {
        for tool in ["bedtools", "mosdepth", "samtools"] {
            let command = measured_command(tool, "ERR2112767", 23, 3, 26);
            assert!(command[2].contains("bam_coverage.awk"));
            assert!(command[2].contains("/input/reference-span.tsv"));
            if tool == "samtools" {
                assert!(!command[2].contains("samtools view -H"));
                assert!(!command[2].contains("depth -aa"));
            }
            if tool == "mosdepth" {
                assert!(command[2].contains("--quantize 0:1:"));
                assert!(!command[2].contains("global.dist.txt\" /input/reference-span.tsv"));
            }
            let required = required_files(tool, 3);
            assert!(required.contains(&"repetitions/repetition-3.metrics.json".to_string()));
            assert!(required.contains(&"repetitions/repetition-3.resource.tsv".to_string()));
        }
    }

    fn normalize(source: &str, inputs: &[PathBuf]) -> anyhow::Result<String> {
        let script = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("benchmarks/execution/bam_coverage.awk");
        let output = Command::new("awk")
            .args(["-v", "mode=summary", "-v", &format!("source={source}"), "-f"])
            .arg(script)
            .args(inputs)
            .output()?;
        anyhow::ensure!(output.status.success(), "coverage AWK failed");
        Ok(String::from_utf8(output.stdout)?)
    }
}
