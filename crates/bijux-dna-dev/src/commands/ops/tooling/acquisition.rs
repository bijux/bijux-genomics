use super::{
    anyhow, artifact_root_path, assert_no_excess_float_precision, ensure_help_only, examples_run,
    find_first_named_file, id_catalog, json, materialize_controlled_file, normalize_benchmark_html,
    path_from_arg, read_json_value, read_utf8, relative_diff, run_program_with_env, sha256_hex,
    sha256_hex_bytes, stable_now_utc_compact, stable_now_utc_string, success_line, toml_string,
    toml_value_string, tooling_simulate_coverage_regime, value_string, write_json_pretty,
    write_utf8, BTreeMap, Context, OpsCommandOutcome, PathBuf, Regex, Result, TomlValue, Value,
    WalkDir, Workspace,
};

pub(in super::super) fn tooling_acquire_reference(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mut download = false;
    let mut verbose = false;
    let mut species_filter = String::new();
    let mut build_filter = String::new();
    let mut cache_root = workspace.path("artifacts/reference_store");
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run acquire-reference -- [--download] [--species <species-id>] [--build <build-id>] [--cache-root <dir>] [--verbose]",
                )
            }
            "--download" => {
                download = true;
                index += 1;
            }
            "--species" => {
                species_filter = args
                    .get(index + 1)
                    .cloned()
                    .context("missing value for --species")?;
                index += 2;
            }
            "--build" => {
                build_filter = args
                    .get(index + 1)
                    .cloned()
                    .context("missing value for --build")?;
                index += 2;
            }
            "--cache-root" => {
                cache_root = path_from_arg(
                    workspace,
                    args.get(index + 1)
                        .context("missing value for --cache-root")?,
                );
                index += 2;
            }
            "--verbose" => {
                verbose = true;
                index += 1;
            }
            other => return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n"))),
        }
    }

    let cfg = toml::from_str::<TomlValue>(&read_utf8(
        &workspace.path("configs/runtime/reference_bank.toml"),
    )?)?;
    let references =
        cfg.get("reference").and_then(TomlValue::as_array).cloned().unwrap_or_default();
    let acquire_log_root = workspace.path("artifacts/containers/smoke/reference-acquire");
    bijux_dna_infra::ensure_dir(&acquire_log_root)
        .with_context(|| format!("create {}", acquire_log_root.display()))?;
    let lock_json = workspace.path("configs/runtime/references/locks/lock.json");
    let lock_sha = workspace.path("configs/runtime/references/locks/lock.json.sha256");
    let mut stdout = String::new();
    let mut rows = Vec::new();
    let mut log_rows = Vec::new();

    for row in references {
        let species = toml_string(row.get("species_id"))?;
        let build = toml_string(row.get("build_id"))?;
        if !species_filter.is_empty() && species_filter != species {
            continue;
        }
        if !build_filter.is_empty() && build_filter != build {
            continue;
        }
        let url = toml_string(row.get("fasta_url"))?;
        let expected = toml_string(row.get("fasta_sha256"))?;
        let license_id = toml_string(row.get("license_id"))?;
        let license_url = toml_string(row.get("license_url"))?;
        let required_indexes = row
            .get("required_indexes")
            .and_then(TomlValue::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|value| toml_value_string(&value))
            .collect::<Vec<_>>();
        let root_dir = cache_root.join(&species).join(&build);
        let raw_dir = root_dir.join("refs/raw");
        let normalized_dir = root_dir.join("refs/normalized");
        let derived_dir = root_dir.join("refs/derived");
        bijux_dna_infra::ensure_dir(&raw_dir)
            .with_context(|| format!("create {}", raw_dir.display()))?;
        bijux_dna_infra::ensure_dir(&normalized_dir)
            .with_context(|| format!("create {}", normalized_dir.display()))?;
        bijux_dna_infra::ensure_dir(&derived_dir)
            .with_context(|| format!("create {}", derived_dir.display()))?;
        let raw_fasta = raw_dir.join("reference.fa.gz");
        let filename = raw_fasta
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("reference.fa.gz")
            .to_string();
        let synthetic = format!("synthetic reference payload for {species}/{build}\n").into_bytes();
        let materialized = materialize_controlled_file(
            &raw_fasta,
            &url,
            &expected,
            &synthetic,
            download,
            verbose,
            &format!("{species}:{build}"),
            &mut stdout,
        )?;
        let mut index_outputs = Vec::new();
        for tool in &required_indexes {
            let output = match tool.as_str() {
                "samtools_faidx" => {
                    let path = normalized_dir.join(format!("{filename}.fai"));
                    write_utf8(&path, &format!("{filename}\t0\t0\t0\t0\n"))?;
                    path
                }
                "bwa_index" => {
                    let path = normalized_dir.join(format!("{filename}.bwt"));
                    write_utf8(&path, "synthetic-bwa-index\n")?;
                    path
                }
                "bowtie2_index" => {
                    let path = normalized_dir.join(format!("{filename}.1.bt2"));
                    write_utf8(&path, "synthetic-bowtie2-index\n")?;
                    path
                }
                "star_genome_generate" => {
                    let path = normalized_dir.join("star/genomeParameters.txt");
                    write_utf8(&path, "synthetic-star-index\n")?;
                    path
                }
                other => return Err(anyhow!("unsupported required index tool: {other}")),
            };
            index_outputs.push(json!({
                "tool": tool,
                "status": "synthetic",
                "output": output.display().to_string(),
            }));
        }
        write_json_pretty(
            &derived_dir.join("materialization.json"),
            &json!({
                "species_id": species,
                "build_id": build,
                "license_id": license_id,
                "license_url": license_url,
                "required_indexes": required_indexes,
                "index_outputs": index_outputs,
            }),
        )?;
        rows.push(json!({
            "species_id": species,
            "build_id": build,
            "fasta_url": url,
            "fasta_sha256": expected,
            "observed_sha256": materialized.observed_sha256,
            "license_id": license_id,
            "license_url": license_url,
            "required_indexes": required_indexes,
            "layout": {
                "raw": raw_dir.strip_prefix(&cache_root).unwrap_or(&raw_dir).display().to_string(),
                "normalized": normalized_dir.strip_prefix(&cache_root).unwrap_or(&normalized_dir).display().to_string(),
                "derived": derived_dir.strip_prefix(&cache_root).unwrap_or(&derived_dir).display().to_string(),
            },
            "action": materialized.action,
        }));
        log_rows.push(json!({
            "species_id": species,
            "build_id": build,
            "download": download,
            "action": materialized.action,
        }));
    }
    rows.sort_by(|left, right| {
        value_string(left.get("species_id")).cmp(&value_string(right.get("species_id"))).then_with(
            || value_string(left.get("build_id")).cmp(&value_string(right.get("build_id"))),
        )
    });
    let payload = json!({
        "schema_version": 1,
        "generated_at_utc": stable_now_utc_string(),
        "source": "configs/runtime/reference_bank.toml",
        "references": rows,
    });
    let raw = format!("{}\n", serde_json::to_string_pretty(&payload)?);
    write_utf8(&lock_json, &raw)?;
    write_utf8(
        &lock_sha,
        &format!(
            "{}  configs/runtime/references/locks/lock.json\n",
            sha256_hex_bytes(raw.as_bytes())
        ),
    )?;
    let run_log =
        acquire_log_root.join(format!("reference-acquire-{}.json", stable_now_utc_compact()));
    write_json_pretty(
        &run_log,
        &json!({
            "rows": log_rows,
            "cache_root": cache_root.display().to_string(),
        }),
    )?;
    stdout.push_str(&format!(
        "wrote {}\nwrote {}\nwrote {}\n",
        workspace.rel(&lock_json).display(),
        workspace.rel(&lock_sha).display(),
        workspace.rel(&run_log).display()
    ));
    Ok(OpsCommandOutcome::success(stdout))
}

