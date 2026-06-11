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
    fn port(
        name: &'static str,
        data_type: &'static str,
        cardinality: PortCardinality,
    ) -> StagePortContract {
        StagePortContract { name, data_type, cardinality }
    }

    let one = PortCardinality::One;
    let many = PortCardinality::Many;
    let contract = match stage {
        VcfDomainStage::Call
        | VcfDomainStage::CallDiploid
        | VcfDomainStage::CallGl
        | VcfDomainStage::CallPseudohaploid => StageIoContract {
            stage,
            inputs: vec![port("bam", "bam", many)],
            outputs: vec![port("vcf", "vcf", one)],
            required_inputs: vec!["bam"],
            required_outputs: vec!["vcf"],
            required_indices: vec!["bam.bai"],
        },
        VcfDomainStage::DamageFilter
        | VcfDomainStage::Filter
        | VcfDomainStage::GlPropagation
        | VcfDomainStage::Phasing
        | VcfDomainStage::Impute
        | VcfDomainStage::Postprocess => StageIoContract {
            stage,
            inputs: vec![port("vcf", "vcf", one)],
            outputs: vec![port("vcf_out", "vcf", one)],
            required_inputs: vec!["vcf"],
            required_outputs: vec!["vcf_out"],
            required_indices: vec!["vcf.tbi"],
        },
        VcfDomainStage::Qc => StageIoContract {
            stage,
            inputs: vec![port("vcf", "vcf", one)],
            outputs: vec![port("qc_report", "json", one)],
            required_inputs: vec!["vcf"],
            required_outputs: vec!["qc_report"],
            required_indices: vec!["vcf.tbi"],
        },
        VcfDomainStage::Stats => StageIoContract {
            stage,
            inputs: vec![port("vcf", "vcf", one)],
            outputs: vec![port("stats_json", "json", one)],
            required_inputs: vec!["vcf"],
            required_outputs: vec!["stats_json"],
            required_indices: vec!["vcf.tbi"],
        },
        VcfDomainStage::ImputationMetrics => StageIoContract {
            stage,
            inputs: vec![port("vcf", "vcf", one)],
            outputs: vec![port("imputation_metrics_json", "json", one)],
            required_inputs: vec!["vcf"],
            required_outputs: vec!["imputation_metrics_json"],
            required_indices: vec!["vcf.tbi"],
        },
        VcfDomainStage::Pca => StageIoContract {
            stage,
            inputs: vec![port("vcf", "vcf", one)],
            outputs: vec![port("pca_report", "json", one)],
            required_inputs: vec!["vcf"],
            required_outputs: vec!["pca_report"],
            required_indices: vec!["vcf.tbi"],
        },
        VcfDomainStage::Admixture => StageIoContract {
            stage,
            inputs: vec![port("vcf", "vcf", one)],
            outputs: vec![port("admixture_report", "json", one)],
            required_inputs: vec!["vcf"],
            required_outputs: vec!["admixture_report"],
            required_indices: vec!["vcf.tbi"],
        },
        VcfDomainStage::PopulationStructure | VcfDomainStage::Roh | VcfDomainStage::Ibd => {
            StageIoContract {
            stage,
            inputs: vec![port("filtered_vcf", "vcf", one)],
            outputs: vec![port("report_json", "json", one)],
            required_inputs: vec!["filtered_vcf"],
            required_outputs: vec!["report_json"],
            required_indices: vec!["vcf.tbi"],
        }
        }
        VcfDomainStage::Demography => StageIoContract {
            stage,
            inputs: vec![port("ibd_segments", "json", one)],
            outputs: vec![port("demography_report", "json", one)],
            required_inputs: vec!["ibd_segments"],
            required_outputs: vec!["demography_report"],
            required_indices: vec![],
        },
        VcfDomainStage::PrepareReferencePanel => StageIoContract {
            stage,
            inputs: vec![port("panel_vcf", "vcf", one)],
            outputs: vec![port("prepared_panel", "vcf", one)],
            required_inputs: vec!["panel_vcf"],
            required_outputs: vec!["prepared_panel"],
            required_indices: vec!["vcf.tbi"],
        },
    };
    Some(contract)
}
