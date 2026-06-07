use super::*;

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
        let raw = read_vcf_text_runtime(&current_input)?;
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
            "stage_id": "vcf.imputation_metrics",
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
            "wrapper=vcf.imputation_metrics\nbackend={}\nphase_ran={}\n",
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
