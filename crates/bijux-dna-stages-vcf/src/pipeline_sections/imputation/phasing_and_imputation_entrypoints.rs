fn run_phasing_stage_inner(
    input_vcf: &Path,
    out_dir: &Path,
    species_context: &SpeciesContext,
    params: &PhasingStageParams,
) -> Result<PhasingStageOutputs> {
    if params.species_id != species_context.species_id
        || params.build_id != species_context.build_id
    {
        bail!("species/build mismatch between phasing params and SpeciesContext");
    }
    if params.threads == 0 {
        bail!("phasing requires threads > 0");
    }

    let raw = std::fs::read_to_string(input_vcf)?;
    let resolved_backend = resolve_phasing_backend(params, &raw);
    let backend_tool = resolved_backend.as_str();
    if matches!(resolved_backend, PhasingBackend::Eagle) && !license_metadata_for_tool_exists("eagle")
    {
        bail!("eagle requires non-bijux license metadata before execution");
    }

    let map = if matches!(
        resolved_backend,
        PhasingBackend::Shapeit5 | PhasingBackend::Eagle
    ) {
        Some(resolve_map(
            &params.species_id,
            &params.build_id,
            params.map_id.as_deref(),
        )?)
    } else {
        params
            .map_id
            .as_deref()
            .map(|map_id| resolve_map(&params.species_id, &params.build_id, Some(map_id)))
            .transpose()?
    };
    if let Some(map_ref) = &map {
        if !map_ref
            .compatibility
            .tool_tags
            .iter()
            .any(|tag| tag == backend_tool)
        {
            bail!(
                "map {} is not compatible with backend {}",
                map_ref.id,
                backend_tool
            );
        }
    }

    let mut out_records = Vec::new();
    let mut header_lines = Vec::new();
    let mut has_gt = false;
    let mut has_gl_or_gp = false;
    let mut diploid_ok = true;
    let mut saw_records = false;
    let mut has_sex_chr = false;
    let mut phase_switches = 0u64;
    let mut prev_gt: Option<String> = None;
    let mut variant_count = 0u64;

    for line in raw.lines() {
        if line.starts_with('#') {
            header_lines.push(line.to_string());
            continue;
        }
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        saw_records = true;
        variant_count += 1;
        let chr = fields[0];
        if matches!(chr, "X" | "Y" | "chrX" | "chrY") {
            has_sex_chr = true;
        }
        let gt_idx = parse_format_index(&fields, "GT");
        let gl_idx = parse_format_index(&fields, "GL");
        let gp_idx = parse_format_index(&fields, "GP");
        if gt_idx.is_some() {
            has_gt = true;
        }
        if gl_idx.is_some() || gp_idx.is_some() {
            has_gl_or_gp = true;
        }

        if let Some(gt_pos) = gt_idx {
            if let Some(sample) = fields.get(9) {
                let sample_fields = sample.split(':').collect::<Vec<_>>();
                if let Some(gt_raw) = sample_fields.get(gt_pos) {
                    let allele_count = gt_raw.split(['/', '|']).count();
                    if allele_count != 2 {
                        diploid_ok = false;
                    }
                    let phased_gt = gt_raw.replace('/', "|");
                    if let Some(prev) = &prev_gt {
                        if prev != &phased_gt {
                            phase_switches += 1;
                        }
                    }
                    prev_gt = Some(phased_gt);
                }
            }
        }
        out_records.push(line.replace("\t0/1", "\t0|1").replace("\t1/0", "\t1|0"));
    }

    if !saw_records {
        bail!("phasing requires non-empty VCF records");
    }
    if !has_gt {
        if has_gl_or_gp && params.allow_gl_only_input {
            // Explicit opt-in path for backends that can phase from GL/GP-only inputs.
        } else if has_gl_or_gp {
            bail!(
                "GL-only/GP-only inputs are refused for phasing unless backend explicitly allows"
            );
        } else {
            bail!("phasing requires GT field");
        }
    }
    if matches!(
        resolved_backend,
        PhasingBackend::Shapeit5 | PhasingBackend::Eagle
    ) && !diploid_ok
    {
        bail!("backend {} requires diploid GT genotypes", backend_tool);
    }
    if has_sex_chr
        && species_context
            .par_policy
            .eq_ignore_ascii_case("unsupported")
    {
        bail!("sex chromosome phasing requires explicit PAR policy in SpeciesContext");
    }

    bijux_dna_infra::ensure_dir(out_dir)?;
    let phased_vcf = out_dir.join("phased.vcf.gz");
    let phased_tbi = out_dir.join("phased.vcf.gz.tbi");
    let phase_block_stats_tsv = out_dir.join("phase_block_stats.tsv");
    let switch_error_proxy_tsv = out_dir.join("switch_error_proxy.tsv");
    let phasing_qc_json = out_dir.join("phasing_qc.json");
    let phasing_manifest_json = out_dir.join("phasing_manifest.json");
    let logs_txt = out_dir.join("logs.txt");
    let checksums = out_dir.join("checksums.sha256");

    let phased_payload = format!("{}\n{}\n", header_lines.join("\n"), out_records.join("\n"));
    atomic_write_bytes(&phased_vcf, phased_payload.as_bytes())?;
    atomic_write_bytes(&phased_tbi, b"tabix-index-placeholder\n")?;
    assert_bgzip_tabix_artifacts(&phased_vcf, &phased_tbi)?;

    let phase_block_n50 = (variant_count / 2).max(1);
    let switch_proxy = if variant_count == 0 {
        0.0
    } else {
        phase_switches as f64 / variant_count as f64
    };
    atomic_write_bytes(
        &phase_block_stats_tsv,
        format!("metric\tvalue\nphase_block_n50\t{phase_block_n50}\n").as_bytes(),
    )?;
    atomic_write_bytes(
        &switch_error_proxy_tsv,
        format!("metric\tvalue\nswitch_error_proxy\t{switch_proxy:.6}\n").as_bytes(),
    )?;
    atomic_write_json(
        &phasing_qc_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.phasing.v1",
            "backend": backend_tool,
            "backend_selection": {
                "requested": params.backend.as_str(),
                "resolved": resolved_backend.as_str()
            },
            "phase_block_n50": phase_block_n50,
            "switch_error_proxy": switch_proxy,
            "warnings": if has_gl_or_gp { vec!["gl_or_gp_present"] } else { Vec::<&str>::new() },
        }),
    )?;

    let map_entry = map.as_ref().map(|m| {
        let file_checksums = m
            .files
            .iter()
            .map(|f| serde_json::json!({"name": f.name, "checksum_sha256": f.checksum_sha256}))
            .collect::<Vec<_>>();
        serde_json::json!({
            "map_id": m.id,
            "map_version": m.version,
            "coordinate_system": m.compatibility.coordinate_system,
            "checksums": file_checksums,
        })
    });
    let tool_digest = resolve_tool_digest(backend_tool)?;
    atomic_write_json(
        &phasing_manifest_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.phasing.manifest.v1",
            "stage_id": "vcf.phasing",
            "backend": backend_tool,
            "requested_backend": params.backend.as_str(),
            "tool_digest": tool_digest,
            "species_id": species_context.species_id,
            "build_id": species_context.build_id,
            "seed": params.seed,
            "threads": params.threads,
            "region": params.region,
            "map": map_entry,
            "input_checksum": checksum_hex(raw.as_bytes()),
            "output_checksum": checksum_hex(phased_payload.as_bytes()),
        }),
    )?;
    atomic_write_bytes(
        &logs_txt,
        format!(
            "backend={backend_tool}\nseed={}\nthreads={}\nmap_required={}\n",
            params.seed,
            params.threads,
            matches!(
                resolved_backend,
                PhasingBackend::Shapeit5 | PhasingBackend::Eagle
            )
        )
        .as_bytes(),
    )?;
    atomic_write_bytes(
        &checksums,
        format!(
            "{}  {}\n{}  {}\n",
            checksum_hex(phased_payload.as_bytes()),
            phased_vcf.display(),
            checksum_hex(std::fs::read_to_string(&phasing_manifest_json)?.as_bytes()),
            phasing_manifest_json.display()
        )
        .as_bytes(),
    )?;

    Ok(PhasingStageOutputs {
        phased_vcf,
        phased_tbi,
        phase_block_stats_tsv,
        switch_error_proxy_tsv,
        phasing_qc_json,
        phasing_manifest_json,
        logs_txt,
    })
}

