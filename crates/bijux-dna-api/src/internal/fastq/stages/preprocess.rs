use bijux_dna_runtime::{
    attrs_from_json, build_telemetry_adapter, TelemetryEventName, TelemetryEventV1,
};
use std::collections::HashMap;

use crate::qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
use crate::tooling::{ensure_bench_runner, filter_tools_by_role, load_workspace_registry};
use crate::{execution_kernel, execution_kernel::NetworkPolicy};
use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::load::sqlite::bench_results_fastq::SqliteBenchResultsRepository;
use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::ContainerImageRefV1;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::RawFailure;
use bijux_dna_planner_fastq::{
    apply_preprocess_policy, preprocess_decisions, resolve_preprocess_pipeline,
    select_preprocess_stage_tools, FastqPlanConfig, FastqPlanner, StageToolSelection,
};
use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
use bijux_dna_runner::backend::docker::executor::resolve_image_for_run;
use bijux_dna_runner::step_runner::StageResultV1;
use bijux_dna_runtime::recording::run_artifacts_dir_for_out;
use bijux_dna_runtime::recording::write_telemetry_event;

use crate::internal::handlers::fastq::jobs::bench_jobs;
use crate::internal::handlers::fastq::summary::{
    render_run_summary, report_stage_step, write_run_manifest, write_scientific_provenance,
    StageExecutionSummary,
};
use crate::internal::handlers::fastq::write_explain_plan_json;
use crate::internal::handlers::fastq::{
    STAGE_PREPROCESS_SUMMARY, STAGE_REPORT_QC, STAGE_TRIM_READS,
};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir};
use bijux_dna_planner_fastq::scale_tool_spec_for_jobs;
use bijux_dna_planner_fastq::stage_api::{
    adapter_bank_context, contaminant_bank_context, polyx_bank_context, polyx_unsupported_warning,
};
use std::io::BufRead;
use std::path::PathBuf;

mod amplicon_governance;
mod amplicon_runtime;
mod runtime_tail;
mod stage_artifacts;
mod stage_backend_policy;

pub(crate) use self::amplicon_governance::resolve_primer_set_governance;
pub use self::runtime_tail::{bench_fastq_preprocess, fastq_preprocess_run};

use self::amplicon_governance::*;
use self::amplicon_runtime::*;
use self::runtime_tail::*;
use self::stage_artifacts::*;
use self::stage_backend_policy::*;

#[derive(Debug, Clone, serde::Serialize)]
struct FastqInvariantsReport {
    schema_version: String,
    r1: FastqFileInvariant,
    r2: Option<FastqFileInvariant>,
    paired_consistent: bool,
    paired_reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct FastqFileInvariant {
    path: PathBuf,
    gzip: bool,
    gzip_valid: bool,
    read_count: u64,
    read_length_min: usize,
    read_length_max: usize,
    read_length_mean: f64,
    read_length_histogram: std::collections::BTreeMap<String, u64>,
    qscore_ascii_min: u8,
    qscore_ascii_max: u8,
    quality_encoding: String,
    quality_encoding_confidence: String,
}

#[derive(Debug, Clone)]
struct FastqScanStats {
    read_count: u64,
    read_length_min: usize,
    read_length_max: usize,
    read_length_mean: f64,
    read_length_histogram: std::collections::BTreeMap<String, u64>,
    qscore_ascii_min: u8,
    qscore_ascii_max: u8,
    first_headers: Vec<String>,
}

fn histogram_bucket_for_read_length(len: usize) -> String {
    if len < 50 {
        "lt50".to_string()
    } else if len < 75 {
        "50_74".to_string()
    } else if len < 100 {
        "75_99".to_string()
    } else if len < 151 {
        "100_150".to_string()
    } else if len < 251 {
        "151_250".to_string()
    } else {
        "ge251".to_string()
    }
}

fn fastq_is_gzip(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.eq_ignore_ascii_case("gz"))
}

