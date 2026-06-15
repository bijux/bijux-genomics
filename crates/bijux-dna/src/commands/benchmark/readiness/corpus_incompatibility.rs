use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::bam_corpus_assignment::{collect_bam_corpus_assignment_rows, BamCorpusAssignmentRow};
use super::fastq_corpus_assignment::{
    collect_fastq_corpus_assignment_rows, FastqCorpusAssignmentRow, FastqCorpusAssignmentStatus,
};
use super::stage_tool_assets::{collect_stage_tool_asset_rows, StageToolAssetRow};
use crate::commands::benchmark::local_corpus_fixture::{
    amplicon::{
        validate_amplicon_corpus_fixture_manifest_path, AmpliconCorpusFixtureValidationReport,
        AMPLICON_CORPUS_FIXTURE_SCHEMA_VERSION, DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH,
    },
    bam::{
        validate_bam_corpus_fixture_manifest_path, BamCorpusFixtureValidationReport,
        BAM_CORPUS_FIXTURE_SCHEMA_VERSION, DEFAULT_CORPUS_01_KINSHIP_MINI_MANIFEST_PATH,
    },
    damage::BAM_DAMAGE_FIXTURE_SCHEMA_VERSION,
    edna::{
        validate_edna_corpus_fixture_manifest_path, EdnaCorpusFixtureValidationReport,
        DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH, EDNA_CORPUS_FIXTURE_SCHEMA_VERSION,
    },
    fastq::FASTQ_CORPUS_FIXTURE_SCHEMA_VERSION,
};
use crate::commands::benchmark::local_corpus_stage_compatibility::{
    validate_corpus_stage_compatibility_path, LocalCorpusStageCompatibilityEntryReport,
    LocalCorpusStageCompatibilityValidationReport, LocalCorpusStageValidatedFixture,
    DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_CORPUS_INCOMPATIBILITY_PATH: &str =
    "benchmarks/readiness/corpus-incompatibility.tsv";
const CORPUS_INCOMPATIBILITY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.corpus_incompatibility.v1";
const NOT_APPLICABLE: &str = "not_applicable";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorpusIncompatibilityKind {
    WrongCorpusFamily,
    MissingAmpliconAsvContract,
    MissingTaxonomyDatabaseBundle,
    MissingKinshipPairManifest,
}

impl CorpusIncompatibilityKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::WrongCorpusFamily => "wrong_corpus_family",
            Self::MissingAmpliconAsvContract => "missing_amplicon_asv_contract",
            Self::MissingTaxonomyDatabaseBundle => "missing_taxonomy_database_bundle",
            Self::MissingKinshipPairManifest => "missing_kinship_pair_manifest",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CorpusIncompatibilityRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) benchmark_status: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) incompatible_fixture_id: String,
    pub(crate) incompatible_corpus_family_id: String,
    pub(crate) required_fixture_id: String,
    pub(crate) required_corpus_family_id: String,
    pub(crate) incompatibility_kind: CorpusIncompatibilityKind,
    pub(crate) required_assets: String,
    pub(crate) required_contract: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CorpusIncompatibilityReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) fixture_count: usize,
    pub(crate) benchmark_ready_binding_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) row_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) incompatibility_kind_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<CorpusIncompatibilityRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FixtureDomain {
    Fastq,
    Bam,
}

#[derive(Debug, Clone)]
struct FixtureDescriptor {
    fixture_id: String,
    corpus_family_id: String,
    summary: String,
    domain: FixtureDomain,
}

#[derive(Debug, Clone)]
struct CorpusIncompatibilityContext {
    compatibility_by_stage: BTreeMap<String, LocalCorpusStageCompatibilityEntryReport>,
    fastq_fixtures: Vec<FixtureDescriptor>,
    bam_fixtures: Vec<FixtureDescriptor>,
    assets_by_binding: BTreeMap<(String, String), Vec<StageToolAssetRow>>,
    taxonomy_contract: String,
    amplicon_asv_contract: String,
    kinship_contract: String,
}

