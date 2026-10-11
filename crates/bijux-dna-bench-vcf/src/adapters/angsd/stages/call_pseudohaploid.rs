//! Owner: bijux-dna-bench-vcf
//! angsd call pseudohaploid command and output contract.
use super::super::commands::{artifact, input_by_id, step, StageContract};
use super::super::{Result, VcfAngsdAdapterArtifact};

pub(crate) fn build(
    output_root: &str,
    inputs: &[VcfAngsdAdapterArtifact],
) -> Result<StageContract> {
    Ok({
        let vcf_output_path = |name: &str| format!("{output_root}/{name}.vcf.gz");
        let json_output_path = |name: &str| format!("{output_root}/{name}.json");
        let arg_output_path = |prefix: &str| format!("{prefix}.arg");
        let prefix = format!("{output_root}/pseudo.native");
        let native_vcf = format!("{prefix}.vcf.gz");
        let output = vcf_output_path("pseudo");
        StageContract {
            command_contract_source: "domain/vcf/fixtures/vcf.call_pseudohaploid/angsd.txt"
                .to_string(),
            reference_path: Some(input_by_id(inputs, "reference_fasta")?.to_string()),
            sites_path: Some(input_by_id(inputs, "sites_vcf")?.to_string()),
            likelihood_model: Some("doHaploCall1_seed42".to_string()),
            output_prefix: prefix.clone(),
            stage_output_ids: vec!["pseudohaploid_vcf".to_string()],
            raw_output_ids: vec![
                "angsd_native_vcf".to_string(),
                "pseudohaploid_vcf".to_string(),
                "pseudohaploid_vcf_tbi".to_string(),
                "angsd_arg".to_string(),
            ],
            parser_output_ids: vec!["pseudohaploid_sampling_log".to_string()],
            declared_outputs: vec![
                artifact("angsd_native_vcf", "raw_variant", &native_vcf),
                artifact("pseudohaploid_vcf", "variant", &output),
                artifact("pseudohaploid_vcf_tbi", "index", &format!("{output}.tbi")),
                artifact("angsd_arg", "report", &arg_output_path(&prefix)),
                artifact(
                    "pseudohaploid_sampling_log",
                    "report_json",
                    &json_output_path("pseudohaploid_sampling_log"),
                ),
            ],
            command_steps: vec![
                step(
                    "call_pseudohaploid",
                    "call",
                    vec![
                        "angsd",
                        "-b",
                        input_by_id(inputs, "bam_list")?,
                        "-ref",
                        input_by_id(inputs, "reference_fasta")?,
                        "-sites",
                        input_by_id(inputs, "sites_vcf")?,
                        "-doHaploCall",
                        "1",
                        "-doCounts",
                        "1",
                        "-seed",
                        "42",
                        "-doVcf",
                        "1",
                        "-out",
                        &prefix,
                    ],
                    &["angsd_native_vcf", "angsd_arg"],
                ),
                step(
                    "normalize_pseudohaploid_vcf",
                    "result_normalizer",
                    vec!["bcftools", "view", "-Oz", "-o", &output, &native_vcf],
                    &["pseudohaploid_vcf"],
                ),
                step(
                    "index_pseudohaploid_vcf",
                    "index",
                    vec!["bcftools", "index", "-t", &output],
                    &["pseudohaploid_vcf_tbi"],
                ),
            ],
        }
    })
}
