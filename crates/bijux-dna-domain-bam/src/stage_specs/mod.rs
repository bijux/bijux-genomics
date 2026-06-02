//! Single source of truth for BAM stage metadata and contracts.
//! Declarative only: no planner logic or runtime decisions live here.

use anyhow::{anyhow, Result};
use bijux_dna_core::contract::canonical::canonicalize_json_value;
use bijux_dna_core::contract::StageId;
use bijux_dna_core::prelude::hashing::params_hash;
use serde::{Deserialize, Serialize};

use crate::params::{
    AlignEffectiveParams, AuthenticityEffectiveParams, BamEffectiveParams,
    BiasMitigationEffectiveParams, BqsrEffectiveParams, ComplexityEffectiveParams,
    ContaminationEffectiveParams, CoverageEffectiveParams, DamageEffectiveParams,
    EndogenousContentEffectiveParams, FilterEffectiveParams, GenotypingEffectiveParams,
    HaplogroupEffectiveParams, KinshipEffectiveParams, MarkDupEffectiveParams,
    QcPreEffectiveParams, SexEffectiveParams, ValidateEffectiveParams,
};

mod specs;
pub use specs::{required_audit_artifacts, stage_spec, stage_spec_opt, stage_specs};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BamArtifactKind {
    Bam,
    BamIndex,
    ReferenceFasta,
    ReferenceIndex,
    ReferenceDict,
    BedRegions,
    Report,
}

