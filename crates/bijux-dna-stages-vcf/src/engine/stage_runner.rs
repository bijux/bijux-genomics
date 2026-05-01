use super::request::{
    map_runner_error, refusal, resolve_call_alias, resolve_stage_tool_digest,
    stage_default_tool_id, try_resume_stage, write_artifact_checksums, write_sidecars,
    write_stage_manifest, DispatchRunner, ToolInvocationBuilder,
};
use super::*;

fn pca_params_for_species(species_id: &str) -> PcaStageParams {
    let mut params = PcaStageParams::default();
    let species = species_id.to_ascii_lowercase();
    if species.contains("homo sapiens") {
        params.preprocessing.maf_threshold = 0.01;
        params.preprocessing.max_missingness = 0.10;
    } else {
        params.preprocessing.maf_threshold = 0.02;
        params.preprocessing.max_missingness = 0.15;
    }
    params
}

fn sample_population_manifest_for_run(run_root: &Path) -> Option<PathBuf> {
    let candidates = [
        run_root.join("population_labels.json"),
        run_root.join("config").join("population_labels.json"),
        run_root.join("artifacts").join("metadata").join("population_labels.json"),
    ];
    candidates.into_iter().find(|path| path.exists())
}

fn population_structure_params_for_species(species_id: &str) -> PopulationStructureStageParams {
    let mut params = PopulationStructureStageParams::default();
    let species = species_id.to_ascii_lowercase();
    if species.contains("homo sapiens") {
        params.smartpca = true;
        params.preprocessing.maf_threshold = 0.01;
    } else {
        params.smartpca = false;
        params.preprocessing.maf_threshold = 0.02;
    }
    params
}

fn roh_params_for_species(species_id: &str) -> RohStageParams {
    let mut params = RohStageParams::default();
    let species = species_id.to_ascii_lowercase();
    if species.contains("homo sapiens") {
        params.min_snp_density_per_mb = 10.0;
        params.max_missingness = 0.20;
    } else {
        params.min_snp_density_per_mb = 5.0;
        params.max_missingness = 0.30;
    }
    params
}

fn ibd_params_for_species(species_id: &str, build_id: &str) -> IbdStageParams {
    let mut params = IbdStageParams::default();
    let species = species_id.to_ascii_lowercase();
    params.expected_build = Some(build_id.to_string());
    if species.contains("homo sapiens") {
        params.min_variant_density_per_mb = 1.0;
        params.max_missingness = 0.20;
    } else {
        params.min_variant_density_per_mb = 0.5;
        params.max_missingness = 0.30;
    }
    params
}

fn demography_params_for_species(species_id: &str, build_id: &str) -> DemographyStageParams {
    let mut params = DemographyStageParams {
        expected_build: Some(build_id.to_string()),
        ..DemographyStageParams::default()
    };
    if species_id.to_ascii_lowercase().contains("homo sapiens") {
        params.min_segments = 3;
    }
    params
}

fn infer_damage_filter_params_from_bam(run_root: &Path) -> Option<DamageFilterStageParams> {
    let candidates = [
        run_root.join("artifacts").join("bam").join("bam_metrics.json"),
        run_root.join("artifacts").join("bam").join("metrics.json"),
        run_root.join("artifacts").join("bam").join("summary.json"),
    ];
    let raw = candidates.iter().find_map(|p| std::fs::read_to_string(p).ok())?;
    let json: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let damage_rate =
        json.get("damage_ct_ga_rate").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
    let udg = json.get("udg_treated").and_then(serde_json::Value::as_bool).unwrap_or(false);
    let params = DamageFilterStageParams {
        udg_regime: if udg {
            crate::pipeline::DamageUdgRegime::Udg
        } else {
            crate::pipeline::DamageUdgRegime::NonUdg
        },
        strict_regime: false,
        max_damage_ratio: if udg {
            (damage_rate + 0.10).clamp(0.10, 0.50)
        } else {
            (damage_rate + 0.15).clamp(0.20, 0.60)
        },
        ..DamageFilterStageParams::recommended()
    };
    Some(params)
}

