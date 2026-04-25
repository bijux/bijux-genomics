use bijux_dna_core::ids::StageId;
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_fastq::params::{DamageMode, PairedMode};
use bijux_dna_domain_fastq::pipeline_contract;
use bijux_dna_pipelines::fastq::{
    fastq_adna_profile, fastq_default_profile, fastq_minimal_profile, fastq_reference_adna_profile,
    validate_fastq_profile,
};
use bijux_dna_pipelines::DefaultParams;

#[test]
fn fastq_profiles_validate_in_tests() {
    for profile in [
        fastq_default_profile(),
        fastq_adna_profile(),
        fastq_reference_adna_profile(),
        fastq_minimal_profile(),
    ] {
        let report = validate_fastq_profile(&profile);
        assert!(
            report.valid,
            "profile {} failed FASTQ invariants: {:?}",
            report.profile_id, report.violations
        );
    }
}

#[test]
fn reference_adna_profile_stage_contract_and_pairing_invariants() {
    let profile = fastq_reference_adna_profile();
    let report = validate_fastq_profile(&profile);
    assert!(report.valid, "reference profile invalid: {:?}", report.violations);
    for stage in [
        id_catalog::FASTQ_VALIDATE_PRE,
        id_catalog::FASTQ_DETECT_ADAPTERS,
        id_catalog::FASTQ_TRIM,
        id_catalog::FASTQ_LOW_COMPLEXITY,
        id_catalog::FASTQ_MERGE,
        id_catalog::FASTQ_STATS_NEUTRAL,
        id_catalog::FASTQ_QC_POST,
    ] {
        assert!(
            profile.capabilities.required_stages.iter().any(|required| required == stage),
            "reference profile must include required stage {stage}"
        );
    }

    let detect_stage = StageId::from_static(id_catalog::FASTQ_DETECT_ADAPTERS);
    let Some(bijux_dna_pipelines::DefaultParams::FastqDetectAdapters(preprocess)) =
        profile.defaults.params.get(&detect_stage)
    else {
        panic!("missing detect_adapters params");
    };
    assert!(
        preprocess.paired_mode == PairedMode::PairedEnd,
        "reference profile must keep paired-end defaults on FASTQ stage params"
    );
}

#[test]
fn adna_profiles_obey_core_stage_and_param_properties() {
    let profile = fastq_adna_profile();
    let report = validate_fastq_profile(&profile);
    assert!(report.valid, "adna profile invalid: {:?}", report.violations);

    let required = &profile.capabilities.required_stages;
    for stage in [
        id_catalog::FASTQ_VALIDATE_PRE,
        id_catalog::FASTQ_DETECT_ADAPTERS,
        id_catalog::FASTQ_TRIM,
        id_catalog::FASTQ_FILTER,
        id_catalog::FASTQ_QC_POST,
    ] {
        assert!(
            required.iter().any(|required_stage| required_stage == stage),
            "aDNA profile must include required stage {stage}"
        );
    }

    let trim_stage = StageId::from_static(id_catalog::FASTQ_TRIM);
    let Some(DefaultParams::FastqTrim(trim)) = profile.defaults.params.get(&trim_stage) else {
        panic!("missing trim params");
    };
    assert!(trim.min_len > 0, "aDNA trim.min_len must be > 0");
    assert_ne!(
        trim.adapter_policy.to_lowercase(),
        "none",
        "aDNA trim.adapter_policy must not be none"
    );

    let trim_polyg_stage = StageId::from_static("fastq.trim_polyg_tails");
    let Some(DefaultParams::FastqTrimPolygTails(trim_polyg)) =
        profile.defaults.params.get(&trim_polyg_stage)
    else {
        panic!("missing trim polyg params");
    };
    assert!(trim_polyg.trim_polyg);
    assert!(trim_polyg.min_polyg_run >= 10);

    let trim_terminal_damage_stage = StageId::from_static("fastq.trim_terminal_damage");
    let Some(DefaultParams::FastqTrimTerminalDamage(trim_terminal_damage)) =
        profile.defaults.params.get(&trim_terminal_damage_stage)
    else {
        panic!("missing terminal damage params");
    };
    assert_eq!(trim_terminal_damage.damage_mode, DamageMode::Ancient);
}