#[derive(Debug, Deserialize)]
struct ManifestSchemaProbe {
    schema_version: String,
}

pub(crate) fn run_render_corpus_incompatibility(
    args: &parse::BenchReadinessRenderCorpusIncompatibilityArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_corpus_incompatibility(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_CORPUS_INCOMPATIBILITY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_corpus_incompatibility(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<CorpusIncompatibilityReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let compatibility = validate_corpus_stage_compatibility_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
    )?;
    let context = load_context(repo_root, &compatibility)?;
    let rows = collect_corpus_incompatibility_rows(repo_root, &context)?;
    let benchmark_ready_binding_count = count_benchmark_ready_bindings(&rows);
    let stage_count = rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len();

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_corpus_incompatibility_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut incompatibility_kind_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *incompatibility_kind_counts
            .entry(row.incompatibility_kind.as_str().to_string())
            .or_default() += 1;
    }

    Ok(CorpusIncompatibilityReport {
        schema_version: CORPUS_INCOMPATIBILITY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        fixture_count: context.fastq_fixtures.len() + context.bam_fixtures.len(),
        benchmark_ready_binding_count,
        stage_count,
        tool_count,
        row_count: rows.len(),
        domain_counts,
        incompatibility_kind_counts,
        rows,
    })
}

fn collect_corpus_incompatibility_rows(
    repo_root: &Path,
    context: &CorpusIncompatibilityContext,
) -> Result<Vec<CorpusIncompatibilityRow>> {
    let (_, _, fastq_rows) = collect_fastq_corpus_assignment_rows(repo_root)?;
    let (_, _, bam_rows) = collect_bam_corpus_assignment_rows(repo_root)?;
    let mut rows = Vec::new();

    for row in fastq_rows.iter().filter(|row| {
        row.benchmark_status == "benchmark_ready"
            && row.assignment_status == FastqCorpusAssignmentStatus::Assigned
    }) {
        let required_fixture_id = row.fixture_id.as_deref().ok_or_else(|| {
            anyhow!("FASTQ row `{}` / `{}` is missing fixture_id", row.stage_id, row.tool_id)
        })?;
        let required_corpus_family_id = row.corpus_family_id.as_deref().ok_or_else(|| {
            anyhow!("FASTQ row `{}` / `{}` is missing corpus_family_id", row.stage_id, row.tool_id)
        })?;
        let stage_contract =
            context.compatibility_by_stage.get(&row.stage_id).ok_or_else(|| {
                anyhow!("missing compatibility entry for FASTQ stage `{}`", row.stage_id)
            })?;

        for fixture in context
            .fastq_fixtures
            .iter()
            .filter(|fixture| fixture.fixture_id != required_fixture_id)
        {
            rows.push(build_fastq_incompatibility_row(
                row,
                fixture,
                required_fixture_id,
                required_corpus_family_id,
                stage_contract,
                context,
            )?);
        }
    }

    for row in bam_rows.iter().filter(|row| row.benchmark_status == "benchmark_ready") {
        let stage_contract =
            context.compatibility_by_stage.get(&row.stage_id).ok_or_else(|| {
                anyhow!("missing compatibility entry for BAM stage `{}`", row.stage_id)
            })?;

        for fixture in
            context.bam_fixtures.iter().filter(|fixture| fixture.fixture_id != row.fixture_id)
        {
            rows.push(build_bam_incompatibility_row(row, fixture, stage_contract, context)?);
        }
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.incompatible_fixture_id.cmp(&right.incompatible_fixture_id))
    });
    ensure_unique_rows(&rows)?;
    ensure_required_incompatibility_coverage(&rows)?;
    Ok(rows)
}

