pub fn median(mut values: Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        values[mid - 1].midpoint(values[mid])
    } else {
        values[mid]
    }
}

#[allow(clippy::cast_precision_loss)]
pub fn u64_to_f64(value: u64) -> f64 {
    value as f64
}

pub fn ratio_u64(num: u64, denom: u64) -> f64 {
    if denom == 0 {
        0.0
    } else {
        u64_to_f64(num) / u64_to_f64(denom)
    }
}
