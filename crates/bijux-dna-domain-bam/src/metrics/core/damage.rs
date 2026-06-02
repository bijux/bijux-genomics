use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DamageComparisonV1 {
    pub tool_a: String,
    pub tool_b: String,
    pub c_to_t_diff: f64,
    pub g_to_a_diff: f64,
    pub exceeds_threshold: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DamageMetricsV1 {
    pub c_to_t_5p: f64,
    pub g_to_a_3p: f64,
    pub pmd_score_histogram: Vec<(u8, u64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MisincorporationPointV1 {
    pub position: u32,
    pub c_to_t_rate: f64,
    pub g_to_a_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MisincorporationCurveSummaryV1 {
    pub five_prime: Vec<MisincorporationPointV1>,
    pub three_prime: Vec<MisincorporationPointV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PmdHistogramBinV1 {
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PmdScoreDistributionV1 {
    pub threshold: f64,
    pub bins: Vec<PmdHistogramBinV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DamageCoreFieldsV1 {
    pub tool: String,
    pub c_to_t_5p: f64,
    pub g_to_a_3p: f64,
    pub reads_considered: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DamageProfilerMetricsV1 {
    pub core: DamageCoreFieldsV1,
    pub misincorporation: MisincorporationCurveSummaryV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PmdtoolsMetricsV1 {
    pub core: DamageCoreFieldsV1,
    pub pmd_distribution: PmdScoreDistributionV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NgsBriggsMetricsV1 {
    pub core: DamageCoreFieldsV1,
    pub pmd_distribution: PmdScoreDistributionV1,
    pub lambda: f64,
    pub delta_s: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdDeamMetricsV1 {
    pub core: DamageCoreFieldsV1,
    pub pmd_distribution: PmdScoreDistributionV1,
    pub cluster_count: u32,
}

impl DamageMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self { c_to_t_5p: 0.0, g_to_a_3p: 0.0, pmd_score_histogram: Vec::new() }
    }
}

#[must_use]
pub fn compare_damage_metrics(
    tool_a: &str,
    metrics_a: &DamageMetricsV1,
    tool_b: &str,
    metrics_b: &DamageMetricsV1,
    threshold: f64,
) -> DamageComparisonV1 {
    let c_to_t_diff = (metrics_a.c_to_t_5p - metrics_b.c_to_t_5p).abs();
    let g_to_a_diff = (metrics_a.g_to_a_3p - metrics_b.g_to_a_3p).abs();
    let exceeds_threshold = c_to_t_diff > threshold || g_to_a_diff > threshold;
    DamageComparisonV1 {
        tool_a: tool_a.to_string(),
        tool_b: tool_b.to_string(),
        c_to_t_diff,
        g_to_a_diff,
        exceeds_threshold,
    }
}

/// # Errors
/// Returns an error if the `PyDamage` JSON cannot be read or parsed.
pub fn parse_pydamage_json(path: &std::path::Path) -> anyhow::Result<DamageMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read pydamage json")?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    Ok(parse_damage_metrics_json_value(&value))
}

/// # Errors
/// Returns an error if the `DamageProfiler` JSON cannot be read or parsed.
pub fn parse_damageprofiler_json(path: &std::path::Path) -> anyhow::Result<DamageMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read damageprofiler json")?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    Ok(parse_damage_metrics_json_value(&value))
}

/// # Errors
/// Returns an error if the `AdDeam` JSON cannot be read or parsed.
pub fn parse_addeam_json(path: &std::path::Path) -> anyhow::Result<DamageMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read addeam json")?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    Ok(parse_damage_metrics_json_value(&value))
}

/// # Errors
/// Returns an error if the `ngsBriggs` JSON cannot be read or parsed.
pub fn parse_ngsbriggs_json(path: &std::path::Path) -> anyhow::Result<DamageMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read ngsbriggs json")?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    Ok(parse_damage_metrics_json_value(&value))
}

/// # Errors
/// Returns an error if the `PMDtools` JSON cannot be read or parsed.
pub fn parse_pmdtools_json(path: &std::path::Path) -> anyhow::Result<DamageMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read pmdtools json")?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    Ok(parse_damage_metrics_json_value(&value))
}

/// # Errors
/// Returns an error if the `mapDamage2` misincorporation file cannot be read or parsed.
pub fn parse_mapdamage2_misincorporation(
    path: &std::path::Path,
) -> anyhow::Result<DamageMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read mapdamage2 misincorporation")?;
    let mut c_to_t = None;
    let mut g_to_a = None;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("pos") {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let c_to_t_val = parts[1].parse::<f64>().unwrap_or(0.0);
        let g_to_a_val = parts[2].parse::<f64>().unwrap_or(0.0);
        c_to_t = Some(c_to_t_val);
        g_to_a = Some(g_to_a_val);
        break;
    }
    Ok(DamageMetricsV1 {
        c_to_t_5p: c_to_t.unwrap_or(0.0),
        g_to_a_3p: g_to_a.unwrap_or(0.0),
        pmd_score_histogram: Vec::new(),
    })
}

fn parse_damage_metrics_json_value(value: &serde_json::Value) -> DamageMetricsV1 {
    let c_to_t = value
        .get("ct_5p")
        .or_else(|| value.get("c_to_t_5p"))
        .or_else(|| value.get("five_prime_c_to_t"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let g_to_a = value
        .get("ga_3p")
        .or_else(|| value.get("g_to_a_3p"))
        .or_else(|| value.get("three_prime_g_to_a"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let pmd_score_histogram = value
        .get("pmd_score_histogram")
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|entry| {
                    let pair = entry.as_array()?;
                    let score = pair.first()?.as_u64()?;
                    let count = pair.get(1)?.as_u64()?;
                    Some((score as u8, count))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    DamageMetricsV1 { c_to_t_5p: c_to_t, g_to_a_3p: g_to_a, pmd_score_histogram }
}
