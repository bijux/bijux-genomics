//! Owner: bijux-dna-bench
//! Container commands and output closure for endogenous-content attempts.

const BAM: &str = "/input/ERR2112767.sorted.bam";

pub(super) fn warmup_command() -> Vec<String> {
    shell(format!("samtools view -c -F 2304 {BAM} > /scratch/warmup.count"))
}

pub(super) fn negative_command() -> Vec<String> {
    vec![
        "samtools".to_string(),
        "view".to_string(),
        "-c".to_string(),
        "/input/missing.bam".to_string(),
    ]
}

pub(super) fn measured_command(
    sample_id: &str,
    bam_bytes: u64,
    bam_index_bytes: u64,
    input_bytes: u64,
) -> Vec<String> {
    shell(format!(
        "set -eu
mkdir -p /result/repetitions
prefix=\"/result/repetitions/repetition-{{repetition}}\"
raw=\"${{prefix}}.counts.tsv\"
summary=\"${{prefix}}.endogenous-content.tsv\"
printf 'metric\\tvalue\\n' > \"$raw\"
printf 'total_primary_reads\\t%s\\n' \"$(samtools view -c -F 2304 {BAM})\" >> \"$raw\"
printf 'mapped_primary_reads\\t%s\\n' \"$(samtools view -c -F 2308 {BAM})\" >> \"$raw\"
printf 'mapped_nonduplicate_primary_reads\\t%s\\n' \"$(samtools view -c -F 3332 {BAM})\" >> \"$raw\"
awk -v mode=summary -f /repository/benchmarks/execution/bam_endogenous_content.awk \"$raw\" > \"$summary\"
awk -v mode=bundle -v tool=samtools -v sample_id={} -f /repository/benchmarks/execution/bam_endogenous_content.awk \"$summary\" > \"${{prefix}}.endogenous-content.json\"
awk -v mode=probe -f /repository/benchmarks/execution/bam_endogenous_content.awk \"$summary\" > \"${{prefix}}.report-qc.json\"
output_bytes=$(wc -c \"$raw\" \"$summary\" \"${{prefix}}.endogenous-content.json\" \"${{prefix}}.report-qc.json\" | awk 'END {{print $1}}')
awk -v mode=metrics -v bam_input_bytes={bam_bytes} -v bam_index_input_bytes={bam_index_bytes} -v input_bytes={input_bytes} -v output_bytes=\"$output_bytes\" -f /repository/benchmarks/execution/bam_endogenous_content.awk \"$summary\" > \"${{prefix}}.metrics.json\"",
        quote(sample_id),
    ))
}

pub(super) fn required_files(repetitions: u32) -> Vec<String> {
    let mut files = Vec::new();
    for repetition in 1..=repetitions {
        let prefix = format!("repetitions/repetition-{repetition}");
        files.extend([
            format!("{prefix}.counts.tsv"),
            format!("{prefix}.endogenous-content.tsv"),
            format!("{prefix}.endogenous-content.json"),
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

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use super::{measured_command, required_files};

    #[test]
    fn endogenous_normalizer_enforces_primary_flag_accounting() -> anyhow::Result<()> {
        let temporary = bijux_dna_infra::temp_dir("bam-endogenous-normalizer")?;
        let raw = temporary.path().join("counts.tsv");
        std::fs::write(
            &raw,
            b"metric\tvalue\ntotal_primary_reads\t10\nmapped_primary_reads\t8\nmapped_nonduplicate_primary_reads\t7\n",
        )?;
        let script = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("benchmarks/execution/bam_endogenous_content.awk");
        let output =
            Command::new("awk").args(["-v", "mode=summary", "-f"]).arg(script).arg(raw).output()?;
        anyhow::ensure!(output.status.success(), "endogenous-content AWK failed");
        assert_eq!(
            String::from_utf8(output.stdout)?,
            "metric\tvalue\ntotal_primary_reads\t10\nmapped_primary_reads\t8\nmapped_fraction\t0.800000000000\nmapped_nonduplicate_primary_reads\t7\n"
        );
        let command = measured_command("ERR2112767", 23, 3, 26);
        assert!(command[2].contains("samtools view -c -F 2304"));
        assert!(command[2].contains("samtools view -c -F 3332"));
        assert_eq!(required_files(3).len(), 18);
        Ok(())
    }
}