fn validate_gzip_path(path: &std::path::Path) -> Result<bool> {
    if !fastq_is_gzip(path) {
        return Ok(true);
    }
    let mut magic = [0_u8; 2];
    let mut file = std::fs::File::open(path)?;
    if file.read_exact(&mut magic).is_err() || magic != [0x1f, 0x8b] {
        return Ok(false);
    }
    let args = vec!["-t".to_string(), path.to_string_lossy().into_owned()];
    let output = bijux_dna_runner::command_runner::run_command("gzip", &args);
    Ok(output.map(|result| result.exit_code == 0).unwrap_or(false))
}

fn quality_encoding_confidence(min_ascii: u8, max_ascii: u8) -> String {
    if (33..=59).contains(&min_ascii) && max_ascii <= 74 {
        "high".to_string()
    } else if min_ascii >= 64 && max_ascii <= 104 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

fn open_fastq_lines(path: &std::path::Path) -> Result<Box<dyn Iterator<Item = String>>> {
    if path
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.eq_ignore_ascii_case("gz"))
    {
        let args = vec!["-cd".to_string(), path.to_string_lossy().into_owned()];
        let output = bijux_dna_runner::command_runner::run_command("gzip", &args)
            .with_context(|| format!("gzip -cd {}", path.display()))?;
        if output.exit_code != 0 {
            return Err(anyhow!(
                "failed to decompress {}: {}",
                path.display(),
                output.stderr
            ));
        }
        let text = output.stdout;
        let lines = text.lines().map(ToString::to_string).collect::<Vec<_>>();
        return Ok(Box::new(lines.into_iter()));
    }
    let f = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = std::io::BufReader::new(f);
    let mut lines = Vec::new();
    for line in reader.lines() {
        lines.push(line?);
    }
    Ok(Box::new(lines.into_iter()))
}

fn quality_encoding_from_ascii(min_ascii: u8, max_ascii: u8) -> String {
    if min_ascii >= 33 && max_ascii <= 74 {
        "phred+33".to_string()
    } else if min_ascii >= 64 && max_ascii <= 104 {
        "phred+64".to_string()
    } else {
        "unclassified".to_string()
    }
}

fn scan_fastq_invariants(path: &std::path::Path) -> Result<FastqScanStats> {
    let mut read_count = 0_u64;
    let mut len_min = usize::MAX;
    let mut len_max = 0_usize;
    let mut len_total = 0_u64;
    let mut q_min = u8::MAX;
    let mut q_max = 0_u8;
    let mut first_headers = Vec::new();
    let mut read_length_histogram = std::collections::BTreeMap::<String, u64>::new();
    let mut i = 0_u64;
    let mut it = open_fastq_lines(path)?;
    loop {
        let h = it.next();
        let seq = it.next();
        let plus = it.next();
        let qual = it.next();
        let (Some(h), Some(seq), Some(plus), Some(qual)) = (h, seq, plus, qual) else {
            break;
        };
        if !h.starts_with('@') || !plus.starts_with('+') {
            return Err(anyhow!(
                "invalid FASTQ record framing in {}",
                path.display()
            ));
        }
        let l = seq.len();
        len_min = len_min.min(l);
        len_max = len_max.max(l);
        len_total += l as u64;
        *read_length_histogram
            .entry(histogram_bucket_for_read_length(l))
            .or_insert(0) += 1;
        for c in qual.bytes() {
            q_min = q_min.min(c);
            q_max = q_max.max(c);
        }
        if i < 16 {
            first_headers.push(h);
        }
        read_count += 1;
        i += 1;
    }
    if read_count == 0 {
        return Err(anyhow!("no reads detected in {}", path.display()));
    }
    Ok(FastqScanStats {
        read_count,
        read_length_min: len_min,
        read_length_max: len_max,
        read_length_mean: u64_to_f64(len_total) / u64_to_f64(read_count),
        read_length_histogram,
        qscore_ascii_min: q_min,
        qscore_ascii_max: q_max,
        first_headers,
    })
}