impl BamArtifactKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bam => "bam",
            Self::BamIndex => "bam_index",
            Self::ReferenceFasta => "reference_fasta",
            Self::ReferenceIndex => "reference_index",
            Self::ReferenceDict => "reference_dict",
            Self::BedRegions => "bed_regions",
            Self::Report => "report",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BamStageContract {
    pub input: BamArtifactKind,
    pub output: BamArtifactKind,
    pub emits_bam: bool,
    pub emits_report: bool,
    pub sorting: &'static str,
    pub indexing: &'static str,
    pub read_group_policy: &'static str,
    pub duplicate_policy: &'static str,
    pub mapping_quality_policy: &'static str,
    pub deterministic: bool,
    pub nondeterminism_reason: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditArtifact {
    pub name: &'static str,
    pub filename: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BamStage {
    Align,
    Validate,
    QcPre,
    MappingSummary,
    Filter,
    MapqFilter,
    LengthFilter,
    Markdup,
    DuplicationMetrics,
    Complexity,
    Coverage,
    InsertSize,
    GcBias,
    EndogenousContent,
    OverlapCorrection,
    Damage,
    Authenticity,
    Contamination,
    Sex,
    BiasMitigation,
    Recalibration,
    Haplogroups,
    Genotyping,
    Kinship,
}

pub const STAGE_PREFIX: &str = "bam.";

impl BamStage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            BamStage::Align => "bam.align",
            BamStage::Validate => "bam.validate",
            BamStage::QcPre => "bam.qc_pre",
            BamStage::MappingSummary => "bam.mapping_summary",
            BamStage::Filter => "bam.filter",
            BamStage::MapqFilter => "bam.mapq_filter",
            BamStage::LengthFilter => "bam.length_filter",
            BamStage::Markdup => "bam.markdup",
            BamStage::DuplicationMetrics => "bam.duplication_metrics",
            BamStage::Complexity => "bam.complexity",
            BamStage::Coverage => "bam.coverage",
            BamStage::InsertSize => "bam.insert_size",
            BamStage::GcBias => "bam.gc_bias",
            BamStage::EndogenousContent => "bam.endogenous_content",
            BamStage::OverlapCorrection => "bam.overlap_correction",
            BamStage::Damage => "bam.damage",
            BamStage::Authenticity => "bam.authenticity",
            BamStage::Contamination => "bam.contamination",
            BamStage::Sex => "bam.sex",
            BamStage::BiasMitigation => "bam.bias_mitigation",
            BamStage::Recalibration => "bam.recalibration",
            BamStage::Haplogroups => "bam.haplogroups",
            BamStage::Genotyping => "bam.genotyping",
            BamStage::Kinship => "bam.kinship",
        }
    }

    #[must_use]
    pub fn id(self) -> StageId {
        StageId::from_static(self.as_str())
    }

    #[must_use]
    pub const fn all() -> &'static [BamStage] {
        &[
            BamStage::Align,
            BamStage::Validate,
            BamStage::QcPre,
            BamStage::MappingSummary,
            BamStage::Filter,
            BamStage::MapqFilter,
            BamStage::LengthFilter,
            BamStage::Markdup,
            BamStage::DuplicationMetrics,
            BamStage::Complexity,
            BamStage::Coverage,
            BamStage::InsertSize,
            BamStage::GcBias,
            BamStage::EndogenousContent,
            BamStage::OverlapCorrection,
            BamStage::Damage,
            BamStage::Authenticity,
            BamStage::Contamination,
            BamStage::Sex,
            BamStage::BiasMitigation,
            BamStage::Recalibration,
            BamStage::Haplogroups,
            BamStage::Genotyping,
            BamStage::Kinship,
        ]
    }

    /// # Errors
    /// Returns an error if the provided JSON does not match the expected
    /// effective params schema for the stage.
    pub fn parse_effective_params(self, value: &serde_json::Value) -> Result<BamEffectiveParams> {
        match self {
            BamStage::Align => serde_json::from_value::<AlignEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Align),
            BamStage::Validate => serde_json::from_value::<ValidateEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Validate),
            BamStage::QcPre => serde_json::from_value::<QcPreEffectiveParams>(value.clone())
                .map(BamEffectiveParams::QcPre),
            BamStage::MappingSummary => {
                serde_json::from_value::<QcPreEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::MappingSummary)
            }
            BamStage::Filter => serde_json::from_value::<FilterEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Filter),
            BamStage::MapqFilter => serde_json::from_value::<FilterEffectiveParams>(value.clone())
                .map(BamEffectiveParams::MapqFilter),
            BamStage::LengthFilter => {
                serde_json::from_value::<FilterEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::LengthFilter)
            }
            BamStage::Markdup => serde_json::from_value::<MarkDupEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Markdup),
            BamStage::DuplicationMetrics => {
                serde_json::from_value::<MarkDupEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::DuplicationMetrics)
            }
            BamStage::Complexity => {
                serde_json::from_value::<ComplexityEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::Complexity)
            }
            BamStage::Coverage => serde_json::from_value::<CoverageEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Coverage),
            BamStage::InsertSize => {
                serde_json::from_value::<CoverageEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::InsertSize)
            }
            BamStage::GcBias => serde_json::from_value::<CoverageEffectiveParams>(value.clone())
                .map(BamEffectiveParams::GcBias),
            BamStage::EndogenousContent => {
                serde_json::from_value::<EndogenousContentEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::EndogenousContent)
            }
            BamStage::OverlapCorrection => {
                serde_json::from_value::<FilterEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::OverlapCorrection)
            }
            BamStage::Damage => serde_json::from_value::<DamageEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Damage),
            BamStage::Authenticity => {
                serde_json::from_value::<AuthenticityEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::Authenticity)
            }
            BamStage::Contamination => {
                serde_json::from_value::<ContaminationEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::Contamination)
            }
            BamStage::Sex => serde_json::from_value::<SexEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Sex),
            BamStage::BiasMitigation => {
                serde_json::from_value::<BiasMitigationEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::BiasMitigation)
            }
            BamStage::Recalibration => serde_json::from_value::<BqsrEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Recalibration),
            BamStage::Haplogroups => {
                serde_json::from_value::<HaplogroupEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::Haplogroups)
            }
            BamStage::Genotyping => {
                serde_json::from_value::<GenotypingEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::Genotyping)
            }
            BamStage::Kinship => serde_json::from_value::<KinshipEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Kinship),
        }
        .map_err(|err| anyhow!("failed to parse params for {}: {err}", self.as_str()))
    }
}

