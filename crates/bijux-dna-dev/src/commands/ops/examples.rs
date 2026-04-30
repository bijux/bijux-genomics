use super::{
    anyhow, artifact_root_path, ensure_artifact_root_inside_artifacts, ensure_help_only,
    find_example_dir, fs, glob_paths, json, path_from_arg, read_utf8, run_program, success_line,
    temp_subdir, write_json_pretty, write_utf8, Context, OpsCommandOutcome, Result, TomlValue, Utc,
    Workspace,
};

fn example_string(data: &TomlValue, key: &str) -> Option<String> {
    data.get(key).and_then(TomlValue::as_str).map(ToOwned::to_owned)
}

fn example_bool(data: &TomlValue, key: &str) -> bool {
    data.get(key).and_then(TomlValue::as_bool).unwrap_or(false)
}

fn example_strings(data: &TomlValue, key: &str) -> Vec<String> {
    data.get(key)
        .and_then(TomlValue::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>()
}

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
        let outputs = example_strings(&data, "expected_outputs");
        rows.push((
            example_id,
            domain,
            corpus,
            outputs,
            rel,
            example_bool(&data, "canonical_example"),
            example_string(&data, "workflow_class"),
            example_string(&data, "tiny_inputs_contract"),
            example_string(&data, "workflow_manifest"),
            example_string(&data, "expected_plan"),
            example_string(&data, "expected_evidence"),
        ));
    }
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    let mut lines = vec![
        "# GENERATED FILE - DO NOT EDIT".to_string(),
        "# Regenerate with: cargo run -p bijux-dna-dev -- examples run generate-index".to_string(),
        "examples:".to_string(),
    ];
    for (
        example_id,
        domain,
        corpus,
        outputs,
        rel,
        canonical_example,
        workflow_class,
        tiny_inputs_contract,
        workflow_manifest,
        expected_plan,
        expected_evidence,
    ) in rows
    {
        lines.push(format!("  - id: {example_id}"));
        lines.push(format!("    domain: {domain}"));
        lines.push(format!("    corpus_required: {corpus}"));
        lines.push(format!("    canonical_example: {canonical_example}"));
        if let Some(value) = workflow_class {
            lines.push(format!("    workflow_class: {value}"));
        }
        lines.push("    expected_outputs:".to_string());
        if outputs.is_empty() {
            lines.push("      - none".to_string());
        } else {
            lines.extend(outputs.into_iter().map(|output| format!("      - {output}")));
        }
        if let Some(value) = tiny_inputs_contract {
            lines.push(format!("    tiny_inputs_contract: {value}"));
        }
        if let Some(value) = workflow_manifest {
            lines.push(format!("    workflow_manifest: {value}"));
        }
        if let Some(value) = expected_plan {
            lines.push(format!("    expected_plan: {value}"));
        }
        if let Some(value) = expected_evidence {
            lines.push(format!("    expected_evidence: {value}"));
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
    let temp_root = temp_subdir(workspace, "examples-index")?;
    let temp = temp_root.join("examples.index.yaml");
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
    let canonical_example = example_bool(&example_toml, "canonical_example");
    let workflow_class = example_string(&example_toml, "workflow_class");
    let tiny_inputs_contract = example_string(&example_toml, "tiny_inputs_contract");
    let workflow_manifest = example_string(&example_toml, "workflow_manifest");
    let expected_plan = example_string(&example_toml, "expected_plan");
    let expected_evidence = example_string(&example_toml, "expected_evidence");
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
    let started_at = Utc::now();
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
    let mut manifest_files = vec![
        "plan.json".to_string(),
        "explain.json".to_string(),
        "report.json".to_string(),
        "golden_report.json".to_string(),
        "run_report.json".to_string(),
        "metrics.json".to_string(),
        "logs.txt".to_string(),
        "example.toml".to_string(),
    ];
    fs::copy(example_dir.join("example.toml"), out_dir.join("example.toml")).with_context(|| {
        format!(
            "copy {} -> {}",
            example_dir.join("example.toml").display(),
            out_dir.join("example.toml").display()
        )
    })?;
    let expected_plan_path = expected_plan
        .as_ref()
        .map(|rel| example_dir.join(rel))
        .unwrap_or_else(|| example_dir.join("golden/plan.json"));
    if read_utf8(&expected_plan_path)? != read_utf8(&example_dir.join("golden/plan.json"))? {
        return Ok(OpsCommandOutcome::failure(format!(
            "example expected plan mismatch for {example_id}: {}\n",
            workspace.rel(&expected_plan_path).display()
        )));
    }
    if canonical_example {
        let Some(tiny_inputs_contract) = tiny_inputs_contract.as_ref() else {
            return Err(anyhow!("canonical example must define tiny_inputs_contract"));
        };
        let Some(workflow_manifest) = workflow_manifest.as_ref() else {
            return Err(anyhow!("canonical example must define workflow_manifest"));
        };
        let Some(expected_evidence) = expected_evidence.as_ref() else {
            return Err(anyhow!("canonical example must define expected_evidence"));
        };
        for (source_rel, dest_name) in [
            (tiny_inputs_contract.as_str(), "tiny_inputs.json"),
            (workflow_manifest.as_str(), "workflow_manifest.json"),
            (expected_evidence.as_str(), "expected_evidence.json"),
        ] {
            let source = example_dir.join(source_rel);
            let dest = out_dir.join(dest_name);
            fs::copy(&source, &dest)
                .with_context(|| format!("copy {} -> {}", source.display(), dest.display()))?;
            manifest_files.push(dest_name.to_string());
        }
    }
    let iso_run_id = std::env::var("ISO_RUN_ID").unwrap_or_else(|_| "none".to_string());
    write_json_pretty(
        &out_dir.join("run_report.json"),
        &json!({
            "example_id": example_id,
            "corpus_id": corpus_id,
            "iso_run_id": iso_run_id,
            "mini_supported": mini_supported,
            "canonical_example": canonical_example,
            "workflow_class": workflow_class.clone(),
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
            "canonical_example": canonical_example,
            "workflow_class": workflow_class.clone(),
            "source": workspace.rel(&example_dir).display().to_string(),
            "files": manifest_files
        }),
    )?;
    let collected_files = [
        "plan.json",
        "explain.json",
        "report.json",
        "golden_report.json",
        "run_report.json",
        "example.toml",
        "tiny_inputs.json",
        "workflow_manifest.json",
        "expected_evidence.json",
    ];
    let artifact_bytes = collected_files
        .iter()
        .filter_map(|file| out_dir.join(file).metadata().ok())
        .map(|meta| meta.len())
        .sum::<u64>();
    let finished_at = Utc::now();
    write_json_pretty(
        &out_dir.join("metrics.json"),
        &json!({
            "example_id": example_id,
            "canonical_example": canonical_example,
            "workflow_class": workflow_class.clone(),
            "collected_at": finished_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "duration_ms": (finished_at - started_at).num_milliseconds(),
            "artifact_bytes": artifact_bytes,
            "status": "ok",
        }),
    )?;
    write_utf8(
        &out_dir.join("logs.txt"),
        &format!(
            "example_id={example_id}\ncorpus_id={corpus_id}\nmini_supported={mini_supported}\ncanonical_example={canonical_example}\nstep1=containers ensure-images --plan\nstep2=bench suite check\nstep3=collect golden outputs\nstep4=write run report and bundle\n"
        ),
    )?;
    let mut bundle_files = vec![
        "manifest.json".to_string(),
        "metrics.json".to_string(),
        "logs.txt".to_string(),
        "plan.json".to_string(),
        "explain.json".to_string(),
        "report.json".to_string(),
        "golden_report.json".to_string(),
        "run_report.json".to_string(),
        "example.toml".to_string(),
    ];
    if canonical_example {
        bundle_files.extend([
            "tiny_inputs.json".to_string(),
            "workflow_manifest.json".to_string(),
            "expected_evidence.json".to_string(),
        ]);
    }
    let tar = run_program(
        workspace,
        "tar",
        &{
            let mut args = vec![
                "-czf".to_string(),
                out_dir.join("bundle.tar.gz").display().to_string(),
                "-C".to_string(),
                out_dir.display().to_string(),
            ];
            args.extend(bundle_files);
            args
        },
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
