//! Owner: bijux-dna-bench
//! Container commands and output closure for BAM genotyping attempts.

const BAM: &str = "/input/ERR2112767.sorted.bam";
const REFERENCE: &str = "/input/EquCab3.0.fna";
const REGIONS: &str = "/input/known-sites.bed";
const ALLELES: &str = "/input/known-sites.alleles.tsv";
const NORMALIZER: &str = "/repository/benchmarks/execution/bam_genotyping.awk";

pub(super) fn warmup_command() -> Vec<String> {
    shell(format!(
        "bcftools mpileup -f {REFERENCE} -R {REGIONS} -q 20 -Q 20 -a FORMAT/DP,FORMAT/AD -Ou {BAM} | bcftools call --threads 2 -A -i -m -C alleles -T {ALLELES} -Oz -o /scratch/warmup.vcf.gz"
    ))
}

pub(super) fn negative_command() -> Vec<String> {
    strings(&["bcftools", "mpileup", "-f", REFERENCE, "/input/missing.bam"])
}

pub(super) fn measured_command(
    sample_id: &str,
    bam_input_bytes: u64,
    reference_input_bytes: u64,
    database_input_bytes: u64,
    input_bytes: u64,
) -> Vec<String> {
    shell(format!(
        "set -eu
mkdir -p /result/repetitions
prefix=\"/result/repetitions/repetition-{{repetition}}\"
vcf=\"${{prefix}}.vcf.gz\"
genotypes=\"${{prefix}}.genotypes.tsv\"
summary=\"${{prefix}}.genotyping-summary.tsv\"
bcftools mpileup -f {REFERENCE} -R {REGIONS} -q 20 -Q 20 -a FORMAT/DP,FORMAT/AD -Ou {BAM} | bcftools call --threads 2 -A -i -m -C alleles -T {ALLELES} -Oz -o \"$vcf\"
bcftools index -f \"$vcf\"
bcftools query -f '[%CHROM\\t%POS\\t%REF\\t%ALT\\t%GT\\t%DP\\n]' \"$vcf\" > \"$genotypes\"
awk -v mode=summary -f {NORMALIZER} \"$genotypes\" > \"$summary\"
awk -v mode=bundle -v tool=bcftools -v sample_id={} -f {NORMALIZER} \"$summary\" > \"${{prefix}}.genotyping-summary.json\"
awk -v mode=probe -f {NORMALIZER} \"$summary\" > \"${{prefix}}.report-qc.json\"
sha256sum \"$genotypes\" > \"${{prefix}}.genotypes.sha256\"
output_bytes=$(wc -c \"$vcf\" \"${{vcf}}.csi\" \"$genotypes\" \"$summary\" \"${{prefix}}.genotyping-summary.json\" \"${{prefix}}.report-qc.json\" \"${{prefix}}.genotypes.sha256\" | awk 'END {{print $1}}')
awk -v mode=metrics -v bam_input_bytes={bam_input_bytes} -v reference_input_bytes={reference_input_bytes} -v database_input_bytes={database_input_bytes} -v input_bytes={input_bytes} -v output_bytes=\"$output_bytes\" -f {NORMALIZER} \"$summary\" > \"${{prefix}}.metrics.json\"",
        quote(sample_id),
    ))
}

pub(super) fn required_files(repetitions: u32) -> Vec<String> {
    let mut files = Vec::new();
    for repetition in 1..=repetitions {
        let prefix = format!("repetitions/repetition-{repetition}");
        files.extend([
            format!("{prefix}.vcf.gz"),
            format!("{prefix}.vcf.gz.csi"),
            format!("{prefix}.genotypes.tsv"),
            format!("{prefix}.genotypes.sha256"),
            format!("{prefix}.genotyping-summary.tsv"),
            format!("{prefix}.genotyping-summary.json"),
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
    fn genotyping_command_closes_calls_indexes_metrics_and_probe() {
        let command = measured_command("ERR2112767", 26, 101, 202, 329);
        assert!(command[2].contains("bcftools mpileup -f"));
        assert!(command[2].contains("bcftools call --threads 2"));
        assert!(command[2].contains("bam_genotyping.awk"));
        assert_eq!(required_files(3).len(), 27);
    }
}
