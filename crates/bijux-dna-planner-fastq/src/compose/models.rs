use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_stage_contract::ArtifactRef;

#[derive(Debug, Clone)]
pub(super) struct ReferenceIndexState {
    pub(super) path: PathBuf,
    pub(super) tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageArtifactInputBinding {
    pub from_stage_node_id: String,
    pub from_output_id: String,
    pub to_input_id: String,
}

pub type StageArtifactInputPolicy = BTreeMap<String, Vec<StageArtifactInputBinding>>;
pub type StageDependencyPolicy = BTreeMap<String, Vec<String>>;

#[derive(Debug, Clone)]
pub struct SyntheticStageArtifact {
    pub artifact: ArtifactRef,
    pub source_tool_id: String,
}

pub type SyntheticStageArtifactPolicy = BTreeMap<String, Vec<SyntheticStageArtifact>>;

#[derive(Debug, Clone)]
pub(super) struct ResolvedStageInputArtifact {
    pub(super) to_input_id: String,
    pub(super) artifact: ArtifactRef,
    pub(super) source_stage_id: String,
    pub(super) source_stage_node_id: String,
    pub(super) source_tool_id: String,
}

#[derive(Debug, Clone)]
pub(super) struct PlannedStageLineage {
    pub(super) reads_r1: PathBuf,
    pub(super) reads_r2: Option<PathBuf>,
    pub(super) feature_table: Option<PathBuf>,
    pub(super) reference_index: Option<ReferenceIndexState>,
    pub(super) qc_inputs: Vec<ArtifactRef>,
    pub(super) lineage_inputs: Vec<ArtifactRef>,
}
