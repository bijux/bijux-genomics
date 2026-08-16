//! Owner: bijux-dna-bench
//! Scientific benchmark readiness derived from typed contracts and admitted evidence.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{
    validate_benchmark_portfolio, validate_defined_campaign_portfolio_alignment,
    validate_executable_stage_campaign_registry, validate_report_admission,
    validate_scientific_readiness, validate_stage_interface_catalog,
};
use bijux_dna_bench_model::{
    benchmark_scientific_readiness_schema, AdmittedBenchmarkStageReport, BenchmarkCorpusClass,
    BenchmarkCorpusStatus, BenchmarkEvidenceScope, BenchmarkEvidenceTier, BenchmarkPortfolio,
    BenchmarkReportVerdict, BenchmarkScientificReadinessCorpus, BenchmarkScientificReadinessDomain,
    BenchmarkScientificReadinessReport, BenchmarkScientificReadinessStage,
    BenchmarkScientificReadinessStatus, BenchmarkScientificReadinessSummary,
    BenchmarkStageCampaignRegistry, BenchmarkStageInterfaceCatalog, BenchmarkStageInterfaceStatus,
};
use bijux_dna_containers::api::ContainerCatalog;

use super::stage_coverage::runnable_stage_tools;
use super::suite_load::load_benchmark_portfolio_from_repo;
use super::{
    load_stage_campaign_registry, load_stage_interface_catalog_from_repo,
    load_stage_reports_from_dir, BenchmarkCoverageWriteMode,
};

const READINESS_JSON: &str = "benchmarks/reports/scientific-benchmark-readiness.json";
const READINESS_MARKDOWN: &str = "benchmarks/reports/scientific-benchmark-readiness.md";
const READINESS_SCHEMA: &str = "benchmarks/contracts/schemas/scientific-readiness.schema.json";

/// Derive scientific readiness without treating qualification or stale evidence as comparative.
///
/// # Errors
/// Returns an error when typed contracts are invalid or their runnable-stage closure disagrees.
pub fn derive_scientific_readiness(
    portfolio: &BenchmarkPortfolio,
    registry: &BenchmarkStageCampaignRegistry,
    interfaces: &BenchmarkStageInterfaceCatalog,
    catalog: &ContainerCatalog,
    reports: &[AdmittedBenchmarkStageReport],
    required_repository_commit: &str,
) -> Result<BenchmarkScientificReadinessReport> {
    let current_report_sources = BTreeSet::from([required_repository_commit.to_string()]);
    derive_scientific_readiness_with_current_sources(
        portfolio,
        registry,
        interfaces,
        catalog,
        reports,
        required_repository_commit,
        &current_report_sources,
    )
}

