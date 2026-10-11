use super::*;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, StageId, StageVersion, ToolConstraints, ToolId,
};
use bijux_dna_stage_contract::{StageIO, StagePlanV1};
use serde_json::{json, Value};

fn plan(stage: &str, params: Value) -> StagePlanV1 {
    StagePlanV1 {
        stage_id: StageId::new(stage.to_string()),
        stage_instance_id: None,
        stage_version: StageVersion(1),
        tool_id: ToolId::from_static("fixture"),
        tool_version: "fixture".into(),
        image: ContainerImageRefV1 { image: "fixture".into(), digest: None },
        command: CommandSpecV1 { template: vec!["must-not-execute".into()] },
        resources: ToolConstraints { runtime: "native".into(), mem_gb: 1, tmp_gb: 1, threads: 1 },
        io: StageIO { inputs: vec![], outputs: vec![] },
        out_dir: "artifacts/benchmark-smoke/readiness/parameters".into(),
        params,
        effective_params: Value::Null,
        operating_mode: bijux_dna_core::contract::StageOperatingMode::default(),
        aux_images: std::collections::BTreeMap::default(),
        canonical_contract: None,
        provenance: None,
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    }
}

#[test]
fn benchmark_parameter_decoding_preserves_explicit_values_and_nullable_defaults() -> Result<()> {
    let source = plan(
        "fastq.correct_errors",
        json!({"quality_encoding":"phred64","threads":4,"genome_size":8_000_000,"conservative_mode":true}),
    );
    let Some(FastqStageParameters::CorrectErrors(decoded)) =
        decode_local_benchmark_stage_parameters("fastq.correct_errors", &source)?
    else {
        anyhow::bail!("correction parameters absent");
    };
    assert_eq!(decoded.threads, Some(4));
    assert_eq!(decoded.genome_size, Some(8_000_000));
    assert!(decoded.conservative_mode);
    assert_eq!(
        decoded.quality_encoding,
        bijux_dna_domain_fastq::params::correct::QualityEncoding::Phred64
    );
    for raw in [Value::Null, json!({}), json!({"threads":null,"genome_size":null})] {
        let Some(FastqStageParameters::CorrectErrors(decoded)) =
            decode_local_benchmark_stage_parameters(
                "fastq.correct_errors",
                &plan("fastq.correct_errors", raw),
            )?
        else {
            anyhow::bail!("correction defaults absent");
        };
        assert_eq!(decoded, crate::CorrectErrorsStageParams::baseline());
    }
    Ok(())
}

#[test]
fn benchmark_parameter_decoding_rejects_invalid_scalars_and_misbound_stage_identity() {
    for raw in [
        json!([]),
        json!({"threads":"4"}),
        json!({"threads":-1}),
        json!({"threads":4_294_967_296_u64}),
        json!({"genome_size":1.5}),
        json!({"conservative_mode":"true"}),
        json!({"quality_encoding":42}),
        json!({"quality_encoding":"unsupported"}),
        json!({"trusted_kmer_artifact":[]}),
    ] {
        assert!(decode_local_benchmark_stage_parameters(
            "fastq.correct_errors",
            &plan("fastq.correct_errors", raw)
        )
        .is_err());
    }
    assert!(decode_local_benchmark_stage_parameters(
        "fastq.merge_pairs",
        &plan("fastq.correct_errors", json!({}))
    )
    .is_err());
    assert!(decode_local_benchmark_stage_parameters(
        "fastq.unknown",
        &plan("fastq.unknown", json!({}))
    )
    .is_err());
    assert!(decode_local_benchmark_stage_parameters(
        "fastq.trim_terminal_damage",
        &plan("fastq.trim_terminal_damage", json!({"damage_mode":"unsupported"}))
    )
    .is_err());
}

#[test]
fn benchmark_parameter_decoding_observes_typed_effective_report_parameters() -> Result<()> {
    use bijux_dna_domain_fastq::params::{
        qc_post::{
            QcAggregationEngine, QcAggregationScope, QcPostEffectiveParams,
            REPORT_QC_SCHEMA_VERSION,
        },
        PairedMode,
    };
    let effective = QcPostEffectiveParams {
        schema_version: REPORT_QC_SCHEMA_VERSION.into(),
        paired_mode: PairedMode::SingleEnd,
        aggregation_engine: QcAggregationEngine::Multiqc,
        aggregation_scope: QcAggregationScope::GovernedQcArtifacts,
    };
    let mut source = plan("fastq.report_qc", Value::Null);
    source.effective_params = serde_json::to_value(&effective)?;
    let Some(FastqStageParameters::ReportQc(decoded)) =
        decode_local_benchmark_stage_parameters("fastq.report_qc", &source)?
    else {
        anyhow::bail!("report parameters absent");
    };
    assert_eq!(serde_json::to_value(decoded)?, source.effective_params);
    source.effective_params["paired_mode"] = json!("unsupported");
    assert!(decode_local_benchmark_stage_parameters("fastq.report_qc", &source).is_err());
    Ok(())
}

#[test]
fn benchmark_parameter_decoding_reuses_domain_enum_wire_contracts() -> Result<()> {
    for (stage, key, field, values) in [
        (
            "fastq.correct_errors",
            "quality_encoding",
            "FASTQ quality_encoding",
            ["phred33", "phred64"],
        ),
        (
            "fastq.merge_pairs",
            "unmerged_read_policy",
            "fastq.merge_pairs unmerged_read_policy",
            ["emit_unmerged_pairs", "omit_unmerged_pairs"],
        ),
        (
            "fastq.trim_terminal_damage",
            "damage_mode",
            "fastq.trim_terminal_damage damage_mode",
            ["ancient", "udg_trimmed"],
        ),
        (
            "fastq.trim_terminal_damage",
            "execution_policy",
            "fastq.trim_terminal_damage execution_policy",
            ["explicit_terminal_trim", "preserve_udg_trimmed_ends"],
        ),
    ] {
        for value in values {
            let mut raw = json!({});
            raw[key] = json!(value);
            let decoded = decode_local_benchmark_stage_parameters(stage, &plan(stage, raw))?
                .ok_or_else(|| anyhow::anyhow!("typed stage parameters absent"))?;
            let observed = match decoded {
                FastqStageParameters::CorrectErrors(params) => {
                    serde_json::to_value(params.quality_encoding)?
                }
                FastqStageParameters::MergePairs(params) => {
                    serde_json::to_value(params.unmerged_read_policy)?
                }
                FastqStageParameters::TrimTerminalDamage(params) if key == "damage_mode" => {
                    serde_json::to_value(params.damage_mode)?
                }
                FastqStageParameters::TrimTerminalDamage(params) => {
                    serde_json::to_value(params.execution_policy)?
                }
                _ => anyhow::bail!("unexpected typed stage parameters"),
            };
            assert_eq!(observed, json!(value));
        }
        for value in ["", "unsupported", " phred33 "] {
            let mut raw = json!({});
            raw[key] = json!(value);
            let error = decode_local_benchmark_stage_parameters(stage, &plan(stage, raw))
                .err()
                .ok_or_else(|| anyhow::anyhow!("unknown domain enum was accepted"))?;
            assert_eq!(error.to_string(), format!("unsupported {field} `{value}`"));
        }
    }
    Ok(())
}
