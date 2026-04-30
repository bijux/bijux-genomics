use super::{
    artifact_root_path, check_schema_doc, collect_warning_strings_json, compare_json_key_drift,
    ensure_exists, ensure_help_only, env_flag, examples_run, json, merge_outcomes, read_json_value,
    read_utf8, smoke_run, sorted_unique, success_line, value_string, write_json_pretty,
    write_utf8, BTreeSet, Context, OpsCommandOutcome, Path, PathBuf, Result, Utc, Value,
    Workspace,
};

pub(in super::super) fn tooling_certification_gate(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("certification-gate", args)?;
    tooling_certify_all(workspace, &[])
}

pub(in super::super) fn tooling_benchmark_smoke_level1(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("benchmark-smoke-level1", args)?;
    let out_dir = artifact_root_path(workspace)?.join("benchmarks/smoke/level1");
    bijux_dna_infra::ensure_dir(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;

    let examples = canonical_level1_examples(workspace);
    let mut rows = Vec::new();
    for (example_id, example_root) in &examples {
        let outcome = examples_run(
            workspace,
            &["--allow-non-isolate".to_string(), example_id.to_string()],
        )?;
        if !outcome.is_success() {
            return Ok(outcome);
        }
        let artifact_dir = workspace.path("artifacts/examples").join(example_id);
        let metrics = read_json_value(&artifact_dir.join("metrics.json"))?;
        let bundle_bytes = std::fs::metadata(artifact_dir.join("bundle.tar.gz"))
            .map(|meta| meta.len())
            .unwrap_or(0);
        let expected_evidence = read_json_value(&example_root.join("expected-evidence.json"))?;
        rows.push(json!({
            "example_id": example_id,
            "workflow_class": value_string(metrics.get("workflow_class")),
            "duration_ms": metrics.get("duration_ms").cloned().unwrap_or(Value::Null),
            "artifact_bytes": metrics.get("artifact_bytes").cloned().unwrap_or(Value::Null),
            "bundle_bytes": bundle_bytes,
            "expected_evidence_count": expected_evidence
                .get("evidence")
                .and_then(Value::as_array)
                .map_or(0, Vec::len),
            "label": "smoke-only; not a scientific performance claim",
        }));
    }

    let report = json!({
        "schema_version": "bijux.smoke_benchmark.level1.v1",
        "generated_at_utc": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "disclaimer": "Smoke-only runtime and artifact-size report. Do not interpret as scientific or production performance evidence.",
        "examples": rows,
    });
    let report_path = out_dir.join("level1_smoke_benchmark.json");
    write_json_pretty(&report_path, &report)?;
    success_line(format!(
        "level1 smoke benchmark: {}",
        workspace.rel(&report_path).display()
    ))
}

pub(in super::super) fn tooling_certify_level1(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("certify-level1", args)?;

    let gate = super::cargo_targets::tooling_cargo_targets(
        workspace,
        &["essential-release".to_string()],
    )?;
    if !gate.is_success() {
        return Ok(gate);
    }

    let bench = tooling_benchmark_smoke_level1(workspace, &[])?;
    if !bench.is_success() {
        return Ok(bench);
    }

    let out_dir = artifact_root_path(workspace)?.join("certification/level1");
    bijux_dna_infra::ensure_dir(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
    let certificate = json!({
        "schema_version": "bijux.level1.certificate.v1",
        "generated_at_utc": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "status": "ok",
        "gate_command": "cargo run -q -p bijux-dna-dev -- tooling run cargo-targets essential-release",
        "benchmark_report": "artifacts/benchmarks/smoke/level1/level1_smoke_benchmark.json",
        "scoreboard": "artifacts/planning/scoreboard.yaml",
        "cards": "artifacts/planning/cards.yaml",
        "artifact_bundles": canonical_level1_examples(workspace)
            .into_iter()
            .map(|(example_id, _)| format!("artifacts/examples/{example_id}/bundle.tar.gz"))
            .collect::<Vec<_>>(),
        "known_gaps": [
            "Smoke benchmarks measure governed bundle flow only and are not scientific performance claims.",
            "Level 1 certification is a local repository completion claim, not an external publication claim."
        ],
    });
    let certificate_path = out_dir.join("level1_certificate.json");
    write_json_pretty(&certificate_path, &certificate)?;
    write_utf8(
        &out_dir.join("level1_certificate.md"),
        &format!(
            "# Level 1 Certificate\n\n- status: ok\n- gate: `cargo run -q -p bijux-dna-dev -- tooling run cargo-targets essential-release`\n- benchmark report: `artifacts/benchmarks/smoke/level1/level1_smoke_benchmark.json`\n- scoreboard: `artifacts/planning/scoreboard.yaml`\n- cards: `artifacts/planning/cards.yaml`\n"
        ),
    )?;
    success_line(format!(
        "level1 certificate: {}",
        workspace.rel(&certificate_path).display()
    ))
}

pub(in super::super) fn tooling_certify_all(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("certify-all", args)?;
    tooling_certify_domains_with_mode(workspace, "all")
}

pub(in super::super) fn tooling_certify_fastq(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("certify-fastq", args)?;
    tooling_certify_domains_with_mode(workspace, "fastq")
}

pub(in super::super) fn tooling_certify_bam(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("certify-bam", args)?;
    tooling_certify_domains_with_mode(workspace, "bam")
}

pub(in super::super) fn tooling_certify_vcf(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("certify-vcf", args)?;
    tooling_certify_domains_with_mode(workspace, "vcf")
}

pub(in super::super) fn tooling_certify_domains(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let Some(mode) = args.first().map(String::as_str) else {
        return Ok(OpsCommandOutcome::failure(
            "Usage: cargo run -p bijux-dna-dev -- tooling run certify-domains -- <fastq|bam|vcf|all>\n",
        ));
    };
    tooling_certify_domains_with_mode(workspace, mode)
}

pub(in super::super) fn tooling_certify_domains_with_mode(
    workspace: &Workspace,
    mode: &str,
) -> Result<OpsCommandOutcome> {
    match mode {
        "fastq" | "bam" | "vcf" | "all" => {}
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run certify-domains -- <fastq|bam|vcf|all>\n",
            ))
        }
    }

    let mut execution = OpsCommandOutcome::success(String::new());
    let cert_root = artifact_root_path(workspace)?.join("certification");
    bijux_dna_infra::ensure_dir(&cert_root)
        .with_context(|| format!("create {}", cert_root.display()))?;

    if matches!(mode, "fastq" | "all") {
        execution = merge_outcomes(
            execution,
            examples_run(
                workspace,
                &["--allow-non-isolate".to_string(), "fastq_edna_mini".to_string()],
            )?,
        );
        if !execution.is_success() {
            return Ok(execution);
        }
    }

    if matches!(mode, "vcf" | "all") {
        for example_id in [
            "vcf_damage_aware_genotype_mini",
            "vcf_downstream_vcf_full_mini",
            "vcf_downstream_demography_mini",
            "vcf_essential_qc_filter",
            "vcf_imputation_mini",
        ] {
            execution = merge_outcomes(
                execution,
                examples_run(
                    workspace,
                    &["--allow-non-isolate".to_string(), example_id.to_string()],
                )?,
            );
            if !execution.is_success() {
                return Ok(execution);
            }
        }
    }

    if matches!(mode, "bam" | "all") {
        let bam_smoke_input = workspace.path("assets/golden/smoke-inputs-v1/bam/sample.bam");
        if bam_smoke_input.exists() {
            execution = merge_outcomes(execution, smoke_run(workspace, &["bam".to_string()])?);
            if !execution.is_success() {
                return Ok(execution);
            }
        } else {
            execution.stdout.push_str(
                "certify-domains: BAM smoke input missing; continuing with fixture-backed BAM certification\n",
            );
        }
    }

    let production_mode = env_flag("BIJUX_CERT_PRODUCTION_MODE");
    let truth_vcf = std::env::var("BIJUX_TRUTH_VCF").unwrap_or_default();
    let doc = read_utf8(&workspace.path("docs/50-reference/MANIFEST_MIGRATION.md"))?;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut domains = serde_json::Map::new();
    let mut seen_schema_versions = BTreeSet::new();

    if matches!(mode, "fastq" | "all") {
        let example_root = workspace.path("examples/fastq/edna-mini");
        let artifact_root = workspace.path("artifacts/examples/fastq_edna_mini");
        let manifest_path = artifact_root.join("manifest.json");
        let metrics_path = artifact_root.join("metrics.json");
        let report_path = artifact_root.join("report.json");
        ensure_exists(&manifest_path, "fastq manifest", &mut errors);
        ensure_exists(&metrics_path, "fastq metrics", &mut errors);
        ensure_exists(&report_path, "fastq report", &mut errors);

        if manifest_path.exists() {
            let manifest = read_json_value(&manifest_path)?;
            check_schema_doc(
                value_string(manifest.get("schema_version")),
                &doc,
                &mut seen_schema_versions,
                &mut errors,
            );
            for key in ["schema_version", "example_id", "files"] {
                if manifest.get(key).is_none() {
                    errors.push(format!("fastq manifest missing key `{key}`"));
                }
            }
        }
        if metrics_path.exists() {
            let metrics = read_json_value(&metrics_path)?;
            for key in ["example_id", "collected_at", "status"] {
                if metrics.get(key).is_none() {
                    errors.push(format!("fastq metrics missing key `{key}`"));
                }
            }
        }
        compare_json_key_drift(
            &report_path,
            &example_root.join("golden/report.json"),
            "fastq report",
            &mut errors,
        )?;

        let mut fastq_warnings = Vec::new();
        if report_path.exists() {
            collect_warning_strings_json(&read_json_value(&report_path)?, &mut fastq_warnings);
        }
        warnings.extend(fastq_warnings.iter().cloned());
        domains.insert(
            "fastq".to_string(),
            json!({
                "status": "ok",
                "warnings": sorted_unique(fastq_warnings),
                "artifacts_dir": artifact_root.display().to_string(),
            }),
        );
    }

    if matches!(mode, "bam" | "all") {
        let fixture_root = workspace.path(
            "crates/bijux-dna-analyze/tests/fixtures/golden_spine/bam-to-bam__adna_shotgun__v1/runs/bam-to-bam__adna_shotgun__v1/artifacts",
        );
        let run_manifest_path = fixture_root.join("run_manifest.json");
        let report_path = fixture_root.join("report.json");
        let facts_path = fixture_root.join("facts.jsonl");
        ensure_exists(&run_manifest_path, "bam run_manifest", &mut errors);
        ensure_exists(&report_path, "bam report", &mut errors);
        ensure_exists(&facts_path, "bam facts", &mut errors);

        if run_manifest_path.exists() {
            let run_manifest = read_json_value(&run_manifest_path)?;
            check_schema_doc(
                value_string(run_manifest.get("schema_version")),
                &doc,
                &mut seen_schema_versions,
                &mut errors,
            );
            for key in ["schema_version", "run_id"] {
                if run_manifest.get(key).is_none() {
                    errors.push(format!("bam run_manifest missing key `{key}`"));
                }
            }
        }
        if report_path.exists() {
            let report = read_json_value(&report_path)?;
            for key in ["schema_version", "stages"] {
                if report.get(key).is_none() {
                    errors.push(format!("bam report missing key `{key}`"));
                }
            }
            check_schema_doc(
                value_string(report.get("schema_version")),
                &doc,
                &mut seen_schema_versions,
                &mut errors,
            );
        }
        if facts_path.exists() {
            let first_line = read_utf8(&facts_path)?
                .lines()
                .find(|line| !line.trim().is_empty())
                .map(ToOwned::to_owned);
            match first_line {
                Some(line) => {
                    let value: Value = serde_json::from_str(&line)
                        .with_context(|| format!("parse {}", facts_path.display()))?;
                    check_schema_doc(
                        value_string(value.get("schema_version")),
                        &doc,
                        &mut seen_schema_versions,
                        &mut errors,
                    );
                    if value.get("metrics").is_none() {
                        errors.push("bam facts.jsonl missing metrics object".to_string());
                    }
                }
                None => errors.push("bam facts.jsonl missing first JSON line".to_string()),
            }
        }
        domains.insert(
            "bam".to_string(),
            json!({
                "status": "ok",
                "warnings": Vec::<String>::new(),
                "artifacts_dir": fixture_root.display().to_string(),
            }),
        );
    }

    if matches!(mode, "vcf" | "all") {
        let mut vcf_warnings = Vec::new();
        for (example_id, example_root) in [
            (
                "vcf_damage_aware_genotype_mini",
                workspace.path("examples/vcf/damage-aware-genotype-mini"),
            ),
            (
                "vcf_downstream_vcf_full_mini",
                workspace.path("examples/vcf/downstream-vcf-full-mini"),
            ),
            (
                "vcf_downstream_demography_mini",
                workspace.path("examples/vcf/downstream-demography-mini"),
            ),
            ("vcf_essential_qc_filter", workspace.path("examples/vcf/essential-qc-filter")),
            ("vcf_imputation_mini", workspace.path("examples/vcf/imputation-mini")),
        ] {
            let artifact_root = workspace.path("artifacts/examples").join(example_id);
            let report_path = artifact_root.join("report.json");
            let explain_path = artifact_root.join("explain.json");
            let metrics_path = artifact_root.join("metrics.json");
            let manifest_path = artifact_root.join("manifest.json");
            ensure_exists(&report_path, &format!("{example_id} report"), &mut errors);
            ensure_exists(&explain_path, &format!("{example_id} explain"), &mut errors);
            ensure_exists(&metrics_path, &format!("{example_id} metrics"), &mut errors);
            ensure_exists(&manifest_path, &format!("{example_id} manifest"), &mut errors);
            compare_json_key_drift(
                &report_path,
                &example_root.join("golden/report.json"),
                &format!("{example_id} report"),
                &mut errors,
            )?;
            compare_json_key_drift(
                &explain_path,
                &example_root.join("golden/explain.json"),
                &format!("{example_id} explain"),
                &mut errors,
            )?;

            if report_path.exists() {
                let report = read_json_value(&report_path)?;
                let report_schema = value_string(report.get("schema_version"));
                if !report_schema.is_empty() {
                    check_schema_doc(report_schema, &doc, &mut seen_schema_versions, &mut errors);
                } else if manifest_path.exists() {
                    let manifest = read_json_value(&manifest_path)?;
                    let manifest_schema = value_string(manifest.get("schema_version"));
                    if manifest_schema.is_empty() {
                        errors.push(format!(
                            "{example_id}: neither report nor manifest declares schema_version"
                        ));
                    } else {
                        check_schema_doc(
                            manifest_schema,
                            &doc,
                            &mut seen_schema_versions,
                            &mut errors,
                        );
                    }
                } else {
                    errors.push(format!(
                        "{example_id}: neither report nor manifest declares schema_version"
                    ));
                }
                collect_warning_strings_json(&report, &mut vcf_warnings);
            }
        }

        let truth_path = truth_vcf.trim();
        let truth_hook = if truth_path.is_empty() {
            json!({
                "enabled": false,
                "truth_vcf": Value::Null,
                "status": "skipped",
                "details": "no truth VCF provided",
            })
        } else if !Path::new(truth_path).exists() {
            errors.push(format!("truth VCF path does not exist: {truth_path}"));
            json!({
                "enabled": true,
                "truth_vcf": truth_path,
                "status": "failed",
                "details": "path missing",
            })
        } else {
            json!({
                "enabled": true,
                "truth_vcf": truth_path,
                "status": "ok",
                "details": "hook enabled; downstream concordance metrics must be consumed from imputation outputs",
            })
        };
        warnings.extend(vcf_warnings.iter().cloned());
        domains.insert(
            "vcf".to_string(),
            json!({
                "status": "ok",
                "warnings": sorted_unique(vcf_warnings),
                "truth_concordance_hook": truth_hook,
                "artifacts_dir": workspace.path("artifacts/examples").display().to_string(),
            }),
        );
    }

    warnings = sorted_unique(warnings);
    if production_mode && !warnings.is_empty() {
        errors.push(format!(
            "production mode forbids warnings; found {} warning entries",
            warnings.len()
        ));
    }

    let generated_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let stamp = json!({
        "schema_version": "bijux.certification_run_stamp.v1",
        "mode": if production_mode { "production" } else { "non_production" },
        "relaxed_thresholds": !production_mode,
        "generated_at_utc": generated_at,
    });
    let bundle = json!({
        "schema_version": "bijux.certification_bundle.v2",
        "generated_at_utc": generated_at,
        "mode": stamp["mode"].clone(),
        "relaxed_thresholds": stamp["relaxed_thresholds"].clone(),
        "domains": Value::Object(domains),
        "golden_drift_policy": {
            "mode": "schema_and_required_keys_only",
            "exact_metric_values_compared": false,
        },
        "artifact_schema_versions_seen": seen_schema_versions.into_iter().collect::<Vec<_>>(),
        "errors": errors,
        "warnings": warnings,
        "status": if errors.is_empty() { "ok" } else { "failed" },
    });

    write_json_pretty(&cert_root.join("run_stamp.json"), &stamp)?;
    write_json_pretty(&cert_root.join("certification_bundle.json"), &bundle)?;

    if bundle["status"] == "failed" {
        execution.stderr.push_str("certification: FAILED\n");
        if let Some(items) = bundle["errors"].as_array() {
            for item in items {
                execution.stderr.push_str("- ");
                execution.stderr.push_str(item.as_str().unwrap_or_default());
                execution.stderr.push('\n');
            }
        }
        execution.exit_code = 1;
        return Ok(execution);
    }

    execution.stdout.push_str("certification: OK\n");
    execution.stdout.push_str(&format!(
        "certify-domains: OK ({})\n",
        cert_root.join("certification_bundle.json").display()
    ));
    Ok(execution)
}

fn canonical_level1_examples(workspace: &Workspace) -> Vec<(String, PathBuf)> {
    vec![
        ("fastq_essential_qc".to_string(), workspace.path("examples/fastq/essential-qc")),
        (
            "bam_essential_alignment_qc".to_string(),
            workspace.path("examples/bam/essential-alignment-qc"),
        ),
        (
            "vcf_essential_qc_filter".to_string(),
            workspace.path("examples/vcf/essential-qc-filter"),
        ),
    ]
}
