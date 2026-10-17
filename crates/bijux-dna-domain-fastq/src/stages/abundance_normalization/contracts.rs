#![cfg(test)]

use super::{
    normalize_abundance_table_text, observe_normalized_abundance_table_text,
    AbundanceTableObservation,
};
use crate::params::edna::AbundanceNormalizationEffectiveParams;
use anyhow::{anyhow, Result};

fn params() -> AbundanceNormalizationEffectiveParams {
    AbundanceNormalizationEffectiveParams {
        schema_version: "bijux.params.edna.v1".to_string(),
        method: "relative_abundance".to_string(),
        expected_columns: vec!["sample_id".into(), "feature_id".into(), "abundance".into()],
        input_value_column: "abundance".into(),
        normalized_value_column: "normalized_abundance".into(),
        compositional_rule: "per_sample_sum_to_one".into(),
        scale_factor: None,
        report_artifact: "report_json".into(),
    }
}

#[test]
fn relative_units_preserve_duplicate_rows_sorted_samples_and_six_decimal_bytes() -> Result<()> {
    let normalized = normalize_abundance_table_text(
        "sample_id\tfeature_id\tabundance\nb\tx\t2\na\tf1\t1\na\tf2\t2\nb\tx\t3\n",
        &params(),
    )?;
    assert_eq!(
        normalized.table_tsv,
        "sample_id\tfeature_id\tnormalized_abundance\na\tf1\t0.333333\na\tf2\t0.666667\nb\tx\t0.400000\nb\tx\t0.600000\n"
    );
    assert_eq!(normalized.observation.table_rows, 4);
    assert_eq!(normalized.observation.feature_count, 3);
    assert_eq!(normalized.observation.sample_count, 2);
    assert_eq!(normalized.observation.per_sample_sums, vec![("a".into(), 1.0), ("b".into(), 1.0)]);
    Ok(())
}

#[test]
fn declared_scale_and_output_column_control_units_while_method_remains_metadata() -> Result<()> {
    let mut effective = params();
    effective.method = "authored_method_label".into();
    effective.scale_factor = Some(1_000_000.0);
    effective.normalized_value_column = "counts_per_million".into();
    let normalized = normalize_abundance_table_text(
        "sample_id\tfeature_id\tabundance\ns\tf1\t25\ns\tf2\t75\n",
        &effective,
    )?;
    assert_eq!(
        normalized.table_tsv,
        "sample_id\tfeature_id\tcounts_per_million\ns\tf1\t250000.000000\ns\tf2\t750000.000000\n"
    );
    assert_eq!(normalized.observation.per_sample_sums, vec![("s".into(), 1_000_000.0)]);
    assert_eq!(effective.method, "authored_method_label");
    Ok(())
}

#[test]
fn malformed_cells_short_rows_and_nonpositive_totals_retain_zero_semantics() -> Result<()> {
    let normalized = normalize_abundance_table_text(
        "sample_id\tfeature_id\tabundance\n z \t n \t -2 \nb\tz\t0\na\tbad\tnot-a-number\na\tvalid\t2\na\tvalid\t0\nshort\trow\n\n",
        &params(),
    )?;
    assert_eq!(
        normalized.table_tsv,
        "sample_id\tfeature_id\tnormalized_abundance\na\tbad\t0.000000\na\tvalid\t1.000000\na\tvalid\t0.000000\nb\tz\t0.000000\nz\tn\t0.000000\n"
    );
    assert_eq!(normalized.observation.table_rows, 5);
    assert_eq!(normalized.observation.feature_count, 4);
    assert_eq!(normalized.observation.sample_count, 3);
    assert!((normalized.observation.zero_fraction - 0.8).abs() < 1e-12);
    assert_eq!(
        normalized.observation.per_sample_sums,
        vec![("a".into(), 1.0), ("b".into(), 0.0), ("z".into(), 0.0)]
    );
    Ok(())
}

#[test]
fn header_failures_keep_input_and_observation_diagnostics_distinct() -> Result<()> {
    let effective = params();
    let input_error = normalize_abundance_table_text("wrong\tcolumns\n", &effective)
        .err()
        .ok_or_else(|| anyhow!("input header must fail"))?;
    assert_eq!(input_error.to_string(), "abundance table header mismatch: expected `sample_id\tfeature_id\tabundance`, got `wrong\tcolumns`");
    let observation_error = observe_normalized_abundance_table_text("wrong\tcolumns\n", &effective)
        .err()
        .ok_or_else(|| anyhow!("observation header must fail"))?;
    assert_eq!(observation_error.to_string(), "normalized abundance table header mismatch: expected `sample_id\tfeature_id\tnormalized_abundance`, got `wrong\tcolumns`");
    Ok(())
}

#[test]
fn observations_count_duplicate_rows_and_invalid_cells_without_rewriting_values() -> Result<()> {
    let observation = observe_normalized_abundance_table_text(
        "sample_id\tfeature_id\tnormalized_abundance\nb\tx\t0\na\tx\t0.2\na\tx\t0.3\na\tbad\tinvalid\nshort\trow\n",
        &params(),
    )?;
    assert_eq!(observation.table_rows, 4);
    assert_eq!(observation.sample_count, 2);
    assert_eq!(observation.feature_count, 2);
    assert!((observation.zero_fraction - 0.5).abs() < 1e-12);
    assert_eq!(observation.per_sample_sums, vec![("a".into(), 0.5), ("b".into(), 0.0)]);
    Ok(())
}

#[test]
fn empty_and_header_only_tables_retain_empty_observations() -> Result<()> {
    let empty = AbundanceTableObservation {
        table_rows: 0,
        sample_count: 0,
        feature_count: 0,
        zero_fraction: 0.0,
        per_sample_sums: vec![],
    };
    let effective = params();
    for input in ["", "sample_id\tfeature_id\tabundance\n"] {
        let normalized = normalize_abundance_table_text(input, &effective)?;
        assert_eq!(normalized.table_tsv, "sample_id\tfeature_id\tnormalized_abundance\n");
        assert_eq!(normalized.observation, empty);
    }
    for input in ["", "sample_id\tfeature_id\tnormalized_abundance\n"] {
        assert_eq!(observe_normalized_abundance_table_text(input, &effective)?, empty);
    }
    Ok(())
}

#[test]
fn nonfinite_cells_remain_observations_for_caller_composition_admission() -> Result<()> {
    let effective = params();
    let normalized = normalize_abundance_table_text(
        "sample_id\tfeature_id\tabundance\na\tf\tNaN\na\tg\t1\nb\tf\tinf\n",
        &effective,
    )?;
    assert_eq!(
        normalized.table_tsv,
        "sample_id\tfeature_id\tnormalized_abundance\na\tf\t0.000000\na\tg\t0.000000\nb\tf\tNaN\n"
    );
    assert!(normalized.observation.per_sample_sums[1].1.is_nan());
    let observed = observe_normalized_abundance_table_text(
        "sample_id\tfeature_id\tnormalized_abundance\na\tf\tNaN\nb\tf\tinf\n",
        &effective,
    )?;
    assert!(observed.per_sample_sums[0].1.is_nan());
    assert!(observed.per_sample_sums[1].1.is_infinite());
    Ok(())
}
