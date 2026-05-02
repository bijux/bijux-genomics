#![allow(clippy::too_many_lines)]

use super::{
    anyhow, ensure_help_only, env_or_default, generate_compatibility_matrix,
    generate_compatibility_reference_docs, generate_docs_graph, generate_domain_coverage_doc,
    generate_repo_root_map, generate_tool_index, json, json_u64, read_json_value, read_utf8,
    resolve_optional_output_arg, resolve_workspace_path, run_check_ids, run_native_ops_command,
    run_program, run_program_with_env, success_line, toml_to_json_value,
    tooling_check_config_snapshot, trim_quoted, value_string, walk_file_list, write_json_pretty,
    write_utf8, BTreeMap, BTreeSet, ContainerApplication, Context, DomainApplication,
    NativeOpsCommandKey, OpsCommandOutcome, Path, PathBuf, Regex, Result, TomlValue, Value,
    WalkDir, Workspace,
};

pub(in super::super) fn tooling_config_inventory(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("config-inventory", args)?;
    let out_txt = workspace.path("artifacts/configs_inventory.txt");
    let out_md = workspace.path("artifacts/inventory/configs.md");
    let mut config_files = WalkDir::new(workspace.path("configs"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| workspace.rel(entry.path()).to_string_lossy().to_string())
        .collect::<Vec<_>>();
    config_files.sort();
    let mut text_lines =
        vec!["# schema_version = 1".to_string(), "# owner = bijux-dna-infra".to_string()];
    text_lines.extend(config_files.iter().cloned());
    write_utf8(&out_txt, &format!("{}\n", text_lines.join("\n")))?;

    let mut md_lines = vec![
        "# Config Inventory".to_string(),
        String::new(),
        "| Path | Schema Version | Owner |".to_string(),
        "|---|---:|---|".to_string(),
    ];
    for rel in config_files {
        let path = workspace.path(&rel);
        let mut schema = "-".to_string();
        let mut owner = "-".to_string();
        for line in read_utf8(&path).unwrap_or_default().lines().take(8) {
            let trimmed = line.trim();
            if let Some(value) = trimmed.strip_prefix("# schema_version = ") {
                schema = value.trim().to_string();
            }
            if let Some(value) = trimmed.strip_prefix("# owner = ") {
                owner = value.trim().to_string();
            }
        }
        md_lines.push(format!("| `{rel}` | `{schema}` | `{owner}` |"));
    }
    write_utf8(&out_md, &format!("{}\n", md_lines.join("\n")))?;
    success_line(format!("wrote {}\nwrote {}", out_txt.display(), out_md.display()))
}

pub(in super::super) fn tooling_architecture_report(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mut base = "HEAD~1".to_string();
    let mut out_json = workspace.path("artifacts/architecture/report.json");
    let mut out_md = workspace.path("artifacts/architecture/report.md");
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run architecture-report -- [--base <rev>] [--out-json <path>] [--out-md <path>]",
                )
            }
            "--base" => {
                base = args.get(index + 1).cloned().context("missing value for --base")?;
                index += 2;
            }
            "--out-json" => {
                out_json = PathBuf::from(
                    args.get(index + 1).cloned().context("missing value for --out-json")?,
                );
                index += 2;
            }
            "--out-md" => {
                out_md = PathBuf::from(
                    args.get(index + 1).cloned().context("missing value for --out-md")?,
                );
                index += 2;
            }
            other => return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n"))),
        }
    }

    let report = build_architecture_report(workspace, &base)?;
    write_json_pretty(&out_json, &report)?;
    write_utf8(&out_md, &render_architecture_report_markdown(&report))?;
    success_line(format!("wrote {}\nwrote {}", out_json.display(), out_md.display()))
}

fn build_architecture_report(workspace: &Workspace, base: &str) -> Result<Value> {
    let crates_dir = workspace.path("crates");
    let mut crates = WalkDir::new(&crates_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_dir() && entry.path().join("Cargo.toml").is_file())
        .map(|entry| collect_architecture_crate_row(workspace, entry.path()))
        .collect::<Result<Vec<_>>>()?;
    crates.sort_by(|left, right| {
        value_string(left.get("crate_name")).cmp(&value_string(right.get("crate_name")))
    });

    let diff = run_program(
        workspace,
        "git",
        &["diff".to_string(), "--unified=0".to_string(), base.to_string(), "--".to_string()],
    )?;
    let name_status = run_program(
        workspace,
        "git",
        &[
            "diff".to_string(),
            "--name-status".to_string(),
            base.to_string(),
            "--".to_string(),
            "configs".to_string(),
            "science/specs".to_string(),
        ],
    )?;

    Ok(json!({
        "schema_version": "bijux.architecture_report.v1",
        "generated_at": super::stable_now_utc_string(),
        "base_revision": base,
        "crates": crates,
        "dependency_additions": extract_workspace_dependency_additions(&diff.stdout),
        "new_config_files": extract_added_paths(&name_status.stdout, "configs/"),
        "new_schema_files": extract_added_paths(&name_status.stdout, "science/specs/"),
    }))
}

fn collect_architecture_crate_row(workspace: &Workspace, crate_dir: &Path) -> Result<Value> {
    let crate_name = crate_dir
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .context("crate directory missing name")?;
    let src_dir = crate_dir.join("src");
    let mut rust_file_count = 0u64;
    let mut rust_loc = 0u64;
    let mut public_item_count = 0u64;
    if src_dir.is_dir() {
        for entry in WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("rs"))
        {
            rust_file_count += 1;
            let raw = read_utf8(entry.path())?;
            rust_loc += raw.lines().count() as u64;
            public_item_count += raw
                .lines()
                .map(str::trim_start)
                .filter(|line| line.starts_with("pub ") || line.starts_with("pub("))
                .count() as u64;
        }
    }
    Ok(json!({
        "crate_name": crate_name,
        "crate_path": workspace.rel(crate_dir).display().to_string(),
        "rust_file_count": rust_file_count,
        "rust_loc": rust_loc,
        "public_item_count": public_item_count,
    }))
}

fn extract_workspace_dependency_additions(diff: &str) -> Vec<String> {
    let mut additions = BTreeSet::new();
    for line in diff.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('+') || trimmed.starts_with("+++") {
            continue;
        }
        let payload = trimmed.trim_start_matches('+').trim();
        if payload.starts_with("bijux-dna-") && payload.contains('=') {
            let dep = payload.split('=').next().unwrap_or(payload).trim();
            additions.insert(dep.to_string());
        }
    }
    additions.into_iter().collect()
}