fn build_fastq_incompatibility_row(
    row: &FastqCorpusAssignmentRow,
    fixture: &FixtureDescriptor,
    required_fixture_id: &str,
    required_corpus_family_id: &str,
    stage_contract: &LocalCorpusStageCompatibilityEntryReport,
    context: &CorpusIncompatibilityContext,
) -> Result<CorpusIncompatibilityRow> {
    let required_assets = required_assets_for_binding(context, &row.stage_id, &row.tool_id);
    let (incompatibility_kind, required_contract, reason) = match row.stage_id.as_str() {
        "fastq.infer_asvs" => {
            let contract = context.amplicon_asv_contract.clone();
            (
                CorpusIncompatibilityKind::MissingAmpliconAsvContract,
                contract.clone(),
                format!(
                    "row `{}` / `{}` is benchmark_ready and must stay on `{}` because `{}` does not own the governed ASV truth contract ({}); candidate fixture summary: {}",
                    row.stage_id,
                    row.tool_id,
                    required_fixture_id,
                    fixture.fixture_id,
                    contract,
                    fixture.summary
                ),
            )
        }
        "fastq.screen_taxonomy" => {
            let contract = context.taxonomy_contract.clone();
            (
                CorpusIncompatibilityKind::MissingTaxonomyDatabaseBundle,
                contract.clone(),
                format!(
                    "row `{}` / `{}` is benchmark_ready and must stay on `{}` because `{}` does not own the governed taxonomy database bundle ({}); candidate fixture summary: {}",
                    row.stage_id,
                    row.tool_id,
                    required_fixture_id,
                    fixture.fixture_id,
                    contract,
                    fixture.summary
                ),
            )
        }
        _ => {
            let contract = stage_contract.compatibility_note.clone();
            (
                CorpusIncompatibilityKind::WrongCorpusFamily,
                contract.clone(),
                format!(
                    "row `{}` / `{}` is benchmark_ready and must stay on `{}` because `{}` does not own the governed corpus contract: {}; candidate fixture summary: {}",
                    row.stage_id,
                    row.tool_id,
                    required_fixture_id,
                    fixture.fixture_id,
                    contract,
                    fixture.summary
                ),
            )
        }
    };

    Ok(CorpusIncompatibilityRow {
        domain: "fastq".to_string(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        benchmark_status: row.benchmark_status.clone(),
        support_status: row.support_status.clone(),
        adapter_status: row.adapter_status.clone(),
        parser_status: row.parser_status.clone(),
        incompatible_fixture_id: fixture.fixture_id.clone(),
        incompatible_corpus_family_id: fixture.corpus_family_id.clone(),
        required_fixture_id: required_fixture_id.to_string(),
        required_corpus_family_id: required_corpus_family_id.to_string(),
        incompatibility_kind,
        required_assets,
        required_contract,
        reason,
    })
}

fn build_bam_incompatibility_row(
    row: &BamCorpusAssignmentRow,
    fixture: &FixtureDescriptor,
    stage_contract: &LocalCorpusStageCompatibilityEntryReport,
    context: &CorpusIncompatibilityContext,
) -> Result<CorpusIncompatibilityRow> {
    let required_assets = required_assets_for_binding(context, &row.stage_id, &row.tool_id);
    let (incompatibility_kind, required_contract, reason) = if row.stage_id == "bam.kinship" {
        let contract = context.kinship_contract.clone();
        (
            CorpusIncompatibilityKind::MissingKinshipPairManifest,
            contract.clone(),
            format!(
                "row `{}` / `{}` is benchmark_ready and must stay on `{}` because `{}` does not own the governed related-pair case contract ({}); candidate fixture summary: {}",
                row.stage_id,
                row.tool_id,
                row.fixture_id,
                fixture.fixture_id,
                contract,
                fixture.summary
            ),
        )
    } else {
        let contract = stage_contract.compatibility_note.clone();
        (
            CorpusIncompatibilityKind::WrongCorpusFamily,
            contract.clone(),
            format!(
                "row `{}` / `{}` is benchmark_ready and must stay on `{}` because `{}` does not own the governed corpus contract: {}; candidate fixture summary: {}",
                row.stage_id,
                row.tool_id,
                row.fixture_id,
                fixture.fixture_id,
                contract,
                fixture.summary
            ),
        )
    };

    Ok(CorpusIncompatibilityRow {
        domain: "bam".to_string(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        benchmark_status: row.benchmark_status.clone(),
        support_status: row.support_status.clone(),
        adapter_status: row.adapter_status.clone(),
        parser_status: row.parser_status.clone(),
        incompatible_fixture_id: fixture.fixture_id.clone(),
        incompatible_corpus_family_id: fixture.corpus_family_id.clone(),
        required_fixture_id: row.fixture_id.clone(),
        required_corpus_family_id: row.corpus_family_id.clone(),
        incompatibility_kind,
        required_assets,
        required_contract,
        reason,
    })
}

fn load_context(
    repo_root: &Path,
    compatibility: &LocalCorpusStageCompatibilityValidationReport,
) -> Result<CorpusIncompatibilityContext> {
    let fixtures = load_fixture_catalog(repo_root, compatibility)?;
    let assets_by_binding = collect_stage_tool_asset_rows(repo_root)?.into_iter().fold(
        BTreeMap::<(String, String), Vec<StageToolAssetRow>>::new(),
        |mut acc, row| {
            acc.entry((row.stage_id.clone(), row.tool_id.clone())).or_default().push(row);
            acc
        },
    );
    let compatibility_by_stage = compatibility
        .stages
        .iter()
        .cloned()
        .map(|entry| (entry.stage_id.clone(), entry))
        .collect::<BTreeMap<_, _>>();
    let taxonomy_report = validate_edna_corpus_fixture_manifest_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH),
    )?;
    let amplicon_report = validate_amplicon_corpus_fixture_manifest_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH),
    )?;
    let kinship_report = validate_bam_corpus_fixture_manifest_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_01_KINSHIP_MINI_MANIFEST_PATH),
    )?;

    Ok(CorpusIncompatibilityContext {
        compatibility_by_stage,
        fastq_fixtures: fixtures
            .iter()
            .filter(|fixture| fixture.domain == FixtureDomain::Fastq)
            .cloned()
            .collect(),
        bam_fixtures: fixtures
            .iter()
            .filter(|fixture| fixture.domain == FixtureDomain::Bam)
            .cloned()
            .collect(),
        assets_by_binding,
        taxonomy_contract: taxonomy_contract(&taxonomy_report),
        amplicon_asv_contract: amplicon_asv_contract(&amplicon_report),
        kinship_contract: kinship_contract(&kinship_report)?,
    })
}

