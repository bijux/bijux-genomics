use bijux_dna_bench_bam_proofs::ProofContext;
use std::path::Path;
mod native;
mod readiness;
pub(super) fn context(root: &Path) -> ProofContext<'_> {
    ProofContext {
        repo_root: root,
        readiness: &readiness::ReadinessAdapter,
        native: &native::NativeAdapter,
    }
}
