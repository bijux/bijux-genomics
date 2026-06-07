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
        validate_bam_corpus_fixture_manifest_path, BamCorpusFixtureGenotypingContractReport,
        BamCorpusFixtureKinshipContractReport, BamCorpusFixtureValidationReport,
        DEFAULT_CORPUS_01_ADNA_BAM_MINI_MANIFEST_PATH,
        DEFAULT_CORPUS_01_GENOTYPING_MINI_MANIFEST_PATH,
        DEFAULT_CORPUS_01_KINSHIP_MINI_MANIFEST_PATH,
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
    "benchmarks/readiness/bam-corpus-assignment.tsv";
const BAM_CORPUS_ASSIGNMENT_SCHEMA_VERSION: &str = "bijux.bench.readiness.bam_corpus_assignment.v3";
const DEFAULT_LOCAL_AUTHENTICITY_CONFIG_PATH: &str =
    "benchmarks/configs/local/bam-authenticity.toml";
const DEFAULT_LOCAL_CONTAMINATION_CONFIG_PATH: &str =
    "benchmarks/configs/local/bam-contamination.toml";
const DEFAULT_LOCAL_DAMAGE_CONFIG_PATH: &str = "benchmarks/configs/local/bam-damage.toml";
const DEFAULT_LOCAL_GENOTYPING_CONFIG_PATH: &str = "benchmarks/configs/local/bam-genotyping.toml";
const DEFAULT_LOCAL_HAPLOGROUPS_CONFIG_PATH: &str = "benchmarks/configs/local/bam-haplogroups.toml";
const DEFAULT_LOCAL_KINSHIP_CONFIG_PATH: &str = "benchmarks/configs/local/bam-kinship.toml";
const DEFAULT_LOCAL_SEX_CONFIG_PATH: &str = "benchmarks/configs/local/bam-sex.toml";
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
    pub(crate) input_contract: String,
    pub(crate) benchmark_limits: String,
    pub(crate) required_assets: String,
    pub(crate) expected_outputs: String,
    pub(crate) skip_behavior: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone)]
struct BamCorpusAssignmentContext {
    stage_evidence: BTreeMap<String, BamStageEvidence>,
}

#[derive(Debug, Clone)]
struct BamStageEvidence {
    sample_id: String,
    input_contract: String,
    benchmark_limits: String,
    required_assets: String,
    expected_outputs: String,
    skip_behavior: String,
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

pub(crate) fn collect_bam_corpus_assignment_rows(
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
            input_contract: evidence.input_contract,
            benchmark_limits: evidence.benchmark_limits,
            required_assets: evidence.required_assets,
            expected_outputs: evidence.expected_outputs,
            skip_behavior: evidence.skip_behavior,
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
    ensure_bam_fixture_assignment_coverage(&rows)?;
    Ok((stage_count, tool_count, rows))
}

impl BamCorpusAssignmentContext {
    fn evidence_for_stage(&self, stage_id: &str) -> BamStageEvidence {
        self.stage_evidence.get(stage_id).cloned().unwrap_or_else(BamStageEvidence::not_applicable)
    }
}

impl BamStageEvidence {
    fn not_applicable() -> Self {
        Self {
            sample_id: NOT_APPLICABLE.to_string(),
            input_contract: NOT_APPLICABLE.to_string(),
            benchmark_limits: NOT_APPLICABLE.to_string(),
            required_assets: NOT_APPLICABLE.to_string(),
            expected_outputs: NOT_APPLICABLE.to_string(),
            skip_behavior: NOT_APPLICABLE.to_string(),
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
    let genotyping_fixture = validate_bam_corpus_fixture_manifest_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_01_GENOTYPING_MINI_MANIFEST_PATH),
    )?;
    let kinship_fixture = validate_bam_corpus_fixture_manifest_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_01_KINSHIP_MINI_MANIFEST_PATH),
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
    let genotyping = load_governed_toml::<LocalGenotypingConfig>(
        &repo_root.join(DEFAULT_LOCAL_GENOTYPING_CONFIG_PATH),
        "BAM genotyping config",
    )?;
    let genotyping_contract = ensure_stage_matches_genotyping_fixture(
        repo_root,
        "bam.genotyping",
        &genotyping,
        &genotyping_fixture,
    )?;
    let kinship = load_governed_toml::<LocalKinshipConfig>(
        &repo_root.join(DEFAULT_LOCAL_KINSHIP_CONFIG_PATH),
        "BAM kinship config",
    )?;
    let kinship_contract =
        ensure_stage_matches_kinship_fixture(repo_root, "bam.kinship", &kinship, &kinship_fixture)?;

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