fn load_fixture_catalog(
    repo_root: &Path,
    compatibility: &LocalCorpusStageCompatibilityValidationReport,
) -> Result<Vec<FixtureDescriptor>> {
    compatibility
        .fixtures
        .iter()
        .map(|fixture| {
            let domain = classify_fixture_domain(repo_root, fixture)?;
            Ok(FixtureDescriptor {
                fixture_id: fixture.fixture_id.clone(),
                corpus_family_id: fixture.corpus_family_id.clone(),
                summary: fixture.summary.clone(),
                domain,
            })
        })
        .collect()
}

fn classify_fixture_domain(
    repo_root: &Path,
    fixture: &LocalCorpusStageValidatedFixture,
) -> Result<FixtureDomain> {
    let manifest_path = repo_root.join(&fixture.fixture_manifest);
    let schema_version = load_manifest_schema_version(&manifest_path)?;
    match schema_version.as_str() {
        FASTQ_CORPUS_FIXTURE_SCHEMA_VERSION
        | EDNA_CORPUS_FIXTURE_SCHEMA_VERSION
        | AMPLICON_CORPUS_FIXTURE_SCHEMA_VERSION => Ok(FixtureDomain::Fastq),
        BAM_CORPUS_FIXTURE_SCHEMA_VERSION | BAM_DAMAGE_FIXTURE_SCHEMA_VERSION => {
            Ok(FixtureDomain::Bam)
        }
        other => Err(anyhow!(
            "unsupported fixture schema `{other}` in compatibility fixture `{}`",
            fixture.fixture_id
        )),
    }
}

