//! Capability-specific ownership protocol for shared execution-evidence publication.
pub(crate) use bijux_dna_bench_artifacts::{
    execution_parent, PublicationPlan, PublishedArtifactPaths, ReportOwner,
};
mod context;
pub(crate) use context::ReportContext;
pub(crate) const SCHEMA: bijux_dna_bench_artifacts::PublicationSchema =
    bijux_dna_bench_artifacts::PublicationSchema {
        inventory_name: "variant-artifact-ownership.json",
        schema_version: "bijux.bench.variant_artifact_ownership.v1",
    };