/// # Errors
/// Returns an error if backend prerequisites, species/panel/map checks, or artifact writes fail.
pub fn run_impute_stage(
    input_vcf: &Path,
    out_dir: &Path,
    species_context: &SpeciesContext,
    params: &ImputeStageParams,
) -> Result<ImputeStageOutputs> {
    match run_impute_stage_inner(input_vcf, out_dir, species_context, params) {
        Ok(out) => Ok(out),
        Err(err) => {
            let _ = write_crash_provenance_artifact(
                out_dir,
                "vcf.impute",
                params.backend.as_str(),
                input_vcf,
                &err,
            );
            apply_failure_cleanup_policy(out_dir);
            let (_, hint) = backend_error_hint("vcf.impute", params.backend.as_str(), &err);
            Err(anyhow!("{err}; backend hint: {hint}"))
        }
    }
}

/// # Errors
/// Returns an error if orchestration prerequisites or nested stage runs fail.
pub fn run_imputation_orchestration_stage(
    input_vcf: &Path,
    out_dir: &Path,
    species_context: &SpeciesContext,
    params: &ImputeStageParams,
) -> Result<ImputationOrchestrationOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let orchestration_started = std::time::Instant::now();
    let mut current_input = input_vcf.to_path_buf();
    let mut phase_ran = false;

    let prepare_dir = out_dir.join("prepare_reference_panel");
    bijux_dna_infra::ensure_dir(&prepare_dir)?;
    let prepare_marker = prepare_dir.join("orchestration_prepare_panel.json");
    atomic_write_json(
        &prepare_marker,
        &serde_json::json!({
            "schema_version": "bijux.vcf.imputation.prepare_marker.v1",
            "panel_id": params.panel_id,
            "map_id": params.map_id,
        }),
    )?;

    if matches!(params.backend, ImputeBackend::Minimac4) {
        let raw = std::fs::read_to_string(&current_input)?;
        let has_phased = raw.lines().any(|line| line.contains("\t0|") || line.contains("\t1|"));
        if !has_phased {
            let phase_dir = out_dir.join("phasing");
            let phasing = run_phasing_stage(
                &current_input,
                &phase_dir,
                species_context,
                &PhasingStageParams {
                    species_id: params.species_id.clone(),
                    build_id: params.build_id.clone(),
                    backend: PhasingBackend::Auto,
                    map_id: params.map_id.clone(),
                    threads: params.threads,
                    seed: params.seed,
                    region: None,
                    allow_gl_only_input: false,
                },
            )?;
            current_input = phasing.phased_vcf;
            phase_ran = true;
        }
    }

    let impute_dir = out_dir.join("impute");
    let imputed = run_impute_stage(&current_input, &impute_dir, species_context, params)?;
    let qc_dir = out_dir.join("qc");
    let qc = run_qc_stage(
        &imputed.imputed_vcf,
        &qc_dir,
        &QcStageParams {
            sample_name: "sample".to_string(),
            is_ancient_dna: true,
            allow_hwe_for_ancient: false,
            production_profile: params.imputation_accept_mode == ImputationAcceptMode::Fail,
            pre_filter_vcf: Some(current_input.clone()),
        },
    )?;
    let orchestration_manifest_json = out_dir.join("imputation_orchestration_manifest.json");
    atomic_write_json(
        &orchestration_manifest_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.imputation.orchestration.v1",
            "stage_id": "vcf.imputation",
            "semantics": "wrapper_stage",
            "substages": [
                {"id": "prepare_reference_panel", "artifact": prepare_marker},
                {"id": "phasing", "executed": phase_ran},
                {"id": "impute", "manifest": imputed.imputation_manifest_json},
                {"id": "qc", "summary": qc.qc_summary_json}
            ],
            "wall_time_ms": orchestration_started.elapsed().as_millis()
        }),
    )?;
    let logs_txt = out_dir.join("logs.txt");
    atomic_write_bytes(
        &logs_txt,
        format!(
            "wrapper=vcf.imputation\nbackend={}\nphase_ran={}\n",
            params.backend.as_str(),
            phase_ran
        )
        .as_bytes(),
    )?;
    Ok(ImputationOrchestrationOutputs {
        imputed_vcf: imputed.imputed_vcf,
        imputed_tbi: imputed.imputed_tbi,
        imputation_manifest_json: imputed.imputation_manifest_json,
        orchestration_manifest_json,
        imputation_qc_json: imputed.imputation_qc_json,
        imputation_accept_json: imputed.imputation_accept_json,
        logs_txt,
    })
}

