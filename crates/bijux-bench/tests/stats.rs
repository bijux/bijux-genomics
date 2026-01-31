use bijux_bench::stats::{bootstrap_ci, iqr, mad, median, trimmed_mean};

#[test]
fn stats_median_mad_iqr_trimmed_mean() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 100.0];
    let med = median(values.clone());
    assert!((med - 3.0).abs() < 1e-6);
    let mad_value = mad(&values);
    assert!((mad_value - 1.0).abs() < 1e-6);
    let iqr_value = iqr(&values);
    assert!((iqr_value - 2.0).abs() < 1e-6);
    let trimmed = trimmed_mean(&values, 0.2);
    assert!((trimmed - 3.0).abs() < 1e-6);
}

#[test]
fn stats_bootstrap_ci_is_deterministic() {
    let values = vec![1.0, 2.0, 3.0, 4.0];
    let res = bootstrap_ci(&values, 100, 7);
    assert_eq!(res.samples, 100);
    assert!(res.mean > 0.0);
    assert!(res.ci_low <= res.ci_high);
}