pub(in super::super) fn tooling_acquire_panels(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mut download = false;
    let mut verbose = false;
    let mut panel_filter = String::new();
    let mut cache_root = workspace.path("artifacts/vcf/reference_store/panels");
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run acquire-panels -- [--download] [--panel <panel-id>] [--cache-root <dir>] [--verbose]",
                )
            }
            "--download" => {
                download = true;
                index += 1;
            }
            "--panel" => {
                panel_filter = args
                    .get(index + 1)
                    .cloned()
                    .context("missing value for --panel")?;
                index += 2;
            }
            "--cache-root" => {
                cache_root = path_from_arg(
                    workspace,
                    args.get(index + 1)
                        .context("missing value for --cache-root")?,
                );
                index += 2;
            }
            "--verbose" => {
                verbose = true;
                index += 1;
            }
            other => return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n"))),
        }
    }

    let cfg = toml::from_str::<TomlValue>(&read_utf8(
        &workspace.path("configs/vcf/panels/panels.toml"),
    )?)?;
    let panels = cfg.get("panel").and_then(TomlValue::as_array).cloned().unwrap_or_default();
    let acquire_log_root = workspace.path("artifacts/containers/smoke/panel-acquire");
    bijux_dna_infra::ensure_dir(&acquire_log_root)
        .with_context(|| format!("create {}", acquire_log_root.display()))?;
    let lock_json = workspace.path("configs/vcf/panels/locks/lock.json");
    let lock_sha = workspace.path("configs/vcf/panels/locks/lock.json.sha256");
    let mut stdout = String::new();
    let mut lock_rows = Vec::new();
    let mut log_rows = Vec::new();

    for panel in panels {
        let panel_id = toml_string(panel.get("id"))?;
        if !panel_filter.is_empty() && panel_filter != panel_id {
            continue;
        }
        let species = toml_string(panel.get("species_id"))?;
        let build = toml_string(panel.get("build_id"))?;
        let version = toml_string(panel.get("version"))?;
        let license = toml_string(panel.get("license"))?;
        let citation = toml_string(panel.get("citation"))?;
        let files = panel.get("files").and_then(TomlValue::as_array).cloned().unwrap_or_default();
        let panel_root = cache_root.join(&species).join(&build).join(&panel_id);
        let raw_dir = panel_root.join("raw");
        let normalized_dir = panel_root.join("normalized");
        let derived_dir = panel_root.join("derived");
        bijux_dna_infra::ensure_dir(&raw_dir)
            .with_context(|| format!("create {}", raw_dir.display()))?;
        bijux_dna_infra::ensure_dir(&normalized_dir)
            .with_context(|| format!("create {}", normalized_dir.display()))?;
        bijux_dna_infra::ensure_dir(&derived_dir)
            .with_context(|| format!("create {}", derived_dir.display()))?;
        let mut manifest_files = Vec::new();
        for file in files {
            let name = toml_string(file.get("name"))?;
            let rel_path = toml_string(file.get("path"))?;
            let url = toml_string(file.get("url"))?;
            let expected = toml_string(file.get("checksum_sha256"))?;
            let format_name = toml_string(file.get("format"))?;
            let dest = raw_dir.join(&rel_path);
            let synthetic = format!("synthetic payload for {panel_id}/{name}\n").into_bytes();
            let materialized = materialize_controlled_file(
                &dest,
                &url,
                &expected,
                &synthetic,
                download,
                verbose,
                &format!("{panel_id}:{name}"),
                &mut stdout,
            )?;
            manifest_files.push(json!({
                "name": name,
                "path": rel_path,
                "materialized_path": dest.strip_prefix(&cache_root).unwrap_or(&dest).display().to_string(),
                "url": url,
                "checksum_sha256": expected,
                "observed_sha256": materialized.observed_sha256,
                "format": format_name,
                "action": materialized.action,
            }));
        }
        write_utf8(
            &derived_dir.join("overlap.tsv"),
            "chr\toverlap_sites\toverlap_fraction\nall\t0\t0.0\n",
        )?;
        let index_stub = normalized_dir.join("panel.vcf.gz.tbi");
        if !index_stub.exists() {
            write_utf8(&index_stub, "tabix-index-placeholder\n")?;
        }
        let file_count = manifest_files.len();
        lock_rows.push(json!({
            "id": panel_id,
            "species_id": species,
            "build_id": build,
            "version": version,
            "license": license,
            "citation": citation,
            "files": manifest_files,
            "storage_layout": {
                "raw": raw_dir.strip_prefix(&cache_root).unwrap_or(&raw_dir).display().to_string(),
                "normalized": normalized_dir.strip_prefix(&cache_root).unwrap_or(&normalized_dir).display().to_string(),
                "derived": derived_dir.strip_prefix(&cache_root).unwrap_or(&derived_dir).display().to_string(),
            },
        }));
        log_rows.push(json!({
            "panel_id": panel_id,
            "species_id": species,
            "build_id": build,
            "download": download,
            "file_count": file_count,
        }));
    }
    lock_rows.sort_by_key(|left| value_string(left.get("id")));
    let payload = json!({
        "schema_version": 2,
        "generated_at_utc": stable_now_utc_string(),
        "source": "configs/vcf/panels/panels.toml",
        "panels": lock_rows,
    });
    let raw = format!("{}\n", serde_json::to_string_pretty(&payload)?);
    write_utf8(&lock_json, &raw)?;
    write_utf8(
        &lock_sha,
        &format!("{}  configs/vcf/panels/locks/lock.json\n", sha256_hex_bytes(raw.as_bytes())),
    )?;
    let run_log = acquire_log_root.join(format!("panel-acquire-{}.json", stable_now_utc_compact()));
    write_json_pretty(
        &run_log,
        &json!({
            "rows": log_rows,
            "cache_root": cache_root.display().to_string(),
        }),
    )?;
    stdout.push_str(&format!(
        "wrote {}\nwrote {}\nwrote {}\n",
        workspace.rel(&lock_json).display(),
        workspace.rel(&lock_sha).display(),
        workspace.rel(&run_log).display()
    ));
    Ok(OpsCommandOutcome::success(stdout))
}