fn extract_added_paths(diff_name_status: &str, prefix: &str) -> Vec<String> {
    diff_name_status
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let status = parts.next()?;
            let path = parts.next()?;
            if status == "A" && path.starts_with(prefix) {
                return Some(path.to_string());
            }
            None
        })
        .collect()
}

fn render_architecture_report_markdown(report: &Value) -> String {
    let mut lines = vec![
        "# Architecture Report".to_string(),
        String::new(),
        format!("- generated_at: `{}`", value_string(report.get("generated_at"))),
        format!("- base_revision: `{}`", value_string(report.get("base_revision"))),
        String::new(),
        "## Crates".to_string(),
        String::new(),
        "| Crate | Rust files | Rust LOC | Public items |".to_string(),
        "| --- | ---: | ---: | ---: |".to_string(),
    ];
    if let Some(rows) = report.get("crates").and_then(Value::as_array) {
        for row in rows {
            lines.push(format!(
                "| `{}` | `{}` | `{}` | `{}` |",
                value_string(row.get("crate_name")),
                json_u64(row.get("rust_file_count")),
                json_u64(row.get("rust_loc")),
                json_u64(row.get("public_item_count"))
            ));
        }
    }
    lines.push(String::new());
    lines.push("## Drift".to_string());
    lines.push(String::new());
    for (label, key) in [
        ("dependency_additions", "dependency_additions"),
        ("new_config_files", "new_config_files"),
        ("new_schema_files", "new_schema_files"),
    ] {
        let rendered = report[key]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_str().map(|item| format!("`{item}`")))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("- {label}: {rendered}"));
    }
    format!("{}\n", lines.join("\n"))
}