#[allow(clippy::too_many_arguments)]
fn derive_scientific_readiness_with_current_sources(
    portfolio: &BenchmarkPortfolio,
    registry: &BenchmarkStageCampaignRegistry,
    interfaces: &BenchmarkStageInterfaceCatalog,
    catalog: &ContainerCatalog,
    reports: &[AdmittedBenchmarkStageReport],
    required_repository_commit: &str,
    current_report_sources: &BTreeSet<String>,
) -> Result<BenchmarkScientificReadinessReport> {
    validate_benchmark_portfolio(portfolio).context("validate benchmark portfolio")?;
    validate_executable_stage_campaign_registry(registry)
        .context("validate benchmark campaigns")?;
    validate_defined_campaign_portfolio_alignment(registry, portfolio)
        .context("align defined campaigns with portfolio")?;
    validate_stage_interface_catalog(interfaces).context("validate benchmark interfaces")?;
    if required_repository_commit.len() != 40
        || !required_repository_commit.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("scientific readiness requires a full repository commit identity");
    }

    let runnable = runnable_stage_tools(catalog)?;
    let reserved = portfolio.stage_runs.keys().cloned().collect::<BTreeSet<_>>();
    let runnable_ids = runnable.keys().cloned().collect::<BTreeSet<_>>();
    if runnable_ids != reserved {
        bail!("scientific readiness portfolio does not close over runnable stages");
    }
    let profiles = portfolio
        .profiles
        .iter()
        .map(|profile| (profile.id.as_str(), profile))
        .collect::<BTreeMap<_, _>>();
    let assignments = portfolio
        .assignments
        .iter()
        .flat_map(|assignment| {
            assignment
                .stage_ids
                .iter()
                .map(move |stage| (stage.as_str(), assignment.profile_id.as_str()))
        })
        .collect::<BTreeMap<_, _>>();
    let corpora = portfolio
        .corpora
        .iter()
        .map(|corpus| (corpus.id.as_str(), corpus))
        .collect::<BTreeMap<_, _>>();
    let campaigns = registry
        .campaigns
        .iter()
        .map(|campaign| (campaign.stage_id.as_str(), campaign))
        .collect::<BTreeMap<_, _>>();
    let interface_ids = interfaces
        .interfaces
        .iter()
        .map(|interface| interface.stage_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut report_by_stage = BTreeMap::new();
    for report in reports {
        validate_report_admission(report).context("validate admitted benchmark report")?;
        if report_by_stage.insert(report.report.stage_id.as_str(), report).is_some() {
            bail!("scientific readiness received duplicate admitted stage reports");
        }
    }

    let mut stages = Vec::with_capacity(runnable.len());
    for (stage_id, tools) in runnable {
        let profile_id = assignments
            .get(stage_id.as_str())
            .context("runnable stage has no scientific profile assignment")?;
        let profile = profiles
            .get(profile_id)
            .context("scientific profile assignment references an unknown profile")?;
        let mut blockers = Vec::new();
        let interface_status = if interface_ids.contains(stage_id.as_str()) {
            BenchmarkStageInterfaceStatus::Closed
        } else {
            blockers.push("no closed stage interface is defined".to_string());
            BenchmarkStageInterfaceStatus::Missing
        };

        let mut available_corpora = BTreeSet::new();
        for role in &profile.required_corpus_roles {
            let candidates = portfolio
                .corpora
                .iter()
                .filter(|corpus| corpus.roles.contains(role))
                .collect::<Vec<_>>();
            let ready = candidates
                .iter()
                .copied()
                .filter(|corpus| comparative_corpus(corpus))
                .collect::<Vec<_>>();
            available_corpora.extend(ready.iter().map(|corpus| corpus.id.as_str()));
            if ready.is_empty() {
                let available = candidates
                    .iter()
                    .map(|corpus| {
                        format!(
                            "{}={}/{}",
                            corpus.id,
                            corpus_status(corpus.status),
                            evidence_scope(corpus.evidence_scope)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                blockers.push(format!("corpus role {role} is not comparative-ready ({available})"));
            }
        }
        let available_comparative_samples = available_corpora
            .iter()
            .filter_map(|id| corpora.get(id).map(|corpus| corpus.sample_count))
            .sum::<u32>();
        if available_comparative_samples < profile.minimum_real_samples {
            blockers.push(format!(
                "available comparative sample count {available_comparative_samples} is below {}",
                profile.minimum_real_samples
            ));
        }

        let reserved_run_id = portfolio
            .stage_runs
            .get(&stage_id)
            .context("runnable stage has no reserved run identity")?
            .clone();
        let campaign = campaigns.get(stage_id.as_str()).copied();
        let (campaign_run_id, campaign_tier) = if let Some(campaign) = campaign {
            let context = CampaignReadinessContext {
                reserved_run_id: &reserved_run_id,
                profile_id,
                profile,
                corpora: &corpora,
                report: report_by_stage.get(stage_id.as_str()).copied(),
                current_report_sources,
            };
            validate_campaign_readiness(campaign, &context, &mut blockers)?;
            (Some(campaign.run_id.clone()), Some(campaign.evidence_tier))
        } else {
            blockers.push("no execution campaign is defined".to_string());
            (None, None)
        };
        let readiness = if blockers.is_empty() {
            BenchmarkScientificReadinessStatus::Ready
        } else {
            BenchmarkScientificReadinessStatus::Blocked
        };
        let domain = stage_id
            .split_once('.')
            .map(|(domain, _)| domain.to_string())
            .context("runnable benchmark stage has no domain")?;
        stages.push(BenchmarkScientificReadinessStage {
            stage_id,
            domain,
            profile_id: (*profile_id).to_string(),
            retained_tools: tools.into_iter().collect(),
            available_comparative_samples,
            campaign_run_id,
            reserved_run_id,
            campaign_tier,
            interface_status,
            readiness,
            blockers,
        });
    }

    let readiness_corpora = portfolio
        .corpora
        .iter()
        .map(|corpus| BenchmarkScientificReadinessCorpus {
            id: corpus.id.clone(),
            status: corpus.status,
            evidence_scope: corpus.evidence_scope,
            class: corpus.class,
            sample_count: corpus.sample_count,
            truth_strategy: corpus.truth_strategy.clone(),
            roles: corpus.roles.clone(),
        })
        .collect::<Vec<_>>();
    let ready_count = stages
        .iter()
        .filter(|stage| stage.readiness == BenchmarkScientificReadinessStatus::Ready)
        .count();
    let domains = ["bam", "fastq", "vcf"]
        .into_iter()
        .map(|domain| {
            let rows = stages.iter().filter(|stage| stage.domain == domain);
            let stage_count = rows.clone().count();
            let ready_count = rows
                .filter(|stage| stage.readiness == BenchmarkScientificReadinessStatus::Ready)
                .count();
            (
                domain.to_string(),
                BenchmarkScientificReadinessDomain {
                    stage_count,
                    ready_count,
                    blocked_count: stage_count - ready_count,
                },
            )
        })
        .collect();
    let summary = BenchmarkScientificReadinessSummary {
        stage_count: stages.len(),
        reserved_run_count: portfolio.stage_runs.len(),
        ready_count,
        blocked_count: stages.len() - ready_count,
        corpus_count: readiness_corpora.len(),
        admitted_comparative_corpus_count: readiness_corpora
            .iter()
            .filter(|corpus| comparative_readiness_corpus(corpus))
            .count(),
        qualification_campaign_count: registry
            .campaigns
            .iter()
            .filter(|campaign| campaign.evidence_tier == BenchmarkEvidenceTier::Qualification)
            .count(),
        comparative_campaign_count: registry
            .campaigns
            .iter()
            .filter(|campaign| campaign.evidence_tier == BenchmarkEvidenceTier::Comparative)
            .count(),
    };
    let report = BenchmarkScientificReadinessReport::v2(
        summary,
        domains,
        portfolio.measurement.clone(),
        readiness_corpora,
        stages,
    );
    validate_scientific_readiness(&report).context("validate derived scientific readiness")?;
    Ok(report)
}

/// Derive and deterministically write or check the checked-in scientific-readiness publication.
///
/// # Errors
/// Returns an error when repository contracts, admitted evidence, or generated publications drift.
pub fn publish_checked_in_scientific_readiness(
    repository_root: &Path,
    mode: BenchmarkCoverageWriteMode,
) -> Result<BenchmarkScientificReadinessReport> {
    let catalog_validation =
        bijux_dna_containers::api::validate_container_catalog(repository_root)?;
    if !catalog_validation.is_valid() {
        bail!("container catalog validation failed: {:?}", catalog_validation.errors);
    }
    let portfolio = load_benchmark_portfolio_from_repo(
        &repository_root.join("benchmarks/contracts/portfolio.toml"),
        repository_root,
    )?;
    let registry = load_stage_campaign_registry(
        &repository_root.join("benchmarks/contracts/stage-campaigns.toml"),
    )?;
    let interfaces = load_stage_interface_catalog_from_repo(
        &repository_root.join("benchmarks/contracts/stage-interfaces.toml"),
        repository_root,
    )?;
    let reports = load_stage_reports_from_dir(
        &repository_root.join("benchmarks/reports"),
        &registry,
        &interfaces,
    )?;
    let catalog = bijux_dna_containers::api::load_container_catalog(repository_root)?;
    let repository_commit = bijux_dna_containers::api::repository_head_commit(repository_root)?;
    let current_report_sources = crate::workflow::stage_report::current_report_sources(
        repository_root,
        &reports,
        &repository_commit,
    )?;
    let report = derive_scientific_readiness_with_current_sources(
        &portfolio,
        &registry,
        &interfaces,
        &catalog,
        &reports,
        &repository_commit,
        &current_report_sources,
    )?;
    let json = serde_json::to_string_pretty(&report)? + "\n";
    let markdown = render_scientific_readiness_markdown(&report);
    let schema = serde_json::to_string_pretty(&benchmark_scientific_readiness_schema())? + "\n";
    for (relative, content) in [
        (READINESS_JSON, json.as_bytes()),
        (READINESS_MARKDOWN, markdown.as_bytes()),
        (READINESS_SCHEMA, schema.as_bytes()),
    ] {
        let path = repository_root.join(relative);
        match mode {
            BenchmarkCoverageWriteMode::Write => {
                bijux_dna_infra::atomic_write_bytes(&path, content)
                    .with_context(|| format!("write scientific readiness {relative}"))?;
            }
            BenchmarkCoverageWriteMode::Check => {
                let current = fs::read(&path)
                    .with_context(|| format!("read scientific readiness {relative}"))?;
                if current != content {
                    bail!("scientific benchmark readiness drifted: {relative}");
                }
            }
        }
    }
    Ok(report)
}

#[must_use]
pub fn render_scientific_readiness_markdown(report: &BenchmarkScientificReadinessReport) -> String {
    let summary = &report.summary;
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: bijux-dna-bench::workflow::scientific_readiness -->".to_string(),
        "# Scientific benchmark readiness".to_string(),
        String::new(),
        "A stage is ready only when real comparative corpora, cohort scale, a closed".to_string(),
        "interface, a comparative campaign, passing admitted execution evidence, and".to_string(),
        "the current repository identity all close over packaged and governed external-artifact tools."
            .to_string(),
        String::new(),
        format!("- Retained FASTQ/BAM/VCF stages: **{}**", summary.stage_count),
        format!("- Scientifically ready stages: **{}**", summary.ready_count),
        format!("- Blocked stages: **{}**", summary.blocked_count),
        format!("- Comparative-ready corpora: **{}**", summary.admitted_comparative_corpus_count),
        format!("- Qualification-only campaigns: **{}**", summary.qualification_campaign_count),
        format!("- Comparative campaigns: **{}**", summary.comparative_campaign_count),
        String::new(),
        "## Domain coverage".to_string(),
        String::new(),
        "| Domain | Stages | Ready | Blocked |".to_string(),
        "|---|---:|---:|---:|".to_string(),
    ];
    for (domain, values) in &report.domains {
        lines.push(format!(
            "| {} | {} | {} | {} |",
            domain.to_uppercase(),
            values.stage_count,
            values.ready_count,
            values.blocked_count
        ));
    }
    lines.extend([
        String::new(),
        "## Corpus portfolio".to_string(),
        String::new(),
        "| Corpus | Class | Status | Scope | Samples | Truth strategy |".to_string(),
        "|---|---|---|---|---:|---|".to_string(),
    ]);
    for corpus in &report.corpora {
        lines.push(format!(
            "| `{}` | {} | {} | {} | {} | `{}` |",
            corpus.id,
            corpus.class.as_str(),
            corpus_status(corpus.status),
            evidence_scope(corpus.evidence_scope),
            corpus.sample_count,
            corpus.truth_strategy
        ));
    }
    lines.extend([
        String::new(),
        "## Stage readiness".to_string(),
        String::new(),
        "| Stage | Tools | Profile | Campaign | Readiness | Principal blockers |".to_string(),
        "|---|---|---|---|---|---|".to_string(),
    ]);
    for stage in &report.stages {
        let tools = stage
            .retained_tools
            .iter()
            .map(|tool| format!("`{tool}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let campaign = stage.campaign_run_id.as_deref().map_or_else(
            || format!("`{}`", stage.reserved_run_id),
            |run_id| {
                format!("`{run_id}` ({})", stage.campaign_tier.map_or("unknown", evidence_tier))
            },
        );
        let blockers = if stage.blockers.is_empty() {
            "none".to_string()
        } else {
            stage.blockers.join("<br>")
        };
        lines.push(format!(
            "| `{}` | {} | `{}` | {} | {} | {} |",
            stage.stage_id,
            tools,
            stage.profile_id,
            campaign,
            readiness_status(stage.readiness),
            blockers
        ));
    }
    lines.join("\n") + "\n"
}

struct CampaignReadinessContext<'a> {
    reserved_run_id: &'a str,
    profile_id: &'a str,
    profile: &'a bijux_dna_bench_model::BenchmarkScientificProfile,
    corpora: &'a BTreeMap<&'a str, &'a bijux_dna_bench_model::BenchmarkCorpusContract>,
    report: Option<&'a AdmittedBenchmarkStageReport>,
    current_report_sources: &'a BTreeSet<String>,
}

fn validate_campaign_readiness(
    campaign: &bijux_dna_bench_model::BenchmarkStageCampaignSpec,
    context: &CampaignReadinessContext<'_>,
    blockers: &mut Vec<String>,
) -> Result<()> {
    if campaign.run_id != context.reserved_run_id {
        blockers.push(format!(
            "campaign run id {} does not match reserved {}",
            campaign.run_id, context.reserved_run_id
        ));
    }
    if campaign.profile_id != context.profile_id {
        blockers.push("campaign does not bind the assigned scientific profile".to_string());
    }
    if campaign.evidence_tier != BenchmarkEvidenceTier::Comparative {
        blockers.push("active campaign is qualification-only".to_string());
        return Ok(());
    }
    let declared = campaign
        .corpus_ids
        .iter()
        .filter_map(|id| context.corpora.get(id.as_str()).copied())
        .collect::<Vec<_>>();
    let missing = campaign
        .corpus_ids
        .iter()
        .filter(|id| !context.corpora.contains_key(id.as_str()))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        blockers.push(format!("campaign declares unknown corpora {missing:?}"));
    }
    let invalid = declared
        .iter()
        .filter(|corpus| !comparative_corpus(corpus))
        .map(|corpus| corpus.id.as_str())
        .collect::<Vec<_>>();
    if !invalid.is_empty() {
        blockers.push(format!("campaign corpora are not comparative-ready: {invalid:?}"));
    }
    let declared_roles = declared
        .iter()
        .flat_map(|corpus| corpus.roles.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();
    let missing_roles = context
        .profile
        .required_corpus_roles
        .iter()
        .filter(|role| !declared_roles.contains(role.as_str()))
        .collect::<Vec<_>>();
    if !missing_roles.is_empty() {
        blockers.push(format!("campaign corpus portfolio omits roles {missing_roles:?}"));
    }
    let declared_samples = declared
        .iter()
        .filter(|corpus| comparative_corpus(corpus))
        .map(|corpus| corpus.sample_count)
        .sum::<u32>();
    if declared_samples < context.profile.minimum_real_samples {
        blockers.push(format!(
            "campaign real sample count {declared_samples} is below {}",
            context.profile.minimum_real_samples
        ));
    }
    match context.report {
        None => blockers.push("no admitted comparative execution evidence".to_string()),
        Some(report)
            if report
                .report
                .tools
                .iter()
                .any(|tool| !context.current_report_sources.contains(&tool.repository_commit)) =>
        {
            blockers.push("admitted comparative execution evidence is stale".to_string())
        }
        Some(report) if report.report.verdict != BenchmarkReportVerdict::Passed => {
            blockers.push("admitted comparative execution evidence failed".to_string());
        }
        Some(_) => {}
    }
    Ok(())
}

fn comparative_corpus(corpus: &bijux_dna_bench_model::BenchmarkCorpusContract) -> bool {
    corpus.status == BenchmarkCorpusStatus::Admitted
        && corpus.evidence_scope == BenchmarkEvidenceScope::Comparative
        && corpus.class == BenchmarkCorpusClass::ProductionReal
}

fn comparative_readiness_corpus(corpus: &BenchmarkScientificReadinessCorpus) -> bool {
    corpus.status == BenchmarkCorpusStatus::Admitted
        && corpus.evidence_scope == BenchmarkEvidenceScope::Comparative
        && corpus.class == BenchmarkCorpusClass::ProductionReal
}

fn corpus_status(status: BenchmarkCorpusStatus) -> &'static str {
    match status {
        BenchmarkCorpusStatus::Missing => "missing",
        BenchmarkCorpusStatus::SourceDefined => "source_defined",
        BenchmarkCorpusStatus::Admitted => "admitted",
    }
}

fn evidence_scope(scope: BenchmarkEvidenceScope) -> &'static str {
    match scope {
        BenchmarkEvidenceScope::Qualification => "qualification",
        BenchmarkEvidenceScope::Comparative => "comparative",
    }
}