    let mut stage_evidence = BTreeMap::new();
    stage_evidence.insert(
        "bam.authenticity".to_string(),
        BamStageEvidence {
            sample_id: authenticity_case.sample_id,
            input_contract: damage_expectation.clone(),
            benchmark_limits: format_key_value_contract([
                ("complexity_min_reads", authenticity_case.complexity_min_reads.to_string()),
                (
                    "coverage_depth_thresholds",
                    format_numeric_list(&authenticity_case.coverage_depth_thresholds),
                ),
            ]),
            required_assets: damage_assets.clone(),
            expected_outputs: "authenticity_report,summary,stage_metrics".to_string(),
            skip_behavior: NOT_APPLICABLE.to_string(),
        },
    );
    stage_evidence.insert(
        "bam.damage".to_string(),
        BamStageEvidence {
            sample_id: damage_case.sample_id,
            input_contract: damage_expectation.clone(),
            benchmark_limits: NOT_APPLICABLE.to_string(),
            required_assets: damage_assets,
            expected_outputs: "damage_report,terminal_position_metrics,stage_metrics".to_string(),
            skip_behavior: NOT_APPLICABLE.to_string(),
        },
    );
    stage_evidence.insert(
        "bam.contamination".to_string(),
        BamStageEvidence {
            sample_id: contamination.sample_id,
            input_contract: inherited_adna_profile.clone(),
            benchmark_limits: format_key_value_contract([(
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
            expected_outputs: "contamination_report,summary,stage_metrics".to_string(),
            skip_behavior: NOT_APPLICABLE.to_string(),
        },
    );
    stage_evidence.insert(
        "bam.sex".to_string(),
        BamStageEvidence {
            sample_id: sex_case.sample_id,
            input_contract: inherited_adna_profile.clone(),
            benchmark_limits: format_key_value_contract([
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
            expected_outputs: "sex_report,summary,stage_metrics".to_string(),
            skip_behavior: NOT_APPLICABLE.to_string(),
        },
    );
    stage_evidence.insert(
        "bam.haplogroups".to_string(),
        BamStageEvidence {
            sample_id: haplogroups.sample_id,
            input_contract: inherited_adna_profile,
            benchmark_limits: format_key_value_contract([(
                "min_coverage",
                format_decimal(haplogroups.min_coverage),
            )]),
            required_assets: format_key_value_contract([
                ("reference_fasta", asset_id_from_path(&haplogroups.reference_fasta)?),
                ("reference_panel", haplogroups.reference_panel_id),
            ]),
            expected_outputs: "haplogroups,summary,stage_metrics".to_string(),
            skip_behavior: NOT_APPLICABLE.to_string(),
        },
    );
    stage_evidence.insert(
        "bam.genotyping".to_string(),
        BamStageEvidence {
            sample_id: genotyping.sample_id,
            input_contract: format_key_value_contract([
                ("reference", asset_id_from_path(&genotyping.reference_fasta)?),
                ("regions", asset_id_from_path(&genotyping.regions)?),
                ("sites", asset_id_from_path(&genotyping.sites_vcf)?),
            ]),
            benchmark_limits: format_key_value_contract([
                ("min_call_rate", format_decimal(genotyping.min_call_rate)),
                ("min_posterior", format_decimal(genotyping.min_posterior)),
            ]),
            required_assets: format_key_value_contract([
                ("reference_fasta", asset_id_from_path(&genotyping.reference_fasta)?),
                ("regions", asset_id_from_path(&genotyping.regions)?),
                ("sites_vcf", asset_id_from_path(&genotyping.sites_vcf)?),
            ]),
            expected_outputs: format_string_list(&genotyping_contract.expected_outputs),
            skip_behavior: NOT_APPLICABLE.to_string(),
        },
    );
    stage_evidence.insert(
        "bam.kinship".to_string(),
        BamStageEvidence {
            sample_id: kinship
                .cases
                .iter()
                .map(|case| case.sample_id.clone())
                .collect::<Vec<_>>()
                .join(","),
            input_contract: format_key_value_contract([
                ("reference", asset_id_from_path(Path::new(&kinship_fixture.reference_fasta))?),
                ("reference_build", kinship_contract.reference_build.clone()),
                ("reference_panel", kinship_contract.reference_panel.clone()),
                ("population_scope", kinship_contract.population_scope.clone()),
            ]),
            benchmark_limits: format_key_value_contract(kinship.cases.iter().flat_map(|case| {
                [
                    (
                        format!("{}.min_overlap_snps", case.sample_id),
                        case.min_overlap_snps.to_string(),
                    ),
                    (
                        format!("{}.observed_max_overlap_snps", case.sample_id),
                        case.expected_observed_max_overlap_snps.to_string(),
                    ),
                ]
            })),
            required_assets: format_key_value_contract([
                (
                    "reference_fasta",
                    asset_id_from_path(Path::new(&kinship_fixture.reference_fasta))?,
                ),
                ("reference_panel", kinship_contract.reference_panel.clone()),
            ]),
            expected_outputs: format_string_list(&kinship_contract.expected_outputs),
            skip_behavior: format_key_value_contract(
                kinship_contract
                    .cases
                    .iter()
                    .map(|case| (case.sample_id.clone(), case.skip_behavior.clone())),
            ),
        },
    );

    Ok(BamCorpusAssignmentContext { stage_evidence })
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
        "tool_id\tstage_id\tbenchmark_status\tsupport_status\tadapter_status\tparser_status\tcorpus_family_id\tfixture_id\tsample_id\tinput_contract\tbenchmark_limits\trequired_assets\texpected_outputs\tskip_behavior\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.benchmark_status),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.corpus_family_id),
            sanitize_tsv(&row.fixture_id),
            sanitize_tsv(&row.sample_id),
            sanitize_tsv(&row.input_contract),
            sanitize_tsv(&row.benchmark_limits),
            sanitize_tsv(&row.required_assets),
            sanitize_tsv(&row.expected_outputs),
            sanitize_tsv(&row.skip_behavior),
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

fn format_string_list(values: &[String]) -> String {
    values.join(",")
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

fn ensure_stage_matches_genotyping_fixture(
    repo_root: &Path,
    stage_id: &str,
    config: &LocalGenotypingConfig,
    fixture: &BamCorpusFixtureValidationReport,
) -> Result<BamCorpusFixtureGenotypingContractReport> {
    let contract = fixture.genotyping_contract.clone().ok_or_else(|| {
        anyhow!("BAM stage `{stage_id}` requires a governed genotyping contract in the fixture")
    })?;
    ensure_stage_sample_matches_bam_fixture(stage_id, &config.sample_id, fixture)?;
    if config.sample_id != contract.sample_id {
        return Err(anyhow!(
            "BAM stage `{stage_id}` sample `{}` must match genotyping contract sample `{}`",
            config.sample_id,
            contract.sample_id
        ));
    }
    ensure_paths_match(
        repo_root,
        stage_id,
        "reference_fasta",
        &config.reference_fasta,
        &repo_root.join(&fixture.reference_fasta),
    )?;
    ensure_paths_match(
        repo_root,
        stage_id,
        "sites_vcf",
        &config.sites_vcf,
        &repo_root.join(&contract.sites_vcf),
    )?;
    ensure_paths_match(
        repo_root,
        stage_id,
        "regions",
        &config.regions,
        &repo_root.join(&contract.regions),
    )?;
    ensure_float_matches(stage_id, "min_posterior", config.min_posterior, contract.min_posterior)?;
    ensure_float_matches(stage_id, "min_call_rate", config.min_call_rate, contract.min_call_rate)?;
    Ok(contract)
}

fn ensure_stage_matches_kinship_fixture(
    repo_root: &Path,
    stage_id: &str,
    config: &LocalKinshipConfig,
    fixture: &BamCorpusFixtureValidationReport,
) -> Result<BamCorpusFixtureKinshipContractReport> {
    let contract = fixture.kinship_contract.clone().ok_or_else(|| {
        anyhow!("BAM stage `{stage_id}` requires a governed kinship contract in the fixture")
    })?;
    if config.cases.len() != contract.cases.len() {
        return Err(anyhow!(
            "BAM stage `{stage_id}` must declare {} kinship cases, found {}",
            contract.cases.len(),
            config.cases.len()
        ));
    }

    let contract_cases = contract
        .cases
        .iter()
        .map(|case| (case.sample_id.clone(), case))
        .collect::<BTreeMap<_, _>>();
    for case in &config.cases {
        ensure_stage_sample_matches_bam_fixture(stage_id, &case.sample_id, fixture)?;
        let expected = contract_cases.get(&case.sample_id).ok_or_else(|| {
            anyhow!(
                "BAM stage `{stage_id}` sample `{}` is missing from the kinship contract",
                case.sample_id
            )
        })?;
        ensure_paths_match(
            repo_root,
            stage_id,
            &format!("{}.bam", case.sample_id),
            &case.bam,
            &repo_root.join(sample_alignment_path(fixture, &case.sample_id)?),
        )?;
        if case.reference_panel != contract.reference_panel {
            return Err(anyhow!(
                "BAM stage `{stage_id}` sample `{}` reference_panel `{}` must match `{}`",
                case.sample_id,
                case.reference_panel,
                contract.reference_panel
            ));
        }
        if case.reference_build != contract.reference_build {
            return Err(anyhow!(
                "BAM stage `{stage_id}` sample `{}` reference_build `{}` must match `{}`",
                case.sample_id,
                case.reference_build,
                contract.reference_build
            ));
        }
        if case.population_scope != contract.population_scope {
            return Err(anyhow!(
                "BAM stage `{stage_id}` sample `{}` population_scope `{}` must match `{}`",
                case.sample_id,
                case.population_scope,
                contract.population_scope
            ));
        }
        if case.min_overlap_snps != expected.min_overlap_snps {
            return Err(anyhow!(
                "BAM stage `{stage_id}` sample `{}` min_overlap_snps `{}` must match `{}`",
                case.sample_id,
                case.min_overlap_snps,
                expected.min_overlap_snps
            ));
        }
        if case.expected_status != expected.expected_status {
            return Err(anyhow!(
                "BAM stage `{stage_id}` sample `{}` expected_status `{}` must match `{}`",
                case.sample_id,
                case.expected_status,
                expected.expected_status
            ));
        }
        if case.expected_observed_max_overlap_snps != expected.expected_observed_max_overlap_snps {
            return Err(anyhow!(
                "BAM stage `{stage_id}` sample `{}` expected_observed_max_overlap_snps `{}` must match `{}`",
                case.sample_id,
                case.expected_observed_max_overlap_snps,
                expected.expected_observed_max_overlap_snps
            ));
        }
        let relationship_labels = case
            .expected_pairwise_results
            .iter()
            .map(|result| result.relationship_label.clone())
            .collect::<Vec<_>>();
        if relationship_labels != expected.expected_relationship_labels {
            return Err(anyhow!(
                "BAM stage `{stage_id}` sample `{}` relationship labels {:?} must match {:?}",
                case.sample_id,
                relationship_labels,
                expected.expected_relationship_labels
            ));
        }
        let inferred_skip_behavior = if relationship_labels.is_empty() {
            "stop_without_pairwise_results"
        } else {
            "emit_pairwise_results"
        };
        if inferred_skip_behavior != expected.skip_behavior {
            return Err(anyhow!(
                "BAM stage `{stage_id}` sample `{}` skip behavior `{}` must match `{}`",
                case.sample_id,
                inferred_skip_behavior,
                expected.skip_behavior
            ));
        }
    }

    Ok(contract)
}

fn ensure_paths_match(
    repo_root: &Path,
    stage_id: &str,
    field_name: &str,
    observed_path: &Path,
    expected_path: &Path,
) -> Result<()> {
    let observed = repo_relative_path(repo_root, observed_path)
        .canonicalize()
        .with_context(|| format!("canonicalize {}", observed_path.display()))?;
    let expected = expected_path
        .canonicalize()
        .with_context(|| format!("canonicalize {}", expected_path.display()))?;
    if observed != expected {
        return Err(anyhow!(
            "BAM stage `{stage_id}` field `{field_name}` path `{}` must match `{}`",
            observed_path.display(),
            expected_path.display()
        ));
    }
    Ok(())
}

fn ensure_float_matches(
    stage_id: &str,
    field_name: &str,
    observed: f64,
    expected: f64,
) -> Result<()> {
    if (observed - expected).abs() > f64::EPSILON {
        return Err(anyhow!(
            "BAM stage `{stage_id}` field `{field_name}` value `{observed}` must match `{expected}`"
        ));
    }
    Ok(())
}

fn sample_alignment_path<'a>(
    fixture: &'a BamCorpusFixtureValidationReport,
    sample_id: &str,
) -> Result<&'a str> {
    fixture
        .samples
        .iter()
        .find(|sample| sample.sample_id == sample_id)
        .map(|sample| sample.alignment_path.as_str())
        .ok_or_else(|| anyhow!("fixture `{}` is missing sample `{sample_id}`", fixture.corpus_id))
}

fn ensure_bam_fixture_assignment_coverage(rows: &[BamCorpusAssignmentRow]) -> Result<()> {
    let expected_rows = [
        BamFixtureAssignmentExpectation {
            stage_id: "bam.authenticity",
            fixture_id: "corpus-01-adna-damage-mini",
            sample_id: "adna_damage_non_udg",
            input_contract: "ct5p=0.18;ga3p=0.11;short_frag=1;signal=moderate;strict_profile_upgraded=false;terminal=ct5p_dominant;udg=non_udg",
            benchmark_limits: "complexity_min_reads=3;coverage_depth_thresholds=1,5,10",
            required_assets: "expected_damage=expected_damage.json;reference_fasta=adna_damage_reference.fasta",
            expected_outputs: "authenticity_report,summary,stage_metrics",
            skip_behavior: NOT_APPLICABLE,
        },
        BamFixtureAssignmentExpectation {
            stage_id: "bam.damage",
            fixture_id: "corpus-01-adna-damage-mini",
            sample_id: "adna_damage_non_udg",
            input_contract: "ct5p=0.18;ga3p=0.11;short_frag=1;signal=moderate;strict_profile_upgraded=false;terminal=ct5p_dominant;udg=non_udg",
            benchmark_limits: NOT_APPLICABLE,
            required_assets: "expected_damage=expected_damage.json;reference_fasta=adna_damage_reference.fasta",
            expected_outputs: "damage_report,terminal_position_metrics,stage_metrics",
            skip_behavior: NOT_APPLICABLE,
        },
        BamFixtureAssignmentExpectation {
            stage_id: "bam.contamination",
            fixture_id: "corpus-01-adna-bam-mini",
            sample_id: "adna_contamination_panel_screen",
            input_contract: "signal=moderate;terminal=ct5p_dominant;udg=non_udg",
            benchmark_limits: "minimum_mean_coverage=0.5",
            required_assets: "reference_fasta=adna_bam_reference;reference_panel=adna_contamination_panel",
            expected_outputs: "contamination_report,summary,stage_metrics",
            skip_behavior: NOT_APPLICABLE,
        },
        BamFixtureAssignmentExpectation {
            stage_id: "bam.sex",
            fixture_id: "corpus-01-adna-bam-mini",
            sample_id: "adna_xy_autosome_coverage",
            input_contract: "signal=moderate;terminal=ct5p_dominant;udg=non_udg",
            benchmark_limits: "expected_autosomal_coverage=1;expected_x_coverage=0.5;expected_y_coverage=0.5;minimum_y_sites=5",
            required_assets: "reference_fasta=adna_bam_reference",
            expected_outputs: "sex_report,summary,stage_metrics",
            skip_behavior: NOT_APPLICABLE,
        },
        BamFixtureAssignmentExpectation {
            stage_id: "bam.haplogroups",
            fixture_id: "corpus-01-adna-bam-mini",
            sample_id: "adna_y_haplogroup_panel",
            input_contract: "signal=moderate;terminal=ct5p_dominant;udg=non_udg",
            benchmark_limits: "min_coverage=2",
            required_assets: "reference_fasta=adna_bam_reference;reference_panel=adna-y-hg38-mini",
            expected_outputs: "haplogroups,summary,stage_metrics",
            skip_behavior: NOT_APPLICABLE,
        },
        BamFixtureAssignmentExpectation {
            stage_id: "bam.genotyping",
            fixture_id: "corpus-01-genotyping-mini",
            sample_id: "human_like_genotyping_candidate_panel",
            input_contract: "reference=corpus_01_bam_reference;regions=human_like_genotyping_target_regions;sites=human_like_genotyping_candidate_sites",
            benchmark_limits: "min_call_rate=0.5;min_posterior=0.9",
            required_assets: "reference_fasta=corpus_01_bam_reference;regions=human_like_genotyping_target_regions;sites_vcf=human_like_genotyping_candidate_sites",
            expected_outputs: "genotyping_bcf,genotyping_vcf,genotyping_vcf_tbi,genotyping_gl,summary,stage_metrics",
            skip_behavior: NOT_APPLICABLE,
        },
        BamFixtureAssignmentExpectation {
            stage_id: "bam.kinship",
            fixture_id: "corpus-01-kinship-mini",
            sample_id: "human_like_kinship_low_overlap_pair,human_like_kinship_related_pair",
            input_contract: "population_scope=human_diploid_panel;reference=corpus_01_bam_reference;reference_build=grch38;reference_panel=human_like_relatedness_panel",
            benchmark_limits: "human_like_kinship_low_overlap_pair.min_overlap_snps=5;human_like_kinship_low_overlap_pair.observed_max_overlap_snps=4;human_like_kinship_related_pair.min_overlap_snps=6;human_like_kinship_related_pair.observed_max_overlap_snps=6",
            required_assets: "reference_fasta=corpus_01_bam_reference;reference_panel=human_like_relatedness_panel",
            expected_outputs: "kinship_report,summary,kinship_segments,stage_metrics",
            skip_behavior: "human_like_kinship_low_overlap_pair=stop_without_pairwise_results;human_like_kinship_related_pair=emit_pairwise_results",
        },
    ];

    for expectation in expected_rows {
        let stage_rows =
            rows.iter().filter(|row| row.stage_id == expectation.stage_id).collect::<Vec<_>>();
        if stage_rows.is_empty() {
            return Err(anyhow!(
                "BAM corpus assignment coverage is missing governed rows for stage `{}`",
                expectation.stage_id
            ));
        }
        for row in stage_rows {
            if row.fixture_id != expectation.fixture_id
                || row.sample_id != expectation.sample_id
                || row.input_contract != expectation.input_contract
                || row.benchmark_limits != expectation.benchmark_limits
                || row.required_assets != expectation.required_assets
                || row.expected_outputs != expectation.expected_outputs
                || row.skip_behavior != expectation.skip_behavior
            {
                return Err(anyhow!(
                    "BAM corpus assignment row `{}` / `{}` drifted away from the governed stage evidence contract",
                    row.stage_id,
                    row.tool_id
                ));
            }
        }
    }

    Ok(())
}

struct BamFixtureAssignmentExpectation {
    stage_id: &'static str,
    fixture_id: &'static str,
    sample_id: &'static str,
    input_contract: &'static str,
    benchmark_limits: &'static str,
    required_assets: &'static str,
    expected_outputs: &'static str,
    skip_behavior: &'static str,
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
struct LocalGenotypingConfig {
    sample_id: String,
    reference_fasta: PathBuf,
    sites_vcf: PathBuf,
    regions: PathBuf,
    min_posterior: f64,
    min_call_rate: f64,
}

#[derive(Debug, Deserialize)]
struct LocalKinshipConfig {
    cases: Vec<LocalKinshipCase>,
}

#[derive(Debug, Deserialize)]
struct LocalKinshipCase {
    sample_id: String,
    bam: PathBuf,
    reference_panel: String,
    reference_build: String,
    population_scope: String,
    min_overlap_snps: u32,
    expected_status: String,
    expected_observed_max_overlap_snps: u32,
    #[serde(default)]
    expected_pairwise_results: Vec<LocalKinshipPairwiseResult>,
}

#[derive(Debug, Deserialize)]
struct LocalKinshipPairwiseResult {
    relationship_label: String,
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

        assert_eq!(report.schema_version, "bijux.bench.readiness.bam_corpus_assignment.v3");
        assert_eq!(report.stage_count, 24);
        assert!(report.row_count > 0);
        assert_eq!(report.corpus_family_counts.get("corpus-01"), Some(&2));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.authenticity"
                && row.tool_id == "authenticct"
                && row.corpus_family_id == "corpus-01-adna-bam"
                && row.fixture_id == "corpus-01-adna-damage-mini"
                && row.sample_id == "adna_damage_non_udg"
                && row.input_contract
                    == "ct5p=0.18;ga3p=0.11;short_frag=1;signal=moderate;strict_profile_upgraded=false;terminal=ct5p_dominant;udg=non_udg"
                && row.benchmark_limits
                    == "complexity_min_reads=3;coverage_depth_thresholds=1,5,10"
                && row.required_assets
                    == "expected_damage=expected_damage.json;reference_fasta=adna_damage_reference.fasta"
                && row.expected_outputs == "authenticity_report,summary,stage_metrics"
                && row.skip_behavior == "not_applicable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.contamination"
                && row.tool_id == "verifybamid2"
                && row.corpus_family_id == "corpus-01-adna-bam"
                && row.fixture_id == "corpus-01-adna-bam-mini"
                && row.sample_id == "adna_contamination_panel_screen"
                && row.input_contract == "signal=moderate;terminal=ct5p_dominant;udg=non_udg"
                && row.benchmark_limits == "minimum_mean_coverage=0.5"
                && row.required_assets
                    == "reference_fasta=adna_bam_reference;reference_panel=adna_contamination_panel"
                && row.expected_outputs == "contamination_report,summary,stage_metrics"
                && row.skip_behavior == "not_applicable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.sex"
                && row.tool_id == "rxy"
                && row.corpus_family_id == "corpus-01-adna-bam"
                && row.fixture_id == "corpus-01-adna-bam-mini"
                && row.sample_id == "adna_xy_autosome_coverage"
                && row.input_contract == "signal=moderate;terminal=ct5p_dominant;udg=non_udg"
                && row.benchmark_limits
                    == "expected_autosomal_coverage=1;expected_x_coverage=0.5;expected_y_coverage=0.5;minimum_y_sites=5"
                && row.required_assets == "reference_fasta=adna_bam_reference"
                && row.expected_outputs == "sex_report,summary,stage_metrics"
                && row.skip_behavior == "not_applicable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.haplogroups"
                && row.tool_id == "yleaf"
                && row.corpus_family_id == "corpus-01-adna-bam"
                && row.fixture_id == "corpus-01-adna-bam-mini"
                && row.sample_id == "adna_y_haplogroup_panel"
                && row.input_contract == "signal=moderate;terminal=ct5p_dominant;udg=non_udg"
                && row.benchmark_limits == "min_coverage=2"
                && row.required_assets
                    == "reference_fasta=adna_bam_reference;reference_panel=adna-y-hg38-mini"
                && row.expected_outputs == "haplogroups,summary,stage_metrics"
                && row.skip_behavior == "not_applicable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.genotyping"
                && row.tool_id == "angsd"
                && row.corpus_family_id == "corpus-01-genotyping"
                && row.fixture_id == "corpus-01-genotyping-mini"
                && row.sample_id == "human_like_genotyping_candidate_panel"
                && row.input_contract
                    == "reference=corpus_01_bam_reference;regions=human_like_genotyping_target_regions;sites=human_like_genotyping_candidate_sites"
                && row.benchmark_limits == "min_call_rate=0.5;min_posterior=0.9"
                && row.required_assets
                    == "reference_fasta=corpus_01_bam_reference;regions=human_like_genotyping_target_regions;sites_vcf=human_like_genotyping_candidate_sites"
                && row.expected_outputs
                    == "genotyping_bcf,genotyping_vcf,genotyping_vcf_tbi,genotyping_gl,summary,stage_metrics"
                && row.skip_behavior == "not_applicable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.kinship"
                && row.tool_id == "king"
                && row.corpus_family_id == "corpus-01-kinship"
                && row.fixture_id == "corpus-01-kinship-mini"
                && row.sample_id
                    == "human_like_kinship_low_overlap_pair,human_like_kinship_related_pair"
                && row.input_contract
                    == "population_scope=human_diploid_panel;reference=corpus_01_bam_reference;reference_build=grch38;reference_panel=human_like_relatedness_panel"
                && row.benchmark_limits
                    == "human_like_kinship_low_overlap_pair.min_overlap_snps=5;human_like_kinship_low_overlap_pair.observed_max_overlap_snps=4;human_like_kinship_related_pair.min_overlap_snps=6;human_like_kinship_related_pair.observed_max_overlap_snps=6"
                && row.required_assets
                    == "reference_fasta=corpus_01_bam_reference;reference_panel=human_like_relatedness_panel"
                && row.expected_outputs
                    == "kinship_report,summary,kinship_segments,stage_metrics"
                && row.skip_behavior
                    == "human_like_kinship_low_overlap_pair=stop_without_pairwise_results;human_like_kinship_related_pair=emit_pairwise_results"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.qc_pre"
                && row.tool_id == "samtools"
                && row.corpus_family_id == "corpus-01-bam"
                && row.fixture_id == "corpus-01-bam-mini"
                && row.sample_id == "not_applicable"
                && row.input_contract == "not_applicable"
                && row.benchmark_limits == "not_applicable"
                && row.required_assets == "not_applicable"
                && row.expected_outputs == "not_applicable"
                && row.skip_behavior == "not_applicable"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.align"
                && row.tool_id == "bwa"
                && row.corpus_family_id == "corpus-01"
                && row.fixture_id == "corpus-01-mini"
                && row.sample_id == "not_applicable"
                && row.input_contract == "not_applicable"
                && row.benchmark_limits == "not_applicable"
                && row.required_assets == "not_applicable"
                && row.expected_outputs == "not_applicable"
                && row.skip_behavior == "not_applicable"
        }));
    }
}
