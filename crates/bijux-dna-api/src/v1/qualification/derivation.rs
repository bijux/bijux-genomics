use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_bench::{AdmittedBenchmarkStageReport, BenchmarkReportVerdict};
use bijux_dna_containers::evidence::{
    ContainerCatalogTool, ContainerEvidenceChain, ContainerImageInventoryTool,
};
use bijux_dna_policies::{
    evaluate_benchmark_evidence, evaluate_execution_evidence, evaluate_identity_evidence,
    evaluate_source_identity, AdmittedSecurityEvidence, ArtifactAvailability, ArtifactDelivery,
    BenchmarkEvidenceObservation, ContainerSecurityEvidencePublication, EvidenceDecision,
    EvidenceDimension, ExecutionEvidenceObservation, ExecutionEvidenceStatus, ProductSupport,
    PublicationDisposition, QualificationAssessment, QualificationEvidenceSource,
    QualificationReport, ToolSecurityEvidence, BUILD_EVIDENCE_RULE_V1, CATALOG_EVIDENCE_RULE_V1,
    FRESHNESS_EVIDENCE_RULE_V1, RELEASE_EVIDENCE_RULE_V1, REQUIRED_EVIDENCE_DIMENSIONS,
    RUNTIME_EVIDENCE_RULE_V1, SECURITY_EVIDENCE_RULE_V1, SUPPORT_DIMENSIONS_RULE_V2,
};
use bijux_dna_runtime::{
    load_run_evidence_index, AttemptExecutionStatus, RunEvidenceRecord, ScientificAcceptanceStatus,
};
use bijux_dna_science::decision::{
    evaluate_scientific_acceptance, ScientificAcceptanceObservation,
    ScientificAssessmentStatus as ScienceAssessmentStatus,
    ScientificExecutionStatus as ScienceExecutionStatus,
};

const CATALOG_PATH: &str = "containers/contracts/catalog.toml";
const INVENTORY_PATH: &str = "containers/reports/image-inventory.json";
const RUNTIME_PATH: &str = "containers/reports/runtime-verification.json";
const RUNS_PATH: &str = "containers/reports/run-evidence.json";
const SECURITY_PATH: &str = "containers/reports/security-evidence.json";

struct QualificationContext<'a> {
    benchmark_reports: &'a [AdmittedBenchmarkStageReport],
    chain: &'a ContainerEvidenceChain,
    freshness_sha256: &'a str,
    runtime_sha256: &'a str,
    security_evidence: Option<&'a AdmittedSecurityEvidence<'a>>,
}

/// Compose a current qualification report from admitted container, runtime, and benchmark facts.
///
/// # Errors
/// Returns an error when an evidence source is missing, stale, malformed, or internally
/// inconsistent.
pub fn derive_container_qualification_report(
    repository_root: &Path,
    chain: &ContainerEvidenceChain,
    benchmark_reports: &[AdmittedBenchmarkStageReport],
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
    let assessments = chain
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

    QualificationReport::v4(sources, assessments)
}

fn load_security_publication(
    repository_root: &Path,
) -> Result<Option<ContainerSecurityEvidencePublication>> {
    let path = repository_root.join(SECURITY_PATH);
    if !path.is_file() {
        return Ok(None);
    }
    serde_json::from_slice(&std::fs::read(&path)?)
        .with_context(|| format!("parse security evidence {}", path.display()))
        .map(Some)
}

