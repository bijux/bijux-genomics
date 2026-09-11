use bijux_dna_bench_bam_proofs::downstream::DownstreamContext;
use std::path::Path;
mod parsers;
mod pipelines;
mod readiness;
mod variants;
pub(super) fn context(root: &Path) -> DownstreamContext<'_> {
    DownstreamContext {
        proofs: super::bam_stage_proofs::context(root),
        parsers: &parsers::ParserAdapter,
        pipelines: &pipelines::PipelineAdapter,
        variants: &variants::VariantAdapter,
        kinship: &readiness::KinshipAdapter,
    }
}
