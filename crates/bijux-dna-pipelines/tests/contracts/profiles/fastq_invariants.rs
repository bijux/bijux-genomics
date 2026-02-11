use bijux_dna_core::ids::StageId;
use bijux_dna_core::prelude::id_catalog;
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
    assert!(
        report.valid,
        "reference profile invalid: {:?}",
        report.violations
    );
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
            profile
                .capabilities
                .required_stages
                .iter()
                .any(|candidate| *candidate == stage),
            "reference profile must include required stage {stage}"
        );
    }

    let preprocess_stage = StageId::from_static(id_catalog::FASTQ_PREPROCESS);
    let Some(bijux_dna_pipelines::DefaultParams::FastqPreprocess(preprocess)) =
        profile.defaults.params.get(&preprocess_stage)
    else {
        panic!("missing preprocess params");
    };
    assert!(
        preprocess.library_declared_paired,
        "reference profile must declare paired library type"
    );
}

#[test]
fn adna_profiles_obey_core_stage_and_param_properties() {
    let profile = fastq_adna_profile();
    let report = validate_fastq_profile(&profile);
    assert!(
        report.valid,
        "adna profile invalid: {:?}",
        report.violations
    );

    let required = &profile.capabilities.required_stages;
    for stage in [
        id_catalog::FASTQ_VALIDATE_PRE,
        id_catalog::FASTQ_DETECT_ADAPTERS,
        id_catalog::FASTQ_TRIM,
        id_catalog::FASTQ_FILTER,
        id_catalog::FASTQ_QC_POST,
    ] {
        assert!(
            required.iter().any(|candidate| *candidate == stage),
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
    profile
        .defaults
        .params
        .insert(trim_stage, DefaultParams::FastqTrim(trim));

    let report = validate_fastq_profile(&profile);
    assert!(!report.valid, "invalid aDNA profile should be rejected");
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.code == "trim_min_len_invalid"),
        "expected trim_min_len_invalid violation"
    );
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.code == "adna_adapter_policy_invalid"),
        "expected adna_adapter_policy_invalid violation"
    );
}