fn evidence_tier(tier: BenchmarkEvidenceTier) -> &'static str {
    match tier {
        BenchmarkEvidenceTier::Qualification => "qualification",
        BenchmarkEvidenceTier::Comparative => "comparative",
    }
}

fn readiness_status(status: BenchmarkScientificReadinessStatus) -> &'static str {
    match status {
        BenchmarkScientificReadinessStatus::Ready => "ready",
        BenchmarkScientificReadinessStatus::Blocked => "blocked",
    }
}

#[cfg(test)]
mod tests {
    use super::derive_scientific_readiness;

    #[test]
    fn repository_portfolio_is_blocked_without_admitted_execution_evidence() -> anyhow::Result<()> {
        let root = crate::repo::resolve_repo_root()?;
        let portfolio = crate::workflow::load_benchmark_portfolio(
            &root.join("benchmarks/contracts/portfolio.toml"),
        )?;
        let registry = crate::workflow::load_stage_campaign_registry(
            &root.join("benchmarks/contracts/stage-campaigns.toml"),
        )?;
        let interfaces = crate::workflow::load_stage_interface_catalog_from_repo(
            &root.join("benchmarks/contracts/stage-interfaces.toml"),
            &root,
        )?;
        let catalog = bijux_dna_containers::api::load_container_catalog(&root)?;

        let report = derive_scientific_readiness(
            &portfolio,
            &registry,
            &interfaces,
            &catalog,
            &[],
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )?;

        assert_eq!(report.summary.stage_count, 71);
        assert_eq!(report.summary.ready_count, 0);
        let read_lengths = report
            .stages
            .iter()
            .find(|stage| stage.stage_id == "fastq.profile_read_lengths")
            .ok_or_else(|| anyhow::anyhow!("read-length stage is absent"))?;
        assert!(read_lengths
            .blockers
            .iter()
            .any(|blocker| blocker == "no admitted comparative execution evidence"));
        Ok(())
    }
}
