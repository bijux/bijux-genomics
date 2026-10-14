use super::*;

fn seed_bam_resume_receipt(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    stage_dir: &Path,
    declared_outputs: &[ArtifactRef],
) -> Result<()> {
    // These accepted receipts test orchestration, not a cold aligner or scientific metrics.
    let mut outputs = declared_outputs.iter().map(|item| item.path.clone()).collect::<Vec<_>>();
    outputs.extend(
        bijux_dna_domain_bam::required_audit_artifacts(stage)
            .iter()
            .map(|item| stage_dir.join(item.filename)),
    );
    outputs.sort();
    outputs.dedup();
    for path in &outputs {
        bijux_dna_infra::ensure_dir(
            path.parent().ok_or_else(|| anyhow!("receipt output has no parent"))?,
        )?;
        let contents: &[u8] = if path.extension().is_some_and(|ext| ext == "bam") {
            b"@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:4\n@RG\tID:rg-sample\tSM:sample\tPL:ILLUMINA\tLB:lib1\nr1\t0\tchr1\t1\t60\t4M\t*\t0\t0\tACGT\tIIII\tRG:Z:rg-sample\n"
        } else if path.extension().is_some_and(|ext| ext == "json") {
            b"{}\n"
        } else {
            b"fixture\n"
        };
        std::fs::write(path, contents)?;
    }
    write_stage_accounting(stage_dir, stage.as_str(), &build_resumed_stage_result(outputs))
}