fn derive_tool_assessment(
    tool: &ContainerCatalogTool,
    inventory: Option<&ContainerImageInventoryTool>,
    runs: &[&RunEvidenceRecord],
    context: &QualificationContext<'_>,
) -> Result<QualificationAssessment> {
    if !tool.is_product_supported() {
        let dimensions = REQUIRED_EVIDENCE_DIMENSIONS
            .into_iter()
            .map(|dimension| (dimension, not_applicable(tool)))
            .collect();
        return QualificationAssessment::v3(
            tool.id.clone(),
            product_support(tool.product_support),
            artifact_delivery(tool.artifact_delivery),
            tool.redistribution_permitted,
            ArtifactAvailability::Unavailable,
            PublicationDisposition::Current,
            dimensions,
        );
    }

    if !tool.is_redistributable_image() {
        return external_artifact_assessment(tool, runs, context);
    }

    let inventory = inventory.ok_or_else(|| {
        anyhow::anyhow!("qualification inventory omits retained tool {}", tool.id)
    })?;
    let image_sha256 = inventory
        .selected_image_sha256
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("qualification inventory has no image for {}", tool.id))?;
    let dimensions = redistributable_image_dimensions(tool, runs, context, image_sha256);
    QualificationAssessment::v3(
        tool.id.clone(),
        ProductSupport::Supported,
        ArtifactDelivery::RedistributableImage,
        tool.redistribution_permitted,
        ArtifactAvailability::Admitted,
        PublicationDisposition::Current,
        dimensions,
    )
}

fn external_artifact_assessment(
    tool: &ContainerCatalogTool,
    runs: &[&RunEvidenceRecord],
    context: &QualificationContext<'_>,
) -> Result<QualificationAssessment> {
    let evidence = context.security_evidence.and_then(|security| security.tool_evidence(&tool.id));
    let availability = evidence.map_or(ArtifactAvailability::Unavailable, |item| item.availability);
    if let Some(item) = evidence {
        validate_external_security_dimensions(tool, item)?;
    }
    let dimensions = match (availability, evidence) {
        (ArtifactAvailability::Admitted, Some(item)) => {
            admitted_external_artifact_dimensions(tool, runs, context, item)?
        }
        _ => unavailable_external_artifact_dimensions(tool, context, availability),
    };
    QualificationAssessment::v3(
        tool.id.clone(),
        ProductSupport::Supported,
        artifact_delivery(tool.artifact_delivery),
        tool.redistribution_permitted,
        availability,
        PublicationDisposition::Current,
        dimensions,
    )
}

fn validate_external_security_dimensions(
    tool: &ContainerCatalogTool,
    evidence: &ToolSecurityEvidence,
) -> Result<()> {
    if evidence.product_support != ProductSupport::Supported
        || evidence.artifact_delivery != artifact_delivery(tool.artifact_delivery)
        || evidence.redistribution_permitted != tool.redistribution_permitted
    {
        anyhow::bail!("security evidence support dimensions differ from catalog for {}", tool.id);
    }
    Ok(())
}

fn admitted_external_artifact_dimensions(
    tool: &ContainerCatalogTool,
    runs: &[&RunEvidenceRecord],
    context: &QualificationContext<'_>,
    evidence: &ToolSecurityEvidence,
) -> Result<BTreeMap<EvidenceDimension, EvidenceDecision>> {
    let artifact_sha256 = evidence
        .artifact_sha256
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("admitted external tool lacks artifact identity"))?;
    let admission_sha256 = evidence
        .admission_evidence_sha256
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("admitted external tool lacks admission identity"))?;
    let execution =
        evaluate_execution_evidence(&execution_observations(runs), &tool.real_test_stage);
    let science =
        evaluate_scientific_acceptance(&scientific_observations(runs), &tool.real_test_stage);
    Ok(BTreeMap::from([
        (
            EvidenceDimension::Catalog,
            evaluate_identity_evidence(
                CATALOG_EVIDENCE_RULE_V1,
                "catalog_sha256",
                &context.chain.catalog_sha256,
            ),
        ),
        (
            EvidenceDimension::Source,
            evaluate_source_identity(
                &tool.source_sha256,
                &tool.pinned_commit,
                &bijux_dna_infra::hash_bytes_sha256(
                    format!("git:{}", tool.pinned_commit).as_bytes(),
                ),
            ),
        ),
        (
            EvidenceDimension::Build,
            evaluate_identity_evidence(
                BUILD_EVIDENCE_RULE_V1,
                "artifact_admission_sha256",
                admission_sha256,
            ),
        ),
        (EvidenceDimension::Runtime, execution.clone()),
        (EvidenceDimension::Stage, execution.clone()),
        (EvidenceDimension::RealData, execution),
        (EvidenceDimension::Science, science),
        (
            EvidenceDimension::Benchmark,
            evaluate_benchmark_evidence(
                &benchmark_observations(&tool.id, context.benchmark_reports),
                &context.chain.repository_commit,
                artifact_sha256,
            ),
        ),
        (EvidenceDimension::Security, security_decision(context, &tool.id, artifact_sha256)),
        (
            EvidenceDimension::Freshness,
            evaluate_identity_evidence(
                FRESHNESS_EVIDENCE_RULE_V1,
                "bound_evidence_set_sha256",
                context.freshness_sha256,
            ),
        ),
        (
            EvidenceDimension::Release,
            missing_publication(
                RELEASE_EVIDENCE_RULE_V1,
                "immutable_publication",
                "evidence_set_coverage",
                "current immutable release publication",
                "no immutable release publication covers this evidence set",
            ),
        ),
    ]))
}

