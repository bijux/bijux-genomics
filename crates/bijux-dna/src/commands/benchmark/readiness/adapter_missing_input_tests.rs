use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use toml::Value;

use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ADAPTER_MISSING_INPUT_TESTS_PATH: &str =
    "target/bench-readiness/adapter-missing-input-tests.json";
const ADAPTER_MISSING_INPUT_TESTS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.adapter_missing_input_tests.v1";
const FASTQ_DOMAIN: &str = "fastq";
const BAM_DOMAIN: &str = "bam";
const PROBE_KIND_LOCAL_READY: &str = "local_ready";
const PROBE_KIND_LOCAL_SMOKE: &str = "local_smoke";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AdapterMissingInputTestRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) probe_kind: String,
    pub(crate) missing_input_role: String,
    pub(crate) missing_input_class: String,
    pub(crate) expected_error_fragment: String,
    pub(crate) observed_error: String,
    pub(crate) passed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdapterMissingInputTestsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) passed_row_count: usize,
    pub(crate) failed_row_count: usize,
    pub(crate) missing_input_class_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<AdapterMissingInputTestRow>,
}

#[derive(Debug, Clone, Copy)]
struct AdapterMissingInputProbe {
    domain: &'static str,
    stage_id: &'static str,
    tool_id: &'static str,
    probe_kind: &'static str,
    config_path: &'static str,
    missing_input_role: &'static str,
    missing_input_class: &'static str,
    expected_error_fragment: &'static str,
}

pub(crate) fn run_render_adapter_missing_input_tests(
    args: &parse::BenchReadinessRenderAdapterMissingInputTestsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_adapter_missing_input_tests(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ADAPTER_MISSING_INPUT_TESTS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_adapter_missing_input_tests(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AdapterMissingInputTestsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_adapter_missing_input_test_rows(repo_root)?;
    let passed_row_count = rows.iter().filter(|row| row.passed).count();
    let failed_row_count = rows.len().saturating_sub(passed_row_count);
    let mut missing_input_class_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *missing_input_class_counts.entry(row.missing_input_class.clone()).or_default() += 1;
    }

    let report = AdapterMissingInputTestsReport {
        schema_version: ADAPTER_MISSING_INPUT_TESTS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        passed_row_count,
        failed_row_count,
        missing_input_class_counts,
        rows,
    };
    let payload = serde_json::to_string_pretty(&report)
        .context("render adapter missing-input report to JSON")?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, payload.as_bytes())?;

    Ok(report)
}

fn collect_adapter_missing_input_test_rows(
    repo_root: &Path,
) -> Result<Vec<AdapterMissingInputTestRow>> {
    let probes = adapter_missing_input_probes();
    probes.iter().map(|probe| execute_probe(repo_root, probe)).collect()
}

