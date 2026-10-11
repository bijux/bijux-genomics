//! Typed views keep required adapter output fields checked by the compiler.
#[derive(Clone, Copy)]
pub(super) struct DeclaredArtifact<'a> {
    pub(super) artifact_id: &'a str,
    pub(super) role: &'a str,
    pub(super) path: &'a str,
}
pub(super) struct OutputObservation<'a> {
    pub(super) stage_id: &'a str,
    pub(super) tool_id: &'a str,
    pub(super) benchmark_status: &'a str,
    pub(super) stage_output_ids: &'a [String],
    pub(super) raw_output_ids: &'a [String],
    pub(super) declared_outputs: Vec<DeclaredArtifact<'a>>,
}
pub(super) trait OutputSource {
    fn output_observation(&self) -> OutputObservation<'_>;
}
macro_rules! output_source {
    ($row:ty) => {
        impl OutputSource for $row {
            fn output_observation(&self) -> OutputObservation<'_> {
                OutputObservation {
                    stage_id: &self.stage_id,
                    tool_id: &self.tool_id,
                    benchmark_status: &self.benchmark_status,
                    stage_output_ids: &self.stage_output_ids,
                    raw_output_ids: &self.raw_output_ids,
                    declared_outputs: self
                        .declared_outputs
                        .iter()
                        .map(|artifact| DeclaredArtifact {
                            artifact_id: &artifact.artifact_id,
                            role: &artifact.role,
                            path: &artifact.path,
                        })
                        .collect(),
                }
            }
        }
    };
}
output_source!(crate::adapters::bcftools::VcfBcftoolsAdapterRow);
output_source!(crate::adapters::angsd::VcfAngsdAdapterRow);
output_source!(crate::adapters::plink::VcfPlinkFamilyAdapterRow);
output_source!(crate::adapters::phasing::VcfPhasingFamilyAdapterRow);
output_source!(crate::adapters::imputation::VcfImputationFamilyAdapterRow);
output_source!(crate::adapters::eigensoft::VcfEigensoftAdapterRow);
output_source!(crate::adapters::descent::VcfDescentFamilyAdapterRow);