#[test]
fn adna_invariants_reject_scientifically_invalid_defaults() {
    let mut profile = fastq_adna_profile();
    let trim_stage = StageId::from_static(id_catalog::FASTQ_TRIM);
    let Some(DefaultParams::FastqTrim(mut trim)) =
        profile.defaults.params.get(&trim_stage).cloned()
    else {
        panic!("missing trim params");
    };
    trim.min_len = 0;
    trim.adapter_policy = "none".to_string();
    profile.defaults.params.insert(trim_stage, DefaultParams::FastqTrim(trim));

    let report = validate_fastq_profile(&profile);
    assert!(!report.valid, "invalid aDNA profile should be rejected");
    assert!(
        report.violations.iter().any(|violation| violation.code == "trim_min_len_invalid"),
        "expected trim_min_len_invalid violation"
    );
    assert!(
        report.violations.iter().any(|violation| violation.code == "adna_adapter_policy_invalid"),
        "expected adna_adapter_policy_invalid violation"
    );

    let invariants_report = report.as_invariants_report();
    assert_eq!(invariants_report.schema_version, "bijux.invariants_report.v1");
    assert!(invariants_report.blocking);
}

fn essential_shotgun_stages() -> Vec<String> {
    pipeline_contract::default_shotgun_preprocess_stage_order()
        .into_iter()
        .map(|stage_id| stage_id.as_str().to_string())
        .collect()
}

#[test]
fn single_end_fastq_profiles_cover_domain_essential_shotgun_stages() {
    let essential = essential_shotgun_stages();
    let default_profile = fastq_default_profile();
    for stage in &essential {
        assert!(
            default_profile.capabilities.required_stages.iter().any(|required| required == stage),
            "profile {} must include domain essential stage {}",
            default_profile.id,
            stage
        );
    }
}

#[test]
fn generic_fastq_profiles_do_not_force_terminal_damage_defaults() {
    for profile in [fastq_default_profile(), fastq_minimal_profile()] {
        let damage_stage = "fastq.trim_terminal_damage";
        assert!(
            !profile.capabilities.required_stages.iter().any(|stage| stage == damage_stage),
            "profile {} must not require terminal-damage trimming",
            profile.id
        );
        assert!(
            !profile.defaults.params.contains_key(&StageId::from_static(damage_stage)),
            "profile {} must not carry terminal-damage params",
            profile.id
        );
        assert!(
            !profile.defaults.tools.contains_key(&StageId::from_static(damage_stage)),
            "profile {} must not carry terminal-damage tool defaults",
            profile.id
        );
    }
}

#[test]
fn minimal_fastq_profile_is_smaller_than_default_profile() {
    let default_profile = fastq_default_profile();
    let minimal_profile = fastq_minimal_profile();
    assert!(
        minimal_profile.capabilities.required_stages.len()
            < default_profile.capabilities.required_stages.len(),
        "minimal FASTQ profile must stay smaller than the default profile"
    );
}

#[test]
fn fastq_pipeline_defaults_follow_domain_active_tools() {
    let default_profile = fastq_default_profile();
    let filter_tool = default_profile
        .defaults
        .tools
        .get(&StageId::from_static(id_catalog::FASTQ_FILTER))
        .expect("filter default tool");
    assert_eq!(
        filter_tool.as_str(),
        "fastp",
        "pipeline defaults must align with domain active default for fastq.filter_reads"
    );

    let merge_tool = default_profile
        .defaults
        .tools
        .get(&StageId::from_static(id_catalog::FASTQ_MERGE))
        .expect("merge default tool");
    assert_eq!(
        merge_tool.as_str(),
        "pear",
        "pipeline defaults must align with domain active default for fastq.merge_pairs"
    );
}

#[test]
fn essential_shotgun_stage_roster_stays_in_sync_with_domain_contract() {
    let pipeline_essentials = essential_shotgun_stages();
    let expected = vec![
        id_catalog::FASTQ_VALIDATE_PRE.to_string(),
        "fastq.profile_read_lengths".to_string(),
        id_catalog::FASTQ_DETECT_ADAPTERS.to_string(),
        "fastq.trim_polyg_tails".to_string(),
        id_catalog::FASTQ_TRIM.to_string(),
        id_catalog::FASTQ_FILTER.to_string(),
        id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
        "fastq.profile_overrepresented_sequences".to_string(),
        id_catalog::FASTQ_QC_POST.to_string(),
    ];
    assert_eq!(
        pipeline_essentials, expected,
        "FASTQ pipeline essential stage roster drifted from the domain pipeline contract"
    );
}