fn normalize_pair_header(header: &str) -> String {
    let core = header
        .trim_start_matches('@')
        .split_whitespace()
        .next()
        .unwrap_or(header);
    core.trim_end_matches("/1")
        .trim_end_matches("/2")
        .to_string()
}

fn write_fastq_entry_invariants(
    root: &std::path::Path,
    r1: &std::path::Path,
    r2: Option<&std::path::Path>,
) -> Result<FastqInvariantsReport> {
    let r1s = scan_fastq_invariants(r1)?;
    let r1_gzip = fastq_is_gzip(r1);
    let r1_gzip_valid = validate_gzip_path(r1)?;
    if r1_gzip && !r1_gzip_valid {
        return Err(anyhow!("invalid gzip FASTQ stream: {}", r1.display()));
    }
    let r1_inv = FastqFileInvariant {
        path: r1.to_path_buf(),
        gzip: r1_gzip,
        gzip_valid: r1_gzip_valid,
        read_count: r1s.read_count,
        read_length_min: r1s.read_length_min,
        read_length_max: r1s.read_length_max,
        read_length_mean: r1s.read_length_mean,
        read_length_histogram: r1s.read_length_histogram.clone(),
        qscore_ascii_min: r1s.qscore_ascii_min,
        qscore_ascii_max: r1s.qscore_ascii_max,
        quality_encoding: quality_encoding_from_ascii(r1s.qscore_ascii_min, r1s.qscore_ascii_max),
        quality_encoding_confidence: quality_encoding_confidence(
            r1s.qscore_ascii_min,
            r1s.qscore_ascii_max,
        ),
    };
    let (r2_inv, paired_consistent, paired_reason) = if let Some(r2_path) = r2 {
        let r2s = scan_fastq_invariants(r2_path)?;
        let r2_gzip = fastq_is_gzip(r2_path);
        let r2_gzip_valid = validate_gzip_path(r2_path)?;
        if r2_gzip && !r2_gzip_valid {
            return Err(anyhow!("invalid gzip FASTQ stream: {}", r2_path.display()));
        }
        let mut ok = r1s.read_count == r2s.read_count;
        let mut reason = None;
        if ok {
            for (lhs, rhs) in r1s.first_headers.iter().zip(r2s.first_headers.iter()) {
                if normalize_pair_header(lhs) != normalize_pair_header(rhs) {
                    ok = false;
                    reason = Some("header pairing mismatch".to_string());
                    break;
                }
            }
        } else {
            reason = Some("read count mismatch between R1 and R2".to_string());
        }
        (
            Some(FastqFileInvariant {
                path: r2_path.to_path_buf(),
                gzip: r2_gzip,
                gzip_valid: r2_gzip_valid,
                read_count: r2s.read_count,
                read_length_min: r2s.read_length_min,
                read_length_max: r2s.read_length_max,
                read_length_mean: r2s.read_length_mean,
                read_length_histogram: r2s.read_length_histogram.clone(),
                qscore_ascii_min: r2s.qscore_ascii_min,
                qscore_ascii_max: r2s.qscore_ascii_max,
                quality_encoding: quality_encoding_from_ascii(
                    r2s.qscore_ascii_min,
                    r2s.qscore_ascii_max,
                ),
                quality_encoding_confidence: quality_encoding_confidence(
                    r2s.qscore_ascii_min,
                    r2s.qscore_ascii_max,
                ),
            }),
            ok,
            reason,
        )
    } else {
        (None, true, None)
    };
    let report = FastqInvariantsReport {
        schema_version: "bijux.fastq.invariants.v1".to_string(),
        r1: r1_inv,
        r2: r2_inv,
        paired_consistent,
        paired_reason,
    };
    bijux_dna_infra::atomic_write_json(&root.join("fastq_invariants.json"), &report)
        .context("write fastq_invariants.json")?;
    Ok(report)
}