pub(in super::super) fn tooling_acquire_maps(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mut download = false;
    let mut verbose = false;
    let mut map_filter = String::new();
    let mut cache_root = workspace.path("artifacts/vcf/reference_store/maps");
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run acquire-maps -- [--download] [--map <map-id>] [--cache-root <dir>] [--verbose]",
                )
            }
            "--download" => {
                download = true;
                index += 1;
            }
            "--map" => {
                map_filter = args
                    .get(index + 1)
                    .cloned()
                    .context("missing value for --map")?;
                index += 2;
            }
            "--cache-root" => {
                cache_root = path_from_arg(
                    workspace,
                    args.get(index + 1)
                        .context("missing value for --cache-root")?,
                );
                index += 2;
            }
            "--verbose" => {
                verbose = true;
                index += 1;
            }
            other => return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n"))),
        }
    }

    let cfg =
        toml::from_str::<TomlValue>(&read_utf8(&workspace.path("configs/vcf/maps/maps.toml"))?)?;
    let maps = cfg.get("map").and_then(TomlValue::as_array).cloned().unwrap_or_default();
    let acquire_log_root = workspace.path("artifacts/containers/smoke/map-acquire");
    bijux_dna_infra::ensure_dir(&acquire_log_root)
        .with_context(|| format!("create {}", acquire_log_root.display()))?;
    let mut stdout = String::new();
    let mut rows = Vec::new();

    for map in maps {
        let map_id = toml_string(map.get("id"))?;
        if !map_filter.is_empty() && map_filter != map_id {
            continue;
        }
        let species = toml_string(map.get("species_id"))?;
        let build = toml_string(map.get("build_id"))?;
        let files = map.get("files").and_then(TomlValue::as_array).cloned().unwrap_or_default();
        let base = cache_root.join(&species).join(&build).join(&map_id);
        let raw_dir = base.join("raw");
        let normalized_dir = base.join("normalized");
        let derived_dir = base.join("derived");
        bijux_dna_infra::ensure_dir(&raw_dir)
            .with_context(|| format!("create {}", raw_dir.display()))?;
        bijux_dna_infra::ensure_dir(&normalized_dir)
            .with_context(|| format!("create {}", normalized_dir.display()))?;
        bijux_dna_infra::ensure_dir(&derived_dir)
            .with_context(|| format!("create {}", derived_dir.display()))?;
        let mut observed = Vec::new();
        for file in files {
            let name = toml_string(file.get("name"))?;
            let rel_path = toml_string(file.get("path"))?;
            let url = toml_string(file.get("url"))?;
            let expected = toml_string(file.get("checksum_sha256"))?;
            let target = raw_dir.join(&rel_path);
            let synthetic = format!("synthetic payload for {map_id}/{name}\n").into_bytes();
            let materialized = materialize_controlled_file(
                &target,
                &url,
                &expected,
                &synthetic,
                download,
                verbose,
                &format!("{map_id}:{name}"),
                &mut stdout,
            )?;
            observed.push(json!({
                "name": name,
                "checksum_sha256": expected,
                "observed_sha256": materialized.observed_sha256,
                "path": target.strip_prefix(&cache_root).unwrap_or(&target).display().to_string(),
                "action": materialized.action,
            }));
        }
        write_utf8(&derived_dir.join("chunk_index.tsv"), "chunk\tregion\n0\tall\n")?;
        rows.push(json!({
            "map_id": map_id,
            "species_id": species,
            "build_id": build,
            "files": observed,
        }));
    }

    let run_log = acquire_log_root.join(format!("map-acquire-{}.json", stable_now_utc_compact()));
    write_json_pretty(
        &run_log,
        &json!({
            "rows": rows,
            "cache_root": cache_root.display().to_string(),
        }),
    )?;
    stdout.push_str(&format!("wrote {}\n", workspace.rel(&run_log).display()));
    Ok(OpsCommandOutcome::success(stdout))
}

