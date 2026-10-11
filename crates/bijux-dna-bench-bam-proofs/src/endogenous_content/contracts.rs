pub(super) fn float_matches(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}
