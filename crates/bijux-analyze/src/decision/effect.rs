//! Owner: bijux-analyze
//! Effect size helpers for decision traces.

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct EffectThresholds {
    pub absolute: f64,
    pub relative: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EffectSize {
    pub absolute: f64,
    pub relative: Option<f64>,
    pub practical: bool,
}

#[must_use]
pub fn default_thresholds() -> EffectThresholds {
    EffectThresholds {
        absolute: 0.05,
        relative: 0.05,
    }
}

#[must_use]
pub fn effect_size(baseline: f64, candidate: f64, thresholds: EffectThresholds) -> EffectSize {
    let absolute = candidate - baseline;
    let relative = if baseline.abs() > f64::EPSILON {
        Some(absolute / baseline)
    } else {
        None
    };
    let practical = absolute.abs() >= thresholds.absolute
        || relative.is_some_and(|rel| rel.abs() >= thresholds.relative);
    EffectSize {
        absolute,
        relative,
        practical,
    }
}
