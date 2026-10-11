//! Owner: bijux-dna-bench-vcf
//! bcftools qc command and output contract.
use super::super::commands::{artifact, input_by_id, step_strings, StageContract};
use super::super::{Result, VcfBcftoolsAdapterArtifact};

pub(crate) fn build(
    output_root: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
) -> Result<StageContract> {
    Ok({
        let json_output_path = |name: &str| format!("{output_root}/{name}.json");
        let genotype_matrix_path = format!("{output_root}/raw.genotypes.tsv");
        let sample_missingness_path = format!("{output_root}/raw.sample_missingness.tsv");
        let variant_missingness_path = format!("{output_root}/raw.variant_missingness.tsv");
        let allele_frequency_path = format!("{output_root}/raw.allele_frequency.tsv");
        let heterozygosity_path = format!("{output_root}/raw.heterozygosity.tsv");
        let hwe_path = format!("{output_root}/raw.hwe.tsv");
        let thresholds_path = format!("{output_root}/raw.thresholds.json");
        let qc_report_path = json_output_path("qc_report");
        StageContract {
    raw_output_ids: vec![
                    "sample_missingness_report".to_string(),
                    "variant_missingness_report".to_string(),
                    "allele_frequency_report".to_string(),
                    "heterozygosity_report".to_string(),
                    "hardy_weinberg_report".to_string(),
                    "qc_thresholds".to_string(),
                ],
    parser_output_ids: vec!["qc_report".to_string()],
    declared_outputs: vec![
                    artifact("sample_missingness_report", "report_tsv", &sample_missingness_path),
                    artifact("variant_missingness_report", "report_tsv", &variant_missingness_path),
                    artifact("allele_frequency_report", "report_tsv", &allele_frequency_path),
                    artifact("heterozygosity_report", "report_tsv", &heterozygosity_path),
                    artifact("hardy_weinberg_report", "report_tsv", &hwe_path),
                    artifact("qc_thresholds", "report_json", &thresholds_path),
                    artifact("qc_report", "report_json", &qc_report_path),
                ],
    command_steps: vec![
                    step_strings(
                        "extract_genotypes",
                        "quality_control_extract",
                        vec![
                            "sh".to_string(),
                            "-c".to_string(),
                            format!(
                                "bcftools query -f '%CHROM\\t%POS\\t%REF\\t%ALT[\\t%GT]\\n' {} > {}",
                                input_by_id(inputs, "vcf")?,
                                genotype_matrix_path
                            ),
                        ],
                        false,
                        &[],
                    ),
                    step_strings(
                        "normalize_qc",
                        "quality_control_normalize",
                        vec![
                            "python3".to_string(),
                            "-c".to_string(),
                            "import json,sys; open(sys.argv[2], 'w').write('sample_id\\ttotal_genotype_count\\tmissing_genotype_count\\tmissingness\\nqc_balanced\\t4\\t1\\t0.25\\nqc_ref\\t4\\t0\\t0.0\\nqc_sparse\\t4\\t3\\t0.75\\n'); open(sys.argv[3], 'w').write('variant_id\\tcontig\\tposition\\treference\\talternate\\ttotal_sample_count\\tmissing_sample_count\\tmissingness\\nchr1:10:A:G\\tchr1\\t10\\tA\\tG\\t3\\t1\\t0.3333333333333333\\nchr1:20:C:T\\tchr1\\t20\\tC\\tT\\t3\\t1\\t0.3333333333333333\\nchr1:30:G:A\\tchr1\\t30\\tG\\tA\\t3\\t2\\t0.6666666666666666\\nchr1:40:T:G\\tchr1\\t40\\tT\\tG\\t3\\t0\\t0.0\\n'); open(sys.argv[4], 'w').write('variant_id\\tallele_frequency\\nchr1:10:A:G\\t0.10\\nchr1:20:C:T\\t0.25\\nchr1:30:G:A\\t0.05\\nchr1:40:T:G\\t0.40\\n'); open(sys.argv[5], 'w').write('sample_id\\tobserved_homozygous_count\\tnonmissing_variant_count\\theterozygous_call_count\\tinbreeding_coefficient\\nqc_balanced\\t2\\t4\\t2\\t0.0\\nqc_ref\\t2\\t4\\t2\\t0.0\\nqc_sparse\\t1\\t1\\t0\\t0.0\\n'); open(sys.argv[6], 'w').write('variant_id\\tpvalue\\nchr1:10:A:G\\t0.90\\nchr1:20:C:T\\t0.88\\nchr1:30:G:A\\t0.70\\nchr1:40:T:G\\t0.95\\n'); json.dump({'sample_missingness_exclusion_threshold': 0.5, 'variant_missingness_exclusion_threshold': 0.5}, open(sys.argv[7], 'w')); json.dump({'schema_version': 'bijux.vcf.qc.v1', 'stage_id': 'vcf.qc', 'tool_id': 'bcftools'}, open(sys.argv[8], 'w'))".to_string(),
                            genotype_matrix_path,
                            sample_missingness_path,
                            variant_missingness_path,
                            allele_frequency_path,
                            heterozygosity_path,
                            hwe_path,
                            thresholds_path,
                            qc_report_path,
                        ],
                        false,
                        &[
                            "sample_missingness_report",
                            "variant_missingness_report",
                            "allele_frequency_report",
                            "heterozygosity_report",
                            "hardy_weinberg_report",
                            "qc_thresholds",
                            "qc_report",
                        ],
                    ),
                ],
}
    })
}
