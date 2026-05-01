use bijux_dna_core::ids::StageId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NonGeneralGenomicsBranchFamily {
    AmpliconAsv,
    AmpliconOtu,
    AmpliconChimeraControl,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NonGeneralGenomicsBranchContractV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub family: NonGeneralGenomicsBranchFamily,
    pub assay_scope: String,
    pub forbidden_from_generic_defaults: bool,
    pub required_predecessors: Vec<String>,
    pub assay_assumptions: Vec<String>,
    pub governed_example_id: String,
    pub caveats: Vec<String>,
}

const NON_GENERAL_GENOMICS_BRANCH_SCHEMA_VERSION: &str = "bijux.fastq.non_general_branch.v1";

#[must_use]
pub fn non_general_genomics_branch_contracts() -> Vec<NonGeneralGenomicsBranchContractV1> {
    vec![
        NonGeneralGenomicsBranchContractV1 {
            schema_version: NON_GENERAL_GENOMICS_BRANCH_SCHEMA_VERSION.to_string(),
            stage_id: "fastq.remove_chimeras".to_string(),
            family: NonGeneralGenomicsBranchFamily::AmpliconChimeraControl,
            assay_scope: "amplicon_or_metabarcoding".to_string(),
            forbidden_from_generic_defaults: true,
            required_predecessors: vec![
                "fastq.normalize_primers".to_string(),
                "fastq.filter_reads".to_string(),
            ],
            assay_assumptions: vec![
                "primer-normalized or merged amplicon reads are required".to_string(),
                "chimera calls are marker- and cohort-dependent".to_string(),
                "removed sequences must not be interpreted as generic contamination depletion"
                    .to_string(),
            ],
            governed_example_id: "fastq_edna_mini".to_string(),
            caveats: vec![
                "run only when marker chemistry and primer governance are declared".to_string(),
                "keep chimera-screen outputs out of generic shotgun defaults".to_string(),
            ],
        },
        NonGeneralGenomicsBranchContractV1 {
            schema_version: NON_GENERAL_GENOMICS_BRANCH_SCHEMA_VERSION.to_string(),
            stage_id: "fastq.infer_asvs".to_string(),
            family: NonGeneralGenomicsBranchFamily::AmpliconAsv,
            assay_scope: "single_marker_amplicon".to_string(),
            forbidden_from_generic_defaults: true,
            required_predecessors: vec![
                "fastq.normalize_primers".to_string(),
                "fastq.remove_chimeras".to_string(),
            ],
            assay_assumptions: vec![
                "error models are learned within a governed marker-specific cohort".to_string(),
                "denoised sequence variants are ecology-oriented features, not generic deduped reads"
                    .to_string(),
                "ASV identifiers are only comparable when primer and filtering contracts match"
                    .to_string(),
            ],
            governed_example_id: "fastq_edna_mini".to_string(),
            caveats: vec![
                "ASV inference remains an explicit branch and does not silently replace OTU defaults"
                    .to_string(),
            ],
        },
        NonGeneralGenomicsBranchContractV1 {
            schema_version: NON_GENERAL_GENOMICS_BRANCH_SCHEMA_VERSION.to_string(),
            stage_id: "fastq.cluster_otus".to_string(),
            family: NonGeneralGenomicsBranchFamily::AmpliconOtu,
            assay_scope: "metabarcoding_or_marker_amplicon".to_string(),
            forbidden_from_generic_defaults: true,
            required_predecessors: vec![
                "fastq.normalize_primers".to_string(),
                "fastq.remove_chimeras".to_string(),
            ],
            assay_assumptions: vec![
                "identity thresholds are marker-specific and must stay explicit".to_string(),
                "OTU centroids are ecology abstractions, not generic FASTQ consensus reads"
                    .to_string(),
                "downstream abundance normalization assumes governed taxonomy/marker scope"
                    .to_string(),
            ],
            governed_example_id: "fastq_edna_mini".to_string(),
            caveats: vec![
                "OTU clustering is the governed eDNA default branch, not a shotgun fallback"
                    .to_string(),
            ],
        },
    ]
}

#[must_use]
pub fn non_general_genomics_branch_contract_for_stage(
    stage_id: &StageId,
) -> Option<NonGeneralGenomicsBranchContractV1> {
    non_general_genomics_branch_contracts()
        .into_iter()
        .find(|contract| contract.stage_id == stage_id.as_str())
}

#[cfg(test)]
mod tests {
    use super::{
        non_general_genomics_branch_contract_for_stage, non_general_genomics_branch_contracts,
    };
    use bijux_dna_core::ids::StageId;

    #[test]
    fn non_general_branch_contracts_cover_specialized_ecology_stages() {
        let stages = non_general_genomics_branch_contracts()
            .into_iter()
            .map(|contract| contract.stage_id)
            .collect::<Vec<_>>();
        assert_eq!(
            stages,
            vec![
                "fastq.remove_chimeras".to_string(),
                "fastq.infer_asvs".to_string(),
                "fastq.cluster_otus".to_string(),
            ]
        );
    }

    #[test]
    fn non_general_branch_lookup_returns_stage_specific_contracts() {
        let contract = non_general_genomics_branch_contract_for_stage(&StageId::from_static(
            "fastq.cluster_otus",
        ))
        .expect("cluster_otus specialized contract");
        assert!(contract.forbidden_from_generic_defaults);
        assert_eq!(contract.governed_example_id, "fastq_edna_mini");
    }
}
