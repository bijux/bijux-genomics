//! Authenticity modeling and inference helpers for BAM metrics.

use crate::metrics::{
    AuthenticityEvidenceV1, AuthenticityScoreV1, BamMetricsV1, LibraryTypeInferenceV1,
    TrimSuggestionV1,
};
use crate::types::LibraryType;

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