pub(in super::super) fn tooling_coverage_summary(
    _workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    #[derive(Default, Clone)]
    struct CoverageEntry {
        lines_hit: u64,
        lines_total: u64,
        funcs_hit: u64,
        funcs_total: u64,
        regions_hit: u64,
        regions_total: u64,
        files: Vec<(String, u64)>,
    }

    let mut report = None;
    let mut baseline = None;
    let mut thresholds = None;
    let mut show_uncovered = false;
    let mut show_worst = false;
    let mut worst_count = 20usize;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run coverage-summary -- <report> [--baseline <path>] [--check-thresholds <path>] [--show-uncovered|--verbose] [--show-worst] [--worst-count N]",
                )
            }
            "--baseline" => {
                baseline = Some(
                    args.get(index + 1)
                        .cloned()
                        .context("missing value for --baseline")?,
                );
                index += 2;
            }
            "--check-thresholds" => {
                thresholds = Some(
                    args.get(index + 1)
                        .cloned()
                        .context("missing value for --check-thresholds")?,
                );
                index += 2;
            }
            "--show-uncovered" | "--verbose" => {
                show_uncovered = true;
                index += 1;
            }
            "--show-worst" => {
                show_worst = true;
                index += 1;
            }
            "--worst-count" => {
                worst_count = args
                    .get(index + 1)
                    .context("missing value for --worst-count")?
                    .parse::<usize>()
                    .context("parse --worst-count")?;
                index += 2;
            }
            value if report.is_none() => {
                report = Some(value.to_string());
                index += 1;
            }
            other => return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n"))),
        }
    }
    let report = report.context("coverage-summary requires <report>")?;
    let show_uncovered =
        show_uncovered || std::env::var("COVERAGE_SHOW_UNCOVERED").ok().as_deref() == Some("1");
    let show_worst =
        show_worst || std::env::var("COVERAGE_SHOW_WORST").ok().as_deref() == Some("1");

    fn percent(hit: u64, total: u64) -> f64 {
        if total == 0 {
            100.0
        } else {
            100.0 * hit as f64 / total as f64
        }
    }

    fn crate_name_for(filename: &str) -> String {
        let path = Path::new(filename);
        let parts = path
            .components()
            .map(|component| component.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        let Some(index) = parts.iter().position(|part| part == "crates") else {
            return "workspace".to_string();
        };
        let Some(crate_dir) = parts.get(index + 1) else {
            return "workspace".to_string();
        };
        let manifest = Path::new("crates").join(crate_dir).join("Cargo.toml");
        if let Ok(raw) = read_utf8(&manifest) {
            for line in raw.lines() {
                let trimmed = line.trim();
                if let Some(value) = trimmed.strip_prefix("name =") {
                    return trim_quoted(value);
                }
            }
        }
        crate_dir.clone()
    }

    fn load_coverage_report(path: &Path) -> Result<BTreeMap<String, CoverageEntry>> {
        let payload = read_json_value(path)?;
        let files = payload["data"]
            .as_array()
            .and_then(|items| items.first())
            .and_then(|root| root.get("files"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mut crates = BTreeMap::<String, CoverageEntry>::new();
        for file in files {
            let filename = value_string(file.get("filename"));
            let crate_name = crate_name_for(&filename);
            let lines = file.get("summary").and_then(|v| v.get("lines"));
            let funcs = file.get("summary").and_then(|v| v.get("functions"));
            let regions = file.get("summary").and_then(|v| v.get("regions"));
            let lines_total = json_u64(lines.and_then(|v| v.get("count")));
            let lines_hit = json_u64(lines.and_then(|v| v.get("covered")));
            let lines_miss_raw = json_u64(lines.and_then(|v| v.get("notcovered")));
            let lines_miss = if lines_total > 0 && lines_hit == 0 && lines_miss_raw == 0 {
                lines_total.saturating_sub(lines_hit)
            } else {
                lines_miss_raw
            };
            let funcs_total = json_u64(funcs.and_then(|v| v.get("count")));
            let mut funcs_hit = json_u64(funcs.and_then(|v| v.get("covered")));
            let funcs_miss_raw = json_u64(funcs.and_then(|v| v.get("notcovered")));
            if funcs_total > 0 && funcs_hit == 0 && funcs_miss_raw == 0 {
                funcs_hit = funcs_total;
            }
            let regions_total = json_u64(regions.and_then(|v| v.get("count")));
            let mut regions_hit = json_u64(regions.and_then(|v| v.get("covered")));
            let regions_miss_raw = json_u64(regions.and_then(|v| v.get("notcovered")));
            if regions_total > 0 && regions_hit == 0 && regions_miss_raw == 0 {
                regions_hit = regions_total;
            }

            let entry = crates.entry(crate_name).or_default();
            entry.lines_hit += lines_hit;
            entry.lines_total += lines_total;
            entry.funcs_hit += funcs_hit;
            entry.funcs_total += funcs_total;
            entry.regions_hit += regions_hit;
            entry.regions_total += regions_total;
            entry.files.push((filename, lines_miss));
        }
        Ok(crates)
    }

    let data = load_coverage_report(&PathBuf::from(&report))?;
    let baseline_data = match baseline {
        Some(path) => Some(load_coverage_report(&PathBuf::from(path))?),
        None => None,
    };

    let mut stdout = String::new();
    let header = if baseline_data.is_some() {
        "crate | lines | covered | lines % | funcs % | regions % | lines Δ | uncovered top files"
    } else {
        "crate | lines | covered | lines % | funcs % | regions % | uncovered top files"
    };
    stdout.push_str(header);
    stdout.push('\n');
    stdout.push_str(if baseline_data.is_some() {
        "----- | ----- | ------- | ------- | ------- | --------- | ------- | -------------------"
    } else {
        "----- | ----- | ------- | ------- | ------- | --------- | -------------------"
    });
    stdout.push('\n');

    let mut rows = Vec::new();
    for (crate_name, entry) in &data {
        let lines_pct = percent(entry.lines_hit, entry.lines_total);
        let funcs_pct = percent(entry.funcs_hit, entry.funcs_total);
        let regions_pct = percent(entry.regions_hit, entry.regions_total);
        let mut top_files = entry.files.clone();
        top_files.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        let top = top_files
            .iter()
            .filter(|(_, misses)| *misses > 0)
            .take(5)
            .map(|(path, misses)| {
                format!(
                    "{}({misses})",
                    Path::new(path).file_name().and_then(|value| value.to_str()).unwrap_or(path)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let delta =
            baseline_data.as_ref().and_then(|baseline| baseline.get(crate_name)).map(|baseline| {
                percent(entry.lines_hit, entry.lines_total)
                    - percent(baseline.lines_hit, baseline.lines_total)
            });
        rows.push((
            crate_name.clone(),
            lines_pct,
            funcs_pct,
            regions_pct,
            delta,
            top,
            entry.clone(),
        ));
    }

    for (crate_name, lines_pct, funcs_pct, regions_pct, delta, top, entry) in &rows {
        if let Some(delta) = delta {
            stdout.push_str(&format!(
                "{crate_name} | {:>5} | {:>7} | {:>6.2}% | {:>6.2}% | {:>7.2}% | {delta:+7.2}% | {top}\n",
                entry.lines_total, entry.lines_hit, lines_pct, funcs_pct, regions_pct
            ));
        } else {
            stdout.push_str(&format!(
                "{crate_name} | {:>5} | {:>7} | {:>6.2}% | {:>6.2}% | {:>7.2}% | {top}\n",
                entry.lines_total, entry.lines_hit, lines_pct, funcs_pct, regions_pct
            ));
        }
        if show_uncovered {
            let mut files = entry.files.clone();
            files.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
            for (path, misses) in files {
                if misses > 0 {
                    stdout.push_str(&format!("  - {path} ({misses} lines)\n"));
                }
            }
        }
    }

    if show_worst {
        let mut worst = rows.clone();
        worst.sort_by(|left, right| {
            left.1.partial_cmp(&right.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        stdout.push_str("\nworst coverage (lines %):\n");
        for (crate_name, lines_pct, ..) in worst.into_iter().take(worst_count) {
            stdout.push_str(&format!("{crate_name}: {lines_pct:6.2}%\n"));
        }
    }

    if let Some(path) = thresholds {
        let thresholds_path = PathBuf::from(path);
        let raw = read_utf8(&thresholds_path)?;
        let value = if thresholds_path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
            toml_to_json_value(toml::from_str::<TomlValue>(&raw)?)
        } else {
            serde_json::from_str::<Value>(&raw)?
        };
        let default_threshold = value["default"].as_f64().unwrap_or(0.0);
        let class_thresholds = value["classes"].as_object().cloned().unwrap_or_default();
        let class_map = value["crate_class"].as_object().cloned().unwrap_or_default();
        let overrides = value["overrides"].as_object().cloned().unwrap_or_default();
        let mut failures = Vec::new();
        for (crate_name, entry) in &data {
            let lines_pct = percent(entry.lines_hit, entry.lines_total);
            let minimum = if let Some(value) = overrides.get(crate_name).and_then(Value::as_f64) {
                value
            } else if let Some(class_name) = class_map.get(crate_name).and_then(Value::as_str) {
                class_thresholds
                    .get(class_name)
                    .and_then(Value::as_f64)
                    .unwrap_or(default_threshold)
            } else {
                default_threshold
            };
            if lines_pct < minimum {
                failures.push((crate_name.clone(), lines_pct, minimum));
            }
        }
        if !failures.is_empty() {
            stdout.push_str("\ncoverage thresholds failed:\n");
            for (crate_name, actual, minimum) in failures {
                stdout.push_str(&format!("{crate_name}: {actual:.2}% < {minimum:.2}%\n"));
            }
            return Ok(OpsCommandOutcome { exit_code: 1, stdout, stderr: String::new() });
        }
    }

    Ok(OpsCommandOutcome::success(stdout))
}

pub(in super::super) fn tooling_crash_triage(
    _workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") || args.is_empty() {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- tooling run crash-triage -- <crash_provenance.json>",
        );
    }
    let path = PathBuf::from(&args[0]);
    if !path.is_file() {
        return Ok(OpsCommandOutcome::failure(format!(
            "crash-triage: missing file {}\n",
            path.display()
        )));
    }
    let payload = read_json_value(&path)?;
    let stderr = payload["stderr_last_lines"]
        .as_array()
        .map(|items| {
            items.iter().filter_map(Value::as_str).collect::<Vec<_>>().join("\n").to_lowercase()
        })
        .unwrap_or_default();
    let command = value_string(payload.get("command")).to_lowercase();
    let exit_code = payload.get("exit_code").and_then(Value::as_i64);
    let mut causes = Vec::<(i32, &str, &str)>::new();
    if stderr.contains("no such file") || stderr.contains("cannot open") {
        causes.push((100, "input_missing", "Input file missing/unreadable."));
    }
    if stderr.contains("index") && (stderr.contains("missing") || stderr.contains("failed")) {
        causes.push((95, "index_missing", "Index missing or invalid."));
    }
    if stderr.contains("out of memory")
        || stderr.contains("cannot allocate memory")
        || stderr.contains("killed")
    {
        causes.push((90, "resource_exhausted", "Process likely hit memory limit."));
    }
    if stderr.contains("header") || stderr.contains("contig") || stderr.contains("chromosome") {
        causes.push((85, "reference_mismatch", "Header/contig/reference mismatch."));
    }
    if stderr.contains("not compressed") && (command.contains("tabix") || command.contains("bgzip"))
    {
        causes.push((80, "compression_contract", "Expected bgzip-compressed input for indexing."));
    }
    if matches!(exit_code, Some(126 | 127)) {
        causes.push((
            75,
            "runner_contract",
            "Command/image contract issue (missing binary or exec failure).",
        ));
    }
    if causes.is_empty() {
        causes.push((10, "unknown", "No high-confidence pattern found; inspect full logs."));
    }
    causes.sort_by(|left, right| right.0.cmp(&left.0));
    let mut stdout = String::from("crash-triage: top causes\n");
    for (_, code, message) in causes.into_iter().take(5) {
        stdout.push_str(&format!("- {code}: {message}\n"));
    }
    Ok(OpsCommandOutcome::success(stdout))
}

pub(in super::super) fn tooling_deprecate_vcf_knob(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- tooling run deprecate-vcf-knob -- --stage <stage_id> --knob <name> --phase <warn|fail|remove> --replacement <name> --rationale <text>";
    let mut stage = None;
    let mut knob = None;
    let mut phase = None;
    let mut replacement = None;
    let mut rationale = None;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => return success_line(usage),
            "--stage" => {
                stage = Some(args.get(index + 1).cloned().context("missing value for --stage")?);
                index += 2;
            }
            "--knob" => {
                knob = Some(args.get(index + 1).cloned().context("missing value for --knob")?);
                index += 2;
            }
            "--phase" => {
                phase = Some(args.get(index + 1).cloned().context("missing value for --phase")?);
                index += 2;
            }
            "--replacement" => {
                replacement =
                    Some(args.get(index + 1).cloned().context("missing value for --replacement")?);
                index += 2;
            }
            "--rationale" => {
                rationale =
                    Some(args.get(index + 1).cloned().context("missing value for --rationale")?);
                index += 2;
            }
            other => {
                return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n{usage}\n")))
            }
        }
    }
    let stage = stage.context(usage)?;
    let knob = knob.context(usage)?;
    let phase = phase.context(usage)?;
    let replacement = replacement.context(usage)?;
    let rationale = rationale.context(usage)?;
    if !matches!(phase.as_str(), "warn" | "fail" | "remove") {
        return Ok(OpsCommandOutcome::failure("phase must be warn|fail|remove\n".to_string()));
    }
    let path = workspace.path("configs/vcf/deprecations/knobs.toml");
    let mut text = read_utf8(&path)?;
    let needle = format!("stage_id = \"{stage}\"\nknob = \"{knob}\"");
    if text.contains(&needle) {
        return Ok(OpsCommandOutcome::failure(format!(
            "deprecation already exists for {stage}:{knob}\n"
        )));
    }
    let entry = format!(
        "\n[[deprecation]]\nstage_id = \"{stage}\"\nknob = \"{knob}\"\nphase = \"{phase}\"\nreplacement = \"{replacement}\"\nrationale = \"{}\"\n",
        rationale.replace('"', "\\\"")
    );
    text = format!("{}{}\n", text.trim_end(), entry);
    write_utf8(&path, &text)?;
    success_line(format!("added knob deprecation {stage}:{knob} ({phase})"))
}

pub(in super::super) fn tooling_deprecate_vcf_panel(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- tooling run deprecate-vcf-panel -- --panel <panel_id> --phase <warn|fail|remove> --replacement <panel_id> --rationale <text>";
    let mut panel = None;
    let mut phase = None;
    let mut replacement = None;
    let mut rationale = None;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => return success_line(usage),
            "--panel" => {
                panel = Some(args.get(index + 1).cloned().context("missing value for --panel")?);
                index += 2;
            }
            "--phase" => {
                phase = Some(args.get(index + 1).cloned().context("missing value for --phase")?);
                index += 2;
            }
            "--replacement" => {
                replacement =
                    Some(args.get(index + 1).cloned().context("missing value for --replacement")?);
                index += 2;
            }
            "--rationale" => {
                rationale =
                    Some(args.get(index + 1).cloned().context("missing value for --rationale")?);
                index += 2;
            }
            other => {
                return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n{usage}\n")))
            }
        }
    }
    let panel = panel.context(usage)?;
    let phase = phase.context(usage)?;
    let replacement = replacement.context(usage)?;
    let rationale = rationale.context(usage)?;
    if !matches!(phase.as_str(), "warn" | "fail" | "remove") {
        return Ok(OpsCommandOutcome::failure("phase must be warn|fail|remove\n".to_string()));
    }
    let path = workspace.path("configs/vcf/deprecations/panels.toml");
    let mut text = read_utf8(&path)?;
    let needle = format!("panel_id = \"{panel}\"");
    if text.contains(&needle) {
        return Ok(OpsCommandOutcome::failure(format!(
            "deprecation already exists for panel {panel}\n"
        )));
    }
    let entry = format!(
        "\n[[deprecation]]\npanel_id = \"{panel}\"\nphase = \"{phase}\"\nreplacement = \"{replacement}\"\nrationale = \"{}\"\n",
        rationale.replace('"', "\\\"")
    );
    text = format!("{}{}\n", text.trim_end(), entry);
    write_utf8(&path, &text)?;
    success_line(format!("added panel deprecation {panel} ({phase})"))
}

pub(in super::super) fn tooling_docs_build(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mode = args.first().map(String::as_str).unwrap_or_default();
    if matches!(mode, "--help" | "-h") || mode.is_empty() {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- tooling run docs-build -- <build|lint|serve>",
        );
    }
    let cfg_path = PathBuf::from(env_or_default("DOCS_CFG", "configs/docs/mkdocs.toml"));
    let cfg_path = if cfg_path.is_absolute() {
        cfg_path
    } else {
        workspace.path(cfg_path.to_string_lossy().as_ref())
    };
    let docs_venv = PathBuf::from(env_or_default("DOCS_VENV", "artifacts/docs/.venv"));
    let docs_venv = if docs_venv.is_absolute() {
        docs_venv
    } else {
        workspace.path(docs_venv.to_string_lossy().as_ref())
    };
    let mkdocs_bin = docs_venv.join("bin/mkdocs");
    if !cfg_path.is_file() || !mkdocs_bin.is_file() {
        return Ok(OpsCommandOutcome::failure(
            "docs-build requires DOCS_CFG and DOCS_VENV/bin/mkdocs to exist\n",
        ));
    }
    let cfg: TomlValue = toml::from_str(&read_utf8(&cfg_path)?)?;
    let mkdocs_config =
        cfg.get("mkdocs_config").and_then(TomlValue::as_str).unwrap_or("mkdocs.yml");
    let site_dir = cfg.get("site_dir").and_then(TomlValue::as_str).unwrap_or("artifacts/docs/site");
    let strict = cfg.get("strict").and_then(TomlValue::as_bool).unwrap_or(true);
    let dev_addr = cfg.get("dev_addr").and_then(TomlValue::as_str).unwrap_or("127.0.0.1:8000");
    if site_dir != "artifacts/docs/site" {
        return Ok(OpsCommandOutcome::failure(format!(
            "docs-build: site_dir must be artifacts/docs/site (got: {site_dir})\n"
        )));
    }
    let cache_dir = workspace.path("artifacts/docs/.cache");
    bijux_dna_infra::ensure_dir(&cache_dir)
        .with_context(|| format!("create {}", cache_dir.display()))?;
    let cmd_args = match mode {
        "build" => vec![
            "build".to_string(),
            "--config-file".to_string(),
            workspace.path(mkdocs_config).display().to_string(),
            "--site-dir".to_string(),
            workspace.path(site_dir).display().to_string(),
        ],
        "lint" => {
            let mut args = vec!["build".to_string()];
            if strict {
                args.push("--strict".to_string());
            }
            args.extend([
                "--config-file".to_string(),
                workspace.path(mkdocs_config).display().to_string(),
                "--site-dir".to_string(),
                workspace.path(site_dir).display().to_string(),
            ]);
            args
        }
        "serve" => vec![
            "serve".to_string(),
            "--config-file".to_string(),
            workspace.path(mkdocs_config).display().to_string(),
            "--dev-addr".to_string(),
            dev_addr.to_string(),
        ],
        other => {
            return Ok(OpsCommandOutcome::failure(format!(
                "unsupported docs-build mode: {other}\n"
            )))
        }
    };
    let program = mkdocs_bin.display().to_string();
    run_program_with_env(
        workspace,
        &program,
        &cmd_args,
        &[("XDG_CACHE_HOME".to_string(), cache_dir.display().to_string())],
    )
}