fn maybe_write_fastq_coverage_classifier(
    root: &std::path::Path,
    invariants: &FastqInvariantsReport,
) -> Result<()> {
    let expected_genome_size_bp = std::env::var("BIJUX_EXPECTED_GENOME_SIZE_BP")
        .ok()
        .and_then(|v| v.parse::<u64>().ok());
    let Some(genome_bp) = expected_genome_size_bp else {
        return Ok(());
    };
    if genome_bp == 0 {
        return Ok(());
    }
    let reads = invariants.r1.read_count + invariants.r2.as_ref().map_or(0, |r2| r2.read_count);
    let mean_len = if let Some(r2) = invariants.r2.as_ref() {
        (invariants.r1.read_length_mean + r2.read_length_mean) / 2.0
    } else {
        invariants.r1.read_length_mean
    };
    let estimated_depth_x = (reads.to_string().parse::<f64>().unwrap_or(0.0) * mean_len)
        / genome_bp.to_string().parse::<f64>().unwrap_or(1.0);
    let thresholds = load_coverage_thresholds_for_fastq("default")?;
    let (selected_regime, trigger) = if estimated_depth_x <= thresholds.gl_max {
        (
            "gl",
            format!(
                "estimated_depth_x <= gl_max_depth ({estimated_depth_x:.4} <= {})",
                thresholds.gl_max
            ),
        )
    } else if estimated_depth_x <= thresholds.pseudohaploid_max {
        (
            "pseudohaploid",
            format!(
                "gl_max_depth < estimated_depth_x <= pseudohaploid_max_depth ({} < {estimated_depth_x:.4} <= {})",
                thresholds.gl_max, thresholds.pseudohaploid_max
            ),
        )
    } else if estimated_depth_x >= thresholds.diploid_min {
        (
            "diploid",
            format!(
                "estimated_depth_x >= diploid_min_depth ({estimated_depth_x:.4} >= {})",
                thresholds.diploid_min
            ),
        )
    } else {
        (
            "pseudohaploid",
            "fallback band between pseudohaploid_max_depth and diploid_min_depth".to_string(),
        )
    };
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.coverage_classifier.v1",
        "expected_genome_size_bp": genome_bp,
        "reads": reads,
        "mean_read_length": mean_len,
        "estimated_depth_x": estimated_depth_x,
        "selected_regime": selected_regime,
        "trigger": trigger,
        "thresholds_used": {
            "gl_max_depth": thresholds.gl_max,
            "pseudohaploid_max_depth": thresholds.pseudohaploid_max,
            "diploid_min_depth": thresholds.diploid_min,
        }
    });
    bijux_dna_infra::atomic_write_json(&root.join("coverage_regime.fastq.json"), &payload)
        .context("write coverage_regime.fastq.json")?;
    bijux_dna_infra::atomic_write_json(&root.join("coverage_explain.fastq.json"), &payload)
        .context("write coverage_explain.fastq.json")
}

#[derive(Debug, Clone, Copy)]
struct FastqCoverageThresholds {
    gl_max: f64,
    pseudohaploid_max: f64,
    diploid_min: f64,
}

fn load_coverage_thresholds_for_fastq(profile: &str) -> Result<FastqCoverageThresholds> {
    let root = crate::support::repo_root::resolve_repo_root()?;
    let raw = std::fs::read_to_string(root.join("configs/runtime/coverage_regimes.toml"))?;
    let parsed: toml::Value = toml::from_str(&raw)?;
    let decision = parsed
        .get("decision")
        .and_then(|v| v.get("coverage_regime"))
        .ok_or_else(|| anyhow!("missing decision.coverage_regime"))?;
    let base = decision
        .get("thresholds")
        .ok_or_else(|| anyhow!("missing decision.coverage_regime.thresholds"))?;
    let profile_thresholds = decision
        .get("profiles")
        .and_then(|v| v.get(profile))
        .unwrap_or(base);
    let read_f = |key: &str| -> Result<f64> {
        profile_thresholds
            .get(key)
            .and_then(toml::Value::as_float)
            .or_else(|| {
                profile_thresholds
                    .get(key)
                    .and_then(toml::Value::as_integer)
                    .and_then(|v| v.to_string().parse::<f64>().ok())
            })
            .ok_or_else(|| anyhow!("missing threshold key `{key}`"))
    };
    Ok(FastqCoverageThresholds {
        gl_max: read_f("gl_max_depth")?,
        pseudohaploid_max: read_f("pseudohaploid_max_depth")?,
        diploid_min: read_f("diploid_min_depth")?,
    })
}

