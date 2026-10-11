//! Owner: bijux-dna-bench-vcf
//! Tool-specific phasing command projection from declared inputs and destinations.
use super::{anyhow, MaterializedPanelInputs, Result, GOVERNED_COHORT_VCF_PATH};

pub(super) fn build(
    tool_id: &str,
    output_prefix: &str,
    materialized_inputs: &MaterializedPanelInputs,
    phased_vcf_path: &str,
    logs_path: &str,
) -> Result<(String, Vec<String>)> {
    Ok(match tool_id {
        "shapeit5" => (
            "domain/vcf/fixtures/vcf.phasing/shapeit5.txt".to_string(),
            vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "shapeit5 phase_common --input '{}' --reference '{}' --map '{}' --region 1:1-1000000 --thread 8 --seed 42 --output '{}' > '{}' 2>&1",
                    GOVERNED_COHORT_VCF_PATH,
                    materialized_inputs.panel_vcf_path,
                    materialized_inputs.genetic_map_path,
                    phased_vcf_path,
                    logs_path
                ),
            ],
        ),
        "shapeit" => (
            "domain/vcf/fixtures/vcf.phasing/shapeit.txt".to_string(),
            vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "shapeit --input '{}' --reference '{}' --map '{}' --region 1:1-1000000 --thread 8 --seed 42 --output '{}' > '{}' 2>&1",
                    GOVERNED_COHORT_VCF_PATH,
                    materialized_inputs.panel_vcf_path,
                    materialized_inputs.genetic_map_path,
                    phased_vcf_path,
                    logs_path
                ),
            ],
        ),
        "eagle" => (
            "domain/vcf/fixtures/vcf.phasing/eagle.txt".to_string(),
            vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "eagle --vcfTarget '{}' --vcfRef '{}' --geneticMapFile '{}' --outPrefix '{}' --numThreads 8 > '{}' 2>&1",
                    GOVERNED_COHORT_VCF_PATH,
                    materialized_inputs.panel_vcf_path,
                    materialized_inputs.genetic_map_path,
                    output_prefix,
                    logs_path
                ),
            ],
        ),
        "beagle" => (
            "domain/vcf/fixtures/vcf.phasing/beagle.txt".to_string(),
            vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "beagle gt='{}' ref='{}' map='{}' out='{}' nthreads=8 seed=42 > '{}' 2>&1",
                    GOVERNED_COHORT_VCF_PATH,
                    materialized_inputs.panel_vcf_path,
                    materialized_inputs.genetic_map_path,
                    output_prefix,
                    logs_path
                ),
            ],
        ),
        other => return Err(anyhow!("VCF phasing-family adapter does not govern tool `{other}`")),
    })
}