pub(in super::super) fn tooling_generate_configs(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("generate-configs", args)?;
    run_program(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "-p".to_string(),
            "bijux-dna-domain-compiler".to_string(),
            "--bin".to_string(),
            "compile_domain_configs".to_string(),
            "--".to_string(),
            "--domain-dir".to_string(),
            "domain".to_string(),
            "--configs-dir".to_string(),
            "configs".to_string(),
        ],
    )
}

pub(in super::super) fn tooling_generate_panel_compatibility_matrix(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let out = resolve_optional_output_arg(
        workspace,
        "generate-panel-compatibility-matrix",
        args,
        "docs/50-reference/PANEL_COMPATIBILITY_MATRIX.md",
    )?;
    let panels = toml::from_str::<TomlValue>(&read_utf8(
        &workspace.path("configs/vcf/panels/panels.toml"),
    )?)?;
    let maps =
        toml::from_str::<TomlValue>(&read_utf8(&workspace.path("configs/vcf/maps/maps.toml"))?)?;
    let panel_rows = panels.get("panel").and_then(TomlValue::as_array).cloned().unwrap_or_default();
    let map_rows = maps.get("map").and_then(TomlValue::as_array).cloned().unwrap_or_default();
    let mut maps_by_sb = BTreeMap::<(String, String), Vec<TomlValue>>::new();
    for row in map_rows {
        let key = (
            row.get("species_id").and_then(TomlValue::as_str).unwrap_or_default().to_string(),
            row.get("build_id").and_then(TomlValue::as_str).unwrap_or_default().to_string(),
        );
        maps_by_sb.entry(key).or_default().push(row);
    }
    let mut panels_sorted = panel_rows;
    panels_sorted.sort_by_key(|row| {
        (
            row.get("species_id").and_then(TomlValue::as_str).unwrap_or_default().to_string(),
            row.get("build_id").and_then(TomlValue::as_str).unwrap_or_default().to_string(),
            row.get("id").and_then(TomlValue::as_str).unwrap_or_default().to_string(),
        )
    });
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-panel-compatibility-matrix -->".to_string(),
        String::new(),
        "# PANEL_COMPATIBILITY_MATRIX".to_string(),
        String::new(),
        "## Purpose".to_string(),
        "Defines generated compatibility coverage for species/build, panel/map pairs, and downstream tool backends.".to_string(),
        String::new(),
        "## Scope".to_string(),
        "Derived from panel and map catalogs to document declared tool-tag compatibility.".to_string(),
        String::new(),
        "## Non-goals".to_string(),
        "- Replacing stage-level validation or runtime compatibility checks.".to_string(),
        String::new(),
        "## Contracts".to_string(),
        "- Matrix rows are generated from catalog authority and must not be hand-edited.".to_string(),
        "- Missing species/build map entries must be represented explicitly as unsupported rows.".to_string(),
        String::new(),
        "| Species | Build | Panel ID | Map ID | Tool Backend | Supported | Notes |".to_string(),
        "|---|---|---|---|---|---|---|".to_string(),
    ];
    for panel in panels_sorted {
        let species = panel.get("species_id").and_then(TomlValue::as_str).unwrap_or_default();
        let build = panel.get("build_id").and_then(TomlValue::as_str).unwrap_or_default();
        let panel_id = panel.get("id").and_then(TomlValue::as_str).unwrap_or_default();
        let compat = panel.get("compatibility").and_then(TomlValue::as_table);
        let tool_tags = compat
            .and_then(|table| table.get("tool_tags"))
            .and_then(TomlValue::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect::<BTreeSet<_>>();
        let maps_for = maps_by_sb.get(&(species.to_string(), build.to_string()));
        if maps_for.is_none() {
            lines.push(format!(
                "| `{species}` | `{build}` | `{panel_id}` | `-` | `-` | `no` | no map catalog for species/build |"
            ));
            continue;
        }
        for map in maps_for.unwrap_or(&Vec::new()) {
            let map_id = map.get("id").and_then(TomlValue::as_str).unwrap_or_default();
            let map_tool_tags = map
                .get("compatibility")
                .and_then(TomlValue::as_table)
                .and_then(|table| table.get("tool_tags"))
                .and_then(TomlValue::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                .collect::<BTreeSet<_>>();
            let union = tool_tags.union(&map_tool_tags).cloned().collect::<BTreeSet<_>>();
            for tool in union {
                let ok = tool_tags.contains(&tool) && map_tool_tags.contains(&tool);
                let mut notes = Vec::new();
                if tool == "minimac4" {
                    notes.push("requires panel m3vcf".to_string());
                }
                if tool == "glimpse" {
                    let format = compat
                        .and_then(|table| table.get("glimpse_reference_format"))
                        .and_then(TomlValue::as_str)
                        .unwrap_or_default();
                    notes.push(format!("GLIMPSE format={format}"));
                }
                let note = if notes.is_empty() { "-".to_string() } else { notes.join("; ") };
                lines.push(format!(
                    "| `{species}` | `{build}` | `{panel_id}` | `{map_id}` | `{tool}` | `{}` | {note} |",
                    if ok { "yes" } else { "no" }
                ));
            }
        }
    }
    write_utf8(&out, &format!("{}\n", lines.join("\n")))?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(in super::super) fn tooling_generate_policy_index(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("generate-policy-index", args)?;
    let out_file = workspace.path("artifacts/policies/index.md");
    let mut lines = vec![
        "# Policy Test Index".to_string(),
        String::new(),
        "Generated from crates/bijux-dna-policies/tests.".to_string(),
        String::new(),
    ];
    let mut files = WalkDir::new(workspace.path("crates/bijux-dna-policies/tests"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .map(|entry| entry.path().to_path_buf())
        .collect::<Vec<_>>();
    files.sort();
    let policy_re = Regex::new(r"(?m)^fn (policy__.+)$")?;
    for path in files {
        let rel = workspace.rel(&path).to_string_lossy().to_string();
        lines.push(format!("## {rel}"));
        for capture in policy_re.captures_iter(&read_utf8(&path)?) {
            if let Some(name) = capture.get(1).map(|value| value.as_str()) {
                lines.push(format!("- {name}"));
            }
        }
        lines.push(String::new());
    }
    write_utf8(&out_file, &format!("{}\n", lines.join("\n")))?;
    success_line(format!("wrote {}", out_file.display()))
}

pub(in super::super) fn tooling_image_qa(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    run_program(
        workspace,
        "cargo",
        &["run".to_string(), "--bin".to_string(), "image_qa".to_string(), "--".to_string()]
            .into_iter()
            .chain(args.iter().cloned())
            .collect::<Vec<_>>(),
    )
}

pub(in super::super) fn tooling_inventory(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("inventory", args)?;
    let out_dir = workspace.path("artifacts/inventory");
    bijux_dna_infra::ensure_dir(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
    let control_plane_out = out_dir.join("control_plane_inventory.txt");
    let configs_out = out_dir.join("configs_inventory.txt");
    let docs_out = out_dir.join("docs_index_coverage.txt");
    let assets_out = out_dir.join("assets_inventory.txt");
    let mut control_plane_lines = walk_file_list(workspace, "makes", Some("mk"))?;
    control_plane_lines.push('\n');
    control_plane_lines.push_str(&walk_file_list(
        workspace,
        "crates/bijux-dna-dev/src",
        Some("rs"),
    )?);
    write_utf8(&control_plane_out, &control_plane_lines)?;
    write_utf8(&configs_out, &walk_file_list(workspace, "configs", None)?)?;
    write_utf8(&assets_out, &walk_file_list(workspace, "assets", None)?)?;
    let mut lines = vec!["docs_index_coverage".to_string()];
    let mut dirs = WalkDir::new(workspace.path("docs"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_dir())
        .map(|entry| entry.path().to_path_buf())
        .collect::<Vec<_>>();
    dirs.sort();
    for dir in dirs {
        let rel = workspace.rel(&dir).to_string_lossy().to_string();
        let present = if dir.join("index.md").is_file() { "present" } else { "missing" };
        lines.push(format!("{rel}/index.md:{present}"));
    }
    write_utf8(&docs_out, &format!("{}\n", lines.join("\n")))?;
    success_line(format!(
        "wrote {}\nwrote {}\nwrote {}\nwrote {}",
        control_plane_out.display(),
        configs_out.display(),
        docs_out.display(),
        assets_out.display()
    ))
}

pub(in super::super) fn tooling_make_help(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let show_internal = match args {
        [] => false,
        [flag] if flag == "--internal" => true,
        [flag] if matches!(flag.as_str(), "--help" | "-h" | "--dry-run" | "--verbose") => {
            return success_line(
                "Usage: cargo run -p bijux-dna-dev -- tooling run make-help -- [--internal]",
            )
        }
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run make-help -- [--internal]\n",
            ))
        }
    };
    let readme = read_utf8(&workspace.path("makes/README.md"))?;
    let mut public = Vec::new();
    let mut in_public = false;
    for line in readme.lines() {
        if line.trim() == "Public targets (stable contract):" {
            in_public = true;
            continue;
        }
        if in_public && line.starts_with("- `") {
            if let Some(target) = line.split('`').nth(1) {
                public.push(target.to_string());
            }
            continue;
        }
        if in_public && !line.trim().is_empty() && !line.starts_with("- ") {
            break;
        }
    }
    let mut out = String::from("Public make targets:\n\n");
    for target in public {
        out.push_str(&format!("  {target:<22} from makes/README.md\n"));
    }
    if show_internal {
        let re = Regex::new(r"^([_a-zA-Z0-9-]+):\s*##\s*(.+)$")?;
        let mut internal = Vec::new();
        for line in read_utf8(&workspace.path("makes/cargo.mk"))?.lines() {
            let Some(capture) = re.captures(line) else {
                continue;
            };
            let name = capture.get(1).map(|value| value.as_str()).unwrap_or_default();
            let desc = capture.get(2).map(|value| value.as_str()).unwrap_or_default();
            if name.starts_with('_') || matches!(name, "domain-validate" | "examples-validate") {
                internal.push((name.to_string(), desc.to_string()));
            }
        }
        if !internal.is_empty() {
            out.push_str("\nInternal make targets:\n\n");
            for (name, desc) in internal {
                out.push_str(&format!("  {name:<22} {desc}\n"));
            }
        }
    }
    out.push_str("\nSee makes/README.md for the public surface contract.\n");
    Ok(OpsCommandOutcome::success(out))
}

pub(in super::super) fn tooling_repo_doctor(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mode = args.first().map_or("--fast", String::as_str);
    if matches!(mode, "--help" | "-h") {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- tooling run repo-doctor -- [--fast|--full]",
        );
    }
    let mut aggregate = String::new();
    let check_ids: Vec<&str> = match mode {
        "--fast" => vec![
            "check-root-layout",
            "check-legacy-automation-removed",
            "check-legacy-automation-references",
        ],
        "--full" => vec![
            "check-root-layout",
            "check-config-layout",
            "check-legacy-automation-removed",
            "check-legacy-automation-references",
        ],
        other => {
            return Ok(OpsCommandOutcome::failure(format!(
                "unsupported repo-doctor mode: {other}\n"
            )))
        }
    };
    run_check_ids(&mut aggregate, &check_ids)?;
    let docs_graph =
        run_native_ops_command(NativeOpsCommandKey::DocsCheckDocsGraph, workspace, &[])?;
    if !docs_graph.is_success() {
        return Ok(docs_graph);
    }
    aggregate.push_str(&docs_graph.stdout);
    if mode == "--full" {
        let generate_configs = tooling_generate_configs(workspace, &[])?;
        if !generate_configs.is_success() {
            return Ok(generate_configs);
        }
        aggregate.push_str(&generate_configs.stdout);
        let check_snapshot = tooling_check_config_snapshot(workspace, &[])?;
        if !check_snapshot.is_success() {
            return Ok(check_snapshot);
        }
        aggregate.push_str(&check_snapshot.stdout);
        let domain = DomainApplication::new()?.run("check-inventory", &[])?;
        if !domain.is_success() {
            return Ok(OpsCommandOutcome {
                exit_code: domain.exit_code,
                stdout: domain.stdout,
                stderr: domain.stderr,
            });
        }
        aggregate.push_str(&domain.stdout);
    }
    aggregate.push_str(&format!("repo-doctor: OK ({mode})\n"));
    Ok(OpsCommandOutcome::success(aggregate))
}

pub(in super::super) fn tooling_run_bijux(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if matches!(args.first().map(String::as_str), Some("--help" | "-h")) {
        return success_line("Usage: cargo run -p bijux-dna-dev -- tooling run bijux -- <args...>");
    }
    let mut command_args =
        vec!["run".to_string(), "--bin".to_string(), "bijux-dna".to_string(), "--".to_string()];
    if let Ok(platform) = std::env::var("BIJUX_PLATFORM") {
        if !platform.trim().is_empty() {
            command_args.push("--platform".to_string());
            command_args.push(platform);
        }
    }
    command_args.extend(args.iter().cloned());
    run_program(workspace, "cargo", &command_args)
}

pub(in super::super) fn tooling_setup_docs_venv(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("setup-docs-venv", args)?;
    let docs_py = env_or_default("DOCS_PY", "python3");
    let docs_venv =
        resolve_workspace_path(workspace, &env_or_default("DOCS_VENV", "artifacts/docs/.venv"));
    let docs_req = resolve_workspace_path(
        workspace,
        &env_or_default("DOCS_REQ", "configs/docs/requirements.txt"),
    );
    let docs_cache = workspace.path("artifacts/docs/.cache/pip");
    bijux_dna_infra::ensure_dir(&docs_cache)
        .with_context(|| format!("create {}", docs_cache.display()))?;
    let venv = run_program(
        workspace,
        &docs_py,
        &["-m".to_string(), "venv".to_string(), docs_venv.display().to_string()],
    )?;
    if !venv.is_success() {
        return Ok(venv);
    }
    let pip = docs_venv.join("bin/pip").display().to_string();
    let upgrade = run_program_with_env(
        workspace,
        &pip,
        &["install".to_string(), "--upgrade".to_string(), "pip".to_string()],
        &[("PIP_CACHE_DIR".to_string(), docs_cache.display().to_string())],
    )?;
    if !upgrade.is_success() {
        return Ok(upgrade);
    }
    run_program_with_env(
        workspace,
        &pip,
        &["install".to_string(), "-r".to_string(), docs_req.display().to_string()],
        &[("PIP_CACHE_DIR".to_string(), docs_cache.display().to_string())],
    )
}

pub(in super::super) fn tooling_simulate_coverage_regime(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if matches!(args.first().map(String::as_str), Some("--help" | "-h")) || args.is_empty() {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- tooling run simulate-coverage-regime -- <mean_depth_x> [--profile <name>]",
        );
    }
    let mean_depth = args[0].parse::<f64>().context("parse mean_depth_x as float")?;
    let mut profile = "default".to_string();
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--profile" => {
                profile = args.get(index + 1).context("missing value for --profile")?.clone();
                index += 2;
            }
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }
    let cfg: TomlValue =
        toml::from_str(&read_utf8(&workspace.path("configs/runtime/coverage_regimes.toml"))?)?;
    let decision = cfg
        .get("decision")
        .and_then(TomlValue::as_table)
        .and_then(|table| table.get("coverage_regime"))
        .and_then(TomlValue::as_table)
        .context("missing decision.coverage_regime")?;
    let base =
        decision.get("thresholds").and_then(TomlValue::as_table).context("missing thresholds")?;
    let profiles =
        decision.get("profiles").and_then(TomlValue::as_table).cloned().unwrap_or_default();
    let selected_profile = if profile == "default" {
        base.clone()
    } else {
        profiles
            .get(&profile)
            .and_then(TomlValue::as_table)
            .cloned()
            .ok_or_else(|| anyhow!("unknown profile: {profile}"))?
    };
    let gl_max = selected_profile
        .get("gl_max_depth")
        .and_then(TomlValue::as_float)
        .or_else(|| {
            selected_profile.get("gl_max_depth").and_then(TomlValue::as_integer).map(|v| v as f64)
        })
        .context("missing gl_max_depth")?;
    let pseudo_max = selected_profile
        .get("pseudohaploid_max_depth")
        .and_then(TomlValue::as_float)
        .or_else(|| {
            selected_profile
                .get("pseudohaploid_max_depth")
                .and_then(TomlValue::as_integer)
                .map(|v| v as f64)
        })
        .context("missing pseudohaploid_max_depth")?;
    let dip_min = selected_profile
        .get("diploid_min_depth")
        .and_then(TomlValue::as_float)
        .or_else(|| {
            selected_profile
                .get("diploid_min_depth")
                .and_then(TomlValue::as_integer)
                .map(|v| v as f64)
        })
        .context("missing diploid_min_depth")?;
    let (selected, pipeline_path) = if mean_depth <= gl_max {
        (
            "gl",
            vec![
                "vcf.call_gl",
                "vcf.damage_filter",
                "vcf.gl_propagation",
                "vcf.impute",
                "vcf.postprocess",
            ],
        )
    } else if mean_depth <= pseudo_max {
        (
            "pseudohaploid",
            vec!["vcf.call_pseudohaploid", "vcf.damage_filter", "vcf.impute", "vcf.postprocess"],
        )
    } else if mean_depth >= dip_min {
        ("diploid", vec!["vcf.call_diploid", "vcf.damage_filter", "vcf.impute", "vcf.postprocess"])
    } else {
        (
            "pseudohaploid",
            vec!["vcf.call_pseudohaploid", "vcf.damage_filter", "vcf.impute", "vcf.postprocess"],
        )
    };
    write_json_pretty(
        &workspace.path("artifacts/tmp/simulate_coverage_regime.last.json"),
        &json!({
            "decision": "decision.coverage_regime",
            "profile": profile,
            "coverage": { "mean_depth_x": mean_depth },
            "thresholds_used": {
                "gl_max_depth": gl_max,
                "pseudohaploid_max_depth": pseudo_max,
                "diploid_min_depth": dip_min,
            },
            "selected_regime": selected,
            "pipeline_path": pipeline_path,
        }),
    )?;
    Ok(OpsCommandOutcome::success(read_utf8(
        &workspace.path("artifacts/tmp/simulate_coverage_regime.last.json"),
    )?))
}

