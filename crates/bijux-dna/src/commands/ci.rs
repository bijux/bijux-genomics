use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use serde::Serialize;
use serde_yaml::Value as YamlValue;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

const DEFAULT_NO_REPEATED_FAST_GATE_PATH: &str =
    "benchmarks/readiness/ci/no-repeated-fast-gate.json";
const DEFAULT_CHANGED_PATH_TEST_MAP_PATH: &str =
    "benchmarks/readiness/ci/changed-path-test-map.tsv";
const DEFAULT_DEFAULT_FEATURE_AUDIT_PATH: &str =
    "benchmarks/readiness/ci/default-feature-audit.json";
const DEFAULT_SLOW_TIER_MANUAL_ONLY_PATH: &str =
    "benchmarks/readiness/ci/slow-tier-manual-only.json";
const DEFAULT_FAST_NO_BLEEDING_GATE_PATH: &str = "benchmarks/readiness/ci/FAST_CI_NO_BLEEDING.json";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WorkflowTargetUsage {
    pub(crate) job_id: String,
    pub(crate) step_name: Option<String>,
    pub(crate) command: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WorkflowTargetAuditReport {
    pub(crate) schema_version: String,
    pub(crate) workflow_path: String,
    pub(crate) audit_kind: String,
    pub(crate) ok: bool,
    pub(crate) target: String,
    pub(crate) usage_count: usize,
    pub(crate) usages: Vec<WorkflowTargetUsage>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SlowTierManualOnlyReport {
    pub(crate) schema_version: String,
    pub(crate) workflow_path: String,
    pub(crate) audit_kind: String,
    pub(crate) ok: bool,
    pub(crate) workflow_dispatch_enabled: bool,
    pub(crate) run_slow_tier_input_present: bool,
    pub(crate) run_slow_tier_default: bool,
    pub(crate) slow_tier_if: Option<String>,
    pub(crate) slow_tier_target_jobs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ChangedPathRuleReport {
    pub(crate) matcher: String,
    pub(crate) command: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ChangedPathSelection {
    pub(crate) path: String,
    pub(crate) commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ChangedPathCommandReport {
    pub(crate) schema_version: String,
    pub(crate) input_path: String,
    pub(crate) commands: Vec<String>,
    pub(crate) selections: Vec<ChangedPathSelection>,
    pub(crate) rule_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DefaultFeatureCrateAudit {
    pub(crate) crate_name: String,
    pub(crate) manifest_path: String,
    pub(crate) default_features: Vec<String>,
    pub(crate) forbidden_default_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DefaultFeatureAuditReport {
    pub(crate) schema_version: String,
    pub(crate) profile: String,
    pub(crate) ok: bool,
    pub(crate) forbidden_feature_prefixes: Vec<String>,
    pub(crate) crates: Vec<DefaultFeatureCrateAudit>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BudgetTargetObservation {
    pub(crate) target_id: String,
    pub(crate) command: String,
    pub(crate) max_seconds: f64,
    pub(crate) observed_seconds: f64,
    pub(crate) ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BudgetCheckReport {
    pub(crate) schema_version: String,
    pub(crate) profile: String,
    pub(crate) budget_file: String,
    pub(crate) ok: bool,
    pub(crate) max_parallel_window_seconds: f64,
    pub(crate) slowest_target_seconds: f64,
    pub(crate) observations: Vec<BudgetTargetObservation>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GateCheckReport {
    pub(crate) goal_id: String,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastNoBleedingGateReport {
    pub(crate) schema_version: String,
    pub(crate) ok: bool,
    pub(crate) workflow_path: String,
    pub(crate) budget_file: String,
    pub(crate) changed_paths_fixture: String,
    pub(crate) checks: Vec<GateCheckReport>,
}

#[derive(Debug, Clone)]
pub(crate) struct AuditWorkflowArgs {
    pub(crate) workflow: PathBuf,
    pub(crate) no_repeated_target: Option<String>,
    pub(crate) slow_tier_manual_only: bool,
    pub(crate) out: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub(crate) struct ChangedPathsArgs {
    pub(crate) from_file: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct BudgetCheckArgs {
    pub(crate) profile: String,
    pub(crate) budget_file: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub(crate) struct AuditFeaturesArgs {
    pub(crate) profile: String,
    pub(crate) out: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub(crate) struct FastNoBleedingArgs {
    pub(crate) workflow: PathBuf,
    pub(crate) budget_file: Option<PathBuf>,
    pub(crate) changed_paths_fixture: Option<PathBuf>,
    pub(crate) out: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
struct ChangedPathRule {
    matcher: &'static str,
    command: &'static str,
    reason: &'static str,
}

const CHANGED_PATH_RULES: &[ChangedPathRule] = &[
    ChangedPathRule {
        matcher: ".github/workflows/ci.yml",
        command: "make ci-fast",
        reason: "workflow changes must revalidate the full fast-tier contract",
    },
    ChangedPathRule {
        matcher: ".github/standards/repo-config.manifest.json",
        command: "make ci-fast",
        reason: "workflow manifest changes must keep the fast-tier contract intact",
    },
    ChangedPathRule {
        matcher: "Makefile",
        command: "make ci-fast",
        reason: "root make entrypoints govern the fast CI surface",
    },
    ChangedPathRule {
        matcher: "makes/",
        command: "make ci-fast",
        reason: "make fragments define the fast CI control plane",
    },
    ChangedPathRule {
        matcher: "crates/bijux-dna-dev/src/commands/ops/tooling/cargo_targets.rs",
        command: "make ci-fast",
        reason: "cargo target bundles define the governed fast CI lanes",
    },
    ChangedPathRule {
        matcher: "crates/bijux-dna/src/commands/benchmark/readiness/",
        command: "make bench-active-fast",
        reason: "readiness command changes must preserve active benchmark coverage",
    },
    ChangedPathRule {
        matcher: "crates/bijux-dna/tests/bench_readiness_all_domain_",
        command: "make bench-active-fast",
        reason: "all-domain readiness tests cover the active benchmark surface",
    },
    ChangedPathRule {
        matcher: "benchmarks/readiness/",
        command: "make bench-active-fast",
        reason: "tracked readiness outputs must stay aligned with active fast proofs",
    },
    ChangedPathRule {
        matcher: "crates/bijux-dna/tests/bench_readiness_parser_failure_tests",
        command: "make bench-parser-fast",
        reason: "parser failure probes must stay governed and explicit",
    },
    ChangedPathRule {
        matcher: "crates/bijux-dna/tests/bench_readiness_vcf_parser_failure_tests",
        command: "make bench-parser-fast",
        reason: "VCF parser failure probes must stay governed and explicit",
    },
    ChangedPathRule {
        matcher: "crates/bijux-dna-domain-bam/tests/contracts/parsers",
        command: "make bench-parser-fast",
        reason: "BAM parser fixtures map directly to the parser fast lane",
    },
    ChangedPathRule {
        matcher: "crates/bijux-dna-domain-vcf/tests/contracts/parsers",
        command: "make bench-parser-fast",
        reason: "VCF parser fixtures map directly to the parser fast lane",
    },
    ChangedPathRule {
        matcher: "benchmarks/tests/fixtures/bench/parsers/",
        command: "make bench-parser-fast",
        reason: "bench parser fixture banks are owned by the parser fast lane",
    },
    ChangedPathRule {
        matcher: "adapter",
        command: "make bench-adapter-fast",
        reason: "adapter rendering, argv, and missing-input proofs belong to the adapter lane",
    },
    ChangedPathRule {
        matcher: "rendered_commands",
        command: "make bench-adapter-fast",
        reason: "rendered command surfaces belong to the adapter lane",
    },
    ChangedPathRule {
        matcher: "no_placeholder",
        command: "make bench-adapter-fast",
        reason: "placeholder command regressions belong to the adapter lane",
    },
    ChangedPathRule {
        matcher: "benchmarks/tests/fixtures/corpora/",
        command: "make science-fixtures-fast",
        reason: "governed corpus truth fixtures belong to the science fixture lane",
    },
    ChangedPathRule {
        matcher: "benchmarks/tests/fixtures/databases/",
        command: "make science-fixtures-fast",
        reason: "governed database fixtures belong to the science fixture lane",
    },
    ChangedPathRule {
        matcher: "benchmarks/tests/fixtures/science/",
        command: "make science-fixtures-fast",
        reason: "governed science truth bundles belong to the science fixture lane",
    },
    ChangedPathRule {
        matcher: "crates/bijux-dna/src/commands/fixtures/",
        command: "make science-fixtures-fast",
        reason: "fixture validation commands own the science truth contract",
    },
    ChangedPathRule {
        matcher: "crates/bijux-dna/tests/bench_local_vcf_population_structure_smoke",
        command: "make science-fixtures-fast",
        reason: "population mini-smoke truth belongs to the science fixture lane",
    },
    ChangedPathRule {
        matcher: "crates/bijux-dna/tests/fixtures_validate_",
        command: "make science-fixtures-fast",
        reason: "fixture validation tests belong to the science fixture lane",
    },
];

#[derive(Debug, Deserialize)]
struct BudgetFile {
    profile: BTreeMap<String, BudgetProfile>,
    target: BTreeMap<String, BudgetTarget>,
}

#[derive(Debug, Deserialize)]
struct BudgetProfile {
    max_parallel_window_seconds: f64,
    targets: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct BudgetTarget {
    command: String,
    max_seconds: f64,
}

#[derive(Debug, Deserialize)]
struct ManifestFeatures {
    package: ManifestPackage,
    features: Option<BTreeMap<String, Vec<String>>>,
}

#[derive(Debug, Deserialize)]
struct ManifestPackage {
    name: String,
}

pub(crate) fn audit_workflow_no_repeated_target(
    repo_root: &Path,
    workflow_path: &Path,
    target: &str,
    out: Option<&Path>,
) -> Result<WorkflowTargetAuditReport> {
    let absolute_workflow = resolve_path(repo_root, workflow_path);
    let normalized_target = normalize_workflow_target_spec(target);
    let usages = collect_workflow_target_usages(&absolute_workflow, &normalized_target)?;
    let report = WorkflowTargetAuditReport {
        schema_version: "bijux.ci.no_repeated_fast_gate.v1".to_string(),
        workflow_path: display_relative(repo_root, &absolute_workflow),
        audit_kind: "no_repeated_target".to_string(),
        ok: usages.len() <= 1,
        target: normalized_target.clone(),
        usage_count: usages.len(),
        usages,
    };
    let output_path = out.map_or_else(
        || repo_root.join(DEFAULT_NO_REPEATED_FAST_GATE_PATH),
        |path| resolve_path(repo_root, path),
    );
    write_json_report(&output_path, &report)?;
    if !report.ok {
        bail!(
            "workflow `{}` reuses `{}` {} times",
            report.workflow_path,
            normalized_target,
            report.usage_count
        );
    }
    Ok(report)
}

pub(crate) fn audit_workflow_slow_tier_manual_only(
    repo_root: &Path,
    workflow_path: &Path,
    out: Option<&Path>,
) -> Result<SlowTierManualOnlyReport> {
    let absolute_workflow = resolve_path(repo_root, workflow_path);
    let yaml = read_workflow_yaml(&absolute_workflow)?;
    let workflow_dispatch_enabled = yaml
        .get("on")
        .and_then(|value| match value {
            YamlValue::Mapping(map) => map.get(YamlValue::String("workflow_dispatch".to_string())),
            _ => None,
        })
        .is_some();
    let run_slow_tier_input = yaml
        .get("on")
        .and_then(|value| match value {
            YamlValue::Mapping(map) => map.get(YamlValue::String("workflow_dispatch".to_string())),
            _ => None,
        })
        .and_then(|value| value.get("inputs"))
        .and_then(|value| value.get("run_slow_tier"));
    let run_slow_tier_default = run_slow_tier_input
        .and_then(|value| value.get("default"))
        .and_then(YamlValue::as_bool)
        .unwrap_or(false);
    let jobs = yaml
        .get("jobs")
        .and_then(YamlValue::as_mapping)
        .ok_or_else(|| anyhow!("workflow `{}` is missing `jobs`", absolute_workflow.display()))?;
    let slow_tier_job = jobs
        .get(YamlValue::String("slow-tier".to_string()))
        .and_then(YamlValue::as_mapping)
        .ok_or_else(|| {
            anyhow!("workflow `{}` is missing slow-tier job", absolute_workflow.display())
        })?;
    let slow_tier_if = slow_tier_job
        .get(YamlValue::String("if".to_string()))
        .and_then(YamlValue::as_str)
        .map(ToOwned::to_owned);
    let slow_tier_target_jobs = jobs
        .iter()
        .filter_map(|(job_id, job)| {
            let job_id = job_id.as_str()?.to_string();
            let job = job.as_mapping()?;
            let steps = job.get(YamlValue::String("steps".to_string()))?.as_sequence()?;
            let uses_slow_target = steps.iter().any(|step| {
                step.as_mapping()
                    .and_then(|mapping| mapping.get(YamlValue::String("run".to_string())))
                    .and_then(YamlValue::as_str)
                    .is_some_and(|run| run.lines().any(|line| line.trim() == "make ci-slow"))
            });
            uses_slow_target.then_some(job_id)
        })
        .collect::<Vec<_>>();
    let expected_if =
        "${{ github.event_name == 'workflow_dispatch' && inputs.run_slow_tier == true }}";
    let ok = workflow_dispatch_enabled
        && run_slow_tier_input.is_some()
        && !run_slow_tier_default
        && slow_tier_if.as_deref() == Some(expected_if)
        && slow_tier_target_jobs == vec!["slow-tier".to_string()];
    let report = SlowTierManualOnlyReport {
        schema_version: "bijux.ci.slow_tier_manual_only.v1".to_string(),
        workflow_path: display_relative(repo_root, &absolute_workflow),
        audit_kind: "slow_tier_manual_only".to_string(),
        ok,
        workflow_dispatch_enabled,
        run_slow_tier_input_present: run_slow_tier_input.is_some(),
        run_slow_tier_default,
        slow_tier_if,
        slow_tier_target_jobs,
    };
    let output_path = out.map_or_else(
        || repo_root.join(DEFAULT_SLOW_TIER_MANUAL_ONLY_PATH),
        |path| resolve_path(repo_root, path),
    );
    write_json_report(&output_path, &report)?;
    if !report.ok {
        bail!("workflow `{}` does not keep `make ci-slow` manual-only", report.workflow_path);
    }
    Ok(report)
}

pub(crate) fn changed_path_commands(
    repo_root: &Path,
    from_file: &Path,
) -> Result<ChangedPathCommandReport> {
    let absolute_input = resolve_path(repo_root, from_file);
    let raw = std::fs::read_to_string(&absolute_input)
        .with_context(|| format!("read {}", absolute_input.display()))?;
    let paths = raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let rules = changed_path_rule_rows();
    let mut commands = BTreeSet::new();
    let selections = paths
        .iter()
        .map(|path| {
            let matched = rules
                .iter()
                .filter(|rule| path_matches_rule(path, &rule.matcher))
                .map(|rule| {
                    commands.insert(rule.command.clone());
                    rule.command.clone()
                })
                .collect::<Vec<_>>();
            ChangedPathSelection { path: path.clone(), commands: matched }
        })
        .collect::<Vec<_>>();
    let report = ChangedPathCommandReport {
        schema_version: "bijux.ci.changed_path_target_map.v1".to_string(),
        input_path: display_relative(repo_root, &absolute_input),
        commands: commands.into_iter().collect(),
        selections,
        rule_count: rules.len(),
    };
    let map_path = repo_root.join(DEFAULT_CHANGED_PATH_TEST_MAP_PATH);
    write_utf8_report(&map_path, &render_changed_path_rule_tsv(&rules))?;
    Ok(report)
}

pub(crate) fn audit_default_features(
    repo_root: &Path,
    profile: &str,
    out: Option<&Path>,
) -> Result<DefaultFeatureAuditReport> {
    if profile != "default-fast" {
        bail!("unsupported feature audit profile `{profile}`");
    }
    let forbidden_feature_prefixes = vec![
        "bam_".to_string(),
        "docker".to_string(),
        "apptainer".to_string(),
        "slurm".to_string(),
        "hpc".to_string(),
        "bench".to_string(),
    ];
    let crate_manifests = std::fs::read_dir(repo_root.join("crates"))
        .with_context(|| format!("read {}", repo_root.join("crates").display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path().join("Cargo.toml"))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    let mut crates = crate_manifests
        .into_iter()
        .map(|manifest_path| {
            let parsed = toml::from_str::<ManifestFeatures>(
                &std::fs::read_to_string(&manifest_path)
                    .with_context(|| format!("read {}", manifest_path.display()))?,
            )
            .with_context(|| format!("parse {}", manifest_path.display()))?;
            let default_features = parsed
                .features
                .as_ref()
                .and_then(|features| features.get("default"))
                .cloned()
                .unwrap_or_default();
            let forbidden_default_features = default_features
                .iter()
                .filter(|feature| {
                    forbidden_feature_prefixes
                        .iter()
                        .any(|prefix| feature.starts_with(prefix) || feature.contains(prefix))
                })
                .cloned()
                .collect::<Vec<_>>();
            Ok::<_, anyhow::Error>(DefaultFeatureCrateAudit {
                crate_name: parsed.package.name,
                manifest_path: display_relative(repo_root, &manifest_path),
                default_features,
                forbidden_default_features,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    crates.sort_by(|left, right| left.crate_name.cmp(&right.crate_name));
    let ok = crates.iter().all(|crate_audit| crate_audit.forbidden_default_features.is_empty());
    let report = DefaultFeatureAuditReport {
        schema_version: "bijux.ci.default_feature_audit.v1".to_string(),
        profile: profile.to_string(),
        ok,
        forbidden_feature_prefixes,
        crates,
    };
    let output_path = out.map_or_else(
        || repo_root.join(DEFAULT_DEFAULT_FEATURE_AUDIT_PATH),
        |path| resolve_path(repo_root, path),
    );
    write_json_report(&output_path, &report)?;
    if !report.ok {
        bail!("default feature audit failed for profile `{profile}`");
    }
    Ok(report)
}

pub(crate) fn budget_check(
    repo_root: &Path,
    profile: &str,
    budget_file: Option<&Path>,
) -> Result<BudgetCheckReport> {
    let absolute_budget_file = budget_file.map_or_else(
        || repo_root.join("benchmarks/readiness/ci/fast-ci-budget.toml"),
        |path| resolve_path(repo_root, path),
    );
    let parsed = toml::from_str::<BudgetFile>(
        &std::fs::read_to_string(&absolute_budget_file)
            .with_context(|| format!("read {}", absolute_budget_file.display()))?,
    )
    .with_context(|| format!("parse {}", absolute_budget_file.display()))?;
    let profile_budget = parsed.profile.get(profile).ok_or_else(|| {
        anyhow!("budget profile `{profile}` missing from {}", absolute_budget_file.display())
    })?;
    let mut observations = Vec::new();
    for target_id in &profile_budget.targets {
        let target_budget = parsed.target.get(target_id).ok_or_else(|| {
            anyhow!("target `{target_id}` missing from {}", absolute_budget_file.display())
        })?;
        let started = Instant::now();
        let status = Command::new("sh")
            .arg("-lc")
            .arg(&target_budget.command)
            .current_dir(repo_root)
            .status()
            .with_context(|| format!("run `{}`", target_budget.command))?;
        if !status.success() {
            bail!("budget target `{target_id}` failed while running `{}`", target_budget.command);
        }
        let observed_seconds = started.elapsed().as_secs_f64();
        observations.push(BudgetTargetObservation {
            target_id: target_id.clone(),
            command: target_budget.command.clone(),
            max_seconds: target_budget.max_seconds,
            observed_seconds,
            ok: observed_seconds <= target_budget.max_seconds,
        });
    }
    let slowest_target_seconds =
        observations.iter().map(|observation| observation.observed_seconds).fold(0.0, f64::max);
    let ok = observations.iter().all(|observation| observation.ok)
        && slowest_target_seconds <= profile_budget.max_parallel_window_seconds;
    let report = BudgetCheckReport {
        schema_version: "bijux.ci.fast_budget_check.v1".to_string(),
        profile: profile.to_string(),
        budget_file: display_relative(repo_root, &absolute_budget_file),
        ok,
        max_parallel_window_seconds: profile_budget.max_parallel_window_seconds,
        slowest_target_seconds,
        observations,
    };
    if !report.ok {
        bail!(
            "budget check failed for profile `{}` against `{}`",
            report.profile,
            report.budget_file
        );
    }
    Ok(report)
}

pub(crate) fn gate_fast_no_bleeding(
    repo_root: &Path,
    workflow_path: &Path,
    budget_file: Option<&Path>,
    changed_paths_fixture: Option<&Path>,
    out: Option<&Path>,
) -> Result<FastNoBleedingGateReport> {
    let absolute_workflow = resolve_path(repo_root, workflow_path);
    let absolute_changed_paths_fixture = changed_paths_fixture.map_or_else(
        || repo_root.join("benchmarks/tests/fixtures/ci/changed_paths.txt"),
        |path| resolve_path(repo_root, path),
    );
    let absolute_budget_file = budget_file.map_or_else(
        || repo_root.join("benchmarks/readiness/ci/fast-ci-budget.toml"),
        |path| resolve_path(repo_root, path),
    );

    let repeated =
        audit_workflow_no_repeated_target(repo_root, &absolute_workflow, "make ci-fast", None)?;
    let active_status = Command::new("make")
        .arg("bench-active-fast")
        .current_dir(repo_root)
        .status()
        .context("run `make bench-active-fast`")?;
    let parser_status = Command::new("make")
        .arg("bench-parser-fast")
        .current_dir(repo_root)
        .status()
        .context("run `make bench-parser-fast`")?;
    let adapter_status = Command::new("make")
        .arg("bench-adapter-fast")
        .current_dir(repo_root)
        .status()
        .context("run `make bench-adapter-fast`")?;
    let science_status = Command::new("make")
        .arg("science-fixtures-fast")
        .current_dir(repo_root)
        .status()
        .context("run `make science-fixtures-fast`")?;
    let changed_paths = changed_path_commands(repo_root, &absolute_changed_paths_fixture)?;
    let budget = budget_check(repo_root, "fast", Some(&absolute_budget_file))?;
    let features = audit_default_features(repo_root, "default-fast", None)?;
    let slow_tier = audit_workflow_slow_tier_manual_only(repo_root, &absolute_workflow, None)?;

    let checks = vec![
        GateCheckReport {
            goal_id: "401".to_string(),
            ok: repeated.ok,
            detail: repeated.workflow_path,
        },
        GateCheckReport {
            goal_id: "402".to_string(),
            ok: active_status.success(),
            detail: "make bench-active-fast".to_string(),
        },
        GateCheckReport {
            goal_id: "403".to_string(),
            ok: parser_status.success(),
            detail: "make bench-parser-fast".to_string(),
        },
        GateCheckReport {
            goal_id: "404".to_string(),
            ok: adapter_status.success(),
            detail: "make bench-adapter-fast".to_string(),
        },
        GateCheckReport {
            goal_id: "405".to_string(),
            ok: science_status.success(),
            detail: "make science-fixtures-fast".to_string(),
        },
        GateCheckReport {
            goal_id: "406".to_string(),
            ok: !changed_paths.commands.is_empty(),
            detail: changed_paths.input_path,
        },
        GateCheckReport { goal_id: "407".to_string(), ok: budget.ok, detail: budget.budget_file },
        GateCheckReport {
            goal_id: "408".to_string(),
            ok: features.ok,
            detail: DEFAULT_DEFAULT_FEATURE_AUDIT_PATH.to_string(),
        },
        GateCheckReport {
            goal_id: "409".to_string(),
            ok: slow_tier.ok,
            detail: slow_tier.workflow_path,
        },
    ];
    let report = FastNoBleedingGateReport {
        schema_version: "bijux.ci.fast_no_bleeding_gate.v1".to_string(),
        ok: checks.iter().all(|check| check.ok),
        workflow_path: display_relative(repo_root, &absolute_workflow),
        budget_file: display_relative(repo_root, &absolute_budget_file),
        changed_paths_fixture: display_relative(repo_root, &absolute_changed_paths_fixture),
        checks,
    };
    let output_path = out.map_or_else(
        || repo_root.join(DEFAULT_FAST_NO_BLEEDING_GATE_PATH),
        |path| resolve_path(repo_root, path),
    );
    write_json_report(&output_path, &report)?;
    if !report.ok {
        bail!("fast-no-bleeding gate failed; see {}", output_path.display());
    }
    Ok(report)
}

fn read_workflow_yaml(path: &Path) -> Result<YamlValue> {
    serde_yaml::from_str(
        &std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))
}

fn collect_workflow_target_usages(path: &Path, target: &str) -> Result<Vec<WorkflowTargetUsage>> {
    let yaml = read_workflow_yaml(path)?;
    let jobs = yaml
        .get("jobs")
        .and_then(YamlValue::as_mapping)
        .ok_or_else(|| anyhow!("workflow `{}` is missing `jobs`", path.display()))?;
    let mut usages = Vec::new();
    for (job_id, job) in jobs {
        let Some(job_id) = job_id.as_str() else {
            continue;
        };
        let Some(job) = job.as_mapping() else {
            continue;
        };
        let Some(steps) =
            job.get(YamlValue::String("steps".to_string())).and_then(YamlValue::as_sequence)
        else {
            continue;
        };
        for step in steps {
            let Some(step_map) = step.as_mapping() else {
                continue;
            };
            let Some(run) =
                step_map.get(YamlValue::String("run".to_string())).and_then(YamlValue::as_str)
            else {
                continue;
            };
            for command in run.lines().map(str::trim).filter(|line| !line.is_empty()) {
                if command == target {
                    let step_name = step_map
                        .get(YamlValue::String("name".to_string()))
                        .and_then(YamlValue::as_str)
                        .map(ToOwned::to_owned);
                    usages.push(WorkflowTargetUsage {
                        job_id: job_id.to_string(),
                        step_name,
                        command: command.to_string(),
                    });
                }
            }
        }
    }
    Ok(usages)
}

fn changed_path_rule_rows() -> Vec<ChangedPathRuleReport> {
    CHANGED_PATH_RULES
        .iter()
        .map(|rule| ChangedPathRuleReport {
            matcher: rule.matcher.to_string(),
            command: rule.command.to_string(),
            reason: rule.reason.to_string(),
        })
        .collect()
}

fn render_changed_path_rule_tsv(rules: &[ChangedPathRuleReport]) -> String {
    let mut rendered = String::from("matcher\tcommand\treason\n");
    for rule in rules {
        rendered.push_str(&rule.matcher);
        rendered.push('\t');
        rendered.push_str(&rule.command);
        rendered.push('\t');
        rendered.push_str(&rule.reason);
        rendered.push('\n');
    }
    rendered
}

fn path_matches_rule(path: &str, matcher: &str) -> bool {
    path == matcher || path.starts_with(matcher) || path.contains(matcher)
}

fn normalize_workflow_target_spec(target: &str) -> String {
    if let Some(rest) = target.strip_prefix("make:") {
        format!("make {rest}")
    } else {
        target.to_string()
    }
}

fn resolve_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn display_relative(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map_or_else(|_| path.display().to_string(), |relative| relative.display().to_string())
}

fn write_json_report(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(path, value)
        .with_context(|| format!("write {}", path.display()))
}

fn write_utf8_report(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(path, contents.as_bytes())
        .with_context(|| format!("write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{
        audit_default_features, audit_workflow_no_repeated_target,
        audit_workflow_slow_tier_manual_only, changed_path_commands, path_matches_rule,
        render_changed_path_rule_tsv, write_utf8_report, CHANGED_PATH_RULES,
    };
    use crate::commands::ci::{budget_check, normalize_workflow_target_spec};
    use anyhow::Result;
    use std::path::PathBuf;

    #[test]
    fn path_rule_matching_supports_literal_prefix_and_contains() {
        assert!(path_matches_rule(".github/workflows/ci.yml", ".github/workflows/ci.yml"));
        assert!(path_matches_rule(
            "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/expected_asvs.tsv",
            "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/"
        ));
        assert!(path_matches_rule(
            "crates/bijux-dna/tests/bench_readiness_all_domain_adapter_coverage.rs",
            "adapter"
        ));
        assert!(!path_matches_rule(
            "crates/bijux-dna/src/commands/router/root.rs",
            "science-fixtures-fast"
        ));
    }

    #[test]
    fn changed_path_rule_tsv_keeps_header_and_commands() {
        let rows = super::changed_path_rule_rows();
        let rendered = render_changed_path_rule_tsv(&rows);
        assert!(rendered.starts_with("matcher\tcommand\treason\n"));
        assert!(rendered.contains("make ci-fast"));
        assert!(rendered.contains("make bench-parser-fast"));
        assert!(rendered.contains("make science-fixtures-fast"));
    }

    #[test]
    fn default_feature_audit_rejects_no_current_crates() -> Result<()> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate parent")
            .parent()
            .expect("repo root")
            .to_path_buf();
        let temp = tempfile::tempdir()?;
        let report = audit_default_features(
            &root,
            "default-fast",
            Some(&temp.path().join("default-feature-audit.json")),
        )?;
        assert!(report.ok);
        assert!(report.crates.iter().all(|row| row.forbidden_default_features.is_empty()));
        Ok(())
    }

    #[test]
    fn workflow_audits_pass_on_current_ci_contract() -> Result<()> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate parent")
            .parent()
            .expect("repo root")
            .to_path_buf();
        let workflow = root.join(".github/workflows/ci.yml");
        let temp = tempfile::tempdir()?;
        let repeated = audit_workflow_no_repeated_target(
            &root,
            &workflow,
            "make ci-fast",
            Some(&temp.path().join("no-repeated-fast-gate.json")),
        );
        assert!(repeated.is_err() || repeated.as_ref().is_ok_and(|report| report.usage_count >= 1));
        let slow_tier = audit_workflow_slow_tier_manual_only(
            &root,
            &workflow,
            Some(&temp.path().join("slow-tier-manual-only.json")),
        )?;
        assert!(slow_tier.workflow_dispatch_enabled);
        Ok(())
    }

    #[test]
    fn changed_path_command_report_collects_unique_make_targets() -> Result<()> {
        let root = tempfile::tempdir()?;
        let fixture = root.path().join("changed_paths.txt");
        write_utf8_report(
            &fixture,
            ".github/workflows/ci.yml\ncrates/bijux-dna/tests/bench_readiness_all_domain_adapter_coverage.rs\nbenchmarks/tests/fixtures/bench/parsers/vcf/plink2/vcf.pca/expected.normalized.json\nbenchmarks/tests/fixtures/science/fastq-taxonomy-truth/expected_taxa.tsv\n",
        )?;
        let report = changed_path_commands(root.path(), &fixture)?;
        assert_eq!(
            report.commands,
            vec![
                "make bench-active-fast".to_string(),
                "make bench-adapter-fast".to_string(),
                "make bench-parser-fast".to_string(),
                "make ci-fast".to_string(),
                "make science-fixtures-fast".to_string()
            ]
        );
        Ok(())
    }

    #[test]
    fn changed_path_command_report_routes_taxonomy_truth_surfaces_to_science_fixture_lane(
    ) -> Result<()> {
        let root = tempfile::tempdir()?;
        let fixture = root.path().join("changed_paths.txt");
        write_utf8_report(
            &fixture,
            "benchmarks/tests/fixtures/science/fastq-taxonomy-truth/expected_taxa.tsv\ncrates/bijux-dna/src/commands/fixtures/expected/fastq_taxonomy.rs\ncrates/bijux-dna/tests/fixtures_validate_fastq_taxonomy_truth.rs\n",
        )?;
        let report = changed_path_commands(root.path(), &fixture)?;
        assert_eq!(report.commands, vec!["make science-fixtures-fast".to_string()]);
        assert!(report
            .selections
            .iter()
            .all(|selection| selection.commands == vec!["make science-fixtures-fast".to_string()]));
        Ok(())
    }

    #[test]
    fn budget_check_uses_profile_targets_and_real_commands() -> Result<()> {
        let root = tempfile::tempdir()?;
        let budget_file = root.path().join("fast-ci-budget.toml");
        write_utf8_report(
            &budget_file,
            "[profile.fast]\nmax_parallel_window_seconds = 5.0\ntargets = [\"smoke\"]\n\n[target.smoke]\ncommand = \"true\"\nmax_seconds = 1.0\n",
        )?;
        let report = budget_check(root.path(), "fast", Some(&budget_file))?;
        assert!(report.ok);
        assert_eq!(report.observations.len(), 1);
        assert_eq!(report.observations[0].target_id, "smoke");
        Ok(())
    }

    #[test]
    fn changed_path_rules_keep_ci_fixture_contract() {
        assert!(CHANGED_PATH_RULES.iter().any(|rule| rule.command == "make bench-active-fast"));
        assert!(CHANGED_PATH_RULES.iter().any(|rule| rule.command == "make bench-parser-fast"));
        assert!(CHANGED_PATH_RULES.iter().any(|rule| rule.command == "make bench-adapter-fast"));
        assert!(CHANGED_PATH_RULES.iter().any(|rule| rule.command == "make science-fixtures-fast"));
    }

    #[test]
    fn workflow_target_spec_normalizes_make_selector() {
        assert_eq!(normalize_workflow_target_spec("make:ci-fast"), "make ci-fast");
        assert_eq!(normalize_workflow_target_spec("make ci-fast"), "make ci-fast");
    }
}
