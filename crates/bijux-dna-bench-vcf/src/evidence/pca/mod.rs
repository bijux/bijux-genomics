//! Typed PCA adapter observations and command publication, independent of scientific readiness.
mod acquisition;
mod commands;
mod outputs;
mod publication;
mod serialization;
pub use acquisition::prepare_pca_adapter_evidence;
use bijux_dna_bench_vcf_evidence::model::pca::PcaAdapterEvidence;
pub use publication::publish_pca_commands;
/// Adapter observations from one admitted acquisition; output addresses remain declarations.
pub struct PreparedPcaAdapterEvidence {
    rows: Vec<PcaAdapterEvidence>,
}
impl PreparedPcaAdapterEvidence {
    /// Borrow the full binding observations for scientific readiness admission.
    #[must_use]
    pub fn rows(&self) -> &[PcaAdapterEvidence] {
        &self.rows
    }
}
pub const VCF_PCA_RENDERED_COMMANDS_PATH: &str =
    "artifacts/bench-readiness/vcf-pca-ready/vcf-pca-rendered-commands.sh";