fn write_stage_path_contract(
    stage_root: &std::path::Path,
    stage_id: &str,
    planned: &ExecutionStep,
    is_paired: bool,
) -> Result<()> {
    bijux_dna_infra::ensure_dir(stage_root).context("create stage root for path contract")?;
    let outputs = planned
        .io
        .outputs
        .iter()
        .map(|x| {
            serde_json::json!({
                "name": x.name,
                "role": x.role.as_str(),
                "path": x.path
            })
        })
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.path_contract.v1",
        "stage_id": stage_id,
        "layout": if is_paired { "pe" } else { "se" },
        "deterministic_root": stage_root,
        "intermediate_root": stage_root.join("tmp"),
        "intermediate_paths": {
            "stdout_log": stage_root.join("stdout.log"),
            "stderr_log": stage_root.join("stderr.log"),
            "runtime_provenance": stage_root.join("runtime_provenance.json"),
            "resume_contract": stage_root.join("stage.resume_contract.json"),
        },
        "outputs": outputs,
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.path_contract.json"), &payload)
        .context("write stage.path_contract.json")
}

fn capture_tool_version(stage_root: &std::path::Path, tool_bin: Option<&str>) -> Result<()> {
    let (declared_tool, ok, raw) = if let Some(tool_bin) = tool_bin.filter(|value| !value.trim().is_empty()) {
        let args = vec!["--version".to_string()];
        let output = bijux_dna_runner::command_runner::run_command(tool_bin, &args);
        let (ok, raw) = match output {
            Ok(out) => {
                let raw = if out.stdout.is_empty() {
                    out.stderr
                } else {
                    out.stdout
                };
                (out.exit_code == 0, raw)
            }
            Err(err) => (false, format!("failed to execute --version: {err}")),
        };
        (tool_bin, ok, raw)
    } else {
        ("", false, "tool command not declared in execution template".to_string())
    };
    let line = raw
        .lines()
        .find(|x| !x.trim().is_empty())
        .unwrap_or("")
        .trim();
    let tokenized = line
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '(' || c == ')')
        .filter(|x| !x.trim().is_empty())
        .collect::<Vec<_>>();
    let version = tokenized
        .iter()
        .find_map(|tok| {
            let t = tok.trim_start_matches('v').trim_start_matches('V');
            if t.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                Some(t.to_string())
            } else {
                None
            }
        });
    let payload = serde_json::json!({
        "schema_version": "bijux.tool_version_capture.v1",
        "tool": declared_tool,
        "ok": ok,
        "raw": raw,
        "parsed": {
            "first_line": line,
            "version": version
        }
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.tool_version.json"), &payload)
        .context("write stage.tool_version.json")
}

use std::io::Read;

pub(crate) fn materialize_amplicon_stage_outputs_for_bench(
    stage_root: &std::path::Path,
    planned: &ExecutionStep,
) -> Result<serde_json::Value> {
    materialize_amplicon_stage_outputs(stage_root, planned)
}

pub(crate) fn enforce_amplicon_qc_thresholds_for_bench(
    stage_root: &std::path::Path,
    stage_id: &str,
    metrics: &serde_json::Value,
) -> Result<()> {
    enforce_amplicon_qc_thresholds(stage_root, stage_id, metrics)
}

fn u64_to_f64(v: u64) -> f64 {
    v.to_string().parse::<f64>().unwrap_or(0.0)
}
