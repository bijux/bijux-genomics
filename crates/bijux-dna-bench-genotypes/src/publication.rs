//! Retained native evidence and stable genotype benchmark artifact publication.
pub(crate) use bijux_dna_bench_artifacts::{
    execution_parent, PublicationPlan, PublishedArtifactPaths, ReportOwner,
};
pub(crate) const SCHEMA: bijux_dna_bench_artifacts::PublicationSchema =
    bijux_dna_bench_artifacts::PublicationSchema {
        inventory_name: "genotype-artifact-ownership.json",
        schema_version: "bijux.bench.genotype_artifact_ownership.v1",
    };
pub(crate) struct ReportContext<'a> {
    pub(crate) paths: PublishedArtifactPaths<'a>,
    pub(crate) started_at: &'a str,
    pub(crate) finished_at: &'a str,
    pub(crate) elapsed_seconds: f64,
}
pub(crate) fn timestamp_marker() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    format!("unix:{seconds}")
}
