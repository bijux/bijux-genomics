struct ExecutionRunResult {
    envelope: ExecutionEnvelope,
    outputs_override: Option<Vec<PathBuf>>,
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn run_stage_execution(
    plan: &StagePlanV1,
    image: &ResolvedImage,
    runner: RunnerKind,
    r1_dir: &Path,
    r1: &Path,
    r2: Option<&Path>,
    container_name: &str,
    canonical_params: &serde_json::Value,
) -> Result<ExecutionRunResult> {
    let mut outputs_override: Option<Vec<PathBuf>> = None;
    let execution = match plan.stage_id.0.as_str() {
        "fastq.merge" => {
            let r2 = r2.ok_or_else(|| anyhow!("merge requires r2 input"))?;
            let exec = run_merge_execution(
                &plan.tool_id.0,
                image,
                r1_dir,
                r1,
                r2,
                &plan.out_dir,
                container_name,
            )?;
            outputs_override = Some(vec![
                exec.merged_fastq.clone(),
                exec.unmerged_r1.clone(),
                exec.unmerged_r2.clone(),
            ]);
            ExecutionEnvelope {
                exit_code: exec.exit_code,
                stdout: exec.stdout,
                stderr: exec.stderr,
                command: exec.command,
            }
        }
        "fastq.qc_post" if plan.tool_id.0 == "multiqc" => {
            let fastqc_image = plan
                .aux_images
                .get("fastqc")
                .ok_or_else(|| anyhow!("fastqc image missing for multiqc qc_post"))?;
            let fastqc_image = resolved_image_for_plan(fastqc_image, runner);
            let fastqc_trimmed_dir = plan.out_dir.join("fastqc_trimmed");
            bijux_infra::ensure_dir(&fastqc_trimmed_dir)?;
            let fastqc_trimmed_container = format!("bijux-stage-fastqc-{}", Uuid::new_v4());
            let fastqc_trimmed_exec = run_validate_execution(
                "fastqc",
                &fastqc_image,
                r1_dir,
                r1,
                &fastqc_trimmed_dir,
                &fastqc_trimmed_container,
            )?;
            cleanup_execution(&fastqc_trimmed_container)?;
            if fastqc_trimmed_exec.exit_code != 0 {
                return Err(anyhow!(
                    "fastqc trimmed exit code {}",
                    fastqc_trimmed_exec.exit_code
                ));
            }

            if let Some(raw_r1) = canonical_params
                .get("raw_r1")
                .and_then(|value| value.as_str())
            {
                let raw_r1 = PathBuf::from(raw_r1);
                if let Some(raw_dir) = raw_r1.parent() {
                    let fastqc_raw_dir = plan.out_dir.join("fastqc_raw");
                    bijux_infra::ensure_dir(&fastqc_raw_dir)?;
                    let fastqc_raw_container = format!("bijux-stage-fastqc-{}", Uuid::new_v4());
                    let fastqc_raw_exec = run_validate_execution(
                        "fastqc",
                        &fastqc_image,
                        raw_dir,
                        &raw_r1,
                        &fastqc_raw_dir,
                        &fastqc_raw_container,
                    )?;
                    cleanup_execution(&fastqc_raw_container)?;
                    if fastqc_raw_exec.exit_code != 0 {
                        return Err(anyhow!(
                            "fastqc raw exit code {}",
                            fastqc_raw_exec.exit_code
                        ));
                    }
                }
            }

            let exec =
                run_multiqc_execution(image, &plan.out_dir, &plan.out_dir, container_name)?;
            ExecutionEnvelope {
                exit_code: exec.exit_code,
                stdout: exec.stdout,
                stderr: exec.stderr,
                command: exec.command,
            }
        }
        "fastq.validate_pre" | "fastq.qc_post" | "fastq.detect_adapters" => {
            let exec = run_validate_execution(
                &plan.tool_id.0,
                image,
                r1_dir,
                r1,
                &plan.out_dir,
                container_name,
            )?;
            ExecutionEnvelope {
                exit_code: exec.exit_code,
                stdout: exec.stdout,
                stderr: exec.stderr,
                command: exec.command,
            }
        }
        "fastq.filter" => {
            let mut filter_params = canonical_params.clone();
            if let Some(kmer_ref) = canonical_params
                .get("kmer_ref")
                .and_then(|value| value.as_str())
            {
                let src = PathBuf::from(kmer_ref);
                if src.exists() {
                    let dest = plan.out_dir.join("kmer_ref.fasta");
                    std::fs::copy(&src, &dest)?;
                    if let Some(obj) = filter_params.as_object_mut() {
                        obj.insert(
                            "kmer_ref".to_string(),
                            serde_json::Value::String("/data/output/kmer_ref.fasta".to_string()),
                        );
                    }
                }
            }
            let exec = run_filter_execution(
                &plan.tool_id.0,
                image,
                r1_dir,
                r1,
                &plan.out_dir,
                container_name,
                &filter_params,
            )?;
            ExecutionEnvelope {
                exit_code: exec.exit_code,
                stdout: exec.stdout,
                stderr: exec.stderr,
                command: exec.command,
            }
        }
        _ => {
            let exec = run_tool_execution(
                &plan.tool_id.0,
                image,
                r1_dir,
                r1,
                &plan.out_dir,
                container_name,
            )?;
            ExecutionEnvelope {
                exit_code: exec.exit_code,
                stdout: exec.stdout,
                stderr: exec.stderr,
                command: exec.command,
            }
        }
    };

    Ok(ExecutionRunResult {
        envelope: execution,
        outputs_override,
    })
}
