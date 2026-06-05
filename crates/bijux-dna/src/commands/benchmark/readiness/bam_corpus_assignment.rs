use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_bam::benchmark_corpus_assignment_for_stage_tool;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::bam_command_adapter_coverage::{
    collect_bam_command_adapter_coverage_rows, BamBenchmarkStatus,
};
use crate::commands::benchmark::local_corpus_fixture::{
    bam::{
        validate_bam_corpus_fixture_manifest_path, BamCorpusFixtureValidationReport,
        DEFAULT_CORPUS_01_ADNA_BAM_MINI_MANIFEST_PATH,
    },
    damage::{
        validate_bam_damage_fixture_manifest_path, BamDamageFixtureValidationReport,
        DEFAULT_CORPUS_01_ADNA_DAMAGE_MANIFEST_PATH,
    },
};
use crate::commands::benchmark::local_corpus_stage_compatibility::{
    validate_corpus_stage_compatibility_path, LocalCorpusStageCompatibilityEntryReport,
    DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
};
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_CORPUS_ASSIGNMENT_PATH: &str =
    "target/bench-readiness/bam-corpus-assignment.tsv";
const BAM_CORPUS_ASSIGNMENT_SCHEMA_VERSION: &str = "bijux.bench.readiness.bam_corpus_assignment.v2";
const DEFAULT_LOCAL_AUTHENTICITY_CONFIG_PATH: &str = "configs/bench/local/bam-authenticity.toml";
const DEFAULT_LOCAL_CONTAMINATION_CONFIG_PATH: &str = "configs/bench/local/bam-contamination.toml";
const DEFAULT_LOCAL_DAMAGE_CONFIG_PATH: &str = "configs/bench/local/bam-damage.toml";
const DEFAULT_LOCAL_HAPLOGROUPS_CONFIG_PATH: &str = "configs/bench/local/bam-haplogroups.toml";
const DEFAULT_LOCAL_SEX_CONFIG_PATH: &str = "configs/bench/local/bam-sex.toml";
const NOT_APPLICABLE: &str = "not_applicable";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BamCorpusAssignmentRow {
    pub(crate) tool_id: String,
    pub(crate) stage_id: String,
    pub(crate) benchmark_status: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_family_id: String,
    pub(crate) fixture_id: String,
    pub(crate) sample_id: String,
    pub(crate) damage_expectation: String,
    pub(crate) coverage_limits: String,
    pub(crate) required_assets: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone)]
struct BamCorpusAssignmentContext {
    adna_stage_evidence: BTreeMap<String, BamAdnaStageEvidence>,
}

#[derive(Debug, Clone)]
struct BamAdnaStageEvidence {
    sample_id: String,
    damage_expectation: String,
    coverage_limits: String,
    required_assets: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamCorpusAssignmentReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) corpus_family_counts: BTreeMap<String, usize>,
    pub(crate) fixture_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<BamCorpusAssignmentRow>,
}

