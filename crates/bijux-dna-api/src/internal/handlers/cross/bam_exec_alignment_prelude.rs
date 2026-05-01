fn base_bam_args(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    profile: &PipelineProfile,
    bam: PathBuf,
    out: PathBuf,
    bam_index: Option<PathBuf>,
    reference: Option<PathBuf>,
) -> BamRunArgs {
    BamRunArgs {
        stage,
        profile: profile.id.to_string(),
        sample_id: None,
        r1: None,
        r2: None,
        bam,
        out,
        tool: None,
        dry_run: false,
        allow_planned: false,
        bai: bam_index,
        reference,
        regions: None,
        udg_model: None,
        pmd_threshold_5p: None,
        pmd_threshold_3p: None,
        trim_5p: None,
        trim_3p: None,
        contamination_scope: None,
        contamination_panel: Vec::new(),
        contamination_prior: None,
        sex_specific_contamination: false,
        contamination_assumptions: None,
        expected_sex: None,
        sex_method: "rxy".to_string(),
        min_mapq: None,
        min_length: None,
        include_flags: Vec::new(),
        exclude_flags: Vec::new(),
        remove_duplicates: false,
        base_quality_threshold: None,
        optical_duplicates: None,
        umi_policy: None,
        duplicate_action: None,
        complexity_min_reads: None,
        complexity_projection_points: Vec::new(),
        depth_thresholds: Vec::new(),
        bqsr_mode: None,
        known_sites: Vec::new(),
        bqsr_min_mean_coverage: None,
        bqsr_min_breadth_1x: None,
        haplogroup_panel: None,
        haplogroup_min_coverage: None,
        kinship_panel: None,
        min_overlap_snps: None,
        caller: None,
        min_posterior: None,
        min_call_rate: None,
        gc_bias_correction: false,
        map_bias_correction: false,
        authenticity_mode: None,
        aligner_preset: None,
        alignment_sensitivity_profile: None,
        alignment_seed_length: None,
        rg_id: None,
        rg_sm: None,
        rg_pl: None,
        rg_lb: None,
        rg_pu: None,
        lane_id: None,
        run_id: None,
        subject_id: None,
        cohort_id: None,
        rg_policy: None,
        build_reference_indices: false,
        params_json: None,
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum AlignmentRegime {
    Adna,
    Modern,
    Edna,
}

fn alignment_meta_value(args: &FastqCrossArgs, key: &str) -> Option<String> {
    for entry in &args.alignment_meta {
        if let Some((found_key, found_value)) = entry.split_once('=') {
            if found_key == key {
                return Some(found_value.to_string());
            }
        }
    }
    None
}

fn infer_alignment_regime(profile: &PipelineProfile, args: &FastqCrossArgs) -> AlignmentRegime {
    if let Some(explicit) = alignment_meta_value(args, "alignment_regime") {
        return match explicit.as_str() {
            "adna" => AlignmentRegime::Adna,
            "edna" | "pollen" => AlignmentRegime::Edna,
            _ => AlignmentRegime::Modern,
        };
    }
    let profile_id = profile.id.as_str().to_ascii_lowercase();
    if profile_id.contains("edna") || profile_id.contains("pollen")
    {
        return AlignmentRegime::Edna;
    }
    if profile_id.contains("adna") {
        return AlignmentRegime::Adna;
    }
    AlignmentRegime::Modern
}