fn execute_probe(
    repo_root: &Path,
    probe: &AdapterMissingInputProbe,
) -> Result<AdapterMissingInputTestRow> {
    let temp_root = repo_root.join("artifacts/bench-readiness");
    let temp = bijux_dna_infra::temp_dir_in(&temp_root, "adapter-missing-input-")
        .map_err(anyhow::Error::from)
        .context("create adapter missing-input probe root")?;
    copy_tool_contract(repo_root, temp.path(), probe.domain, probe.tool_id)?;
    copy_runtime_profile(repo_root, temp.path())?;
    write_probe_config(repo_root, temp.path(), probe)?;
    let observed_error = match invoke_probe_planner(temp.path(), probe) {
        Ok(()) => format!(
            "{} unexpectedly accepted missing input role `{}`",
            probe.stage_id, probe.missing_input_role
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(probe.expected_error_fragment);

    Ok(AdapterMissingInputTestRow {
        domain: probe.domain.to_string(),
        stage_id: probe.stage_id.to_string(),
        tool_id: probe.tool_id.to_string(),
        probe_kind: probe.probe_kind.to_string(),
        missing_input_role: probe.missing_input_role.to_string(),
        missing_input_class: probe.missing_input_class.to_string(),
        expected_error_fragment: probe.expected_error_fragment.to_string(),
        observed_error,
        passed,
    })
}

fn copy_tool_contract(
    repo_root: &Path,
    probe_root: &Path,
    domain: &str,
    tool_id: &str,
) -> Result<()> {
    let source = repo_root.join(format!("domain/{domain}/tools/{tool_id}.yaml"));
    let target = probe_root.join(format!("domain/{domain}/tools/{tool_id}.yaml"));
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::copy(&source, &target)
        .with_context(|| format!("copy {} -> {}", source.display(), target.display()))?;
    Ok(())
}

fn copy_runtime_profile(repo_root: &Path, probe_root: &Path) -> Result<()> {
    let source = repo_root.join("configs/runtime/profiles/local.toml");
    let target = probe_root.join("configs/runtime/profiles/local.toml");
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::copy(&source, &target)
        .with_context(|| format!("copy {} -> {}", source.display(), target.display()))?;
    Ok(())
}

fn write_probe_config(
    repo_root: &Path,
    probe_root: &Path,
    probe: &AdapterMissingInputProbe,
) -> Result<()> {
    let mut value = load_governed_config(repo_root, probe.config_path)?;
    match probe.stage_id {
        "fastq.deplete_host"
        | "fastq.deplete_reference_contaminants"
        | "fastq.deplete_rrna"
        | "fastq.screen_taxonomy" => {
            mutate_fastq_local_ready_config(&mut value, repo_root, probe_root, probe)?;
        }
        "bam.contamination" | "bam.genotyping" | "bam.haplogroups" => {
            mutate_bam_local_ready_config(&mut value, repo_root, probe_root, probe)?;
        }
        "bam.recalibration" => {
            mutate_bam_recalibration_smoke_config(&mut value, repo_root, probe_root, probe)?;
        }
        _ => {
            return Err(anyhow!(
                "unsupported adapter missing-input probe stage `{}`",
                probe.stage_id
            ));
        }
    }

    let output_path = probe_root.join(probe.config_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let rendered =
        toml::to_string_pretty(&value).context("render adapter missing-input probe config")?;
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered.as_bytes())?;
    Ok(())
}

fn mutate_fastq_local_ready_config(
    value: &mut Value,
    repo_root: &Path,
    probe_root: &Path,
    probe: &AdapterMissingInputProbe,
) -> Result<()> {
    let table = root_table_mut(value)?;
    set_string(table, "tool_id", probe.tool_id.to_string());
    set_string(
        table,
        "output_dir",
        readiness_output_dir(
            probe.probe_kind,
            probe.stage_id,
            probe.tool_id,
            probe.missing_input_role,
        ),
    );
    rewrite_path_field(
        table,
        "input_r1",
        repo_root,
        missing_path_for_role(probe_root, probe, "input_r1"),
        probe.missing_input_role == "input_r1",
    )?;

    match probe.stage_id {
        "fastq.deplete_host" | "fastq.deplete_reference_contaminants" => {
            rewrite_path_field(
                table,
                "reference_index",
                repo_root,
                missing_path_for_role(probe_root, probe, "reference_index"),
                probe.missing_input_role == "reference_index",
            )?;
        }
        "fastq.deplete_rrna" => {
            rewrite_path_field(
                table,
                "rrna_db",
                repo_root,
                missing_path_for_role(probe_root, probe, "rrna_db"),
                probe.missing_input_role == "rrna_db",
            )?;
        }
        "fastq.screen_taxonomy" => {
            rewrite_path_field(
                table,
                "database_root",
                repo_root,
                missing_path_for_role(probe_root, probe, "database_root"),
                probe.missing_input_role == "database_root",
            )?;
        }
        _ => {}
    }

    Ok(())
}

fn mutate_bam_local_ready_config(
    value: &mut Value,
    repo_root: &Path,
    probe_root: &Path,
    probe: &AdapterMissingInputProbe,
) -> Result<()> {
    let table = root_table_mut(value)?;
    set_string(table, "tool_id", probe.tool_id.to_string());
    set_string(
        table,
        "output_dir",
        readiness_output_dir(
            probe.probe_kind,
            probe.stage_id,
            probe.tool_id,
            probe.missing_input_role,
        ),
    );

    match probe.stage_id {
        "bam.contamination" => {
            rewrite_path_field(
                table,
                "bam",
                repo_root,
                missing_path_for_role(probe_root, probe, "bam"),
                probe.missing_input_role == "bam",
            )?;
            rewrite_path_field(
                table,
                "bai",
                repo_root,
                missing_path_for_role(probe_root, probe, "bai"),
                false,
            )?;
            rewrite_path_field(
                table,
                "reference_fasta",
                repo_root,
                missing_path_for_role(probe_root, probe, "reference_fasta"),
                probe.missing_input_role == "reference_fasta",
            )?;
            rewrite_first_array_path_entry(
                table,
                "reference_panels",
                repo_root,
                missing_path_for_role(probe_root, probe, "reference_panel"),
                probe.missing_input_role == "reference_panel",
            )?;
        }
        "bam.genotyping" => {
            rewrite_path_field(
                table,
                "bam",
                repo_root,
                missing_path_for_role(probe_root, probe, "bam"),
                probe.missing_input_role == "bam",
            )?;
            rewrite_path_field(
                table,
                "bai",
                repo_root,
                missing_path_for_role(probe_root, probe, "bai"),
                false,
            )?;
            rewrite_path_field(
                table,
                "reference_fasta",
                repo_root,
                missing_path_for_role(probe_root, probe, "reference_fasta"),
                probe.missing_input_role == "reference_fasta",
            )?;
            rewrite_path_field(
                table,
                "sites_vcf",
                repo_root,
                missing_path_for_role(probe_root, probe, "sites_vcf"),
                probe.missing_input_role == "sites_vcf",
            )?;
            rewrite_path_field(
                table,
                "regions",
                repo_root,
                missing_path_for_role(probe_root, probe, "regions"),
                probe.missing_input_role == "regions",
            )?;
        }
        "bam.haplogroups" => {
            rewrite_path_field(
                table,
                "bam",
                repo_root,
                missing_path_for_role(probe_root, probe, "bam"),
                probe.missing_input_role == "bam",
            )?;
            rewrite_path_field(
                table,
                "bai",
                repo_root,
                missing_path_for_role(probe_root, probe, "bai"),
                false,
            )?;
            rewrite_path_field(
                table,
                "reference_fasta",
                repo_root,
                missing_path_for_role(probe_root, probe, "reference_fasta"),
                probe.missing_input_role == "reference_fasta",
            )?;
            rewrite_path_field(
                table,
                "reference_panel",
                repo_root,
                missing_path_for_role(probe_root, probe, "reference_panel"),
                probe.missing_input_role == "reference_panel",
            )?;
        }
        _ => {}
    }

    Ok(())
}

fn mutate_bam_recalibration_smoke_config(
    value: &mut Value,
    repo_root: &Path,
    probe_root: &Path,
    probe: &AdapterMissingInputProbe,
) -> Result<()> {
    let table = root_table_mut(value)?;
    set_string(table, "tool_id", probe.tool_id.to_string());
    set_string(
        table,
        "output_dir",
        readiness_output_dir(
            probe.probe_kind,
            probe.stage_id,
            probe.tool_id,
            probe.missing_input_role,
        ),
    );

    let cases = table
        .get_mut("cases")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow!("bam.recalibration probe config must contain cases"))?;
    let case = cases
        .get_mut(0)
        .and_then(Value::as_table_mut)
        .ok_or_else(|| anyhow!("bam.recalibration probe config must contain a first case"))?;
    rewrite_path_field(
        case,
        "bam",
        repo_root,
        missing_path_for_role(probe_root, probe, "bam"),
        probe.missing_input_role == "bam",
    )?;
    rewrite_path_field(
        case,
        "reference",
        repo_root,
        missing_path_for_role(probe_root, probe, "reference"),
        probe.missing_input_role == "reference",
    )?;
    rewrite_first_array_path_entry(
        case,
        "known_sites",
        repo_root,
        missing_path_for_role(probe_root, probe, "known_sites"),
        probe.missing_input_role == "known_sites",
    )?;
    Ok(())
}

fn invoke_probe_planner(repo_root: &Path, probe: &AdapterMissingInputProbe) -> Result<()> {
    match probe.stage_id {
        "fastq.deplete_host" => {
            bijux_dna_planner_fastq::stage_api::local_deplete_host_plan(repo_root).map(|_| ())
        }
        "fastq.deplete_reference_contaminants" => {
            bijux_dna_planner_fastq::stage_api::local_deplete_reference_contaminants_plan(repo_root)
                .map(|_| ())
        }
        "fastq.deplete_rrna" => {
            bijux_dna_planner_fastq::stage_api::local_deplete_rrna_plan(repo_root).map(|_| ())
        }
        "fastq.screen_taxonomy" => {
            bijux_dna_planner_fastq::stage_api::local_screen_taxonomy_plan(repo_root).map(|_| ())
        }
        "bam.contamination" => {
            bijux_dna_planner_bam::stage_api::local_contamination_plan(repo_root).map(|_| ())
        }
        "bam.genotyping" => {
            #[cfg(feature = "bam_downstream")]
            {
                return bijux_dna_planner_bam::stage_api::local_genotyping_plan(repo_root)
                    .map(|_| ());
            }
            #[cfg(not(feature = "bam_downstream"))]
            {
                Err(anyhow!("bam.genotyping probe requires the `bam_downstream` feature"))
            }
        }
        "bam.haplogroups" => {
            #[cfg(feature = "bam_downstream")]
            {
                return bijux_dna_planner_bam::stage_api::local_haplogroups_plan(repo_root)
                    .map(|_| ());
            }
            #[cfg(not(feature = "bam_downstream"))]
            {
                Err(anyhow!("bam.haplogroups probe requires the `bam_downstream` feature"))
            }
        }
        "bam.recalibration" => {
            bijux_dna_planner_bam::stage_api::local_recalibration_smoke_plans(repo_root).map(|_| ())
        }
        _ => Err(anyhow!("unsupported adapter missing-input planner stage `{}`", probe.stage_id)),
    }
}

fn load_governed_config(repo_root: &Path, path: &str) -> Result<Value> {
    let full_path = repo_root.join(path);
    let raw = std::fs::read_to_string(&full_path)
        .with_context(|| format!("read {}", full_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", full_path.display()))
}

fn root_table_mut(value: &mut Value) -> Result<&mut toml::map::Map<String, Value>> {
    value.as_table_mut().ok_or_else(|| anyhow!("probe config root must be a TOML table"))
}

fn set_string(table: &mut toml::map::Map<String, Value>, key: &str, value: String) {
    table.insert(key.to_string(), Value::String(value));
}

fn rewrite_path_field(
    table: &mut toml::map::Map<String, Value>,
    key: &str,
    repo_root: &Path,
    missing_path: PathBuf,
    use_missing_path: bool,
) -> Result<()> {
    let original = table
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("probe config missing string field `{key}`"))?;
    let path =
        if use_missing_path { missing_path } else { absolute_config_path(repo_root, original) };
    set_string(table, key, path.display().to_string());
    Ok(())
}

fn rewrite_first_array_path_entry(
    table: &mut toml::map::Map<String, Value>,
    key: &str,
    repo_root: &Path,
    missing_path: PathBuf,
    use_missing_path: bool,
) -> Result<()> {
    let array = table
        .get_mut(key)
        .and_then(Value::as_array_mut)
        .ok_or_else(|| anyhow!("probe config missing array field `{key}`"))?;
    let original = array
        .first()
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("probe config field `{key}` must contain a string entry"))?;
    let path =
        if use_missing_path { missing_path } else { absolute_config_path(repo_root, original) };
    array.clear();
    array.push(Value::String(path.display().to_string()));
    Ok(())
}

fn absolute_config_path(repo_root: &Path, raw: &str) -> PathBuf {
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    }
}