impl TryFrom<&str> for BamStage {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "bam.align" => Ok(BamStage::Align),
            "bam.validate" => Ok(BamStage::Validate),
            "bam.qc_pre" => Ok(BamStage::QcPre),
            "bam.mapping_summary" => Ok(BamStage::MappingSummary),
            "bam.filter" => Ok(BamStage::Filter),
            "bam.mapq_filter" => Ok(BamStage::MapqFilter),
            "bam.length_filter" => Ok(BamStage::LengthFilter),
            "bam.markdup" => Ok(BamStage::Markdup),
            "bam.duplication_metrics" => Ok(BamStage::DuplicationMetrics),
            "bam.complexity" => Ok(BamStage::Complexity),
            "bam.coverage" => Ok(BamStage::Coverage),
            "bam.insert_size" => Ok(BamStage::InsertSize),
            "bam.gc_bias" => Ok(BamStage::GcBias),
            "bam.endogenous_content" => Ok(BamStage::EndogenousContent),
            "bam.overlap_correction" => Ok(BamStage::OverlapCorrection),
            "bam.damage" => Ok(BamStage::Damage),
            "bam.authenticity" => Ok(BamStage::Authenticity),
            "bam.contamination" => Ok(BamStage::Contamination),
            "bam.sex" => Ok(BamStage::Sex),
            "bam.bias_mitigation" => Ok(BamStage::BiasMitigation),
            "bam.recalibration" => Ok(BamStage::Recalibration),
            "bam.haplogroups" => Ok(BamStage::Haplogroups),
            "bam.genotyping" => Ok(BamStage::Genotyping),
            "bam.kinship" => Ok(BamStage::Kinship),
            _ => Err(anyhow!("unknown bam stage: {value}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArtifactPolicy {
    pub required_outputs: &'static [&'static str],
    pub required_audit: &'static [AuditArtifact],
}

#[derive(Debug, Clone)]
pub struct BamStageSpec {
    pub stage: BamStage,
    pub required_inputs: &'static [&'static str],
    pub artifact_policy: ArtifactPolicy,
    pub default_params: BamEffectiveParams,
}

#[derive(Debug, Clone)]
pub struct StageSpec {
    pub id: StageId,
    pub stage: BamStage,
    pub contract: BamStageContract,
}

#[must_use]
pub fn contract_for_stage(stage_id: &str) -> Option<BamStageContract> {
    let stage = BamStage::try_from(stage_id).ok()?;
    match stage {
        BamStage::Align => Some(BamStageContract {
            input: BamArtifactKind::ReferenceFasta,
            output: BamArtifactKind::Bam,
            emits_bam: true,
            emits_report: true,
            sorting: "output_sorting_tool_specific",
            indexing: "produces_index_if_requested",
            read_group_policy: "adds_or_regenerates_read_groups",
            duplicate_policy: "no_duplicate_marking",
            mapping_quality_policy: "no_mapping_quality_filter",
            deterministic: true,
            nondeterminism_reason: None,
        }),
        BamStage::Filter
        | BamStage::MapqFilter
        | BamStage::LengthFilter
        | BamStage::OverlapCorrection
        | BamStage::Markdup
        | BamStage::DuplicationMetrics
        | BamStage::Recalibration => Some(BamStageContract {
            input: BamArtifactKind::Bam,
            output: BamArtifactKind::Bam,
            emits_bam: true,
            emits_report: true,
            sorting: "requires_coordinate_sorted_input",
            indexing: "requires_index_and_produces_index",
            read_group_policy: "preserves_read_groups",
            duplicate_policy: if stage == BamStage::Markdup || stage == BamStage::DuplicationMetrics
            {
                "marks_duplicates"
            } else {
                "preserves_duplicates"
            },
            mapping_quality_policy: if stage == BamStage::Filter || stage == BamStage::MapqFilter {
                "filters_by_mapping_quality_threshold"
            } else {
                "no_mapping_quality_filter"
            },
            deterministic: stage != BamStage::Markdup,
            nondeterminism_reason: if stage == BamStage::Markdup {
                Some("duplicate marking can be tool-dependent")
            } else {
                None
            },
        }),
        BamStage::Coverage
        | BamStage::InsertSize
        | BamStage::GcBias
        | BamStage::EndogenousContent
        | BamStage::Damage
        | BamStage::Authenticity
        | BamStage::Contamination
        | BamStage::Sex
        | BamStage::Haplogroups
        | BamStage::Genotyping
        | BamStage::Kinship => Some(BamStageContract {
            input: BamArtifactKind::Bam,
            output: BamArtifactKind::Report,
            emits_bam: false,
            emits_report: true,
            sorting: "requires_coordinate_sorted_input",
            indexing: "requires_index_and_refuses_missing_index",
            read_group_policy: "requires_read_groups_for_sample_metadata",
            duplicate_policy: "preserves_duplicates",
            mapping_quality_policy: "no_mapping_quality_filter",
            deterministic: true,
            nondeterminism_reason: None,
        }),
        _ => Some(BamStageContract {
            input: BamArtifactKind::Bam,
            output: BamArtifactKind::Report,
            emits_bam: false,
            emits_report: true,
            sorting: "accepts_unsorted_bam",
            indexing: "index_optional",
            read_group_policy: "requires_read_groups_for_sample_metadata",
            duplicate_policy: "no_duplicate_marking",
            mapping_quality_policy: "no_mapping_quality_filter",
            deterministic: true,
            nondeterminism_reason: None,
        }),
    }
}

