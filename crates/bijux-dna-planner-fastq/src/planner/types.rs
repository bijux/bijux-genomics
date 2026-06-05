use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::PipelineSpec;
use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::prelude::{ContainerImageRefV1, ToolExecutionSpecV1};
use bijux_dna_stage_contract::ArtifactRef;
use bijux_dna_stage_contract::PlanDecisionReason;

#[derive(Debug, Clone)]
pub struct FastqPlanConfig {
    pub pipeline_id: String,
    pub policy: PlanPolicy,
    pub selection_objective: bijux_dna_core::contract::Objective,
    pub pipeline_spec: Option<PipelineSpec>,
    pub stage_bindings: Vec<FastqStageBinding>,
    pub stage_toolsets: Vec<FastqStageToolsetBinding>,
    pub aux_images: BTreeMap<String, ContainerImageRefV1>,
    pub adapter_bank: Option<serde_json::Value>,
    pub polyx_bank: Option<serde_json::Value>,
    pub contaminant_bank: Option<serde_json::Value>,
    pub enable_contaminant_removal: bool,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub reference_fasta: Option<PathBuf>,
    pub out_dir: PathBuf,
    pub allow_planned: bool,
}

#[derive(Debug, Clone)]
pub struct FastqStageBinding {
    pub stage_id: String,
    pub stage_instance_id: Option<String>,
    pub tool: ToolExecutionSpecV1,
    pub reason: Option<PlanDecisionReason>,
    pub params: Option<FastqStageParameters>,
}

#[derive(Debug, Clone)]
pub struct FastqStageToolsetBinding {
    pub stage_id: String,
    pub stage_instance_id: Option<String>,
    pub tools: Vec<ToolExecutionSpecV1>,
    pub reason: Option<PlanDecisionReason>,
    pub params: Option<FastqStageParameters>,
}

