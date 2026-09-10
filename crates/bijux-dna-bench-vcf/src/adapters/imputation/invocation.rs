//! Owner: bijux-dna-bench-vcf
//! Tool-specific imputation command projection from declared inputs and destinations.
use super::{anyhow, MaterializedPanelInputs, Result, VcfDomainStage, GOVERNED_REGION_LITERAL};

pub(super) fn build(
    tool_id: &str,
    stage: VcfDomainStage,
    target_vcf_path: &str,
    output_prefix: &str,
    materialized_inputs: &MaterializedPanelInputs,
    log_output_path: &str,
    imputed_vcf_path: &str,
) -> Result<(String, Vec<String>)> {
    Ok(match tool_id {
        "beagle" => (
            format!("domain/vcf/fixtures/{}//beagle.txt", stage.as_str()).replace("//", "/"),
            vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "beagle gt='{}' ref='{}' map='{}' out='{}' impute=true nthreads=8 seed=42 > '{}' 2>&1",
                    target_vcf_path,
                    materialized_inputs.panel_vcf_path,
                    materialized_inputs.genetic_map_path,
                    output_prefix,
                    log_output_path
                ),
            ],
        ),
        "glimpse" => (
            format!("domain/vcf/fixtures/{}//glimpse.txt", stage.as_str()).replace("//", "/"),
            vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "GLIMPSE_phase --input '{}' --reference '{}' --map '{}' --input-region '{}' --output-region '{}' --threads 8 --seed 42 --output '{}' > '{}' 2>&1",
                    target_vcf_path,
                    materialized_inputs.panel_vcf_path,
                    materialized_inputs.genetic_map_path,
                    GOVERNED_REGION_LITERAL,
                    GOVERNED_REGION_LITERAL,
                    imputed_vcf_path,
                    log_output_path
                ),
            ],
        ),
        "impute5" => (
            format!("domain/vcf/fixtures/{}//impute5.txt", stage.as_str()).replace("//", "/"),
            vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "impute5 --g '{}' --h '{}' --m '{}' --r '{}' --o '{}' --threads 8 --seed 42 > '{}' 2>&1",
                    target_vcf_path,
                    materialized_inputs.panel_vcf_path,
                    materialized_inputs.genetic_map_path,
                    GOVERNED_REGION_LITERAL,
                    imputed_vcf_path,
                    log_output_path
                ),
            ],
        ),
        "minimac4" => (
            format!("domain/vcf/fixtures/{}//minimac4.txt", stage.as_str()).replace("//", "/"),
            vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "minimac4 --threads 8 --map '{}' --output '{}' '{}' '{}' > '{}' 2>&1",
                    materialized_inputs.genetic_map_path,
                    imputed_vcf_path,
                    materialized_inputs
                        .panel_msav_path
                        .as_deref()
                        .ok_or_else(|| anyhow!("missing panel.msav for minimac4"))?,
                    target_vcf_path,
                    log_output_path
                ),
            ],
        ),
        other => {
            return Err(anyhow!(
                "VCF imputation-family adapter does not govern tool `{other}`"
            ));
        }
    })
}

/// Region arguments apply only to engines whose command contracts declare them.
pub(super) fn region_literal(tool_id: &str) -> Option<String> {
    matches!(tool_id, "glimpse" | "impute5").then(|| GOVERNED_REGION_LITERAL.to_string())
}
