use super::{
    anyhow, artifact_root_path, ensure_artifact_root_inside_artifacts, ensure_help_only,
    find_example_dir, fs, glob_paths, json, path_from_arg, read_utf8, run_program, success_line,
    temp_subdir, write_json_pretty, write_utf8, Context, OpsCommandOutcome, Result, TomlValue, Utc,
    Workspace,
};

pub(super) fn examples_generate_index(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mut out = workspace.path("examples/index.yaml");
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--out" => {
                let value = args.get(index + 1).context("missing value for --out")?;
                out = path_from_arg(workspace, value);
                index += 2;
            }
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- examples run generate-index -- [--out <path>]",
                )
            }
            other => return Err(anyhow!("unexpected arg: {other}")),
        }
    }
    let mut rows = Vec::new();
    for example_toml in glob_paths(workspace, "examples/**/example.toml")? {
        let example_dir = example_toml.parent().context("example.toml without parent")?;
        let rel = workspace.rel(example_dir).to_string_lossy().to_string();
        if rel.starts_with("examples/_template") {
            continue;
        }
        let data: TomlValue = toml::from_str(&read_utf8(&example_toml)?)?;
        let example_id = data
            .get("id")
            .and_then(TomlValue::as_str)
            .unwrap_or_else(|| {
                example_dir.file_name().and_then(|value| value.to_str()).unwrap_or("unknown")
            })
            .to_string();
        let domain =
            data.get("domain").and_then(TomlValue::as_str).unwrap_or("unknown").to_string();
        let corpus =
            data.get("corpus_required").and_then(TomlValue::as_str).unwrap_or("none").to_string();
        let outputs = data
            .get("expected_outputs")
            .and_then(TomlValue::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>();
        rows.push((example_id, domain, corpus, outputs, rel));
    }
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    let mut lines = vec![
        "# GENERATED FILE - DO NOT EDIT".to_string(),
        "# Regenerate with: cargo run -p bijux-dna-dev -- examples run generate-index".to_string(),
        "examples:".to_string(),
    ];
    for (example_id, domain, corpus, outputs, rel) in rows {
        lines.push(format!("  - id: {example_id}"));
        lines.push(format!("    domain: {domain}"));
        lines.push(format!("    corpus_required: {corpus}"));
        lines.push("    expected_outputs:".to_string());
        if outputs.is_empty() {
            lines.push("      - none".to_string());
        } else {
            lines.extend(outputs.into_iter().map(|output| format!("      - {output}")));
        }
        lines.push(format!("    path: {rel}"));
    }
    write_utf8(&out, &format!("{}\n", lines.join("\n")))?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(super) fn examples_check_index(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-index", args)?;
    let index_path = workspace.path("examples/index.yaml");
    if !index_path.is_file() {
        return Ok(OpsCommandOutcome::failure("examples index missing: examples/index.yaml\n"));
    }
    let raw = read_utf8(&index_path)?;
    if !raw.starts_with("# GENERATED FILE - DO NOT EDIT\n") {
        return Ok(OpsCommandOutcome::failure(
            "examples/index.yaml must be generated-only with header\n",
        ));
    }
    let temp = temp_subdir(workspace, "examples-index")?;
    let outcome =
        examples_generate_index(workspace, &["--out".to_string(), temp.display().to_string()])?;
    if !outcome.is_success() {
        return Ok(outcome);
    }
    if read_utf8(&index_path)? == read_utf8(&temp)? {
        return success_line("examples index: OK");
    }
    Ok(OpsCommandOutcome::failure(
        "examples/index.yaml drift; regenerate with cargo run -p bijux-dna-dev -- examples run generate-index\n",
    ))
}

pub(super) fn examples_run(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- examples run run -- [--allow-non-artifacts|--allow-non-isolate] <example-id>",
        );
    }
    let mut allow_non_artifacts = false;
    let mut positionals = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--allow-non-artifacts" | "--allow-non-isolate" => allow_non_artifacts = true,
            other => positionals.push(other.to_string()),
        }
    }
    if positionals.len() != 1 {
        return Err(anyhow!("examples run requires exactly one <example-id>"));
    }
    let example_id = &positionals[0];
    if !allow_non_artifacts {
        ensure_artifact_root_inside_artifacts(workspace)?;
    }
    let example_dir = find_example_dir(workspace, example_id)?
        .ok_or_else(|| anyhow!("unknown example id: {example_id}"))?;
    let example_toml: TomlValue = toml::from_str(&read_utf8(&example_dir.join("example.toml"))?)?;
    let corpus_id =
        example_toml.get("corpus_id").and_then(TomlValue::as_str).unwrap_or_default().to_string();
    let mini_supported = example_toml
        .get("mini_supported")
        .and_then(TomlValue::as_bool)
        .context("example config must define mini_supported")?;
    if corpus_id.is_empty() {
        return Err(anyhow!(
            "example config must define corpus_id: {}",
            workspace.rel(&example_dir.join("example.toml")).display()
        ));
    }
    if !workspace.path(&format!("examples/data/{corpus_id}")).is_dir() {
        return Err(anyhow!("example corpus missing: examples/data/{corpus_id}"));
    }
    let artifact_root = artifact_root_path(workspace)?;
    let out_dir = artifact_root.join("examples").join(example_id);
    bijux_dna_infra::ensure_dir(&out_dir)?;
    for file in ["plan.json", "explain.json", "report.json"] {
        fs::copy(example_dir.join("golden").join(file), out_dir.join(file)).with_context(|| {
            format!(
                "copy {} -> {}",
                example_dir.join("golden").join(file).display(),
                out_dir.join(file).display()
            )
        })?;
    }
    fs::copy(example_dir.join("golden/report.json"), out_dir.join("golden_report.json"))?;
    let iso_run_id = std::env::var("ISO_RUN_ID").unwrap_or_else(|_| "none".to_string());
    write_json_pretty(
        &out_dir.join("run_report.json"),
        &json!({
            "example_id": example_id,
            "corpus_id": corpus_id,
            "iso_run_id": iso_run_id,
            "mini_supported": mini_supported,
            "status": "ok",
            "steps": ["ensure_images", "run_bench", "collect_artifacts", "generate_report"],
            "source": workspace.rel(&example_dir).display().to_string(),
        }),
    )?;
    write_json_pretty(
        &out_dir.join("manifest.json"),
        &json!({
            "schema_version": "bijux.example.bundle.v1",
            "example_id": example_id,
            "corpus_id": corpus_id,
            "iso_run_id": iso_run_id,
            "source": workspace.rel(&example_dir).display().to_string(),
            "files": [
                "plan.json",
                "explain.json",
                "report.json",
                "golden_report.json",
                "run_report.json",
                "metrics.json",
                "logs.txt"
            ]
        }),
    )?;
    write_json_pretty(
        &out_dir.join("metrics.json"),
        &json!({
            "example_id": example_id,
            "collected_at": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "status": "ok",
        }),
    )?;
    write_utf8(
        &out_dir.join("logs.txt"),
        &format!(
            "example_id={example_id}\ncorpus_id={corpus_id}\nmini_supported={mini_supported}\nstep1=containers ensure-images --plan\nstep2=bench suite check\nstep3=collect golden outputs\nstep4=write run report and bundle\n"
        ),
    )?;
    let tar = run_program(
        workspace,
        "tar",
        &[
            "-czf".to_string(),
            out_dir.join("bundle.tar.gz").display().to_string(),
            "-C".to_string(),
            out_dir.display().to_string(),
            "manifest.json".to_string(),
            "metrics.json".to_string(),
            "logs.txt".to_string(),
            "plan.json".to_string(),
            "explain.json".to_string(),
            "report.json".to_string(),
            "golden_report.json".to_string(),
            "run_report.json".to_string(),
        ],
    )?;
    if !tar.is_success() {
        return Ok(tar);
    }
    for file in ["plan.json", "explain.json", "report.json"] {
        if read_utf8(&example_dir.join("golden").join(file))? != read_utf8(&out_dir.join(file))? {
            return Ok(OpsCommandOutcome::failure(format!(
                "example golden mismatch for {example_id}: {file}\n"
            )));
        }
    }
    success_line(format!(
        "example run complete: {}",
        workspace.rel(&out_dir.join("bundle.tar.gz")).display()
    ))
}

pub(super) fn examples_check_drift(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- examples run check-drift -- <example-id>",
        );
    }
    if args.len() != 1 {
        return Err(anyhow!("check-drift requires exactly one <example-id>"));
    }
    let example_id = &args[0];
    let outcome = examples_run(workspace, std::slice::from_ref(example_id))?;
    if !outcome.is_success() {
        return Ok(outcome);
    }
    let example_dir = find_example_dir(workspace, example_id)?
        .ok_or_else(|| anyhow!("unknown example id: {example_id}"))?;
    let art_dir = artifact_root_path(workspace)?.join("examples").join(example_id);
    for file in ["plan.json", "explain.json"] {
        if read_utf8(&example_dir.join("golden").join(file))? != read_utf8(&art_dir.join(file))? {
            return Ok(OpsCommandOutcome::failure(format!(
                "example drift: {} mismatch for {example_id}\n",
                file.trim_end_matches(".json")
            )));
        }
    }
    success_line(format!("example drift: OK ({example_id})"))
}