pub(in super::super) fn tooling_generate_domain_coverage_doc(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let out = match args {
        [] => workspace.path("docs/20-science/DOMAIN_COVERAGE.generated.md"),
        [flag, value] if flag == "--out" => resolve_workspace_path(workspace, value),
        [flag] if flag == "--help" || flag == "-h" => {
            return success_line(
                "Usage: cargo run -p bijux-dna-dev -- tooling run generate-domain-coverage-doc -- --out <path>",
            )
        }
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run generate-domain-coverage-doc -- --out <path>\n",
            ))
        }
    };
    generate_domain_coverage_doc(workspace, &out)?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(in super::super) fn tooling_generate_repo_root_map(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let out = resolve_optional_output_arg(
        workspace,
        "generate-repo-root-map",
        args,
        "docs/00-intro/REPO_ROOT_MAP.generated.md",
    )?;
    generate_repo_root_map(workspace, &out)?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(in super::super) fn tooling_generate_compatibility_matrix(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let out = resolve_optional_output_arg(
        workspace,
        "generate-compatibility-matrix",
        args,
        "docs/50-reference/COMPATIBILITY_MATRIX.md",
    )?;
    generate_compatibility_matrix(workspace, &out)?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(in super::super) fn tooling_generate_docs_graph(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let out = resolve_optional_output_arg(
        workspace,
        "generate-docs-graph",
        args,
        "docs/DOCS_GRAPH.toml",
    )?;
    generate_docs_graph(workspace, &out)?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(in super::super) fn tooling_generate_docs(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let out_root =
        match args {
            [] => workspace.path("docs"),
            [flag] if flag == "--help" || flag == "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run generate-docs -- [out-root]",
                )
            }
            [out] => resolve_workspace_path(workspace, out),
            _ => return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run generate-docs -- [out-root]\n",
            )),
        };
    bijux_dna_infra::ensure_dir(out_root.join("00-intro"))
        .with_context(|| format!("create {}", out_root.join("00-intro").display()))?;
    bijux_dna_infra::ensure_dir(out_root.join("20-science"))
        .with_context(|| format!("create {}", out_root.join("20-science").display()))?;
    bijux_dna_infra::ensure_dir(out_root.join("30-operations"))
        .with_context(|| format!("create {}", out_root.join("30-operations").display()))?;
    bijux_dna_infra::ensure_dir(out_root.join("50-reference"))
        .with_context(|| format!("create {}", out_root.join("50-reference").display()))?;

    generate_tool_index(workspace, &out_root.join("20-science/TOOL_INDEX.md"))?;
    generate_domain_coverage_doc(
        workspace,
        &out_root.join("20-science/DOMAIN_COVERAGE.generated.md"),
    )?;
    let container_outcome = ContainerApplication::new()?.run(
        "generate-qa-matrix",
        &[out_root.join("30-operations/APPTAINER_QA_MATRIX.md").display().to_string()],
    )?;
    if !container_outcome.is_success() {
        return Ok(OpsCommandOutcome {
            exit_code: container_outcome.exit_code,
            stdout: container_outcome.stdout,
            stderr: container_outcome.stderr,
        });
    }
    generate_repo_root_map(workspace, &out_root.join("00-intro/REPO_ROOT_MAP.generated.md"))?;
    generate_compatibility_matrix(
        workspace,
        &out_root.join("50-reference/COMPATIBILITY_MATRIX.md"),
    )?;
    generate_compatibility_reference_docs(workspace, &out_root.join("50-reference"))?;
    generate_docs_graph(workspace, &out_root.join("DOCS_GRAPH.toml"))?;
    success_line(format!("generated docs into {}", out_root.display()))
}