#[derive(Debug, Clone)]
pub struct FastqStageExplicitInput {
    pub input_id: String,
    pub artifact: ArtifactRef,
    pub source_tool_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum FastqStageParameters {
    Validate(bijux_dna_domain_fastq::params::validate::ValidateEffectiveParams),
    DetectAdapters(DetectAdaptersStageParams),
    FilterReads(FilterReadsStageParams),
    FilterLowComplexity(FilterLowComplexityStageParams),
    ExtractUmis(ExtractUmisStageParams),
    ProfileReadLengths(bijux_dna_domain_fastq::FastqReadLengthProfileParams),
    ProfileOverrepresented(bijux_dna_domain_fastq::FastqOverrepresentedProfileParams),
    ProfileReads(bijux_dna_domain_fastq::params::stats::FastqStatsParams),
    RemoveDuplicates(
        bijux_dna_domain_fastq::params::remove_duplicates::RemoveDuplicatesEffectiveParams,
    ),
    RemoveChimeras(bijux_dna_domain_fastq::params::edna::ChimeraDetectionEffectiveParams),
    ReportQc(bijux_dna_domain_fastq::params::qc_post::QcPostEffectiveParams),
    Trim(bijux_dna_domain_fastq::params::trim::TrimEffectiveParams),
    TrimPolygTails(bijux_dna_domain_fastq::params::trim::TrimPolygTailsParams),
    MergePairs(MergePairsStageParams),
    NormalizePrimers(NormalizePrimersStageParams),
    NormalizeAbundance(NormalizeAbundanceStageParams),
    Screen(bijux_dna_domain_fastq::params::screen::ScreenEffectiveParams),
    IndexReference(IndexReferenceStageParams),
    InferAsvs(InferAsvsStageParams),
    ClusterOtus(ClusterOtusStageParams),
    CorrectErrors(CorrectErrorsStageParams),
    TrimTerminalDamage(TrimTerminalDamageStageParams),
    DepleteRrna(DepleteRrnaStageParams),
    DepleteHost(DepleteHostStageParams),
    DepleteReferenceContaminants(DepleteReferenceContaminantsStageParams),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FilterReadsStageParams {
    pub threads: Option<u32>,
    pub max_n: Option<u32>,
    pub max_n_fraction: Option<f64>,
    pub max_n_count: Option<u32>,
    pub low_complexity_threshold: Option<f64>,
    pub entropy_threshold: Option<f64>,
    pub kmer_ref: Option<PathBuf>,
    pub polyx_policy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FilterLowComplexityStageParams {
    pub entropy_threshold: Option<f64>,
    pub polyx_threshold: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExtractUmisStageParams {
    pub threads: Option<u32>,
    pub umi_pattern: Option<String>,
    pub extraction_location: Option<String>,
    pub read_name_transform: Option<String>,
    pub failed_extraction_policy: Option<String>,
    pub grouping_policy: Option<String>,
    pub downstream_dedup_policy: Option<String>,
    pub downstream_propagation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DetectAdaptersStageParams {
    pub threads: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergePairsStageParams {
    pub threads: Option<u32>,
    pub merge_overlap: Option<u32>,
    pub min_len: Option<u32>,
    pub unmerged_read_policy: bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy,
}

impl MergePairsStageParams {
    #[must_use]
    pub fn baseline() -> Self {
        Self {
            threads: None,
            merge_overlap: None,
            min_len: None,
            unmerged_read_policy:
                bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy::EmitUnmergedPairs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizeAbundanceStageParams {
    pub method: String,
}

impl NormalizeAbundanceStageParams {
    #[must_use]
    pub fn baseline() -> Self {
        Self { method: "relative_abundance".to_string() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizePrimersStageParams {
    pub primer_set_id: String,
    pub marker_id: Option<String>,
    pub primer_fasta: Option<PathBuf>,
    pub orientation_policy: String,
    pub max_mismatch_rate: f64,
    pub min_overlap_bp: u32,
    pub strict_5p_anchor: bool,
    pub allow_iupac_codes: bool,
}

impl NormalizePrimersStageParams {
    #[must_use]
    pub fn baseline() -> Self {
        Self {
            primer_set_id: "default".to_string(),
            marker_id: None,
            primer_fasta: None,
            orientation_policy: "normalize_to_forward_primer".to_string(),
            max_mismatch_rate: 0.10,
            min_overlap_bp: 10,
            strict_5p_anchor: true,
            allow_iupac_codes: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IndexReferenceStageParams {
    pub threads: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InferAsvsStageParams {
    pub denoising_method: String,
    pub pooling_mode: String,
    pub chimera_policy: String,
    pub threads: Option<u32>,
}

impl InferAsvsStageParams {
    #[must_use]
    pub fn baseline() -> Self {
        Self {
            denoising_method: "dada2".to_string(),
            pooling_mode: "independent".to_string(),
            chimera_policy: "remove_bimera_denovo".to_string(),
            threads: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterOtusStageParams {
    pub otu_identity: f64,
    pub threads: Option<u32>,
}

impl ClusterOtusStageParams {
    #[must_use]
    pub fn baseline() -> Self {
        Self {
            otu_identity: bijux_dna_domain_fastq::params::edna::DEFAULT_OTU_IDENTITY_THRESHOLD,
            threads: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrectErrorsStageParams {
    pub threads: Option<u32>,
    pub quality_encoding: bijux_dna_domain_fastq::params::correct::QualityEncoding,
    pub kmer_size: Option<u32>,
    pub musket_kmer_budget: Option<u64>,
    pub genome_size: Option<u64>,
    pub max_memory_gb: Option<u32>,
    pub trusted_kmer_artifact: Option<PathBuf>,
    pub conservative_mode: bool,
}

impl CorrectErrorsStageParams {
    #[must_use]
    pub fn baseline() -> Self {
        Self {
            threads: None,
            quality_encoding: bijux_dna_domain_fastq::params::correct::QualityEncoding::Phred33,
            kmer_size: None,
            musket_kmer_budget: None,
            genome_size: None,
            max_memory_gb: None,
            trusted_kmer_artifact: None,
            conservative_mode: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrimTerminalDamageStageParams {
    pub threads: Option<u32>,
    pub damage_mode: bijux_dna_domain_fastq::params::DamageMode,
    pub execution_policy:
        Option<bijux_dna_domain_fastq::params::trim::TerminalDamageExecutionPolicy>,
    pub trim_5p_bases: u32,
    pub trim_3p_bases: u32,
}

impl TrimTerminalDamageStageParams {
    #[must_use]
    pub fn baseline() -> Self {
        Self {
            threads: None,
            damage_mode: bijux_dna_domain_fastq::params::DamageMode::Ancient,
            execution_policy: None,
            trim_5p_bases: 2,
            trim_3p_bases: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DepleteRrnaStageParams {
    pub rrna_db: String,
    pub min_identity: f64,
    pub threads: Option<u32>,
}

impl DepleteRrnaStageParams {
    #[must_use]
    pub fn baseline() -> Self {
        Self { rrna_db: "rrna_reference".to_string(), min_identity: 0.95, threads: None }
    }
}

#[derive(Debug, Clone)]
pub struct DepleteHostStageParams {
    pub host_identity_threshold: f64,
    pub retain_unmapped_only: bool,
    pub threads: Option<u32>,
}

impl DepleteHostStageParams {
    #[must_use]
    pub fn baseline() -> Self {
        Self { host_identity_threshold: 0.95, retain_unmapped_only: true, threads: None }
    }
}

#[derive(Debug, Clone)]
pub struct DepleteReferenceContaminantsStageParams {
    pub decoy_mode: String,
    pub threads: Option<u32>,
}

impl DepleteReferenceContaminantsStageParams {
    #[must_use]
    pub fn baseline() -> Self {
        Self { decoy_mode: "phix_and_spikeins".to_string(), threads: None }
    }
}

#[derive(Debug, Clone)]
pub struct FastqStageBenchmarkConfig {
    pub pipeline_id: String,
    pub policy: PlanPolicy,
    pub stage_id: String,
    pub tools: Vec<ToolExecutionSpecV1>,
    pub params: Option<FastqStageParameters>,
    pub aux_images: BTreeMap<String, ContainerImageRefV1>,
    pub adapter_bank: Option<serde_json::Value>,
    pub polyx_bank: Option<serde_json::Value>,
    pub contaminant_bank: Option<serde_json::Value>,
    pub enable_contaminant_removal: bool,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub reference_fasta: Option<PathBuf>,
    pub out_dir: PathBuf,
    pub allow_planned: bool,
}
