#[test]
fn damage_and_auth_models_have_shared_core_fields() -> anyhow::Result<()> {
    let core = bijux_dna_domain_bam::metrics::DamageCoreFieldsV1 {
        tool: "damageprofiler".to_string(),
        c_to_t_5p: 0.21,
        g_to_a_3p: 0.19,
        reads_considered: 10_000,
    };
    let model = bijux_dna_domain_bam::metrics::DamageProfilerMetricsV1 {
        core: core.clone(),
        misincorporation: bijux_dna_domain_bam::metrics::MisincorporationCurveSummaryV1 {
            five_prime: vec![bijux_dna_domain_bam::metrics::MisincorporationPointV1 {
                position: 1,
                c_to_t_rate: 0.21,
                g_to_a_rate: 0.02,
            }],
            three_prime: vec![bijux_dna_domain_bam::metrics::MisincorporationPointV1 {
                position: 1,
                c_to_t_rate: 0.03,
                g_to_a_rate: 0.19,
            }],
        },
    };
    let raw = serde_json::to_value(&model)?;
    assert_eq!(raw["core"]["tool"], "damageprofiler");
    assert_eq!(raw["core"]["reads_considered"], 10_000);
    Ok(())
}

#[test]
fn contamination_models_require_inputs_assumptions_and_warnings() -> anyhow::Result<()> {
    let model = bijux_dna_domain_bam::metrics::SchmutziMetricsV1 {
        contamination: bijux_dna_domain_bam::metrics::ContaminationToolMetricsV1 {
            tool: "schmutzi".to_string(),
            estimate: 0.03,
            ci_low: 0.01,
            ci_high: 0.05,
            model_assumptions: vec!["mtDNA-only estimate".to_string()],
            required_inputs: bijux_dna_domain_bam::metrics::ContaminationRequiredInputsV1 {
                reference_panel: "1000g-hg19".to_string(),
                scope: bijux_dna_domain_bam::metrics::ContaminationInputScopeV1::MtOnly,
            },
            warnings: vec![bijux_dna_domain_bam::metrics::ContaminationWarningV1 {
                code: "LOW_COVERAGE".to_string(),
                message: "mtDNA coverage below recommended depth".to_string(),
            }],
        },
    };
    let raw = serde_json::to_value(&model)?;
    assert_eq!(
        raw["contamination"]["required_inputs"]["reference_panel"],
        "1000g-hg19"
    );
    assert_eq!(raw["contamination"]["warnings"][0]["code"], "LOW_COVERAGE");
    Ok(())
}