#[cfg(test)]
mod architecture_report_tests {
    use super::{
        extract_added_paths, extract_workspace_dependency_additions,
        render_architecture_report_markdown, PathBuf, Value,
    };

    fn snapshot_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/snapshots/bijux-dna-dev__tooling__architecture_report.md")
    }

    #[test]
    fn architecture_report_extracts_added_workspace_dependencies_only() {
        let diff = r#"
+bijux-dna-api = { path = "../bijux-dna-api" }
+serde = "1"
 other = "ignored"
"#;
        assert_eq!(extract_workspace_dependency_additions(diff), vec!["bijux-dna-api".to_string()]);
    }

    #[test]
    fn architecture_report_extracts_added_config_and_schema_paths() {
        let diff = "A\tconfigs/ci/policy_layers.toml\nM\tconfigs/ci/public_api_tiers.toml\nA\tscience/specs/data/example.json\n";
        assert_eq!(
            extract_added_paths(diff, "configs/"),
            vec!["configs/ci/policy_layers.toml".to_string()]
        );
        assert_eq!(
            extract_added_paths(diff, "science/specs/"),
            vec!["science/specs/data/example.json".to_string()]
        );
    }

    #[test]
    fn architecture_report_markdown_matches_snapshot() {
        let report: Value = serde_json::json!({
            "generated_at": "1970-01-01T00:00:00Z",
            "base_revision": "HEAD~1",
            "crates": [
                {
                    "crate_name": "bijux-dna-api",
                    "rust_file_count": 12,
                    "rust_loc": 4200,
                    "public_item_count": 44
                },
                {
                    "crate_name": "bijux-dna-runtime",
                    "rust_file_count": 9,
                    "rust_loc": 2100,
                    "public_item_count": 17
                }
            ],
            "dependency_additions": ["bijux-dna-api"],
            "new_config_files": ["configs/ci/crate-boundaries.toml"],
            "new_schema_files": ["science/specs/data/example.json"]
        });
        let rendered = render_architecture_report_markdown(&report);
        let expected = std::fs::read_to_string(snapshot_path())
            .unwrap_or_else(|err| panic!("read {}: {err}", snapshot_path().display()));
        assert_eq!(rendered, expected);
    }
}
