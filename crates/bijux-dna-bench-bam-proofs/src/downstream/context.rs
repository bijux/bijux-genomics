use super::ports::{KinshipReadiness, ParserEvidence, PipelineEvidence, VariantEvidence};
use crate::ProofContext;
/// Independent canonical observations for cross-stage scientific proof.
pub struct DownstreamContext<'a> {
    pub proofs: ProofContext<'a>,
    pub parsers: &'a dyn ParserEvidence,
    pub pipelines: &'a dyn PipelineEvidence,
    pub variants: &'a dyn VariantEvidence,
    pub kinship: &'a dyn KinshipReadiness,
}