pub(in super::super) fn tooling_benchmark_integrity_mini(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mut sample_id = "mini_bench".to_string();
    let mut r1 = workspace.path("assets/toy/core-v1/fastq/reads_1.fastq");
    let mut base_out = artifact_root_path(workspace)?
        .join("benchmarks/integrity-mini")
        .join(std::env::var("ISO_RUN_ID").unwrap_or_else(|_| "manual".to_string()));
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run benchmark-integrity-mini -- [--sample-id <id>] [--r1 <fastq>] [--out <dir>]",
                )
            }
            "--sample-id" => {
                sample_id = args
                    .get(index + 1)
                    .cloned()
                    .context("missing value for --sample-id")?;
                index += 2;
            }
            "--r1" => {
                r1 = path_from_arg(
                    workspace,
                    args.get(index + 1).context("missing value for --r1")?,
                );
                index += 2;
            }
            "--out" => {
                base_out = path_from_arg(
                    workspace,
                    args.get(index + 1).context("missing value for --out")?,
                );
                index += 2;
            }
            other => return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n"))),
        }
    }
    if sample_id.is_empty() {
        return Ok(OpsCommandOutcome::failure("empty --sample-id\n"));
    }
    if !r1.is_file() {
        return Ok(OpsCommandOutcome::failure(format!("missing r1 fastq: {}\n", r1.display())));
    }
    bijux_dna_infra::ensure_dir(&base_out)
        .with_context(|| format!("create {}", base_out.display()))?;
    let run_a = base_out.join("run_a");
    let run_b = base_out.join("run_b");
    bijux_dna_infra::ensure_dir(&run_a).with_context(|| format!("create {}", run_a.display()))?;
    bijux_dna_infra::ensure_dir(&run_b).with_context(|| format!("create {}", run_b.display()))?;
    let first = run_program_with_env(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "-q".to_string(),
            "-p".to_string(),
            "bijux-dna".to_string(),
            "--".to_string(),
            "bench".to_string(),
            "fastq".to_string(),
            "validate".to_string(),
            "--sample-id".to_string(),
            sample_id.clone(),
            "--r1".to_string(),
            r1.display().to_string(),
            "--out".to_string(),
            run_a.display().to_string(),
        ],
        &[],
    )?;
    if !first.is_success() {
        return Ok(first);
    }
    let second = run_program_with_env(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "-q".to_string(),
            "-p".to_string(),
            "bijux-dna".to_string(),
            "--".to_string(),
            "bench".to_string(),
            "fastq".to_string(),
            "validate".to_string(),
            "--sample-id".to_string(),
            sample_id.clone(),
            "--r1".to_string(),
            r1.display().to_string(),
            "--out".to_string(),
            run_b.display().to_string(),
        ],
        &[],
    )?;
    if !second.is_success() {
        return Ok(second);
    }

    let knobs =
        toml::from_str::<TomlValue>(&read_utf8(&workspace.path("configs/bench/knobs.toml"))?)?;
    let variance = knobs.get("variance").and_then(TomlValue::as_table).cloned().unwrap_or_default();
    let runtime_rel_max =
        variance.get("runtime_relative_max").and_then(TomlValue::as_float).unwrap_or(0.20);
    let memory_rel_max =
        variance.get("memory_relative_max").and_then(TomlValue::as_float).unwrap_or(0.25);
    let mut errors = Vec::new();
    for path in [&run_a, &run_b] {
        if path.display().to_string().contains("containers/smoke") {
            errors.push(format!("{}: benchmark output path overlaps smoke", path.display()));
        }
    }
    let m_a = find_first_named_file(&run_a, "metrics.json");
    let m_b = find_first_named_file(&run_b, "metrics.json");
    let t_a = find_first_named_file(&run_a, "telemetry.jsonl");
    let t_b = find_first_named_file(&run_b, "telemetry.jsonl");
    let h_a = find_first_named_file(&run_a, "report.html");
    let h_b = find_first_named_file(&run_b, "report.html");
    for (tag, path) in [
        ("run_a", &m_a),
        ("run_b", &m_b),
        ("run_a", &t_a),
        ("run_b", &t_b),
        ("run_a", &h_a),
        ("run_b", &h_b),
    ] {
        if path.is_none() {
            errors.push(format!(
                "{tag}: missing required artifact (metrics.json/telemetry.jsonl/report.html)"
            ));
        }
    }
    let mut runtime_values = Vec::new();
    let mut memory_values = Vec::new();
    let number_re =
        Regex::new(r#""(?:runtime_s|runtime_ms|duration_ms)"\s*:\s*([0-9]+(?:\.[0-9]+)?)"#)?;
    let memory_re = Regex::new(r#""memory_mb"\s*:\s*([0-9]+(?:\.[0-9]+)?)"#)?;
    for (tag, path) in [("run_a", m_a.as_ref()), ("run_b", m_b.as_ref())] {
        if let Some(path) = path {
            let payload = read_json_value(path)?;
            assert_no_excess_float_precision(&payload, tag, &mut errors);
            let raw = read_utf8(path)?;
            if let Some(found) = memory_re.captures(&raw).and_then(|caps| caps.get(1)) {
                if let Ok(value) = found.as_str().parse::<f64>() {
                    memory_values.push(value);
                }
            }
            if let Some(found) = number_re.captures(&raw).and_then(|caps| caps.get(1)) {
                if let Ok(value) = found.as_str().parse::<f64>() {
                    runtime_values.push(value);
                }
            }
        }
    }
    let host_path_re = Regex::new(r"/Users/|/home/|\btmp/")?;
    for (tag, path) in [("run_a", t_a.as_ref()), ("run_b", t_b.as_ref())] {
        if let Some(path) = path {
            let mut by_stage = BTreeMap::new();
            for (line_number, line) in read_utf8(path)?.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                let row: Value = serde_json::from_str(line).with_context(|| {
                    format!("parse {} line {}", path.display(), line_number + 1)
                })?;
                let stage = value_string(row.get("stage_id"));
                let trace = value_string(row.get("trace_id"));
                if stage.is_empty() || trace.is_empty() {
                    errors.push(format!("{tag}:{}: missing stage_id/trace_id", line_number + 1));
                    continue;
                }
                if let Some(previous) = by_stage.insert(stage.clone(), trace.clone()) {
                    if previous != trace {
                        errors.push(format!(
                            "{tag}:{}: trace_id drift within stage {stage}",
                            line_number + 1
                        ));
                    }
                }
                if host_path_re.is_match(line) {
                    errors.push(format!("{tag}:{}: telemetry leaks host path", line_number + 1));
                }
            }
        }
    }
    if let (Some(h_a), Some(h_b)) = (h_a.as_ref(), h_b.as_ref()) {
        if normalize_benchmark_html(&read_utf8(h_a)?)?
            != normalize_benchmark_html(&read_utf8(h_b)?)?
        {
            errors.push(
                "report.html normalized structure differs across consecutive mini benchmark runs"
                    .to_string(),
            );
        }
    }
    if runtime_values.len() == 2 {
        let diff = relative_diff(runtime_values[0], runtime_values[1]);
        if diff > runtime_rel_max {
            errors
                .push(format!("runtime variance {diff:.4} exceeds threshold {runtime_rel_max:.4}"));
        }
    }
    if memory_values.len() == 2 {
        let diff = relative_diff(memory_values[0], memory_values[1]);
        if diff > memory_rel_max {
            errors.push(format!("memory variance {diff:.4} exceeds threshold {memory_rel_max:.4}"));
        }
    }
    let summary_path = base_out.join("integrity_summary.json");
    write_json_pretty(
        &summary_path,
        &json!({
            "schema_version": "bijux.benchmark.integrity.frontend-mini.v1",
            "run_a": run_a.display().to_string(),
            "run_b": run_b.display().to_string(),
            "runtime_relative_max": runtime_rel_max,
            "memory_relative_max": memory_rel_max,
            "runtime_values": runtime_values,
            "memory_values": memory_values,
            "ok": errors.is_empty(),
            "errors": errors,
        }),
    )?;
    let mut stdout = format!("{}\n", summary_path.display());
    if errors.is_empty() {
        stdout.push_str("benchmark integrity mini: OK\n");
        return Ok(OpsCommandOutcome::success(stdout));
    }
    let mut stderr = String::from("benchmark integrity mini: FAILED\n");
    for error in &errors {
        stderr.push_str(&format!("- {error}\n"));
    }
    Ok(OpsCommandOutcome { exit_code: 1, stdout, stderr })
}

