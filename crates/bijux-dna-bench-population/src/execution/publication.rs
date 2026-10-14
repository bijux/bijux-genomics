//! Publish population artifacts by consuming their admitted disposable staging root.
use anyhow::Result;
use bijux_dna_bench_artifacts::{PublicationSchema, StagedPublicationPlan};
use std::path::Path;

pub(crate) use bijux_dna_bench_artifacts::ReportOwner;

const SCHEMA: PublicationSchema = PublicationSchema {
    inventory_name: "population-artifact-ownership.json",
    schema_version: "bijux.bench.population_artifact_ownership.v1",
};

pub(crate) fn publish(staged: &Path, destination: &Path, owner: &ReportOwner<'_>) -> Result<()> {
    StagedPublicationPlan::admit(staged, destination, owner, &SCHEMA)?.publish()
}