fn load_manifest_schema_version(manifest_path: &Path) -> Result<String> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let probe: ManifestSchemaProbe =
        toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))?;
    Ok(probe.schema_version)
}

fn taxonomy_contract(report: &EdnaCorpusFixtureValidationReport) -> String {
    format!("community_id={};expected_taxa_path={}", report.community_id, report.expected_taxa_path)
}

fn amplicon_asv_contract(report: &AmpliconCorpusFixtureValidationReport) -> String {
    format!(
        "assay_id={};marker_id={};expected_asvs_path={}",
        report.assay_id, report.marker_id, report.expected_asvs_path
    )
}

fn kinship_contract(report: &BamCorpusFixtureValidationReport) -> Result<String> {
    let contract = report
        .kinship_contract
        .as_ref()
        .ok_or_else(|| anyhow!("governed kinship fixture must declare a kinship contract"))?;
    let cases = contract
        .cases
        .iter()
        .map(|case| format!("{}:min_overlap_snps={}", case.sample_id, case.min_overlap_snps))
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!(
        "reference_panel={};reference_build={};cases={}",
        contract.reference_panel, contract.reference_build, cases
    ))
}

fn required_assets_for_binding(
    context: &CorpusIncompatibilityContext,
    stage_id: &str,
    tool_id: &str,
) -> String {
    let Some(rows) = context.assets_by_binding.get(&(stage_id.to_string(), tool_id.to_string()))
    else {
        return NOT_APPLICABLE.to_string();
    };
    let mut parts =
        rows.iter().map(|row| format!("{}={}", row.asset_role, row.asset_id)).collect::<Vec<_>>();
    parts.sort();
    parts.join(",")
}

fn count_benchmark_ready_bindings(rows: &[CorpusIncompatibilityRow]) -> usize {
    rows.iter()
        .map(|row| (row.domain.clone(), row.stage_id.clone(), row.tool_id.clone()))
        .collect::<BTreeSet<_>>()
        .len()
}

fn ensure_unique_rows(rows: &[CorpusIncompatibilityRow]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for row in rows {
        let key = (
            row.domain.clone(),
            row.stage_id.clone(),
            row.tool_id.clone(),
            row.incompatible_fixture_id.clone(),
        );
        if !seen.insert(key) {
            return Err(anyhow!(
                "corpus incompatibility report repeats `{}` / `{}` / `{}` against `{}`",
                row.domain,
                row.stage_id,
                row.tool_id,
                row.incompatible_fixture_id
            ));
        }
    }
    Ok(())
}

fn ensure_required_incompatibility_coverage(rows: &[CorpusIncompatibilityRow]) -> Result<()> {
    ensure_row(
        rows,
        "fastq",
        "fastq.infer_asvs",
        "dada2",
        "corpus-01-mini",
        CorpusIncompatibilityKind::MissingAmpliconAsvContract,
        "expected_asvs_path=",
    )?;
    for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
        ensure_row(
            rows,
            "fastq",
            "fastq.screen_taxonomy",
            tool_id,
            "corpus-01-mini",
            CorpusIncompatibilityKind::MissingTaxonomyDatabaseBundle,
            "taxonomy_database_root=taxonomy_reference",
        )?;
    }
    for tool_id in ["angsd", "king"] {
        ensure_row(
            rows,
            "bam",
            "bam.kinship",
            tool_id,
            "corpus-01-bam-mini",
            CorpusIncompatibilityKind::MissingKinshipPairManifest,
            "reference_panel=human_like_relatedness_panel",
        )?;
    }
    Ok(())
}