fn redistributable_image_dimensions(
    tool: &ContainerCatalogTool,
    runs: &[&RunEvidenceRecord],
    context: &QualificationContext<'_>,
    image_sha256: &str,
) -> BTreeMap<EvidenceDimension, EvidenceDecision> {
    let stage = evaluate_execution_evidence(&execution_observations(runs), &tool.real_test_stage);
    let science =
        evaluate_scientific_acceptance(&scientific_observations(runs), &tool.real_test_stage);
    BTreeMap::from([
        (
            EvidenceDimension::Catalog,
            evaluate_identity_evidence(
                CATALOG_EVIDENCE_RULE_V1,
                "catalog_sha256",
                &context.chain.catalog_sha256,
            ),
        ),
        (
            EvidenceDimension::Source,
            evaluate_source_identity(
                &tool.source_sha256,
                &tool.pinned_commit,
                &bijux_dna_infra::hash_bytes_sha256(
                    format!("git:{}", tool.pinned_commit).as_bytes(),
                ),
            ),
        ),
        (
            EvidenceDimension::Build,
            evaluate_identity_evidence(BUILD_EVIDENCE_RULE_V1, "image_sha256", image_sha256),
        ),
        (
            EvidenceDimension::Runtime,
            evaluate_identity_evidence(
                RUNTIME_EVIDENCE_RULE_V1,
                "runtime_publication_sha256",
                context.runtime_sha256,
            ),
        ),
        (EvidenceDimension::Stage, stage.clone()),
        (EvidenceDimension::RealData, stage),
        (EvidenceDimension::Science, science),
        (
            EvidenceDimension::Benchmark,
            evaluate_benchmark_evidence(
                &benchmark_observations(&tool.id, context.benchmark_reports),
                &context.chain.repository_commit,
                image_sha256,
            ),
        ),
        (EvidenceDimension::Security, security_decision(context, &tool.id, image_sha256)),
        (
            EvidenceDimension::Freshness,
            evaluate_identity_evidence(
                FRESHNESS_EVIDENCE_RULE_V1,
                "bound_evidence_set_sha256",
                context.freshness_sha256,
            ),
        ),
        (
            EvidenceDimension::Release,
            missing_publication(
                RELEASE_EVIDENCE_RULE_V1,
                "immutable_publication",
                "evidence_set_coverage",
                "current immutable release publication",
                "no immutable release publication covers this evidence set",
            ),
        ),
    ])
}

fn product_support(
    support: bijux_dna_containers::evidence::ContainerProductSupport,
) -> ProductSupport {
    match support {
        bijux_dna_containers::evidence::ContainerProductSupport::Supported => {
            ProductSupport::Supported
        }
        bijux_dna_containers::evidence::ContainerProductSupport::Retired => ProductSupport::Retired,
        bijux_dna_containers::evidence::ContainerProductSupport::Excluded => {
            ProductSupport::Excluded
        }
    }
}

fn artifact_delivery(
    delivery: bijux_dna_containers::evidence::ContainerArtifactDelivery,
) -> ArtifactDelivery {
    match delivery {
        bijux_dna_containers::evidence::ContainerArtifactDelivery::RedistributableImage => {
            ArtifactDelivery::RedistributableImage
        }
        bijux_dna_containers::evidence::ContainerArtifactDelivery::ExternalArtifact => {
            ArtifactDelivery::ExternalArtifact
        }
        bijux_dna_containers::evidence::ContainerArtifactDelivery::PrivateSiteBuild => {
            ArtifactDelivery::PrivateSiteBuild
        }
        bijux_dna_containers::evidence::ContainerArtifactDelivery::SiteModule => {
            ArtifactDelivery::SiteModule
        }
    }
}