#[test]
fn resumed_alignment_keeps_profile_downstream_summaries() -> Result<()> {
    use bijux_dna_planner_bam::stage_api::BamStage;
    const CHILD_ROOT: &str = "BIJUX_DNA_BAM_RESUME_CONTRACT_ROOT";

    let workspace = crate::support::workspace::resolve_repo_root()?;
    let Some(root) = std::env::var_os(CHILD_ROOT).map(PathBuf::from) else {
        let owned = bijux_dna_infra::temp_dir_in(
            &workspace.join("artifacts/testing/bam-resume-contract"),
            "receipt-",
        )?;
        let empty_path = owned.path().join("provider-path");
        let home = owned.path().join("home");
        let scratch = owned.path().join("scratch");
        for path in [&empty_path, &home, &scratch] {
            bijux_dna_infra::ensure_dir(path)?;
        }
        let bindings = owned.path().join("execution-bindings.json");
        bijux_dna_infra::atomic_write_json(
            &bindings,
            &bijux_dna_runner::command_runner::ExecutionBindingManifestV1 {
                schema_version:
                    bijux_dna_runner::command_runner::RUNNER_EXECUTION_BINDINGS_SCHEMA_V1
                        .to_string(),
                bindings: vec![bijux_dna_runner::command_runner::CommandExecutionBindingV1 {
                    command: std::env::current_exe()?.to_string_lossy().into_owned(),
                    program: std::env::current_exe()?.to_string_lossy().into_owned(),
                    prefix_args: Vec::new(),
                    required_files: Vec::new(),
                }],
            },
        )?;
        let envs = std::collections::BTreeMap::from([
            (CHILD_ROOT.to_string(), owned.path().display().to_string()),
            ("BIJUX_REPO_ROOT".to_string(), workspace.display().to_string()),
            ("PATH".to_string(), empty_path.display().to_string()),
            ("HOME".to_string(), home.display().to_string()),
            ("TMPDIR".to_string(), scratch.display().to_string()),
            ("TMP".to_string(), scratch.display().to_string()),
            ("TEMP".to_string(), scratch.display().to_string()),
            ("TEST_TMP_DIR".to_string(), scratch.display().to_string()),
            (
                bijux_dna_runner::command_runner::RUNNER_EXECUTION_BINDINGS_ENV.to_string(),
                bindings.display().to_string(),
            ),
        ]);
        let (_, module) = module_path!()
            .split_once("::")
            .ok_or_else(|| anyhow!("contract module has no crate prefix"))?;
        let output = bijux_dna_runner::command_runner::run_command_with_context_and_timeout(
            std::env::current_exe()?.to_string_lossy().as_ref(),
            &[
                format!("{module}::resumed_alignment_keeps_profile_downstream_summaries"),
                "--exact".to_string(),
                "--nocapture".to_string(),
                "--test-threads=1".to_string(),
            ],
            Some(&workspace),
            Some(&envs),
            std::time::Duration::from_secs(30),
        )?;
        eprint!("{}", output.output.stderr);
        assert!(!output.timed_out, "resume contract child exceeded its deadline");
        assert_eq!(
            output.output.exit_code, 0,
            "resume contract child failed:\n{}\n{}",
            output.output.stdout, output.output.stderr
        );
        assert!(
            output.output.stderr.contains("BAM_RESUME_CONTINUATION_PASS"),
            "the exact contract child must execute; a zero-case child is not success"
        );
        return Ok(());
    };

    let registry_path = workspace.join("configs/ci/registry/tool_registry.toml");
    let registry = crate::support::workspace::registry::load_registry(&registry_path)?;
    let catalog = bijux_dna_containers::api::load_image_catalog_from_paths(
        &workspace.join("configs/ci/tools/images.toml"),
        &registry_path,
    )?;
    let platform = bijux_dna_containers::api::PlatformSpec {
        name: "receipt-contract".to_string(),
        runner: bijux_dna_containers::api::RuntimeKind::Local,
        container_dir: root.join("containers"),
        image_prefix: "bijuxdna".to_string(),
        arch: std::env::consts::ARCH.to_string(),
    };
    let profile = bijux_dna_pipelines::bam::bam_default_profile();
    let expected_ids = bijux_dna_planner_bam::pipeline_id_catalog(profile.id.as_str());
    assert!(!expected_ids.is_empty(), "default profile must declare downstream stages");
    let out_dir = root.join("run");
    let align_dir = out_dir.join("bam/align");
    seed_bam_resume_receipt(BamStage::Align, &align_dir, &[])?;
    let fasta = root.join("reference.fasta");
    std::fs::write(&fasta, b">chr1\nACGT\n")?;
    let fai = PathBuf::from(format!("{}.fai", fasta.display()));
    let dict = PathBuf::from(format!("{}.dict", fasta.display()));
    std::fs::write(&fai, b"chr1\t4\t6\t4\t5\n")?;
    std::fs::write(&dict, b"@HD\tVN:1.6\n@SQ\tSN:chr1\tLN:4\n")?;
    std::fs::write(PathBuf::from(format!("{}.bwt", fasta.display())), b"fixture\n")?;
    let reference = ReferenceRecord {
        digest: bijux_dna_infra::hash_file_sha256(&fasta)?,
        root: root.clone(),
        fasta: fasta.clone(),
        fai,
        dict,
        bwa_prefix: fasta.clone(),
        bowtie2_prefix: fasta.clone(),
    };
    let r1 = root.join("reads.fastq");
    std::fs::write(&r1, b"@r1\nACGT\n+\nIIII\n")?;
    let args = FastqCrossArgs {
        sample_id: Some("sample".to_string()),
        r1: Some(r1),
        r2: None,
        alignment_bam: None,
        alignment_bai: None,
        alignment_reference: Some(fasta.clone()),
        alignment_rg_policy: None,
        alignment_meta: Vec::new(),
    };
    let bam = align_dir.join("align.bam");
    let bai = align_dir.join("align.bam.bai");
    let boundary = AlignmentBoundary {
        bam_path: bam.display().to_string(),
        bai_path: Some(bai.display().to_string()),
        reference: Some(fasta.display().to_string()),
        rg_policy: None,
        aligner_meta: None,
    };
    let mut expected_outputs = std::collections::BTreeMap::new();
    for stage_id in &expected_ids {
        let stage = BamStage::try_from(stage_id.as_str())?;
        let stage_dir = out_dir.join("bam").join(stage_id.trim_start_matches(STAGE_PREFIX));
        let tool_id = profile
            .defaults
            .tools
            .get(&StageId::new(stage_id.clone()))
            .ok_or_else(|| anyhow!("default stage has no tool: {stage_id}"))?;
        let spec =
            build_tool_execution_spec(stage_id, tool_id.as_str(), &registry, &catalog, &platform)?;
        let mut stage_args = base_bam_args(
            stage,
            &profile,
            bam.clone(),
            stage_dir.clone(),
            Some(bai.clone()),
            Some(fasta.clone()),
        );
        stage_args.tool = Some(tool_id.to_string());
        let plan =
            plan_for_bam_stage_with_profile(stage, &spec, &stage_args, &profile, &stage_dir)?;
        seed_bam_resume_receipt(stage, &stage_dir, &plan.io.outputs)?;
        expected_outputs.insert(stage_id.clone(), plan.io.outputs);
    }

    // Qualify the same inputs through the real downstream loop before testing continuation.
    let direct =
        run_bam_truth_stages(&registry, &catalog, &platform, &profile, &boundary, &out_dir)?;
    assert_eq!(
        direct.iter().map(|entry| entry.plan.step_id.to_string()).collect::<Vec<_>>(),
        expected_ids,
        "direct downstream control must retain the authoritative default catalog"
    );
    for entry in &direct {
        let stage_id = entry.plan.step_id.to_string();
        assert_eq!(Some(&entry.plan.io.outputs), expected_outputs.get(&stage_id));
        assert!(entry.plan.io.outputs.iter().all(|item| item.path.is_file()));
        assert_eq!(entry.result.command, "resume-skip");
        assert!(entry.plan.out_dir.join("stage_resume.json").is_file());
    }
    eprintln!("BAM_RESUME_DIRECT_CONTROL_PASS: {expected_ids:?}");

    let cross_profile = bijux_dna_pipelines::registry::profile_by_id(
        bijux_dna_pipelines::Domain::Cross,
        "fastq-to-bam__default__v1",
    )?;
    let fastq_summary = serde_json::json!({ "run_id": "receipt-projection", "stages": [] });
    crate::internal::handlers::cross::manifests::write_defaults_ledger(&out_dir, &cross_profile)?;
    let boundary_path =
        crate::internal::handlers::cross::manifests::write_alignment_boundary(&out_dir, &boundary)?;
    let reference_path = crate::internal::handlers::cross::manifests::write_reference_manifest(
        &out_dir, &reference,
    )?;
    let publish = |runs: &[StageExecutionSummary],
                   expected: &[String],
                   capture: &str|
     -> Result<serde_json::Value> {
        let reports = crate::internal::handlers::bam_summary::render_bam_summary(
            &out_dir.join("bam"),
            runs,
            &[],
        )?;
        let summary: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&reports.summary_json_path)?)?;
        let rows = summary["stages"].as_array().ok_or_else(|| anyhow!("summary stages missing"))?;
        assert_eq!(
            rows.iter().map(|row| row["stage_id"].as_str()).collect::<Vec<_>>(),
            expected.iter().map(|id| Some(id.as_str())).collect::<Vec<_>>()
        );
        let tsv = std::fs::read_to_string(&reports.summary_tsv_path)?;
        assert_eq!(
            tsv.lines().skip(1).map(|line| line.split('\t').next()).collect::<Vec<_>>(),
            expected.iter().map(|id| Some(id.as_str())).collect::<Vec<_>>()
        );
        assert!(reports.report_html_path.is_file());
        crate::internal::handlers::cross::manifests::write_cross_run_manifest(
            &out_dir,
            &cross_profile,
            &fastq_summary,
            runs,
            Some(&boundary_path),
            Some(&reference_path),
        )?;
        let manifest_path = out_dir.join("run_manifest.json");
        let manifest: serde_json::Value = serde_json::from_slice(&std::fs::read(&manifest_path)?)?;
        let rows =
            manifest["stages"].as_array().ok_or_else(|| anyhow!("manifest stages missing"))?;
        assert_eq!(
            rows.iter()
                .filter(|row| row["domain"].as_str() == Some("bam"))
                .map(|row| row["stage_id"].as_str())
                .collect::<Vec<_>>(),
            expected.iter().map(|id| Some(id.as_str())).collect::<Vec<_>>()
        );
        let outputs = manifest["output_artifacts"]
            .as_array()
            .ok_or_else(|| anyhow!("manifest declared outputs missing"))?;
        for (stage_id, declared) in &expected_outputs {
            for artifact in declared {
                let projected = outputs
                    .iter()
                    .filter(|item| {
                        item["stage_id"].as_str() == Some(stage_id.as_str())
                            && item["name"].as_str() == Some(artifact.name.as_str())
                    })
                    .collect::<Vec<_>>();
                assert_eq!(projected.len(), 1, "one declared output identity must be retained");
                let item = projected[0];
                assert_eq!(item["role"].as_str(), Some(artifact.role.as_str()));
                assert_eq!(item["optional"].as_bool(), Some(artifact.optional));
                assert_eq!(
                    item["path"].as_str(),
                    Some(artifact.path.strip_prefix(&out_dir)?.to_string_lossy().as_ref())
                );
                assert_eq!(
                    item["sha256"].as_str(),
                    Some(bijux_dna_infra::hash_file_sha256(&artifact.path)?.as_str())
                );
            }
        }
        let capture_dir = root.join("publication-captures").join(capture);
        bijux_dna_infra::ensure_dir(&capture_dir)?;
        for (source, name) in [
            (&reports.summary_json_path, "summary.json"),
            (&reports.summary_tsv_path, "summary.tsv"),
            (&reports.report_html_path, "report.html"),
            (&manifest_path, "run_manifest.json"),
        ] {
            std::fs::copy(source, capture_dir.join(name))?;
        }
        Ok(manifest)
    };
    let direct_published = publish(&direct, &expected_ids, "direct")?;
    eprintln!("BAM_RESUME_PUBLISHER_DIRECT_CONTROL_PASS");

    let combined = run_bam_align_and_truth_stages(
        &registry, &catalog, &platform, &profile, &reference, &args, &out_dir,
    )?;
    let align = combined.first().ok_or_else(|| anyhow!("missing alignment summary"))?;
    assert_eq!(align.plan.step_id.as_str(), BamStage::Align.as_str());
    assert_eq!(align.result.command, "resume-skip");
    assert!(align_dir.join("stage_resume.json").is_file());
    let expected_combined = std::iter::once(BamStage::Align.as_str().to_string())
        .chain(expected_ids)
        .collect::<Vec<_>>();
    assert_eq!(
        combined.iter().map(|entry| entry.plan.step_id.to_string()).collect::<Vec<_>>(),
        expected_combined,
        "resumed alignment must retain its qualified downstream stage summaries"
    );
    for (resumed, control) in combined.iter().skip(1).zip(&direct) {
        assert_eq!(resumed.plan.io.outputs, control.plan.io.outputs);
        assert!(resumed.plan.io.outputs.iter().all(|item| item.path.is_file()));
        assert_eq!(resumed.result.command, "resume-skip");
        assert!(resumed.plan.out_dir.join("stage_resume.json").is_file());
    }
    let combined_published = publish(&combined, &expected_combined, "combined")?;
    let downstream_outputs = |manifest: &serde_json::Value| -> Result<Vec<serde_json::Value>> {
        Ok(manifest["output_artifacts"]
            .as_array()
            .ok_or_else(|| anyhow!("manifest declared outputs missing"))?
            .iter()
            .filter(|item| {
                item["stage_id"]
                    .as_str()
                    .is_some_and(|stage_id| expected_outputs.contains_key(stage_id))
            })
            .cloned()
            .collect())
    };
    assert_eq!(downstream_outputs(&direct_published)?, downstream_outputs(&combined_published)?);
    eprintln!("BAM_RESUME_PUBLISHER_CONTINUATION_PASS");

    let first_stage = direct
        .first()
        .ok_or_else(|| anyhow!("missing downstream control stage"))?
        .plan
        .step_id
        .as_str();
    let absent_tool = "missing-resume-contract-tool";
    let mut missing_binding_profile = profile.clone();
    missing_binding_profile
        .defaults
        .tools
        .insert(StageId::new(first_stage), ToolId::new(absent_tool));
    let Err(direct_error) = run_bam_truth_stages(
        &registry,
        &catalog,
        &platform,
        &missing_binding_profile,
        &boundary,
        &out_dir,
    ) else {
        return Err(anyhow!("direct downstream control accepted an absent tool binding"));
    };
    let refusal = format!("tool {absent_tool} missing from manifest for {first_stage}");
    assert!(format!("{direct_error:#}").contains(&refusal));
    eprintln!("BAM_RESUME_ABSENT_BINDING_DIRECT_REFUSAL_PASS: {refusal}");
    let Err(combined_error) = run_bam_align_and_truth_stages(
        &registry,
        &catalog,
        &platform,
        &missing_binding_profile,
        &reference,
        &args,
        &out_dir,
    ) else {
        return Err(anyhow!("alignment reuse hid an absent downstream tool binding"));
    };
    assert!(format!("{combined_error:#}").contains(&refusal));
    eprintln!("BAM_RESUME_ABSENT_BINDING_CONTINUATION_PASS");
    eprintln!("BAM_RESUME_CONTINUATION_PASS");
    Ok(())
}