impl VcfStageRunner for DispatchRunner {
    fn stage(&self) -> VcfDomainStage {
        self.stage
    }

    fn run(&self, ctx: &VcfStageRunContext<'_>, input_vcf: &Path) -> Result<VcfStageOutputs> {
        let stage = self.stage;
        let stage_dir = ctx.artifact_root.join(stage.as_str().replace('.', "_"));
        bijux_dna_infra::ensure_dir(&stage_dir)?;
        if std::env::var("BIJUX_VCF_ALLOW_NETWORK").ok().as_deref() == Some("1") {
            return Err(refusal(
                VcfRefusalCode::PlanningFailed,
                "no-network policy violation: BIJUX_VCF_ALLOW_NETWORK=1 is not permitted",
            ));
        }
        if let Some(resumed) = try_resume_stage(stage, &stage_dir)? {
            return Ok(resumed);
        }
        let stage_tmp_dir = ctx.request.run_root.join("tmp").join(stage.as_str().replace('.', "_"));
        bijux_dna_infra::ensure_dir(&stage_tmp_dir)?;
        let started = Instant::now();
        let mut artifacts = Vec::<PathBuf>::new();
        let mut primary_output = None;
        let mut tool_id = stage_default_tool_id(stage).to_string();
        let runtime =
            std::env::var("BIJUX_CONTAINER_RUNTIME").unwrap_or_else(|_| "apptainer".to_string());
        let mut version = "captured-in-stage-artifacts".to_string();
        let mut argv = vec![tool_id.clone(), stage.as_str().to_string()];

        match stage {
            VcfDomainStage::Call
            | VcfDomainStage::CallGl
            | VcfDomainStage::CallDiploid
            | VcfDomainStage::CallPseudohaploid => {
                let params = VcfCallParams {
                    sample_name: ctx.request.sample_name.clone(),
                    reference_fasta: ctx.request.reference_fasta.clone(),
                    ..VcfCallParams::default()
                };
                let effective =
                    if stage == VcfDomainStage::Call { resolve_call_alias(ctx)? } else { stage };
                let out = match effective {
                    VcfDomainStage::CallGl => run_call_gl_stage(input_vcf, &stage_dir, &params),
                    VcfDomainStage::CallDiploid => {
                        run_call_diploid_stage(input_vcf, &stage_dir, &params)
                    }
                    VcfDomainStage::CallPseudohaploid => {
                        run_call_pseudohaploid_stage(input_vcf, &stage_dir, &params)
                    }
                    _ => Err(anyhow!("unsupported call stage {}", effective.as_str())),
                }
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                primary_output = Some(out.called_vcf.clone());
                artifacts.extend([
                    out.called_vcf,
                    out.called_tbi,
                    out.call_metrics_json,
                    out.call_metrics_tsv,
                    out.call_manifest_json,
                ]);
            }
            VcfDomainStage::Filter => {
                let out = run_filter_stage_real(
                    input_vcf,
                    &stage_dir,
                    &VcfFilterParams {
                        sample_name: ctx.request.sample_name.clone(),
                        production_profile: ctx.request.production_profile,
                        ..VcfFilterParams::default()
                    },
                )
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                primary_output = Some(out.filtered_vcf.clone());
                artifacts.extend([
                    out.filtered_vcf,
                    out.filtered_tbi,
                    out.filter_breakdown_json,
                    out.filter_breakdown_tsv,
                    out.filter_explain_json,
                ]);
            }
            VcfDomainStage::DamageFilter => {
                let params = ctx
                    .request
                    .damage_filter
                    .clone()
                    .or_else(|| infer_damage_filter_params_from_bam(&ctx.request.run_root))
                    .unwrap_or_else(DamageFilterStageParams::recommended);
                let out =
                    run_damage_filter_stage(input_vcf, &stage_dir, &params).map_err(|err| {
                        let (code, hint) = map_runner_error(&err.to_string());
                        refusal(code, hint)
                    })?;
                primary_output = Some(out.filtered_vcf.clone());
                artifacts.extend([
                    out.filtered_vcf,
                    out.filtered_tbi,
                    out.damage_filter_summary_json,
                    out.damage_filter_counts_json,
                    out.warnings_json,
                    out.damage_genotype_manifest_json,
                ]);
            }
            VcfDomainStage::GlPropagation => {
                let params = ctx
                    .request
                    .gl_propagation
                    .clone()
                    .unwrap_or_else(GlPropagationStageParams::recommended);
                let out =
                    run_gl_propagation_stage(input_vcf, &stage_dir, &params).map_err(|err| {
                        let (code, hint) = map_runner_error(&err.to_string());
                        refusal(code, hint)
                    })?;
                primary_output = Some(out.normalized_vcf.clone());
                artifacts.push(out.normalized_vcf);
                artifacts.push(out.normalized_tbi);
                if let Some(bcf) = out.normalized_bcf {
                    artifacts.push(bcf);
                }
                if let Some(csi) = out.normalized_bcf_csi {
                    artifacts.push(csi);
                }
                artifacts.push(out.gl_propagation_report_json);
            }
            VcfDomainStage::Stats => {
                let out = run_stats_stage_real(
                    input_vcf,
                    &stage_dir,
                    &VcfStatsParams {
                        sample_name: ctx.request.sample_name.clone(),
                        ..VcfStatsParams::default()
                    },
                )
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                artifacts.extend([out.bcftools_stats_txt, out.stats_json]);
            }
            VcfDomainStage::Qc => {
                let out = run_qc_stage(
                    input_vcf,
                    &stage_dir,
                    &ctx.request.qc.clone().unwrap_or(QcStageParams {
                        sample_name: ctx.request.sample_name.clone(),
                        is_ancient_dna: true,
                        allow_hwe_for_ancient: false,
                        production_profile: ctx.request.production_profile,
                        pre_filter_vcf: None,
                    }),
                )
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                artifacts.extend([
                    out.qc_summary_json,
                    out.qc_tables_tsv,
                    out.imputation_qc_tsv,
                    out.warnings_json,
                    out.qc_histograms_json,
                ]);
            }
            VcfDomainStage::PrepareReferencePanel => {
                let params = ctx.request.prepare_panel.clone().ok_or_else(|| {
                    refusal(
                        VcfRefusalCode::PlanningFailed,
                        "missing prepare_reference_panel params",
                    )
                })?;
                let panel_vcf = ctx.request.panel_vcf.clone().ok_or_else(|| {
                    refusal(VcfRefusalCode::PlanningFailed, "missing panel_vcf path")
                })?;
                let out = run_prepare_reference_panel_stage(
                    input_vcf,
                    &panel_vcf,
                    &stage_dir,
                    &ctx.request.species_context,
                    &params,
                )
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                primary_output = Some(out.prepared_panel_vcf.clone());
                artifacts.extend([
                    out.prepared_panel_vcf,
                    out.prepared_panel_tbi,
                    out.panel_manifest_json,
                    out.overlap_json,
                    out.panel_overlap_json,
                    out.panel_files_json,
                    out.overlap_tsv,
                    out.chunks_json,
                ]);
            }
            VcfDomainStage::Phasing => {
                let params = ctx.request.phasing.clone().ok_or_else(|| {
                    refusal(VcfRefusalCode::PlanningFailed, "missing phasing params")
                })?;
                tool_id = params.backend.as_str().to_string();
                argv[0] = tool_id.clone();
                if params.seed == 0 {
                    return Err(refusal(
                        VcfRefusalCode::PlanningFailed,
                        "deterministic seed required for vcf.phasing",
                    ));
                }
                argv.push(format!("--seed={}", params.seed));
                let out =
                    run_phasing_stage(input_vcf, &stage_dir, &ctx.request.species_context, &params)
                        .map_err(|err| {
                            let (code, hint) = map_runner_error(&err.to_string());
                            refusal(code, hint)
                        })?;
                primary_output = Some(out.phased_vcf.clone());
                artifacts.extend([
                    out.phased_vcf,
                    out.phased_tbi,
                    out.phase_block_stats_tsv,
                    out.switch_error_proxy_tsv,
                    out.phasing_qc_json,
                    out.phasing_manifest_json,
                    out.logs_txt,
                ]);
            }
            VcfDomainStage::Impute => {
                let params = ctx.request.impute.clone().ok_or_else(|| {
                    refusal(VcfRefusalCode::PlanningFailed, "missing impute params")
                })?;
                tool_id = params.backend.as_str().to_string();
                argv[0] = tool_id.clone();
                if params.seed == 0 {
                    return Err(refusal(
                        VcfRefusalCode::PlanningFailed,
                        "deterministic seed required for vcf.impute",
                    ));
                }
                argv.push(format!("--seed={}", params.seed));
                let out =
                    run_impute_stage(input_vcf, &stage_dir, &ctx.request.species_context, &params)
                        .map_err(|err| {
                            let (code, hint) = map_runner_error(&err.to_string());
                            refusal(code, hint)
                        })?;
                primary_output = Some(out.imputed_vcf.clone());
                artifacts.extend([
                    out.imputed_vcf,
                    out.imputed_tbi,
                    out.imputation_qc_json,
                    out.imputation_qc_tsv,
                    out.maf_bin_quality_tsv,
                    out.info_hist_json,
                    out.warnings_json,
                    out.imputation_accept_json,
                    out.overlap_stats_json,
                    out.imputation_manifest_json,
                    out.panel_mismatch_diagnostics_json,
                    out.logs_txt,
                ]);
            }
            VcfDomainStage::Imputation => {
                let params = ctx.request.impute.clone().ok_or_else(|| {
                    refusal(VcfRefusalCode::PlanningFailed, "missing impute params")
                })?;
                tool_id = params.backend.as_str().to_string();
                argv[0] = tool_id.clone();
                let out = run_imputation_orchestration_stage(
                    input_vcf,
                    &stage_dir,
                    &ctx.request.species_context,
                    &params,
                )
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                primary_output = Some(out.imputed_vcf.clone());
                artifacts.extend([
                    out.imputed_vcf,
                    out.imputed_tbi,
                    out.imputation_qc_json,
                    out.imputation_accept_json,
                    out.imputation_manifest_json,
                    out.orchestration_manifest_json,
                    out.logs_txt,
                ]);
            }
            VcfDomainStage::Postprocess => {
                let params = ctx.request.postprocess.clone().ok_or_else(|| {
                    refusal(VcfRefusalCode::PlanningFailed, "missing postprocess params")
                })?;
                let out = run_postprocess_stage(
                    input_vcf,
                    &stage_dir,
                    &ctx.request.species_context,
                    &params,
                )
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                primary_output = Some(out.merged_vcf.clone());
                artifacts.push(out.merged_vcf);
                artifacts.push(out.merged_tbi);
                if let Some(bcf) = out.merged_bcf {
                    artifacts.push(bcf);
                }
                artifacts.push(out.artifact_checksums_json);
                artifacts.push(out.normalization_contract_json);
                artifacts.push(out.validate_outputs_json);
                artifacts.push(out.final_manifest_json);
                artifacts.push(out.logs_txt);
            }
            VcfDomainStage::Pca => {
                let mut params = pca_params_for_species(&ctx.request.species_context.species_id);
                params.sample_metadata_manifest =
                    sample_population_manifest_for_run(&ctx.request.run_root);
                let out = run_pca_stage(input_vcf, &stage_dir, &params).map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                artifacts.extend([
                    out.eigenvec_tsv,
                    out.eigenval_tsv,
                    out.pca_manifest_json,
                    out.logs_txt,
                ]);
            }
            VcfDomainStage::PopulationStructure => {
                let mut params = population_structure_params_for_species(
                    &ctx.request.species_context.species_id,
                );
                params.sample_metadata_manifest =
                    sample_population_manifest_for_run(&ctx.request.run_root);
                let out = run_population_structure_stage(input_vcf, &stage_dir, &params).map_err(
                    |err| {
                        let (code, hint) = map_runner_error(&err.to_string());
                        refusal(code, hint)
                    },
                )?;
                artifacts.extend([
                    out.pruned_variants_tsv,
                    out.population_structure_json,
                    out.logs_txt,
                ]);
            }
            VcfDomainStage::Admixture => {
                let out = run_admixture_stage(
                    input_vcf,
                    &stage_dir,
                    &AdmixtureStageParams {
                        sample_metadata_manifest: sample_population_manifest_for_run(
                            &ctx.request.run_root,
                        ),
                        ..AdmixtureStageParams::default()
                    },
                )
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                artifacts.extend([out.q_matrix_tsv, out.k_selection_json, out.logs_txt]);
            }
            VcfDomainStage::Roh => {
                let out = run_roh_stage(
                    input_vcf,
                    &stage_dir,
                    &roh_params_for_species(&ctx.request.species_context.species_id),
                )
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                artifacts.extend([
                    out.roh_segments_tsv,
                    out.roh_per_sample_tsv,
                    out.roh_json,
                    out.metrics_json,
                    out.roh_summary_json,
                    out.roh_metrics_json,
                    out.logs_txt,
                ]);
            }
            VcfDomainStage::Ibd => {
                let out = run_ibd_stage(
                    input_vcf,
                    &stage_dir,
                    &ibd_params_for_species(
                        &ctx.request.species_context.species_id,
                        &ctx.request.species_context.build_id,
                    ),
                )
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                primary_output = Some(out.ibd_filtered_segments_tsv.clone());
                artifacts.extend([
                    out.ibd_input_tsv,
                    out.ibd_segments_tsv,
                    out.ibd_merged_segments_tsv,
                    out.ibd_filtered_segments_tsv,
                    out.ibd_summary_json,
                    out.ibd_metrics_json,
                    out.logs_txt,
                ]);
            }
            VcfDomainStage::Demography => {
                let out = run_demography_stage(
                    input_vcf,
                    &stage_dir,
                    &demography_params_for_species(
                        &ctx.request.species_context.species_id,
                        &ctx.request.species_context.build_id,
                    ),
                )
                .map_err(|err| {
                    let (code, hint) = map_runner_error(&err.to_string());
                    refusal(code, hint)
                })?;
                artifacts.extend([
                    out.ne_trajectory_tsv,
                    out.demography_json,
                    out.demography_metrics_json,
                    out.logs_txt,
                ]);
            }
        }

        let image_digest = resolve_stage_tool_digest(&tool_id)
            .map_err(|err| refusal(VcfRefusalCode::PlanningFailed, err.to_string()))?;
        if stage == VcfDomainStage::Admixture {
            version = "refused-or-captured-in-stage-artifacts".to_string();
        }
        let invocation = ToolInvocationBuilder::new(&tool_id, &runtime, &image_digest)
            .argv(argv.clone())
            .io(vec![input_vcf.to_path_buf()], artifacts.clone())
            .build()
            .map_err(|err| refusal(VcfRefusalCode::PlanningFailed, err.to_string()))?;
        atomic_write_json(&stage_dir.join("tool_invocation.json"), &invocation)?;
        atomic_write_bytes(&stage_dir.join("tool_version.txt"), format!("{version}\n").as_bytes())?;
        write_sidecars(&stage_dir, stage, &argv, &stage_tmp_dir)?;
        let runtime = StageRuntimeStats {
            wall_time_ms: started.elapsed().as_millis(),
            exit_code: 0,
            rss_kb: None,
        };
        let checksums_path = write_artifact_checksums(&stage_dir, &artifacts)?;
        artifacts.push(checksums_path);
        let stage_manifest =
            write_stage_manifest(&stage_dir, stage, input_vcf, &artifacts, &runtime, &invocation)?;

        Ok(VcfStageOutputs {
            stage_id: stage.as_str().to_string(),
            artifact_dir: stage_dir,
            primary_output,
            artifacts,
            stage_manifest,
            runtime,
        })
    }
}
