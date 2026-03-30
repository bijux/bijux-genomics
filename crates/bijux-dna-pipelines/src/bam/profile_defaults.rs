//! BAM pipeline profile support helpers.

use std::collections::BTreeMap;

use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_bam::params::BamEffectiveParams;
use bijux_dna_domain_bam::BamStage;

use crate::{DefaultParams, EffectiveDefaults};

#[derive(Debug, Clone)]
pub(super) struct BamStageDefault {
    pub(super) stage: BamStage,
    pub(super) params: BamEffectiveParams,
}

pub(super) fn defaults_for(
    stages: &[BamStage],
    params_for_stage: fn(BamStage) -> serde_json::Value,
) -> Vec<BamStageDefault> {
    stages
        .iter()
        .map(|stage| BamStageDefault {
            stage: *stage,
            params: stage
                .parse_effective_params(&params_for_stage(*stage))
                .unwrap_or_else(|err| {
                    panic!(
                        "failed to parse typed BAM defaults for stage {}: {err}",
                        stage.as_str()
                    )
                }),
        })
        .collect()
}

pub(super) fn to_effective_defaults(defaults: &[BamStageDefault]) -> EffectiveDefaults {
    let mut tools = BTreeMap::new();
    let mut params = BTreeMap::new();
    let mut rationales = BTreeMap::new();
    for entry in defaults {
        let default_tool = match entry.stage {
            BamStage::Align => id_catalog::TOOL_BWA,
            BamStage::Validate => id_catalog::TOOL_SAMTOOLS,
            BamStage::QcPre => id_catalog::TOOL_SAMTOOLS,
            BamStage::MappingSummary => id_catalog::TOOL_SAMTOOLS,
            BamStage::Filter => id_catalog::TOOL_SAMTOOLS,
            BamStage::MapqFilter => id_catalog::TOOL_SAMTOOLS,
            BamStage::LengthFilter => id_catalog::TOOL_SAMTOOLS,
            BamStage::Markdup => id_catalog::TOOL_GATK,
            BamStage::DuplicationMetrics => id_catalog::TOOL_SAMTOOLS,
            BamStage::Complexity => id_catalog::TOOL_PRESEQ,
            BamStage::Coverage => id_catalog::TOOL_MOSDEPTH,
            BamStage::InsertSize => id_catalog::TOOL_GATK,
            BamStage::GcBias => id_catalog::TOOL_GATK,
            BamStage::EndogenousContent => id_catalog::TOOL_SAMTOOLS,
            BamStage::OverlapCorrection => id_catalog::TOOL_SAMTOOLS,
            BamStage::Damage => id_catalog::TOOL_PYDAMAGE,
            BamStage::Authenticity => id_catalog::TOOL_AUTHENTICCT,
            BamStage::Contamination => id_catalog::TOOL_AUTHENTICCT,
            BamStage::Sex => id_catalog::TOOL_RXY,
            BamStage::BiasMitigation => id_catalog::TOOL_ANGSD,
            BamStage::Recalibration => id_catalog::TOOL_GATK,
            BamStage::Haplogroups => id_catalog::TOOL_YLEAF,
            BamStage::Genotyping => id_catalog::TOOL_ANGSD,
            BamStage::Kinship => id_catalog::TOOL_KING,
        };
        tools.insert(
            StageId::from_static(entry.stage.as_str()),
            ToolId::from_static(default_tool),
        );
        params.insert(
            StageId::from_static(entry.stage.as_str()),
            DefaultParams::Bam(entry.params.clone()),
        );
        rationales.insert(
            StageId::from_static(entry.stage.as_str()),
            "pipeline default".to_string(),
        );
    }
    EffectiveDefaults {
        tools,
        params,
        rationales,
    }
}

pub(super) fn filter_downstream(stages: &mut Vec<BamStage>) {
    if cfg!(feature = "bam_downstream") {
        return;
    }
    stages.retain(|stage| {
        !matches!(
            stage,
            BamStage::Haplogroups | BamStage::Genotyping | BamStage::Kinship
        )
    });
}

pub(super) fn stable_bam_stages() -> Vec<BamStage> {
    vec![
        BamStage::Validate,
        BamStage::QcPre,
        BamStage::MappingSummary,
        BamStage::Filter,
        BamStage::Coverage,
        BamStage::Damage,
    ]
}

fn bam_stage_order(stage: &BamStage) -> usize {
    match stage {
        BamStage::Align => 0,
        BamStage::Validate => 1,
        BamStage::QcPre => 2,
        BamStage::MappingSummary => 3,
        BamStage::Filter => 4,
        BamStage::MapqFilter => 5,
        BamStage::LengthFilter => 6,
        BamStage::Markdup => 7,
        BamStage::DuplicationMetrics => 8,
        BamStage::Complexity => 9,
        BamStage::Coverage => 10,
        BamStage::InsertSize => 11,
        BamStage::GcBias => 12,
        BamStage::EndogenousContent => 13,
        BamStage::OverlapCorrection => 14,
        BamStage::Damage => 15,
        BamStage::Authenticity => 16,
        BamStage::Contamination => 17,
        BamStage::Sex => 18,
        BamStage::BiasMitigation => 19,
        BamStage::Recalibration => 20,
        BamStage::Haplogroups => 21,
        BamStage::Genotyping => 22,
        BamStage::Kinship => 23,
    }
}

pub(super) fn catalog_bam_stages() -> Vec<BamStage> {
    let parsed: toml::Value = include_str!("../../../../configs/ci/stages/stages.toml")
        .parse()
        .expect("generated configs/ci/stages/stages.toml must parse");
    let mut stages = parsed
        .get("stages")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| {
            entry
                .get("id")
                .and_then(toml::Value::as_str)
                .and_then(|id| {
                    if id.starts_with("bam.") {
                        BamStage::try_from(id).ok()
                    } else {
                        None
                    }
                })
        })
        .collect::<Vec<_>>();
    stages.sort_by_key(bam_stage_order);
    stages.dedup();
    stages
}
