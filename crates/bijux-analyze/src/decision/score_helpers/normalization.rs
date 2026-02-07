use crate::decision::score::RankInput;

pub(super) fn format_optional(value: Option<f64>) -> String {
    value.map_or_else(|| "NA".to_string(), |val| format!("{val:.3}"))
}

pub(super) fn min_max<I: Iterator<Item = f64>>(mut iter: I) -> (f64, f64) {
    let Some(first) = iter.next() else {
        return (0.0, 0.0);
    };
    let mut min_val = first;
    let mut max_val = first;
    for value in iter {
        if value < min_val {
            min_val = value;
        }
        if value > max_val {
            max_val = value;
        }
    }
    (min_val, max_val)
}

pub(super) fn normalize_inverted(value: f64, min_val: f64, max_val: f64) -> f64 {
    if (max_val - min_val).abs() < f64::EPSILON {
        return 1.0;
    }
    let norm = (value - min_val) / (max_val - min_val);
    1.0 - norm
}

pub(super) fn penalties_for_input(input: &RankInput) -> Vec<crate::decision::score::RankingPenalty> {
    let mut penalties = Vec::new();
    if input.runtime_s <= 0.0 {
        penalties.push(crate::decision::score::RankingPenalty {
            reason: "runtime_s missing or non-positive".to_string(),
            severity: "high".to_string(),
        });
    }
    if input.memory_mb <= 0.0 {
        penalties.push(crate::decision::score::RankingPenalty {
            reason: "memory_mb missing or non-positive".to_string(),
            severity: "medium".to_string(),
        });
    }
    if input.read_retention.is_none() {
        penalties.push(crate::decision::score::RankingPenalty {
            reason: "read_retention missing".to_string(),
            severity: "low".to_string(),
        });
    }
    penalties
}
