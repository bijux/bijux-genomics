use serde::Serialize;

use crate::taxonomy::VcfDomainStage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PortCardinality {
    One,
    Many,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StagePortContract {
    pub name: &'static str,
    pub data_type: &'static str,
    pub cardinality: PortCardinality,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StageIoContract {
    pub stage: VcfDomainStage,
    pub inputs: Vec<StagePortContract>,
    pub outputs: Vec<StagePortContract>,
    pub required_inputs: Vec<&'static str>,
    pub required_outputs: Vec<&'static str>,
    pub required_indices: Vec<&'static str>,
}

#[must_use]
pub fn stage_io_contract(stage: VcfDomainStage) -> Option<StageIoContract> {
    let one = PortCardinality::One;
    let many = PortCardinality::Many;
    Some(match stage {
        VcfDomainStage::Call
        | VcfDomainStage::CallDiploid
        | VcfDomainStage::CallGl
        | VcfDomainStage::CallPseudohaploid => contract(
            stage,
            vec![port("bam", "bam", many)],
            vec![port("vcf", "vcf", one)],
            vec!["bam"],
            vec!["vcf"],
            vec!["bam.bai"],
        ),
        VcfDomainStage::DamageFilter
        | VcfDomainStage::Filter
        | VcfDomainStage::GlPropagation
        | VcfDomainStage::Phasing
        | VcfDomainStage::Impute
        | VcfDomainStage::Postprocess => indexed_vcf_transform_contract(stage, "vcf", "vcf_out"),
        VcfDomainStage::Qc => indexed_vcf_report_contract(stage, "vcf", "qc_report"),
        VcfDomainStage::Stats => indexed_vcf_report_contract(stage, "vcf", "stats_json"),
        VcfDomainStage::ImputationMetrics => {
            indexed_vcf_report_contract(stage, "vcf", "imputation_metrics_json")
        }
        VcfDomainStage::Pca => indexed_vcf_report_contract(stage, "vcf", "pca_report"),
        VcfDomainStage::Admixture => indexed_vcf_report_contract(stage, "vcf", "admixture_report"),
        VcfDomainStage::PopulationStructure => {
            indexed_vcf_report_contract(stage, "filtered_vcf", "population_structure_report")
        }
        VcfDomainStage::Roh => indexed_vcf_report_contract(stage, "filtered_vcf", "roh_report"),
        VcfDomainStage::Ibd => contract(
            stage,
            vec![port("filtered_vcf", "vcf", one)],
            vec![port("ibd_segments", "tsv", one)],
            vec!["filtered_vcf"],
            vec!["ibd_segments"],
            vec!["vcf.tbi"],
        ),
        VcfDomainStage::Demography => contract(
            stage,
            vec![port("ibd_segments", "tsv", one)],
            vec![port("demography_report", "json", one)],
            vec!["ibd_segments"],
            vec!["demography_report"],
            vec![],
        ),
        VcfDomainStage::PrepareReferencePanel => {
            indexed_vcf_transform_contract(stage, "panel_vcf", "prepared_panel")
        }
    })
}

fn port(
    name: &'static str,
    data_type: &'static str,
    cardinality: PortCardinality,
) -> StagePortContract {
    StagePortContract { name, data_type, cardinality }
}

fn contract(
    stage: VcfDomainStage,
    inputs: Vec<StagePortContract>,
    outputs: Vec<StagePortContract>,
    required_inputs: Vec<&'static str>,
    required_outputs: Vec<&'static str>,
    required_indices: Vec<&'static str>,
) -> StageIoContract {
    StageIoContract { stage, inputs, outputs, required_inputs, required_outputs, required_indices }
}

fn indexed_vcf_transform_contract(
    stage: VcfDomainStage,
    input_name: &'static str,
    output_name: &'static str,
) -> StageIoContract {
    contract(
        stage,
        vec![port(input_name, "vcf", PortCardinality::One)],
        vec![port(output_name, "vcf", PortCardinality::One)],
        vec![input_name],
        vec![output_name],
        vec!["vcf.tbi"],
    )
}

fn indexed_vcf_report_contract(
    stage: VcfDomainStage,
    input_name: &'static str,
    output_name: &'static str,
) -> StageIoContract {
    contract(
        stage,
        vec![port(input_name, "vcf", PortCardinality::One)],
        vec![port(output_name, "json", PortCardinality::One)],
        vec![input_name],
        vec![output_name],
        vec!["vcf.tbi"],
    )
}
