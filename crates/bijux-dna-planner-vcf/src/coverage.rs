use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_domain_vcf::taxonomy::CoverageRegime;

#[derive(Debug, Clone)]
pub(crate) struct CoverageThresholds {
    pub gl_max_depth: f64,
    pub pseudohaploid_max_depth: f64,
    pub diploid_min_depth: f64,
}

#[derive(Debug, serde::Deserialize)]
struct CoverageRegimesToml {
    decision: CoverageDecisionToml,
}

#[derive(Debug, serde::Deserialize)]
struct CoverageDecisionToml {
    coverage_regime: CoverageDecisionRuleToml,
}

#[derive(Debug, serde::Deserialize)]
struct CoverageDecisionRuleToml {
    thresholds: CoverageThresholdsToml,
    #[serde(default)]
    profiles: BTreeMap<String, CoverageThresholdsToml>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct CoverageThresholdsToml {
    gl_max_depth: f64,
    pseudohaploid_max_depth: f64,
    diploid_min_depth: f64,
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn load_coverage_thresholds(profile: Option<&str>) -> Result<CoverageThresholds> {
    let path = workspace_root().join("configs/runtime/coverage_regimes.toml");
    let raw = fs::read_to_string(&path)?;
    let parsed: CoverageRegimesToml = toml::from_str(&raw)?;
    let selected = profile
        .and_then(|name| parsed.decision.coverage_regime.profiles.get(name))
        .cloned()
        .unwrap_or(parsed.decision.coverage_regime.thresholds);
    Ok(CoverageThresholds {
        gl_max_depth: selected.gl_max_depth,
        pseudohaploid_max_depth: selected.pseudohaploid_max_depth,
        diploid_min_depth: selected.diploid_min_depth,
    })
}

pub(crate) fn classify_coverage_regime(
    requested: CoverageRegime,
    mean_depth_x: Option<f64>,
    profile: Option<&str>,
) -> Result<(CoverageRegime, String, CoverageThresholds)> {
    let thresholds = load_coverage_thresholds(profile)?;
    let Some(depth) = mean_depth_x else {
        return Ok((
            requested,
            "mean_depth_x absent; using caller-requested coverage_regime".to_string(),
            thresholds,
        ));
    };
    let resolved = if depth <= thresholds.gl_max_depth {
        CoverageRegime::LowCovGl
    } else if depth <= thresholds.pseudohaploid_max_depth {
        CoverageRegime::Pseudohaploid
    } else if depth >= thresholds.diploid_min_depth {
        CoverageRegime::Diploid
    } else {
        CoverageRegime::Pseudohaploid
    };
    Ok((
        resolved,
        format!(
            "classified from mean_depth_x={depth:.3} using thresholds gl<= {:.3}, pseudohaploid<= {:.3}, diploid>= {:.3}",
            thresholds.gl_max_depth, thresholds.pseudohaploid_max_depth, thresholds.diploid_min_depth
        ),
        thresholds,
    ))
}

pub(crate) fn damage_aware_policy_for_regime(regime: CoverageRegime) -> serde_json::Value {
    match regime {
        CoverageRegime::LowCovGl => serde_json::json!({
            "policy_id": "damage-aware-genotype-policy.v1",
            "mode": "ancient_dna_lowcov",
            "filtering": {"ct_ga_transition_bias_filter": true, "min_baseq": 20, "min_mapq": 30},
            "masking": {"terminal_damage_sites": true, "mask_fraction_ends": 0.02},
            "udg_thresholds": {"udg":"relaxed", "non_udg":"strict"},
        }),
        CoverageRegime::Pseudohaploid => serde_json::json!({
            "policy_id": "damage-aware-genotype-policy.v1",
            "mode": "pseudohaploid",
            "filtering": {"ct_ga_transition_bias_filter": true, "min_baseq": 20, "min_mapq": 30},
            "masking": {"terminal_damage_sites": true, "mask_fraction_ends": 0.02},
            "udg_thresholds": {"udg":"relaxed", "non_udg":"strict"},
        }),
        CoverageRegime::Diploid => serde_json::json!({
            "policy_id": "damage-aware-genotype-policy.v1",
            "mode": "diploid",
            "filtering": {"ct_ga_transition_bias_filter": false, "min_baseq": 20, "min_mapq": 30},
            "masking": {"terminal_damage_sites": false, "mask_fraction_ends": 0.0},
            "udg_thresholds": {"udg":"standard", "non_udg":"strict"},
        }),
    }
}