pub(crate) fn run_render_bam_corpus_assignment(
    args: &parse::BenchReadinessRenderBamCorpusAssignmentArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_corpus_assignment(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_CORPUS_ASSIGNMENT_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_corpus_assignment(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamCorpusAssignmentReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let stage_count = load_local_stage_inventory(repo_root, BenchLocalDomain::Bam)?.stage_count;
    let (_, tool_count, rows) = collect_bam_corpus_assignment_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_bam_corpus_assignment_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let mut corpus_family_counts = BTreeMap::<String, usize>::new();
    let mut fixture_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *corpus_family_counts.entry(row.corpus_family_id.clone()).or_default() += 1;
        *fixture_counts.entry(row.fixture_id.clone()).or_default() += 1;
    }

    Ok(BamCorpusAssignmentReport {
        schema_version: BAM_CORPUS_ASSIGNMENT_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        benchmark_ready_row_count,
        corpus_family_counts,
        fixture_counts,
        rows,
    })
}

fn collect_bam_corpus_assignment_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<BamCorpusAssignmentRow>)> {
    let compatibility_by_stage = load_bam_stage_compatibility(repo_root)?;
    let context = load_bam_corpus_assignment_context(repo_root)?;
    let (stage_count, tool_count, coverage_rows) =
        collect_bam_command_adapter_coverage_rows(repo_root)?;
    let mut rows = Vec::with_capacity(coverage_rows.len());

    for row in coverage_rows {
        let stage_id = StageId::new(row.stage_id.clone());
        let tool_id = ToolId::new(row.tool_id.clone());
        let domain_assignment = benchmark_corpus_assignment_for_stage_tool(&stage_id, &tool_id)
            .ok_or_else(|| {
                anyhow!("missing BAM corpus assignment for `{}` / `{}`", row.stage_id, row.tool_id)
            })?;
        let stage_compatibility = compatibility_by_stage.get(&row.stage_id).ok_or_else(|| {
            anyhow!("missing BAM corpus compatibility for stage `{}`", row.stage_id)
        })?;
        let compatibility_family =
            stage_compatibility.corpus_family_id.as_deref().ok_or_else(|| {
                anyhow!(
                    "BAM stage `{}` is missing corpus_family_id in local compatibility",
                    row.stage_id
                )
            })?;
        let fixture_id = stage_compatibility.fixture_id.as_deref().ok_or_else(|| {
            anyhow!("BAM stage `{}` is missing fixture_id in local compatibility", row.stage_id)
        })?;
        let assigned_family = domain_assignment.assigned_family();
        if compatibility_family != assigned_family.as_str() {
            return Err(anyhow!(
                "BAM stage `{}` assigns `{}` in the domain contract but `{}` in local compatibility",
                row.stage_id,
                assigned_family.as_str(),
                compatibility_family
            ));
        }
        let evidence = context.evidence_for_stage(&row.stage_id);

        rows.push(BamCorpusAssignmentRow {
            tool_id: row.tool_id,
            stage_id: row.stage_id,
            benchmark_status: benchmark_status_label(row.benchmark_status).to_string(),
            support_status: row.support_status,
            adapter_status: row.adapter_status,
            parser_status: row.parser_status,
            corpus_family_id: compatibility_family.to_string(),
            fixture_id: fixture_id.to_string(),
            sample_id: evidence.sample_id,
            damage_expectation: evidence.damage_expectation,
            coverage_limits: evidence.coverage_limits,
            required_assets: evidence.required_assets,
            reason: format!(
                "row `{}` / `{}` is {} and maps to `{}` via fixture `{}`: {}",
                stage_id.as_str(),
                tool_id.as_str(),
                benchmark_status_label(row.benchmark_status),
                compatibility_family,
                fixture_id,
                domain_assignment.rationale()
            ),
        });
    }

    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then_with(|| left.stage_id.cmp(&right.stage_id))
    });
    ensure_row_completeness(&rows)?;
    ensure_bam_adna_assignment_coverage(&rows)?;
    Ok((stage_count, tool_count, rows))
}

impl BamCorpusAssignmentContext {
    fn evidence_for_stage(&self, stage_id: &str) -> BamAdnaStageEvidence {
        self.adna_stage_evidence
            .get(stage_id)
            .cloned()
            .unwrap_or_else(BamAdnaStageEvidence::not_applicable)
    }
}

impl BamAdnaStageEvidence {
    fn not_applicable() -> Self {
        Self {
            sample_id: NOT_APPLICABLE.to_string(),
            damage_expectation: NOT_APPLICABLE.to_string(),
            coverage_limits: NOT_APPLICABLE.to_string(),
            required_assets: NOT_APPLICABLE.to_string(),
        }
    }
}