fn ensure_row(
    rows: &[CorpusIncompatibilityRow],
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    incompatible_fixture_id: &str,
    incompatibility_kind: CorpusIncompatibilityKind,
    required_fragment: &str,
) -> Result<()> {
    let row = rows
        .iter()
        .find(|row| {
            row.domain == domain
                && row.stage_id == stage_id
                && row.tool_id == tool_id
                && row.incompatible_fixture_id == incompatible_fixture_id
        })
        .ok_or_else(|| {
            anyhow!(
                "corpus incompatibility report is missing `{domain}` / `{stage_id}` / `{tool_id}` against `{incompatible_fixture_id}`"
            )
        })?;
    if row.incompatibility_kind != incompatibility_kind
        || (!row.required_contract.contains(required_fragment)
            && !row.required_assets.contains(required_fragment))
    {
        return Err(anyhow!(
            "corpus incompatibility row `{domain}` / `{stage_id}` / `{tool_id}` against `{incompatible_fixture_id}` drifted from its governed blocker contract"
        ));
    }
    Ok(())
}

fn render_corpus_incompatibility_tsv(rows: &[CorpusIncompatibilityRow]) -> String {
    let mut rendered = String::from(
        "domain\tstage_id\ttool_id\tbenchmark_status\tsupport_status\tadapter_status\tparser_status\tincompatible_fixture_id\tincompatible_corpus_family_id\trequired_fixture_id\trequired_corpus_family_id\tincompatibility_kind\trequired_assets\trequired_contract\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.benchmark_status),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.incompatible_fixture_id),
            sanitize_tsv(&row.incompatible_corpus_family_id),
            sanitize_tsv(&row.required_fixture_id),
            sanitize_tsv(&row.required_corpus_family_id),
            sanitize_tsv(row.incompatibility_kind.as_str()),
            sanitize_tsv(&row.required_assets),
            sanitize_tsv(&row.required_contract),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_corpus_incompatibility, CorpusIncompatibilityKind,
        CORPUS_INCOMPATIBILITY_SCHEMA_VERSION, DEFAULT_CORPUS_INCOMPATIBILITY_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn corpus_incompatibility_report_tracks_governed_blocker_rows() {
        let root = repo_root();
        let report = render_corpus_incompatibility(
            &root,
            PathBuf::from(DEFAULT_CORPUS_INCOMPATIBILITY_PATH),
        )
        .expect("render corpus incompatibility");

        assert_eq!(report.schema_version, CORPUS_INCOMPATIBILITY_SCHEMA_VERSION);
        assert!(report.row_count > 0);
        assert!(report.benchmark_ready_binding_count > 0);
        assert!(
            report.rows.iter().any(|row| {
                row.stage_id == "fastq.infer_asvs"
                    && row.tool_id == "dada2"
                    && row.incompatible_fixture_id == "corpus-01-mini"
                    && row.incompatibility_kind
                        == CorpusIncompatibilityKind::MissingAmpliconAsvContract
            }),
            "ASV inference must emit an explicit corpus-01 incompatibility row"
        );
        assert!(
            report.rows.iter().any(|row| {
                row.stage_id == "fastq.screen_taxonomy"
                    && row.tool_id == "kraken2"
                    && row.incompatible_fixture_id == "corpus-03-amplicon-mini"
                    && row.incompatibility_kind
                        == CorpusIncompatibilityKind::MissingTaxonomyDatabaseBundle
                    && row.required_assets.contains("database_artifact_id=taxonomy_db")
            }),
            "taxonomy screening must emit an explicit non-corpus-02 database incompatibility row"
        );
        assert!(
            report.rows.iter().any(|row| {
                row.stage_id == "bam.kinship"
                    && row.tool_id == "king"
                    && row.incompatible_fixture_id == "corpus-01-bam-mini"
                    && row.incompatibility_kind
                        == CorpusIncompatibilityKind::MissingKinshipPairManifest
            }),
            "kinship must emit an explicit incompatible BAM fixture row without the pair manifest"
        );
    }
}