fn unavailable_external_artifact_dimensions(
    tool: &ContainerCatalogTool,
    context: &QualificationContext<'_>,
    availability: ArtifactAvailability,
) -> BTreeMap<EvidenceDimension, EvidenceDecision> {
    REQUIRED_EVIDENCE_DIMENSIONS
        .into_iter()
        .map(|dimension| {
            let decision = match dimension {
                EvidenceDimension::Catalog => evaluate_identity_evidence(
                    CATALOG_EVIDENCE_RULE_V1,
                    "catalog_sha256",
                    &context.chain.catalog_sha256,
                ),
                EvidenceDimension::Source => evaluate_source_identity(
                    &tool.source_sha256,
                    &tool.pinned_commit,
                    &bijux_dna_infra::hash_bytes_sha256(
                        format!("git:{}", tool.pinned_commit).as_bytes(),
                    ),
                ),
                EvidenceDimension::Security => context.security_evidence.map_or_else(
                    || {
                        missing_publication(
                            SECURITY_EVIDENCE_RULE_V1,
                            "admitted_publication",
                            "artifact_coverage",
                            "current admitted security publication",
                            "no admitted security publication covers this external artifact",
                        )
                    },
                    |security| security.tool_availability_decision(&tool.id),
                ),
                _ => EvidenceDecision::blocked(
                    SUPPORT_DIMENSIONS_RULE_V2,
                    BTreeMap::from([
                        ("product_support".to_string(), "supported".to_string()),
                        (
                            "artifact_delivery".to_string(),
                            tool.artifact_delivery.as_str().to_string(),
                        ),
                        (
                            "availability".to_string(),
                            artifact_availability_name(availability).to_string(),
                        ),
                    ]),
                    BTreeMap::from([(
                        "availability".to_string(),
                        "admitted checksum-bound artifact".to_string(),
                    )]),
                    "no admitted external artifact is present in the current qualification evidence chain",
                ),
            };
            (dimension, decision)
        })
        .collect()
}

const fn artifact_availability_name(value: ArtifactAvailability) -> &'static str {
    match value {
        ArtifactAvailability::Admitted => "admitted",
        ArtifactAvailability::BlockedExternalAccess => "blocked_external_access",
        ArtifactAvailability::Unavailable => "unavailable",
    }
}

fn security_decision(
    context: &QualificationContext<'_>,
    tool_id: &str,
    image_sha256: &str,
) -> EvidenceDecision {
    context.security_evidence.map_or_else(
        || {
            missing_publication(
                SECURITY_EVIDENCE_RULE_V1,
                "admitted_publication",
                "image_coverage",
                "current admitted security publication",
                "no admitted security publication covers this image",
            )
        },
        |security| security.tool_decision(tool_id, image_sha256),
    )
}

fn benchmark_observations(
    tool_id: &str,
    reports: &[AdmittedBenchmarkStageReport],
) -> Vec<BenchmarkEvidenceObservation> {
    reports
        .iter()
        .filter_map(|admission| {
            let tool = admission.report.tools.iter().find(|tool| tool.tool == tool_id)?;
            Some(BenchmarkEvidenceObservation {
                run_id: admission.report.run_id.clone(),
                repository_commit: tool.repository_commit.clone(),
                image_sha256: tool.image.sha256.clone(),
                evidence_sha256: admission.source_sha256.clone(),
                report_passed: admission.report.verdict == BenchmarkReportVerdict::Passed,
                tool_passed: tool.verdict == BenchmarkReportVerdict::Passed,
            })
        })
        .collect()
}

