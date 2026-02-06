fn qa_qc_post_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
) -> Result<()> {
    let contract = tool_contract(registry, STAGE_QC_POST.as_str(), tool)?;
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.toml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("qc_post", tool)?;
    let container_name = format!("bijux-qa-qc_post-{}-{}", tool, Uuid::new_v4());
    let timeout = Duration::from_secs(QA_TIMEOUT_SECS);
    if tool == "multiqc" {
        let fastqc_spec = catalog
            .get("fastqc")
            .ok_or_else(|| anyhow!("fastqc missing from images.toml"))?;
        let fastqc_image = resolve_image_for_run(fastqc_spec, platform)?;
        let fastqc_dir = out_dir.join("fastqc");
        bijux_infra::ensure_dir(&fastqc_dir).context("create fastqc output dir")?;
        let fastqc_container = format!("bijux-qa-qc_post-fastqc-{}", Uuid::new_v4());
        let fastqc_exec = match run_validate_container_with_timeout(
            "fastqc",
            &fastqc_image,
            &dataset.r1_dir,
            &dataset.r1,
            &fastqc_dir,
            &fastqc_container,
            timeout,
        ) {
            Ok(execution) => execution,
            Err(err) => {
                let _ = docker_rm(&fastqc_container);
                return Err(err);
            }
        };
        docker_rm(&fastqc_container)?;
        if fastqc_exec.exit_code != 0 {
            return Err(anyhow!("fastqc exit code {}", fastqc_exec.exit_code));
        }
        let execution = match run_multiqc_container_with_timeout(
            &image,
            &fastqc_dir,
            &out_dir,
            &container_name,
            timeout,
        ) {
            Ok(execution) => execution,
            Err(err) => {
                let _ = docker_rm(&container_name);
                return Err(err);
            }
        };
        docker_rm(&container_name)?;
        if execution.exit_code != 0 {
            return Err(anyhow!("exit code {}", execution.exit_code));
        }
        validate_execution_outputs(contract, &out_dir)?;
        if !dir_has_files(&out_dir) {
            return Err(anyhow!("qc_post output missing"));
        }
        return Ok(());
    }

    let execution = match run_validate_container_with_timeout(
        tool,
        &image,
        &dataset.r1_dir,
        &dataset.r1,
        &out_dir,
        &container_name,
        timeout,
    ) {
        Ok(execution) => execution,
        Err(err) => {
            let _ = docker_rm(&container_name);
            return Err(err);
        }
    };
    docker_rm(&container_name)?;
    if execution.exit_code != 0 {
        return Err(anyhow!("exit code {}", execution.exit_code));
    }
    validate_execution_outputs(contract, &out_dir)?;
    if !dir_has_files(&out_dir) {
        return Err(anyhow!("qc_post output missing"));
    }
    Ok(())
}

fn qa_umi_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
    seqkit_image: &ResolvedImage,
) -> Result<()> {
    let contract = tool_contract(registry, STAGE_UMI.as_str(), tool)?;
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.toml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("umi", tool)?;
    let container_name = format!("bijux-qa-umi-{}-{}", tool, Uuid::new_v4());
    let timeout = Duration::from_secs(QA_TIMEOUT_SECS);
    let execution = match run_tool_container_with_timeout(
        tool,
        &image,
        &dataset.r1_dir,
        &dataset.r1,
        &out_dir,
        &container_name,
        timeout,
    ) {
        Ok(execution) => execution,
        Err(err) => {
            let _ = docker_rm(&container_name);
            return Err(err);
        }
    };
    docker_rm(&container_name)?;
    if execution.exit_code != 0 {
        return Err(anyhow!("exit code {}", execution.exit_code));
    }
    validate_execution_outputs(contract, &out_dir)?;
    let out_fastq = execution
        .output_fastq
        .ok_or_else(|| anyhow!("output FASTQ missing"))?;
    if !out_fastq.exists() {
        return Err(anyhow!("output FASTQ not found: {}", out_fastq.display()));
    }
    let output_stats = output_fastq_stats(seqkit_image, &out_dir, &out_fastq)?;
    if output_stats.reads > dataset.input_stats_r1.reads {
        return Err(anyhow!(
            "reads_out {} exceeds reads_in {}",
            output_stats.reads,
            dataset.input_stats_r1.reads
        ));
    }
    Ok(())
}

fn qa_stats_tool(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
) -> Result<()> {
    let contract = tool_contract(registry, STAGE_STATS_NEUTRAL.as_str(), tool)?;
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.toml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let out_dir = temp_out_dir("stats", tool)?;
    let container_name = format!("bijux-qa-stats-{}-{}", tool, Uuid::new_v4());
    let timeout = Duration::from_secs(QA_TIMEOUT_SECS);
    let execution = match run_validate_container_with_timeout(
        tool,
        &image,
        &dataset.r1_dir,
        &dataset.r1,
        &out_dir,
        &container_name,
        timeout,
    ) {
        Ok(execution) => execution,
        Err(err) => {
            let _ = docker_rm(&container_name);
            return Err(err);
        }
    };
    docker_rm(&container_name)?;
    if execution.exit_code != 0 {
        return Err(anyhow!("exit code {}", execution.exit_code));
    }
    validate_execution_outputs(contract, &out_dir)?;
    Ok(())
}

fn qa_screen_tool(
    _tool: &str,
    _platform: &PlatformSpec,
    _catalog: &HashMap<String, ToolImageSpec>,
    _registry: &ToolRegistry,
    _dataset: &QaDataset,
) -> Result<()> {
    Err(anyhow!(
        "screen QA requires BIJUX_SCREEN_DB and is not enabled"
    ))
}

fn tool_contract<'a>(
    registry: &'a ToolRegistry,
    stage_id: &str,
    tool_id: &str,
) -> Result<&'a bijux_core::contract::ExecutionContract> {
    let stage_id = bijux_core::ids::StageId::try_from(stage_id)
        .map_err(|err| anyhow!("invalid stage id: {err}"))?;
    let tool_id = bijux_core::ids::ToolId::try_from(tool_id)
        .map_err(|err| anyhow!("invalid tool id: {err}"))?;
    let tool = registry
        .tool_by_id(&stage_id, &tool_id)
        .ok_or_else(|| anyhow!("tool {tool_id} missing from manifests"))?;
    Ok(&tool.execution_contract)
}

fn dir_has_files(path: &std::path::Path) -> bool {
    match std::fs::read_dir(path) {
        Ok(mut iter) => iter.next().is_some(),
        Err(_) => false,
    }
}

fn find_fastq_in_dir(dir: &std::path::Path) -> Result<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir).context("read output dir")?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if is_fastq_path(&path) {
            return Ok(path);
        }
    }
    Err(anyhow!("output FASTQ not found in {}", dir.display()))
}

fn is_fastq_path(path: &std::path::Path) -> bool {
    let ext = path.extension().and_then(|ext| ext.to_str());
    if let Some(ext) = ext {
        if ext.eq_ignore_ascii_case("fastq") || ext.eq_ignore_ascii_case("fq") {
            return true;
        }
        if ext.eq_ignore_ascii_case("gz") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let nested = std::path::Path::new(stem);
                if let Some(stem_ext) = nested.extension().and_then(|s| s.to_str()) {
                    return stem_ext.eq_ignore_ascii_case("fastq")
                        || stem_ext.eq_ignore_ascii_case("fq");
                }
            }
        }
    }
    false
}
