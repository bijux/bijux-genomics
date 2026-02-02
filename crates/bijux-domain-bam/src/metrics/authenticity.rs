use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::metrics::BamMetricsV1;
use crate::types::LibraryType;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthenticityEvidenceV1 {
    pub damage_high: bool,
    pub fragments_short: bool,
    pub mapq_low_with_damage: bool,
}

impl AuthenticityEvidenceV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            damage_high: false,
            fragments_short: false,
            mapq_low_with_damage: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LibraryTypeInferenceV1 {
    pub inferred: LibraryType,
    pub confidence: f64,
    pub rationale: String,
    #[serde(default)]
    pub declared: Option<LibraryType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrimSuggestionV1 {
    pub trim_5p: u8,
    pub trim_3p: u8,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthenticityScoreV1 {
    pub score: f64,
    pub confidence: f64,
    pub evidence: AuthenticityEvidenceV1,
    #[serde(default)]
    pub library_type_inference: Option<LibraryTypeInferenceV1>,
    #[serde(default)]
    pub trim_suggestion: Option<TrimSuggestionV1>,
    #[serde(default)]
    pub rationale: Vec<String>,
}

impl AuthenticityScoreV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            score: 0.0,
            confidence: 0.0,
            evidence: AuthenticityEvidenceV1::empty(),
            library_type_inference: None,
            trim_suggestion: None,
            rationale: Vec::new(),
        }
    }
}

impl Default for AuthenticityScoreV1 {
    fn default() -> Self {
        Self::empty()
    }
}

#[must_use]
pub fn infer_library_type_from_damage(damage_5p: f64, damage_3p: f64) -> LibraryTypeInferenceV1 {
    let damage = damage_5p.max(damage_3p);
    let (inferred, confidence, rationale) = if damage >= 0.20 {
        (
            LibraryType::NonUdg,
            0.85,
            "high terminal damage signal".to_string(),
        )
    } else if damage >= 0.10 {
        (
            LibraryType::HalfUdg,
            0.65,
            "moderate terminal damage signal".to_string(),
        )
    } else {
        (
            LibraryType::Udg,
            0.55,
            "low terminal damage signal".to_string(),
        )
    };
    LibraryTypeInferenceV1 {
        inferred,
        confidence,
        rationale,
        declared: None,
    }
}

#[must_use]
pub fn suggest_trim_from_damage(damage_5p: f64, damage_3p: f64) -> Option<TrimSuggestionV1> {
    let damage = damage_5p.max(damage_3p);
    if damage < 0.05 {
        return None;
    }
    let trim = if damage >= 0.20 {
        5
    } else if damage >= 0.10 {
        3
    } else {
        2
    };
    Some(TrimSuggestionV1 {
        trim_5p: trim,
        trim_3p: trim,
        rationale: format!("terminal damage {damage:.2} suggests trim of {trim}bp"),
    })
}

#[must_use]
pub fn authenticity_score(metrics: &BamMetricsV1) -> AuthenticityScoreV1 {
    let damage = metrics.damage.c_to_t_5p.max(metrics.damage.g_to_a_3p);
    let damage_score = (damage / 0.3).min(1.0);
    let short_score = (metrics.fragment_length.short_fraction / 0.5).min(1.0);
    let mapq_low = metrics.mapq.mean <= 30.0;
    let mapq_damage = if mapq_low && damage >= 0.10 { 1.0 } else { 0.0 };
    let score = (damage_score + short_score + mapq_damage) / 3.0;

    let evidence = AuthenticityEvidenceV1 {
        damage_high: damage >= 0.10,
        fragments_short: metrics.fragment_length.short_fraction >= 0.20,
        mapq_low_with_damage: mapq_low && damage >= 0.10,
    };

    let library_type_inference = Some(infer_library_type_from_damage(
        metrics.damage.c_to_t_5p,
        metrics.damage.g_to_a_3p,
    ));
    let trim_suggestion =
        suggest_trim_from_damage(metrics.damage.c_to_t_5p, metrics.damage.g_to_a_3p);

    let mut rationale = Vec::new();
    if evidence.damage_high {
        rationale.push("terminal damage supports authenticity".to_string());
    }
    if evidence.fragments_short {
        rationale.push("short fragment profile supports authenticity".to_string());
    }
    if !evidence.damage_high && metrics.mapq.mean >= 40.0 {
        rationale.push("high MAPQ with low damage suggests modern contamination".to_string());
    }

    AuthenticityScoreV1 {
        score,
        confidence: 0.6 + 0.4 * score,
        evidence,
        library_type_inference,
        trim_suggestion,
        rationale,
    }
}

#[must_use]
pub fn contamination_cross_check(damage: f64, contamination: f64) -> String {
    if contamination >= 0.10 && damage >= 0.10 {
        "high contamination but strong damage signal suggests endogenous aDNA".to_string()
    } else if contamination >= 0.10 && damage < 0.05 {
        "high contamination with weak damage suggests modern contamination".to_string()
    } else {
        "contamination and damage are consistent".to_string()
    }
}