fn execution_observations(runs: &[&RunEvidenceRecord]) -> Vec<ExecutionEvidenceObservation> {
    runs.iter()
        .map(|run| ExecutionEvidenceObservation {
            run_id: run.run_id.clone(),
            stage: run.stage.clone(),
            evidence_sha256: run.evidence_lock_sha256.clone(),
            status: match run.execution_status {
                AttemptExecutionStatus::Passed => ExecutionEvidenceStatus::Passed,
                AttemptExecutionStatus::Failed => ExecutionEvidenceStatus::Failed,
                AttemptExecutionStatus::Blocked => ExecutionEvidenceStatus::Blocked,
                AttemptExecutionStatus::Incomplete => ExecutionEvidenceStatus::Incomplete,
            },
        })
        .collect()
}

fn scientific_observations(runs: &[&RunEvidenceRecord]) -> Vec<ScientificAcceptanceObservation> {
    runs.iter()
        .map(|run| ScientificAcceptanceObservation {
            run_id: run.run_id.clone(),
            stage: run.stage.clone(),
            evidence_sha256: run.evidence_lock_sha256.clone(),
            execution_status: match run.execution_status {
                AttemptExecutionStatus::Passed => ScienceExecutionStatus::Passed,
                AttemptExecutionStatus::Failed => ScienceExecutionStatus::Failed,
                AttemptExecutionStatus::Blocked => ScienceExecutionStatus::Blocked,
                AttemptExecutionStatus::Incomplete => ScienceExecutionStatus::Incomplete,
            },
            scientific_status: match run.scientific_status {
                ScientificAcceptanceStatus::Passed => ScienceAssessmentStatus::Passed,
                ScientificAcceptanceStatus::Failed => ScienceAssessmentStatus::Failed,
                ScientificAcceptanceStatus::NotAssessed => ScienceAssessmentStatus::NotAssessed,
            },
        })
        .collect()
}

fn repository_source(
    repository_root: &Path,
    relative: &str,
) -> Result<QualificationEvidenceSource> {
    let bytes = fs::read(repository_root.join(relative))
        .with_context(|| format!("read qualification source {relative}"))?;
    Ok(QualificationEvidenceSource {
        uri: format!("repository://{relative}"),
        sha256: bijux_dna_infra::hash_bytes_sha256(&bytes),
    })
}

fn freshness_identity(
    repository_commit: &str,
    sources: &BTreeMap<String, QualificationEvidenceSource>,
) -> Result<String> {
    Ok(bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&(repository_commit, sources))?))
}

fn not_applicable(tool: &ContainerCatalogTool) -> EvidenceDecision {
    EvidenceDecision::not_applicable(
        SUPPORT_DIMENSIONS_RULE_V2,
        BTreeMap::from([
            ("product_support".to_string(), tool.product_support.as_str().to_string()),
            ("artifact_delivery".to_string(), tool.artifact_delivery.as_str().to_string()),
        ]),
        BTreeMap::from([(
            "product_support".to_string(),
            "supported for qualification".to_string(),
        )]),
        "tool is retired or excluded from the product qualification population",
    )
}

fn missing_publication(
    rule_version: &str,
    measured_key: &str,
    threshold_key: &str,
    threshold: &str,
    reason: &str,
) -> EvidenceDecision {
    EvidenceDecision::missing(
        rule_version,
        BTreeMap::from([(measured_key.to_string(), "missing".to_string())]),
        BTreeMap::from([(threshold_key.to_string(), threshold.to_string())]),
        reason,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::freshness_identity;
    use bijux_dna_policies::QualificationEvidenceSource;

    #[test]
    fn freshness_identity_covers_benchmark_sources() -> anyhow::Result<()> {
        let mut sources = BTreeMap::from([(
            "benchmark:run-1001".to_string(),
            QualificationEvidenceSource {
                uri: "repository://benchmarks/reports/run-1001.json".to_string(),
                sha256: "a".repeat(64),
            },
        )]);
        let original = freshness_identity(&"b".repeat(40), &sources)?;
        let source = sources
            .get_mut("benchmark:run-1001")
            .ok_or_else(|| anyhow::anyhow!("benchmark source fixture is absent"))?;
        source.sha256 = "c".repeat(64);
        assert_ne!(original, freshness_identity(&"b".repeat(40), &sources)?);
        Ok(())
    }
}
