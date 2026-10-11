//! Corpus-owned panel authority observations and metadata fixture reports.
use serde::Serialize;
pub const GOVERNED_VCF_PANEL_SPECIES_ID: &str = "Homo sapiens";
pub const GOVERNED_VCF_PANEL_BUILD_ID: &str = "GRCh38";
pub const GOVERNED_VCF_PANEL_ID: &str = "hsapiens_grch38_1kg_3202_chr22";
pub const GOVERNED_VCF_MAP_ID: &str = "hsapiens_grch38_glimpse_chr22";
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedVcfPanelAssetContract {
    pub schema_version: &'static str,
    pub fixture_class: &'static str,
    pub species_id: String,
    pub build_id: String,
    pub panel_id: String,
    pub map_id: String,
    pub panel_files: Vec<String>,
    pub panel_derivations: Vec<ResolvedVcfPanelDerivationContract>,
    pub map_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedVcfPanelDerivationContract {
    pub path: String,
    pub source_file_name: String,
    pub tool_id: String,
    pub tool_version: String,
    pub tool_artifact_sha256: String,
    pub argv: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalVcfPanelFixtureReport {
    pub schema_version: &'static str,
    pub fixture_class: &'static str,
    pub panel_id: String,
    pub map_id: String,
    pub materialized_files: Vec<String>,
}
