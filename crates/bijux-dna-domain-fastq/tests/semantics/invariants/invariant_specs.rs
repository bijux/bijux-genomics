#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::map_unwrap_or,
    clippy::unnecessary_wraps,
    clippy::unwrap_used
)]

use anyhow::Result;
use bijux_dna_core::{ids::StageId, prelude::InvariantStatusV1};
use bijux_dna_domain_fastq::invariants::{evaluate_invariants, thresholds_from_env};
use bijux_dna_domain_fastq::metrics::{
    FastqDeltaMetricsV1, FastqFilterMetricsV1, FastqMergeMetricsV1, FastqTrimMetricsV1,
    FastqValidateMetricsV1, RetentionReportMetricV1,
};
use bijux_dna_domain_fastq::params::filter::FilterEffectiveParams;
use bijux_dna_domain_fastq::params::merge::{
    MergeEffectiveParams, MergeEngine, UnmergedReadPolicy, MERGE_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::trim::TrimEffectiveParams;
use bijux_dna_domain_fastq::params::validate::{
    PairSyncPolicy, ValidateEffectiveParams, ValidationMode, VALIDATE_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::{fastq_invariant_specs, PairedMode, EVALUATED_STAGES};

fn retention_metric() -> RetentionReportMetricV1 {
    RetentionReportMetricV1 {
        value: 0.9,
        numerator_reads: 90,
        denominator_reads: 100,
        numerator_bases: 900,
        denominator_bases: 1000,
        definition: "reads_out/reads_in".to_string(),
        stage_boundary: "stage".to_string(),
        conditions: serde_json::json!({}),
    }
}

fn trim_metrics(read_retention: f64, mean_q_delta: f64) -> FastqTrimMetricsV1 {
    FastqTrimMetricsV1 {
        reads_in: 100,
        reads_out: 90,
        bases_in: 1000,
        bases_out: 900,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 30.0 + mean_q_delta,
        delta_metrics: FastqDeltaMetricsV1 {
            read_retention,
            base_retention: 0.9,
            mean_q_delta,
            gc_delta: 0.0,
        },
        paired_mode: None,
        adapter_policy: None,
        polyx_policy: None,
        n_policy: None,
        contaminant_policy: None,
        raw_backend_report_format: None,
        retention: retention_metric(),
    }
}

fn filter_metrics(n_rate: f64) -> FastqFilterMetricsV1 {
    let reads_in = 100_u64;
    let reads_removed_by_n = (n_rate * reads_in as f64).round() as u64;
    FastqFilterMetricsV1 {
        reads_in,
        reads_out: reads_in.saturating_sub(reads_removed_by_n),
        reads_dropped: reads_removed_by_n,
        reads_removed_by_n,
        reads_removed_by_entropy: 0,
        reads_removed_low_complexity: 0,
        reads_removed_by_kmer: 0,
        reads_removed_contaminant_kmer: 0,
        reads_removed_by_length: 0,
        bases_in: 1000,
        bases_out: 900,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: FastqDeltaMetricsV1 {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.0,
        },
        retention: retention_metric(),
    }
}

fn merge_metrics(merge_rate: f64) -> FastqMergeMetricsV1 {
    FastqMergeMetricsV1 {
        reads_in: 100,
        reads_out: 100,
        bases_in: 1000,
        bases_out: 1000,
        pairs_in: Some(50),
        pairs_out: Some(50),
        reads_r1: 50,
        reads_r2: 50,
        reads_merged: 60,
        reads_unmerged: 40,
        reads_discarded: 0,
        input_pair_count: 50,
        merged_pair_count: 50,
        unmerged_pair_count: 0,
        discarded_pair_count: 0,
        merge_rate,
        merge_q_delta: 0.0,
    }
}

fn validate_metrics(reads_invalid: u64) -> FastqValidateMetricsV1 {
    FastqValidateMetricsV1 {
        reads_in: 100,
        reads_out: 100,
        bases_in: 1000,
        bases_out: 1000,
        pairs_in: None,
        pairs_out: None,
        reads_total: 100,
        reads_valid: 100 - reads_invalid,
        reads_invalid,
        mean_q: 30.0,
        validated_inputs: None,
        validated_pairs: None,
        pair_sync_checked: None,
        pair_sync_pass: None,
        pair_count_match: None,
        strict_pass: None,
        failure_class: None,
    }
}

fn effective_params_trim() -> serde_json::Value {
    serde_json::to_value(TrimEffectiveParams {
        paired_mode: PairedMode::SingleEnd,
        threads: 2,
        min_len: 30,
        q_cutoff: Some(20),
        adapter_policy: "auto".to_string(),
        damage_mode: None,
        polyx_policy: None,
        n_policy: None,
        contaminant_policy: None,
    })
    .unwrap()
}

fn effective_params_filter() -> serde_json::Value {
    serde_json::to_value(FilterEffectiveParams {
        paired_mode: PairedMode::SingleEnd,
        threads: 2,
        max_n: Some(1),
        max_n_fraction: None,
        max_n_count: None,
        low_complexity_threshold: None,
        entropy_threshold: None,
        contaminant_db: None,
        n_policy: None,
        polyx_policy: None,
        damage_mode: None,
    })
    .unwrap()
}

fn effective_params_merge() -> serde_json::Value {
    serde_json::to_value(MergeEffectiveParams {
        schema_version: MERGE_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::PairedEnd,
        threads: 2,
        merge_overlap: Some(10),
        min_len: Some(30),
        merge_engine: MergeEngine::Pear,
        unmerged_read_policy: UnmergedReadPolicy::EmitUnmergedPairs,
    })
    .unwrap()
}

fn effective_params_validate() -> serde_json::Value {
    serde_json::to_value(ValidateEffectiveParams {
        schema_version: VALIDATE_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::SingleEnd,
        threads: 2,
        validation_mode: ValidationMode::Strict,
        pair_sync_policy: PairSyncPolicy::NotApplicable,
    })
    .unwrap()
}

fn fixture_for_invariant(id: &str) -> (String, serde_json::Value, serde_json::Value) {
    match id {
        "effective_params_present" => (
            "fastq.trim_reads".to_string(),
            serde_json::to_value(trim_metrics(0.9, 1.0)).unwrap(),
            serde_json::json!({}),
        ),
        "metrics_parse" => {
            ("fastq.trim_reads".to_string(), serde_json::json!({}), effective_params_trim())
        }
        "retention_sanity" => (
            "fastq.trim_reads".to_string(),
            serde_json::to_value(trim_metrics(0.1, 1.0)).unwrap(),
            effective_params_trim(),
        ),
        "quality_direction" => (
            "fastq.trim_reads".to_string(),
            serde_json::to_value(trim_metrics(0.9, -10.0)).unwrap(),
            effective_params_trim(),
        ),
        "merge_rate_range" => (
            "fastq.merge_pairs".to_string(),
            serde_json::to_value(merge_metrics(1.5)).unwrap(),
            effective_params_merge(),
        ),
        "n_rate_sanity" => (
            "fastq.filter_reads".to_string(),
            serde_json::to_value(filter_metrics(0.1)).unwrap(),
            effective_params_filter(),
        ),
        "validate_malformed_reads" => (
            "fastq.validate_reads".to_string(),
            serde_json::to_value(validate_metrics(1)).unwrap(),
            effective_params_validate(),
        ),
        "validate_pair_integrity" => (
            "fastq.validate_reads".to_string(),
            serde_json::json!({
                "reads_in": 100,
                "reads_out": 100,
                "bases_in": 1000,
                "bases_out": 1000,
                "pairs_in": 50,
                "pairs_out": 50,
                "reads_total": 100,
                "reads_valid": 100,
                "reads_invalid": 0,
                "mean_q": 30.0,
                "validated_inputs": 2,
                "validated_pairs": 49,
                "pair_sync_checked": true,
                "pair_sync_pass": false,
                "pair_count_match": false,
                "strict_pass": false,
                "failure_class": "header_sync_mismatch"
            }),
            effective_params_validate(),
        ),
        "validate_strict_outcome" => (
            "fastq.validate_reads".to_string(),
            serde_json::json!({
                "reads_in": 100,
                "reads_out": 100,
                "bases_in": 1000,
                "bases_out": 1000,
                "pairs_in": null,
                "pairs_out": null,
                "reads_total": 100,
                "reads_valid": 100,
                "reads_invalid": 0,
                "mean_q": 30.0,
                "validated_inputs": 1,
                "validated_pairs": null,
                "pair_sync_checked": false,
                "pair_sync_pass": null,
                "pair_count_match": null,
                "strict_pass": false,
                "failure_class": "validator_error"
            }),
            effective_params_validate(),
        ),
        _ => panic!("missing fixture for invariant {id}"),
    }
}

#[test]
fn fastq_invariants_have_specs_and_fixtures() -> Result<()> {
    let specs = fastq_invariant_specs();
    let mut ids = std::collections::BTreeSet::new();
    for spec in &specs {
        assert!(ids.insert(spec.id.clone()), "duplicate invariant {}", spec.id);
        assert!(!spec.definition.trim().is_empty());
        assert!(!spec.threshold_provenance.trim().is_empty());
        assert!(!spec.next_steps.trim().is_empty());
        let (stage_id, metrics, params) = fixture_for_invariant(&spec.id);
        let stage_id = StageId::new(stage_id);
        let eval = evaluate_invariants(&stage_id, &metrics, &params, &thresholds_from_env());
        let status = eval
            .results
            .iter()
            .find(|entry| entry.id == spec.id)
            .map(|entry| entry.status.clone())
            .unwrap_or_else(|| panic!("missing invariant {} for {stage_id}", spec.id));
        assert!(status != InvariantStatusV1::Pass, "fixture did not trigger {}", spec.id);
    }
    Ok(())
}

#[test]
fn evaluated_stage_set_matches_fastq_invariant_scope() {
    let stages = EVALUATED_STAGES.iter().map(ToString::to_string).collect::<Vec<_>>();
    assert_eq!(
        stages,
        vec!["fastq.validate_reads", "fastq.trim_reads", "fastq.merge_pairs", "fastq.filter_reads",],
        "FASTQ invariant evaluation should list only stages with concrete evaluator logic"
    );
}