fn tool_ids_for_stage(stage_id: &str) -> Vec<&'static str> {
    match stage_id {
        "bam.align" => vec!["bwa", "bowtie2"],
        "bam.validate" | "bam.qc_pre" | "bam.mapping_summary" | "bam.endogenous_content" => {
            vec!["samtools"]
        }
        "bam.filter" | "bam.mapq_filter" => vec!["samtools", "bamtools"],
        "bam.length_filter" | "bam.duplication_metrics" => vec!["samtools", "picard"],
        "bam.markdup" => vec!["samtools", "picard"],
        "bam.insert_size" | "bam.gc_bias" => vec!["picard"],
        "bam.recalibration" => vec!["gatk"],
        "bam.coverage" => vec!["mosdepth", "samtools", "bedtools"],
        "bam.overlap_correction" => vec!["bamutil"],
        "bam.damage" => vec![
            "addeam",
            "damageprofiler",
            "mapdamage2",
            "ngsbriggs",
            "pmdtools",
            "pydamage",
        ],
        "bam.complexity" => vec!["preseq"],
        "bam.authenticity" => vec!["authenticct", "pmdtools"],
        "bam.haplogroups" => vec!["yleaf"],
        "bam.kinship" => vec!["king"],
        "bam.contamination" => vec!["schmutzi", "contammix", "verifybamid2"],
        "bam.sex" => vec!["rxy", "angsd"],
        "bam.bias_mitigation" => vec!["mapdamage2"],
        "bam.genotyping" => vec!["angsd", "bcftools"],
        _ => Vec::new(),
    }
}

#[must_use]
pub fn stage_contract_json(stage_id: &str) -> Option<serde_json::Value> {
    let stage = BamStage::try_from(stage_id).ok()?;
    let spec = stage_spec(stage);
    let contract = contract_for_stage(stage_id)?;
    let required_outputs = spec.artifact_policy.required_outputs;
    let required_audit: Vec<serde_json::Value> = spec
        .artifact_policy
        .required_audit
        .iter()
        .map(|artifact| {
            serde_json::json!({
                "name": artifact.name,
                "filename": artifact.filename,
            })
        })
        .collect();
    Some(serde_json::json!({
        "schema_version": "bijux.stage_contract.v1",
        "stage_id": stage_id,
        "inputs": spec.required_inputs,
        "outputs": required_outputs,
        "audit_artifacts": required_audit,
        "io": {
            "input_kind": contract.input.as_str(),
            "output_kind": contract.output.as_str(),
            "emits_bam": contract.emits_bam,
            "emits_report": contract.emits_report,
            "sorting": contract.sorting,
            "indexing": contract.indexing,
            "read_group_policy": contract.read_group_policy,
            "duplicate_policy": contract.duplicate_policy,
            "mapping_quality_policy": contract.mapping_quality_policy,
            "deterministic": contract.deterministic,
            "nondeterminism_reason": contract.nondeterminism_reason,
        },
        "tool_ids": tool_ids_for_stage(stage_id),
    }))
}

/// # Errors
/// Returns an error if JSON canonicalization fails.
pub fn stage_contract_hash(stage_id: &str) -> Option<anyhow::Result<String>> {
    let json = stage_contract_json(stage_id)?;
    let canonical = canonicalize_json_value(&json);
    Some(params_hash(&canonical).map_err(anyhow::Error::from))
}