fn load_bam_corpus_assignment_context(repo_root: &Path) -> Result<BamCorpusAssignmentContext> {
    let damage_fixture = validate_bam_damage_fixture_manifest_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_01_ADNA_DAMAGE_MANIFEST_PATH),
    )?;
    let adna_bam_fixture = validate_bam_corpus_fixture_manifest_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_01_ADNA_BAM_MINI_MANIFEST_PATH),
    )?;

    let authenticity = load_governed_toml::<LocalAuthenticityConfig>(
        &repo_root.join(DEFAULT_LOCAL_AUTHENTICITY_CONFIG_PATH),
        "BAM authenticity config",
    )?;
    let authenticity_case = authenticity
        .cases
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("BAM authenticity config must declare at least one case"))?;
    ensure_stage_sample_matches_damage_fixture(
        "bam.authenticity",
        &authenticity_case.sample_id,
        &damage_fixture,
    )?;

    let damage = load_governed_toml::<LocalDamageConfig>(
        &repo_root.join(DEFAULT_LOCAL_DAMAGE_CONFIG_PATH),
        "BAM damage config",
    )?;
    let damage_case = damage
        .cases
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("BAM damage config must declare at least one case"))?;
    ensure_stage_sample_matches_damage_fixture(
        "bam.damage",
        &damage_case.sample_id,
        &damage_fixture,
    )?;

    let contamination = load_governed_toml::<LocalContaminationConfig>(
        &repo_root.join(DEFAULT_LOCAL_CONTAMINATION_CONFIG_PATH),
        "BAM contamination config",
    )?;
    ensure_stage_sample_matches_bam_fixture(
        "bam.contamination",
        &contamination.sample_id,
        &adna_bam_fixture,
    )?;

    let sex = load_governed_toml::<LocalSexConfig>(
        &repo_root.join(DEFAULT_LOCAL_SEX_CONFIG_PATH),
        "BAM sex config",
    )?;
    let sex_case = sex
        .cases
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("BAM sex config must declare at least one case"))?;
    ensure_stage_sample_matches_bam_fixture("bam.sex", &sex_case.sample_id, &adna_bam_fixture)?;

    let haplogroups = load_governed_toml::<LocalHaplogroupsConfig>(
        &repo_root.join(DEFAULT_LOCAL_HAPLOGROUPS_CONFIG_PATH),
        "BAM haplogroups config",
    )?;
    ensure_stage_sample_matches_bam_fixture(
        "bam.haplogroups",
        &haplogroups.sample_id,
        &adna_bam_fixture,
    )?;

    let damage_expectation = format_key_value_contract([
        ("ct5p", format_decimal(damage_fixture.expected_damage.terminal_c_to_t_5p)),
        ("ga3p", format_decimal(damage_fixture.expected_damage.terminal_g_to_a_3p)),
        ("signal", damage_fixture.expected_damage.damage_signal.clone()),
        ("short_frag", format_decimal(damage_fixture.expected_damage.short_fragment_fraction)),
        (
            "strict_profile_upgraded",
            damage_fixture.expected_damage.strict_profile_upgraded.to_string(),
        ),
        ("terminal", damage_fixture.expected_terminal_pattern_class.clone()),
        ("udg", damage_fixture.udg_model.clone()),
    ]);
    let damage_assets = format_key_value_contract([
        ("expected_damage", file_name_string(&damage_fixture.expected_damage_path)?),
        ("reference_fasta", file_name_string(&damage_fixture.reference_fasta)?),
    ]);
    let inherited_adna_profile = format_key_value_contract([
        ("signal", adna_bam_fixture.damage_signal.clone().unwrap_or_default()),
        ("terminal", adna_bam_fixture.expected_terminal_pattern_class.clone().unwrap_or_default()),
        ("udg", adna_bam_fixture.udg_model.clone().unwrap_or_default()),
    ]);

    let mut adna_stage_evidence = BTreeMap::new();
    adna_stage_evidence.insert(
        "bam.authenticity".to_string(),
        BamAdnaStageEvidence {
            sample_id: authenticity_case.sample_id,
            damage_expectation: damage_expectation.clone(),
            coverage_limits: format_key_value_contract([
                ("complexity_min_reads", authenticity_case.complexity_min_reads.to_string()),
                (
                    "coverage_depth_thresholds",
                    format_numeric_list(&authenticity_case.coverage_depth_thresholds),
                ),
            ]),
            required_assets: damage_assets.clone(),
        },
    );
    adna_stage_evidence.insert(
        "bam.damage".to_string(),
        BamAdnaStageEvidence {
            sample_id: damage_case.sample_id,
            damage_expectation: damage_expectation.clone(),
            coverage_limits: NOT_APPLICABLE.to_string(),
            required_assets: damage_assets,
        },
    );
    adna_stage_evidence.insert(
        "bam.contamination".to_string(),
        BamAdnaStageEvidence {
            sample_id: contamination.sample_id,
            damage_expectation: inherited_adna_profile.clone(),
            coverage_limits: format_key_value_contract([(
                "minimum_mean_coverage",
                format_decimal(contamination.minimum_mean_coverage),
            )]),
            required_assets: format_key_value_contract([
                ("reference_fasta", asset_id_from_path(&contamination.reference_fasta)?),
                (
                    "reference_panel",
                    asset_id_from_path(contamination.reference_panels.first().ok_or_else(
                        || {
                            anyhow!(
                                "BAM contamination config must declare at least one reference panel"
                            )
                        },
                    )?)?,
                ),
            ]),
        },
    );
    adna_stage_evidence.insert(
        "bam.sex".to_string(),
        BamAdnaStageEvidence {
            sample_id: sex_case.sample_id,
            damage_expectation: inherited_adna_profile.clone(),
            coverage_limits: format_key_value_contract([
                (
                    "expected_autosomal_coverage",
                    format_decimal(sex_case.expected_autosomal_coverage),
                ),
                ("expected_x_coverage", format_decimal(sex_case.expected_x_coverage)),
                ("expected_y_coverage", format_decimal(sex_case.expected_y_coverage)),
                ("minimum_y_sites", sex_case.minimum_y_sites.to_string()),
            ]),
            required_assets: format_key_value_contract([(
                "reference_fasta",
                asset_id_from_path(&sex_case.reference)?,
            )]),
        },
    );
    adna_stage_evidence.insert(
        "bam.haplogroups".to_string(),
        BamAdnaStageEvidence {
            sample_id: haplogroups.sample_id,
            damage_expectation: inherited_adna_profile,
            coverage_limits: format_key_value_contract([(
                "min_coverage",
                format_decimal(haplogroups.min_coverage),
            )]),
            required_assets: format_key_value_contract([
                ("reference_fasta", asset_id_from_path(&haplogroups.reference_fasta)?),
                ("reference_panel", haplogroups.reference_panel_id),
            ]),
        },
    );

    Ok(BamCorpusAssignmentContext { adna_stage_evidence })
}

