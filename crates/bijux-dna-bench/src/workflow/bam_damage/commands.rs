//! Owner: bijux-dna-bench
//! Container commands and output closure for BAM damage attempts.

const BAM: &str = "/input/ERR2112767.sorted.bam";
const REFERENCE: &str = "/input/EquCab3.0.fna";

pub(super) fn warmup_command(tool: &str) -> Vec<String> {
    shell(tool_command(tool, "/scratch/bam-damage-warmup"))
}

pub(super) fn negative_command(tool: &str) -> Vec<String> {
    shell(tool_command_for_inputs(
        tool,
        "/input/missing.bam",
        REFERENCE,
        "/scratch/bam-damage-negative",
    ))
}

pub(super) fn measured_command(
    tool: &str,
    sample_id: &str,
    bam_bytes: u64,
    bam_index_bytes: u64,
    reference_bytes: u64,
    input_bytes: u64,
) -> Vec<String> {
    let native = "/result/repetitions/repetition-{repetition}.native";
    let command = tool_command(tool, native);
    shell(format!(
        "set -eu
mkdir -p /result/repetitions
prefix=\"/result/repetitions/repetition-{{repetition}}\"
native=\"${{prefix}}.native\"
rm -rf \"$native\"
{command}
native_file_count=$(find \"$native\" -type f -size +0c | wc -l | awk '{{print $1}}')
native_output_bytes=$(find \"$native\" -type f -size +0c -exec wc -c {{}} \\; | awk '{{sum += $1}} END {{print sum + 0}}')
test \"$native_file_count\" -gt 0
test \"$native_output_bytes\" -gt 0
raw=\"${{prefix}}.damage-native-summary.tsv\"
summary=\"${{prefix}}.damage-summary.tsv\"
printf 'metric\\tvalue\\nnormalization_complete\\t1\\nnative_file_count\\t%s\\nnative_output_bytes\\t%s\\n' \"$native_file_count\" \"$native_output_bytes\" > \"$raw\"
awk -v mode=summary -f /repository/benchmarks/execution/bam_damage.awk \"$raw\" > \"$summary\"
awk -v mode=bundle -v tool={} -v sample_id={} -f /repository/benchmarks/execution/bam_damage.awk \"$summary\" > \"${{prefix}}.damage-summary.json\"
awk -v mode=probe -f /repository/benchmarks/execution/bam_damage.awk \"$summary\" > \"${{prefix}}.report-qc.json\"
output_bytes=$({{ find \"$native\" -type f -size +0c -exec wc -c {{}} \\;; wc -c \"$raw\" \"$summary\" \"${{prefix}}.damage-summary.json\" \"${{prefix}}.report-qc.json\"; }} | awk '$2 != \"total\" {{sum += $1}} END {{print sum + 0}}')
awk -v mode=metrics -v bam_input_bytes={bam_bytes} -v bam_index_input_bytes={bam_index_bytes} -v reference_input_bytes={reference_bytes} -v input_bytes={input_bytes} -v output_bytes=\"$output_bytes\" -f /repository/benchmarks/execution/bam_damage.awk \"$summary\" > \"${{prefix}}.metrics.json\"",
        shell_quote(tool),
        shell_quote(sample_id),
    ))
}

pub(super) fn required_files(repetitions: u32) -> Vec<String> {
    let mut files = Vec::new();
    for repetition in 1..=repetitions {
        let prefix = format!("repetitions/repetition-{repetition}");
        files.extend([
            format!("{prefix}.native"),
            format!("{prefix}.damage-native-summary.tsv"),
            format!("{prefix}.damage-summary.tsv"),
            format!("{prefix}.damage-summary.json"),
            format!("{prefix}.report-qc.json"),
            format!("{prefix}.metrics.json"),
            format!("{prefix}.resource.tsv"),
        ]);
    }
    files
}

fn tool_command(tool: &str, output: &str) -> String {
    tool_command_for_inputs(tool, BAM, REFERENCE, output)
}

fn tool_command_for_inputs(tool: &str, bam: &str, reference: &str, output: &str) -> String {
    match tool {
        "addeam" => format!(
            "set -eu; mkdir -p '{output}'; bam_list=/scratch/addeam-bam-list.txt; printf '%s\\n' '{bam}' > \"$bam_list\"; addeam profile \"$bam_list\" -classic -both -minq 20 -minl 30 -length 10 -precision 0.001 -minAligned 10000 -minConverge 0.01 -stepsConverge 500 -threads 1 -o '{output}'"
        ),
        "damageprofiler" => format!(
            "set -eu; mkdir -p '{output}'; damageprofiler -i '{bam}' -o '{output}' -title ERR2112767"
        ),
        "mapdamage2" => format!(
            "set -eu; mkdir -p '{output}'; mapdamage2 analyze --bam '{bam}' --reference '{reference}' --out-dir '{output}'"
        ),
        "ngsbriggs" => format!(
            "set -eu; mkdir -p '{output}'; ngsbriggs analyze --bam '{bam}' --reference '{reference}' --json-out '{output}/damage.ngsbriggs.json' --model nb"
        ),
        "pmdtools" => format!(
            "set -eu; mkdir -p '{output}'; samtools view -h '{bam}' | pmdtools --printDS --stats > '{output}/pmd-scores.tsv' 2> '{output}/pmd-statistics.txt'"
        ),
        "pydamage" => format!(
            "set -eu; mkdir -p '{output}'; pydamage --outdir '{output}' analyze --group --process 2 '{bam}'"
        ),
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
    fn commands_preserve_native_output_for_every_damage_tool() -> anyhow::Result<()> {
        for tool in ["addeam", "damageprofiler", "mapdamage2", "ngsbriggs", "pmdtools", "pydamage"]
        {
            assert_ne!(warmup_command(tool)[2], "false");
            assert!(negative_command(tool)[2].contains("/input/missing.bam"));
            let measured = measured_command(tool, "ERR2112767", 10, 2, 20, 32);
            assert!(measured[2].contains("bam_damage.awk"));
            assert!(measured[2].contains("native_file_count"));
            assert!(std::process::Command::new("sh")
                .args(["-n", "-c", &measured[2]])
                .status()?
                .success());
        }
        assert_eq!(required_files(3).len(), 21);
        Ok(())
    }
}
