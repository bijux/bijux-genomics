use crate::adapters::{
    descent::VcfDescentFamilyAdapterReport, imputation::VcfImputationFamilyAdapterReport,
    phasing::VcfPhasingFamilyAdapterReport,
};
use crate::evidence::smoke::VcfLocalContainerSmokeReport;
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::model::{
    command_reports::VcfRenderedCommandsReport, expectations::VcfExpectedBenchmarkResultsReport,
    parsers::VcfParserFixtureCoverageReport, reports::VcfReportMapReport,
    scope::VcfActiveStageToolMatrixReport,
};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
/// Scientific readiness request identity, independent of concrete service crate dependencies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectReadinessStage {
    Call,
    CallDiploid,
    CallPseudohaploid,
    CallGl,
    DamageFilter,
    GlPropagation,
    Filter,
    Stats,
    Qc,
    PrepareReferencePanel,
    ImputationMetrics,
    Pca,
    Admixture,
    PopulationStructure,
}
impl DirectReadinessStage {
    /// Canonical stage identity used for external execution evidence admission.
    #[must_use]
    pub const fn stage_id(self) -> &'static str {
        match self {
            Self::Call => "vcf.call",
            Self::CallDiploid => "vcf.call_diploid",
            Self::CallPseudohaploid => "vcf.call_pseudohaploid",
            Self::CallGl => "vcf.call_gl",
            Self::DamageFilter => "vcf.damage_filter",
            Self::GlPropagation => "vcf.gl_propagation",
            Self::Filter => "vcf.filter",
            Self::Stats => "vcf.stats",
            Self::Qc => "vcf.qc",
            Self::PrepareReferencePanel => "vcf.prepare_reference_panel",
            Self::ImputationMetrics => "vcf.imputation_metrics",
            Self::Pca => "vcf.pca",
            Self::Admixture => "vcf.admixture",
            Self::PopulationStructure => "vcf.population_structure",
        }
    }
}
/// Composition provides concrete scientific execution and canonical external bindings.
pub struct DirectReadinessPorts {
    pub output_path: fn(DirectReadinessStage) -> &'static str,
    pub render: fn(&Path, DirectReadinessStage, PathBuf) -> Result<()>,
    pub external_bindings: ExternalBindingProvider,
}
/// Typed evidence publication ports preserve their independently owned report contracts.
pub struct EvidencePublicationPorts {
    pub matrix: fn(&Path, PathBuf) -> Result<VcfActiveStageToolMatrixReport>,
    pub smoke: fn(&Path, PathBuf) -> Result<VcfLocalContainerSmokeReport>,
    pub parsers: fn(&Path, PathBuf) -> Result<VcfParserFixtureCoverageReport>,
    pub commands: fn(&Path, PathBuf) -> Result<VcfRenderedCommandsReport>,
    pub expectations: fn(&Path, PathBuf) -> Result<VcfExpectedBenchmarkResultsReport>,
    pub reports: fn(&Path, PathBuf) -> Result<VcfReportMapReport>,
}
/// Adapter execution is supplied explicitly while completion owns persisted report admission.
pub struct AdapterPublicationPorts {
    pub phasing: fn(&Path, &str, PathBuf) -> Result<VcfPhasingFamilyAdapterReport>,
    pub imputation: fn(&Path, PathBuf) -> Result<VcfImputationFamilyAdapterReport>,
    pub descent: fn(&Path, PathBuf) -> Result<VcfDescentFamilyAdapterReport>,
}
/// Explicit execution authorities for the complete retained-tool diagnostic.
pub struct VcfCompletionPorts {
    pub direct: DirectReadinessPorts,
    pub evidence: EvidencePublicationPorts,
    pub adapters: AdapterPublicationPorts,
}

/// Canonical external stage/tool bindings supplied by delivery composition.
pub type ExternalBindingProvider = fn(&Path) -> Result<BTreeSet<(String, String)>>;