fn load_bam_stage_compatibility(
    repo_root: &Path,
) -> Result<BTreeMap<String, LocalCorpusStageCompatibilityEntryReport>> {
    let matrix_path = repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH);
    let report = validate_corpus_stage_compatibility_path(repo_root, &matrix_path)?;
    report
        .stages
        .into_iter()
        .filter(|stage| stage.stage_id.starts_with("bam."))
        .map(|stage| Ok((stage.stage_id.clone(), stage)))
        .collect()
}

fn ensure_row_completeness(rows: &[BamCorpusAssignmentRow]) -> Result<()> {
    let mut seen = BTreeSet::<(&str, &str)>::new();
    for row in rows {
        if !seen.insert((&row.stage_id, &row.tool_id)) {
            return Err(anyhow!(
                "BAM corpus assignment report repeats row `{}` / `{}`",
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}

fn benchmark_status_label(status: BamBenchmarkStatus) -> &'static str {
    match status {
        BamBenchmarkStatus::BenchmarkReady => "benchmark_ready",
        BamBenchmarkStatus::NotBenchmarkReady => "not_benchmark_ready",
    }
}

fn render_bam_corpus_assignment_tsv(rows: &[BamCorpusAssignmentRow]) -> String {
    let mut rendered = String::from(
        "tool_id\tstage_id\tbenchmark_status\tsupport_status\tadapter_status\tparser_status\tcorpus_family_id\tfixture_id\tsample_id\tdamage_expectation\tcoverage_limits\trequired_assets\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.benchmark_status),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.corpus_family_id),
            sanitize_tsv(&row.fixture_id),
            sanitize_tsv(&row.sample_id),
            sanitize_tsv(&row.damage_expectation),
            sanitize_tsv(&row.coverage_limits),
            sanitize_tsv(&row.required_assets),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn load_governed_toml<T>(path: &Path, label: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {label} {}", path.display()))
}

fn ensure_stage_sample_matches_damage_fixture(
    stage_id: &str,
    sample_id: &str,
    fixture: &BamDamageFixtureValidationReport,
) -> Result<()> {
    if sample_id != fixture.sample_id {
        return Err(anyhow!(
            "BAM stage `{stage_id}` sample `{sample_id}` must match damage fixture sample `{}`",
            fixture.sample_id
        ));
    }
    Ok(())
}

fn ensure_stage_sample_matches_bam_fixture(
    stage_id: &str,
    sample_id: &str,
    fixture: &BamCorpusFixtureValidationReport,
) -> Result<()> {
    if fixture.samples.iter().any(|sample| sample.sample_id == sample_id) {
        return Ok(());
    }
    Err(anyhow!(
        "BAM stage `{stage_id}` sample `{sample_id}` is missing from BAM fixture `{}`",
        fixture.corpus_id
    ))
}

fn format_key_value_contract<I, K, V>(entries: I) -> String
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    entries
        .into_iter()
        .map(|(key, value)| (key.into(), value.into()))
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(";")
}

fn format_numeric_list(values: &[u64]) -> String {
    values.iter().map(u64::to_string).collect::<Vec<_>>().join(",")
}

fn format_decimal(value: f64) -> String {
    let rendered = format!("{value:.6}");
    rendered.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn file_name_string(path: &str) -> Result<String> {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("path `{path}` is missing a terminal file name"))
}

fn asset_id_from_path(path: &Path) -> Result<String> {
    path.file_stem().and_then(|value| value.to_str()).map(ToOwned::to_owned).ok_or_else(|| {
        anyhow!("path `{}` is missing a stable asset identifier stem", path.display())
    })
}

fn ensure_bam_adna_assignment_coverage(rows: &[BamCorpusAssignmentRow]) -> Result<()> {
    let expected_rows = [
        BamAdnaAssignmentExpectation {
            stage_id: "bam.authenticity",
            fixture_id: "corpus-01-adna-damage-mini",
            sample_id: "adna_damage_non_udg",
            damage_expectation:
                "ct5p=0.18;ga3p=0.11;short_frag=1;signal=moderate;strict_profile_upgraded=false;terminal=ct5p_dominant;udg=non_udg",
            coverage_limits: "complexity_min_reads=3;coverage_depth_thresholds=1,5,10",
            required_assets: "expected_damage=expected_damage.json;reference_fasta=adna_damage_reference.fasta",
        },
        BamAdnaAssignmentExpectation {
            stage_id: "bam.damage",
            fixture_id: "corpus-01-adna-damage-mini",
            sample_id: "adna_damage_non_udg",
            damage_expectation:
                "ct5p=0.18;ga3p=0.11;short_frag=1;signal=moderate;strict_profile_upgraded=false;terminal=ct5p_dominant;udg=non_udg",
            coverage_limits: NOT_APPLICABLE,
            required_assets: "expected_damage=expected_damage.json;reference_fasta=adna_damage_reference.fasta",
        },
        BamAdnaAssignmentExpectation {
            stage_id: "bam.contamination",
            fixture_id: "corpus-01-adna-bam-mini",
            sample_id: "adna_contamination_panel_screen",
            damage_expectation: "signal=moderate;terminal=ct5p_dominant;udg=non_udg",
            coverage_limits: "minimum_mean_coverage=0.5",
            required_assets:
                "reference_fasta=adna_bam_reference;reference_panel=adna_contamination_panel",
        },
        BamAdnaAssignmentExpectation {
            stage_id: "bam.sex",
            fixture_id: "corpus-01-adna-bam-mini",
            sample_id: "adna_xy_autosome_coverage",
            damage_expectation: "signal=moderate;terminal=ct5p_dominant;udg=non_udg",
            coverage_limits:
                "expected_autosomal_coverage=1;expected_x_coverage=0.5;expected_y_coverage=0.5;minimum_y_sites=5",
            required_assets: "reference_fasta=adna_bam_reference",
        },
        BamAdnaAssignmentExpectation {
            stage_id: "bam.haplogroups",
            fixture_id: "corpus-01-adna-bam-mini",
            sample_id: "adna_y_haplogroup_panel",
            damage_expectation: "signal=moderate;terminal=ct5p_dominant;udg=non_udg",
            coverage_limits: "min_coverage=2",
            required_assets: "reference_fasta=adna_bam_reference;reference_panel=adna-y-hg38-mini",
        },
    ];

    for expectation in expected_rows {
        let stage_rows =
            rows.iter().filter(|row| row.stage_id == expectation.stage_id).collect::<Vec<_>>();
        if stage_rows.is_empty() {
            return Err(anyhow!(
                "BAM aDNA corpus assignment coverage is missing governed rows for stage `{}`",
                expectation.stage_id
            ));
        }
        for row in stage_rows {
            if row.fixture_id != expectation.fixture_id
                || row.sample_id != expectation.sample_id
                || row.damage_expectation != expectation.damage_expectation
                || row.coverage_limits != expectation.coverage_limits
                || row.required_assets != expectation.required_assets
            {
                return Err(anyhow!(
                    "BAM aDNA corpus assignment row `{}` / `{}` drifted away from the governed stage evidence contract",
                    row.stage_id,
                    row.tool_id
                ));
            }
        }
    }

    Ok(())
}

struct BamAdnaAssignmentExpectation {
    stage_id: &'static str,
    fixture_id: &'static str,
    sample_id: &'static str,
    damage_expectation: &'static str,
    coverage_limits: &'static str,
    required_assets: &'static str,
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

#[derive(Debug, Deserialize)]
struct LocalAuthenticityConfig {
    cases: Vec<LocalAuthenticityCase>,
}

#[derive(Debug, Deserialize)]
struct LocalAuthenticityCase {
    sample_id: String,
    complexity_min_reads: u64,
    coverage_depth_thresholds: Vec<u64>,
}

#[derive(Debug, Deserialize)]
struct LocalContaminationConfig {
    sample_id: String,
    minimum_mean_coverage: f64,
    reference_fasta: PathBuf,
    reference_panels: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalDamageConfig {
    cases: Vec<LocalDamageCase>,
}

#[derive(Debug, Deserialize)]
struct LocalDamageCase {
    sample_id: String,
}

#[derive(Debug, Deserialize)]
struct LocalHaplogroupsConfig {
    sample_id: String,
    min_coverage: f64,
    reference_fasta: PathBuf,
    reference_panel_id: String,
}

#[derive(Debug, Deserialize)]
struct LocalSexConfig {
    cases: Vec<LocalSexCase>,
}

#[derive(Debug, Deserialize)]
struct LocalSexCase {
    sample_id: String,
    minimum_y_sites: u64,
    expected_x_coverage: f64,
    expected_y_coverage: f64,
    expected_autosomal_coverage: f64,
    reference: PathBuf,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{render_bam_corpus_assignment, DEFAULT_BAM_CORPUS_ASSIGNMENT_PATH};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn bam_corpus_assignment_reports_precise_bam_fixture_routing() {
        let report = render_bam_corpus_assignment(
            &repo_root(),
            PathBuf::from(DEFAULT_BAM_CORPUS_ASSIGNMENT_PATH),
        )
        .expect("render BAM corpus assignment");

        assert_eq!(report.schema_version, "bijux.bench.readiness.bam_corpus_assignment.v2");
        assert_eq!(report.stage_count, 24);
        assert!(report.row_count > 0);
        assert_eq!(report.corpus_family_counts.get("corpus-01"), Some(&2));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.authenticity"
                && row.tool_id == "authenticct"
                && row.corpus_family_id == "corpus-01-adna-bam"
                && row.fixture_id == "corpus-01-adna-damage-mini"
                && row.sample_id == "adna_damage_non_udg"
                && row.damage_expectation
                    == "ct5p=0.18;ga3p=0.11;short_frag=1;signal=moderate;strict_profile_upgraded=false;terminal=ct5p_dominant;udg=non_udg"
                && row.coverage_limits == "complexity_min_reads=3;coverage_depth_thresholds=1,5,10"
                && row.required_assets
                    == "expected_damage=expected_damage.json;reference_fasta=adna_damage_reference.fasta"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.contamination"
                && row.tool_id == "verifybamid2"
                && row.corpus_family_id == "corpus-01-adna-bam"
                && row.fixture_id == "corpus-01-adna-bam-mini"
                && row.sample_id == "adna_contamination_panel_screen"
                && row.damage_expectation == "signal=moderate;terminal=ct5p_dominant;udg=non_udg"
                && row.coverage_limits == "minimum_mean_coverage=0.5"
                && row.required_assets
                    == "reference_fasta=adna_bam_reference;reference_panel=adna_contamination_panel"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.sex"
                && row.tool_id == "rxy"
                && row.corpus_family_id == "corpus-01-adna-bam"
                && row.fixture_id == "corpus-01-adna-bam-mini"
                && row.sample_id == "adna_xy_autosome_coverage"
                && row.damage_expectation == "signal=moderate;terminal=ct5p_dominant;udg=non_udg"
                && row.coverage_limits
                    == "expected_autosomal_coverage=1;expected_x_coverage=0.5;expected_y_coverage=0.5;minimum_y_sites=5"
                && row.required_assets == "reference_fasta=adna_bam_reference"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.haplogroups"
                && row.tool_id == "yleaf"
                && row.corpus_family_id == "corpus-01-adna-bam"
                && row.fixture_id == "corpus-01-adna-bam-mini"
                && row.sample_id == "adna_y_haplogroup_panel"
                && row.damage_expectation == "signal=moderate;terminal=ct5p_dominant;udg=non_udg"
                && row.coverage_limits == "min_coverage=2"
                && row.required_assets
                    == "reference_fasta=adna_bam_reference;reference_panel=adna-y-hg38-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.genotyping"
                && row.tool_id == "angsd"
                && row.corpus_family_id == "corpus-01-genotyping"
                && row.fixture_id == "corpus-01-genotyping-mini"
                && row.sample_id == "not_applicable"
                && row.damage_expectation == "not_applicable"
                && row.coverage_limits == "not_applicable"
                && row.required_assets == "not_applicable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.kinship"
                && row.tool_id == "king"
                && row.corpus_family_id == "corpus-01-kinship"
                && row.fixture_id == "corpus-01-kinship-mini"
                && row.sample_id == "not_applicable"
                && row.damage_expectation == "not_applicable"
                && row.coverage_limits == "not_applicable"
                && row.required_assets == "not_applicable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.qc_pre"
                && row.tool_id == "samtools"
                && row.corpus_family_id == "corpus-01-bam"
                && row.fixture_id == "corpus-01-bam-mini"
                && row.sample_id == "not_applicable"
                && row.damage_expectation == "not_applicable"
                && row.coverage_limits == "not_applicable"
                && row.required_assets == "not_applicable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.align"
                && row.tool_id == "bwa"
                && row.corpus_family_id == "corpus-01"
                && row.fixture_id == "corpus-01-mini"
                && row.sample_id == "not_applicable"
                && row.damage_expectation == "not_applicable"
                && row.coverage_limits == "not_applicable"
                && row.required_assets == "not_applicable"
        }));
    }
}