pub(in super::super) fn tooling_validate_frontend_mini_domain_stacks(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("validate-frontend-mini-domain-stacks", args)?;
    let out_dir = std::env::var("OUT_DIR").map(PathBuf::from).unwrap_or_else(|_| {
        artifact_root_path(workspace)
            .unwrap_or_else(|_| workspace.path("artifacts"))
            .join("domain/frontend-mini-validation")
    });
    bijux_dna_infra::ensure_dir(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
    let examples = [
        ("fastq_edna_mini", workspace.path("examples/fastq/edna-mini")),
        (
            "vcf_damage_aware_genotype_mini",
            workspace.path("examples/vcf/damage-aware-genotype-mini"),
        ),
        ("vcf_downstream_vcf_full_mini", workspace.path("examples/vcf/downstream-vcf-full-mini")),
        (
            "vcf_downstream_demography_mini",
            workspace.path("examples/vcf/downstream-demography-mini"),
        ),
        ("vcf_essential_qc", workspace.path("examples/vcf/essential-qc")),
        ("vcf_imputation_mini", workspace.path("examples/vcf/imputation-mini")),
    ];
    for (example_id, _) in &examples {
        let outcome = examples_run(
            workspace,
            &["--allow-non-artifacts".to_string(), (*example_id).to_string()],
        )?;
        if !outcome.is_success() {
            return Ok(outcome);
        }
    }
    let mut errors = Vec::new();
    let mut checks = Vec::new();
    for (example_id, example_dir) in &examples {
        let artifact_dir = workspace.path("artifacts/examples").join(example_id);
        for name in [
            "plan.json",
            "explain.json",
            "report.json",
            "golden_report.json",
            "run_report.json",
            "metrics.json",
            "logs.txt",
        ] {
            if !artifact_dir.join(name).exists() {
                errors.push(format!("{example_id}: missing {name}"));
            }
        }
        for json_file in ["plan.json", "explain.json", "report.json"] {
            let artifact_path = artifact_dir.join(json_file);
            let golden_path = example_dir.join("golden").join(json_file);
            if artifact_path.is_file()
                && golden_path.is_file()
                && read_utf8(&artifact_path)? != read_utf8(&golden_path)?
            {
                errors.push(format!("{example_id}: {json_file} differs from golden"));
            }
        }
        let suite =
            toml::from_str::<TomlValue>(&read_utf8(&example_dir.join("bench-suite.toml"))?)?;
        let stages = suite
            .get("stages")
            .and_then(TomlValue::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|value| toml_value_string(&value))
            .collect::<Vec<_>>();
        let plan = read_json_value(&artifact_dir.join("plan.json"))?;
        let got_stages = plan
            .get("stages")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|value| value_string(Some(&value)))
            .collect::<Vec<_>>();
        for stage in stages {
            if !got_stages.contains(&stage) {
                errors.push(format!("{example_id}: stage {stage} missing in plan.json stages"));
            }
        }
        let logs = read_utf8(&artifact_dir.join("logs.txt")).unwrap_or_default();
        for key in
            ["example_id=", "corpus_id=", "mini_supported=", "step1=", "step2=", "step3=", "step4="]
        {
            if !logs.contains(key) {
                errors.push(format!("{example_id}: logs.txt missing {key}"));
            }
        }
        let metrics = read_json_value(&artifact_dir.join("metrics.json"))?;
        for key in ["example_id", "collected_at", "status"] {
            if metrics.get(key).is_none() {
                errors.push(format!("{example_id}: metrics.json missing {key}"));
            }
        }
        if example_id.starts_with("vcf_") {
            for (doc_name, payload) in [
                ("explain.json", read_json_value(&artifact_dir.join("explain.json"))?),
                ("report.json", read_json_value(&artifact_dir.join("report.json"))?),
            ] {
                let coverage = payload.get("coverage_regime").cloned().unwrap_or(Value::Null);
                let selected = value_string(coverage.get("selected"));
                if !matches!(selected.as_str(), "gl" | "pseudohaploid" | "diploid") {
                    errors
                        .push(format!("{example_id}: {doc_name} coverage_regime.selected invalid"));
                }
                for key in ["thresholds_used", "observed_coverage_stats"] {
                    if coverage.get(key).is_none() {
                        errors.push(format!(
                            "{example_id}: {doc_name} coverage_regime missing {key}"
                        ));
                    }
                }
            }
        }
        checks.push(json!({
            "example_id": example_id,
            "artifact_dir": artifact_dir.display().to_string(),
            "plan_sha256": sha256_hex(&artifact_dir.join("plan.json"))?,
            "explain_sha256": sha256_hex(&artifact_dir.join("explain.json"))?,
            "report_sha256": sha256_hex(&artifact_dir.join("report.json"))?,
        }));
    }
    for (profile, depth, want) in [
        ("adna_lowcov_capture", "1", "gl"),
        ("adna_lowcov_capture", "6", "pseudohaploid"),
        ("modern_wgs_shotgun", "20", "diploid"),
    ] {
        let outcome = tooling_simulate_coverage_regime(
            workspace,
            &[depth.to_string(), "--profile".to_string(), profile.to_string()],
        )?;
        if !outcome.is_success() {
            errors
                .push(format!("coverage_regime simulate failed: profile={profile} depth={depth}"));
            continue;
        }
        let payload: Value = serde_json::from_str(&outcome.stdout)
            .with_context(|| "parse simulate-coverage-regime output")?;
        let got = value_string(payload.get("selected_regime"));
        if got != want {
            errors.push(format!(
                "coverage_regime mismatch: profile={profile} depth={depth} expected={want} got={got}"
            ));
        }
    }
    let auth_text = read_utf8(&workspace.path("domain/bam/stages/authenticity.yaml"))?;
    let mut tools = Vec::new();
    let mut in_tools = false;
    for line in auth_text.lines() {
        let raw = line.trim_end();
        if raw.trim_start().starts_with("compatible_tools:") {
            in_tools = true;
            continue;
        }
        if in_tools {
            if raw.starts_with("  - ") {
                tools.push(
                    raw.split_once("- ")
                        .map(|(_, value)| value.trim().to_string())
                        .unwrap_or_default(),
                );
                continue;
            }
            if !raw.is_empty() && !raw.starts_with(' ') {
                break;
            }
        }
    }
    tools.sort();
    let authenticity_stage = id_catalog::BAM_AUTHENTICITY;
    if tools
        != vec!["authenticct".to_string(), "damageprofiler".to_string(), "pmdtools".to_string()]
    {
        errors.push(format!("{authenticity_stage} compatible_tools mismatch: {tools:?}"));
    }
    for entry in WalkDir::new(workspace.path("domain/bam/fixtures/bam.authenticity"))
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|ext| ext.to_str()) != Some("txt")
        {
            continue;
        }
        let mut kv = BTreeMap::new();
        for line in read_utf8(entry.path())?.lines() {
            if let Some((key, value)) = line.split_once('=') {
                kv.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        if kv.get("stage").map(String::as_str) != Some(authenticity_stage) {
            errors.push(format!("{}: stage must be {authenticity_stage}", entry.path().display()));
        }
        if kv.get("domain").map(String::as_str) != Some("bam") {
            errors.push(format!("{}: domain must be bam", entry.path().display()));
        }
        if kv.get("expected_outputs").map(String::as_str) != Some("contract_artifacts") {
            errors.push(format!(
                "{}: expected_outputs must be contract_artifacts",
                entry.path().display()
            ));
        }
        if kv.get("expected_stdout_patterns").map(String::as_str) != Some("contract_ok") {
            errors.push(format!(
                "{}: expected_stdout_patterns must be contract_ok",
                entry.path().display()
            ));
        }
    }
    let summary_path = out_dir.join("summary.json");
    write_json_pretty(
        &summary_path,
        &json!({
            "schema_version": "bijux.frontend.mini_domain_validation.v1",
            "ok": errors.is_empty(),
            "checks": checks,
            "errors": errors,
        }),
    )?;
    let mut stdout = format!("{}\n", summary_path.display());
    if errors.is_empty() {
        stdout.push_str("frontend mini domain validation: OK\n");
        return Ok(OpsCommandOutcome::success(stdout));
    }
    let mut stderr = String::from("frontend mini domain validation: FAILED\n");
    for error in &errors {
        stderr.push_str(&format!("- {error}\n"));
    }
    Ok(OpsCommandOutcome { exit_code: 1, stdout, stderr })
}