fn missing_path_for_role(
    probe_root: &Path,
    probe: &AdapterMissingInputProbe,
    role: &str,
) -> PathBuf {
    probe_root
        .join("artifacts/bench-readiness/missing-inputs")
        .join(probe.stage_id.replace('.', "/"))
        .join(probe.tool_id)
        .join(role)
}

fn readiness_output_dir(
    probe_kind: &str,
    stage_id: &str,
    tool_id: &str,
    missing_input_role: &str,
) -> String {
    format!("target/bench-readiness/{probe_kind}/{stage_id}/{tool_id}/missing-{missing_input_role}")
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

fn adapter_missing_input_probes() -> Vec<AdapterMissingInputProbe> {
    let mut probes = Vec::new();
    probes.extend([
        fastq_probe(
            "fastq.deplete_host",
            "bowtie2",
            "input_r1",
            "fastq",
            "input FASTQ is missing",
            "benchmarks/configs/local/fastq-deplete-host.toml",
        ),
        fastq_probe(
            "fastq.deplete_host",
            "bowtie2",
            "reference_index",
            "reference",
            "reference index prefix is incomplete",
            "benchmarks/configs/local/fastq-deplete-host.toml",
        ),
        fastq_probe(
            "fastq.deplete_reference_contaminants",
            "bowtie2",
            "input_r1",
            "fastq",
            "input FASTQ is missing",
            "benchmarks/configs/local/fastq-deplete-reference-contaminants.toml",
        ),
        fastq_probe(
            "fastq.deplete_reference_contaminants",
            "bowtie2",
            "reference_index",
            "reference",
            "reference index prefix is incomplete",
            "benchmarks/configs/local/fastq-deplete-reference-contaminants.toml",
        ),
        fastq_probe(
            "fastq.deplete_rrna",
            "sortmerna",
            "input_r1",
            "fastq",
            "input FASTQ is missing",
            "benchmarks/configs/local/fastq-deplete-rrna.toml",
        ),
        fastq_probe(
            "fastq.deplete_rrna",
            "sortmerna",
            "rrna_db",
            "reference",
            "rRNA reference is missing",
            "benchmarks/configs/local/fastq-deplete-rrna.toml",
        ),
        fastq_probe(
            "fastq.screen_taxonomy",
            "centrifuge",
            "input_r1",
            "fastq",
            "input FASTQ is missing",
            "benchmarks/configs/local/fastq-screen-taxonomy.toml",
        ),
        fastq_probe(
            "fastq.screen_taxonomy",
            "centrifuge",
            "database_root",
            "database",
            "taxonomy database root is missing",
            "benchmarks/configs/local/fastq-screen-taxonomy.toml",
        ),
        fastq_probe(
            "fastq.screen_taxonomy",
            "kaiju",
            "input_r1",
            "fastq",
            "input FASTQ is missing",
            "benchmarks/configs/local/fastq-screen-taxonomy.toml",
        ),
        fastq_probe(
            "fastq.screen_taxonomy",
            "kaiju",
            "database_root",
            "database",
            "taxonomy database root is missing",
            "benchmarks/configs/local/fastq-screen-taxonomy.toml",
        ),
        fastq_probe(
            "fastq.screen_taxonomy",
            "kraken2",
            "input_r1",
            "fastq",
            "input FASTQ is missing",
            "benchmarks/configs/local/fastq-screen-taxonomy.toml",
        ),
        fastq_probe(
            "fastq.screen_taxonomy",
            "kraken2",
            "database_root",
            "database",
            "taxonomy database root is missing",
            "benchmarks/configs/local/fastq-screen-taxonomy.toml",
        ),
        fastq_probe(
            "fastq.screen_taxonomy",
            "krakenuniq",
            "input_r1",
            "fastq",
            "input FASTQ is missing",
            "benchmarks/configs/local/fastq-screen-taxonomy.toml",
        ),
        fastq_probe(
            "fastq.screen_taxonomy",
            "krakenuniq",
            "database_root",
            "database",
            "taxonomy database root is missing",
            "benchmarks/configs/local/fastq-screen-taxonomy.toml",
        ),
        bam_ready_probe(
            "bam.contamination",
            "contammix",
            "bam",
            "bam",
            "bam is missing",
            "benchmarks/configs/local/bam-contamination.toml",
        ),
        bam_ready_probe(
            "bam.contamination",
            "contammix",
            "reference_fasta",
            "reference",
            "reference FASTA is missing",
            "benchmarks/configs/local/bam-contamination.toml",
        ),
        bam_ready_probe(
            "bam.contamination",
            "contammix",
            "reference_panel",
            "reference",
            "reference panel is missing",
            "benchmarks/configs/local/bam-contamination.toml",
        ),
        bam_ready_probe(
            "bam.contamination",
            "schmutzi",
            "bam",
            "bam",
            "bam is missing",
            "benchmarks/configs/local/bam-contamination.toml",
        ),
        bam_ready_probe(
            "bam.contamination",
            "schmutzi",
            "reference_fasta",
            "reference",
            "reference FASTA is missing",
            "benchmarks/configs/local/bam-contamination.toml",
        ),
        bam_ready_probe(
            "bam.contamination",
            "schmutzi",
            "reference_panel",
            "reference",
            "reference panel is missing",
            "benchmarks/configs/local/bam-contamination.toml",
        ),
        bam_ready_probe(
            "bam.contamination",
            "verifybamid2",
            "bam",
            "bam",
            "bam is missing",
            "benchmarks/configs/local/bam-contamination.toml",
        ),
        bam_ready_probe(
            "bam.contamination",
            "verifybamid2",
            "reference_fasta",
            "reference",
            "reference FASTA is missing",
            "benchmarks/configs/local/bam-contamination.toml",
        ),
        bam_ready_probe(
            "bam.contamination",
            "verifybamid2",
            "reference_panel",
            "reference",
            "reference panel is missing",
            "benchmarks/configs/local/bam-contamination.toml",
        ),
        bam_smoke_probe(
            "bam.recalibration",
            "gatk",
            "bam",
            "bam",
            "BAM fixture is missing",
            "benchmarks/configs/local/bam-recalibration.toml",
        ),
        bam_smoke_probe(
            "bam.recalibration",
            "gatk",
            "reference",
            "reference",
            "reference fixture is missing",
            "benchmarks/configs/local/bam-recalibration.toml",
        ),
        bam_smoke_probe(
            "bam.recalibration",
            "gatk",
            "known_sites",
            "reference",
            "known-sites fixture is missing",
            "benchmarks/configs/local/bam-recalibration.toml",
        ),
    ]);

    #[cfg(feature = "bam_downstream")]
    {
        probes.extend([
            bam_ready_probe(
                "bam.genotyping",
                "angsd",
                "bam",
                "bam",
                "bam is missing",
                "benchmarks/configs/local/bam-genotyping.toml",
            ),
            bam_ready_probe(
                "bam.genotyping",
                "angsd",
                "reference_fasta",
                "reference",
                "reference FASTA is missing",
                "benchmarks/configs/local/bam-genotyping.toml",
            ),
            bam_ready_probe(
                "bam.genotyping",
                "angsd",
                "sites_vcf",
                "reference",
                "sites VCF is missing",
                "benchmarks/configs/local/bam-genotyping.toml",
            ),
            bam_ready_probe(
                "bam.genotyping",
                "angsd",
                "regions",
                "reference",
                "regions list is missing",
                "benchmarks/configs/local/bam-genotyping.toml",
            ),
            bam_ready_probe(
                "bam.haplogroups",
                "yleaf",
                "bam",
                "bam",
                "bam is missing",
                "benchmarks/configs/local/bam-haplogroups.toml",
            ),
            bam_ready_probe(
                "bam.haplogroups",
                "yleaf",
                "reference_fasta",
                "reference",
                "reference FASTA is missing",
                "benchmarks/configs/local/bam-haplogroups.toml",
            ),
            bam_ready_probe(
                "bam.haplogroups",
                "yleaf",
                "reference_panel",
                "reference",
                "reference panel is missing",
                "benchmarks/configs/local/bam-haplogroups.toml",
            ),
        ]);
    }

    probes
}

fn fastq_probe(
    stage_id: &'static str,
    tool_id: &'static str,
    missing_input_role: &'static str,
    missing_input_class: &'static str,
    expected_error_fragment: &'static str,
    config_path: &'static str,
) -> AdapterMissingInputProbe {
    AdapterMissingInputProbe {
        domain: FASTQ_DOMAIN,
        stage_id,
        tool_id,
        probe_kind: PROBE_KIND_LOCAL_READY,
        config_path,
        missing_input_role,
        missing_input_class,
        expected_error_fragment,
    }
}

fn bam_ready_probe(
    stage_id: &'static str,
    tool_id: &'static str,
    missing_input_role: &'static str,
    missing_input_class: &'static str,
    expected_error_fragment: &'static str,
    config_path: &'static str,
) -> AdapterMissingInputProbe {
    AdapterMissingInputProbe {
        domain: BAM_DOMAIN,
        stage_id,
        tool_id,
        probe_kind: PROBE_KIND_LOCAL_READY,
        config_path,
        missing_input_role,
        missing_input_class,
        expected_error_fragment,
    }
}

fn bam_smoke_probe(
    stage_id: &'static str,
    tool_id: &'static str,
    missing_input_role: &'static str,
    missing_input_class: &'static str,
    expected_error_fragment: &'static str,
    config_path: &'static str,
) -> AdapterMissingInputProbe {
    AdapterMissingInputProbe {
        domain: BAM_DOMAIN,
        stage_id,
        tool_id,
        probe_kind: PROBE_KIND_LOCAL_SMOKE,
        config_path,
        missing_input_role,
        missing_input_class,
        expected_error_fragment,
    }
}
