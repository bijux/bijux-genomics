//! Owner: benchmark qualification composition.
//! Compose container qualification through repository, policy and science owners.

mod assessment;
mod decisions;
mod external;
mod image;
mod observations;
mod publication;

use self::assessment::derive_tool_assessment;
use self::publication::{apply_release_evidence, freshness_identity};
use crate::repo::container_qualification::{
    load_release_publication, load_security_publication, repository_source, INVENTORY_PATH,
    RUNS_PATH, RUNTIME_PATH, SECURITY_PATH,
};

use crate::AdmittedBenchmarkStageReport;
use anyhow::Result;
use bijux_dna_containers::evidence::{
    load_run_evidence_index, ContainerEvidenceChain, RunEvidenceRecord,
};
use bijux_dna_policies::{
    admit_release_evidence, qualification_sources_sha256, AdmittedSecurityEvidence,
    QualificationEvidenceSource, QualificationReport,
};
use std::collections::BTreeMap;
use std::path::Path;

const CATALOG_PATH: &str = "containers/contracts/catalog.toml";

struct QualificationContext<'a> {
    benchmark_reports: &'a [AdmittedBenchmarkStageReport],
    chain: &'a ContainerEvidenceChain,
    freshness_sha256: &'a str,
    runtime_sha256: &'a str,
    security_evidence: Option<&'a AdmittedSecurityEvidence<'a>>,
}

/// Compose qualification from admitted container, runtime and benchmark evidence.
///
/// # Errors
/// Returns an error for missing, stale, malformed or inconsistent evidence.
pub fn derive_container_qualification_report(
    repository_root: &Path,
    chain: &ContainerEvidenceChain,
    benchmark_reports: &[AdmittedBenchmarkStageReport],
) -> Result<QualificationReport> {
    derive_container_qualification_report_inner(repository_root, chain, benchmark_reports, None)
}

/// Compose qualification including an explicit immutable release publication.
///
/// # Errors
/// Returns an error when a release or supporting source cannot be admitted.
pub fn derive_container_qualification_report_with_release(
    repository_root: &Path,
    chain: &ContainerEvidenceChain,
    benchmark_reports: &[AdmittedBenchmarkStageReport],
    release_publication_path: &Path,
) -> Result<QualificationReport> {
    derive_container_qualification_report_inner(
        repository_root,
        chain,
        benchmark_reports,
        Some(release_publication_path),
    )
}

fn derive_container_qualification_report_inner(
    repository_root: &Path,
    chain: &ContainerEvidenceChain,
    benchmark_reports: &[AdmittedBenchmarkStageReport],
    release_publication_path: Option<&Path>,
) -> Result<QualificationReport> {
    let inventory_source = repository_source(repository_root, INVENTORY_PATH)?;
    let runtime_source = repository_source(repository_root, RUNTIME_PATH)?;
    let security_publication = load_security_publication(repository_root)?;
    let security_source = security_publication
        .as_ref()
        .map(|_| repository_source(repository_root, SECURITY_PATH))
        .transpose()?;
    let security_evidence = security_publication
        .as_ref()
        .zip(security_source.as_ref())
        .map(|(publication, source)| {
            bijux_dna_policies::admit_security_evidence(
                publication,
                &source.sha256,
                &chain.repository_commit,
                &chain.catalog_sha256,
                &inventory_source.sha256,
            )
        })
        .transpose()?;
    let runs_source = repository_source(repository_root, RUNS_PATH)?;
    let runtime_sha256 = runtime_source.sha256.clone();
    let mut sources = BTreeMap::from([
        (
            "catalog".to_string(),
            QualificationEvidenceSource {
                uri: format!("repository://{CATALOG_PATH}"),
                sha256: chain.catalog_sha256.clone(),
            },
        ),
        ("inventory".to_string(), inventory_source),
        ("runs".to_string(), runs_source),
        ("runtime".to_string(), runtime_source),
    ]);
    for admission in benchmark_reports {
        sources.insert(
            format!("benchmark:{}", admission.report.run_id),
            QualificationEvidenceSource {
                uri: format!("repository://benchmarks/reports/{}", admission.report_file_name),
                sha256: admission.source_sha256.clone(),
            },
        );
    }
    if let Some(source) = security_source {
        sources.insert("security".to_string(), source);
    }
    let freshness_sha256 = freshness_identity(&chain.repository_commit, &sources)?;
    let context = QualificationContext {
        benchmark_reports,
        chain,
        freshness_sha256: &freshness_sha256,
        runtime_sha256: &runtime_sha256,
        security_evidence: security_evidence.as_ref(),
    };
    let runs = load_run_evidence_index(&repository_root.join(RUNS_PATH))?;
    let inventory_tools = chain
        .inventory
        .tools
        .iter()
        .map(|tool| (tool.tool.as_str(), tool))
        .collect::<BTreeMap<_, _>>();
    let run_records = runs.runs.iter().fold(
        BTreeMap::<&str, Vec<&RunEvidenceRecord>>::new(),
        |mut records, run| {
            records.entry(run.tool.as_str()).or_default().push(run);
            records
        },
    );
    let provisional_assessments = chain
        .catalog
        .tools
        .iter()
        .map(|tool| {
            derive_tool_assessment(
                tool,
                inventory_tools.get(tool.id.as_str()).copied(),
                run_records.get(tool.id.as_str()).map(Vec::as_slice).unwrap_or_default(),
                &context,
            )
        })
        .collect::<Result<Vec<_>>>()?;

    let release_publication = release_publication_path.map(load_release_publication).transpose()?;
    let assessments = if let Some((publication, source)) = release_publication.as_ref() {
        let sources_sha256 = qualification_sources_sha256(&sources)?;
        let admitted = admit_release_evidence(
            publication,
            &source.sha256,
            &chain.repository_commit,
            &chain.catalog_sha256,
            &sources_sha256,
            &provisional_assessments,
        )?;
        let assessments = apply_release_evidence(provisional_assessments, &admitted)?;
        sources.insert("release".to_string(), source.clone());
        assessments
    } else {
        provisional_assessments
    };

    QualificationReport::v4(sources, assessments)
}

#[cfg(test)]
#[path = "../../../tests/contracts/qualification/owner.rs"]
mod tests;
