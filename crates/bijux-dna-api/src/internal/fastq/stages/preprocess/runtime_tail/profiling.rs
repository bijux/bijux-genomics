use super::{open_fastq_lines, Result};

pub(crate) fn infer_udg_classification(input: &std::path::Path) -> String {
    if let Ok(configured) = std::env::var("BIJUX_UDG_CLASSIFICATION") {
        let normalized = configured.trim().to_ascii_lowercase();
        if matches!(normalized.as_str(), "udg" | "partial" | "non_udg") {
            return normalized;
        }
    }
    let stem = input
        .file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if stem.contains("partial_udg") || stem.contains("partial-udg") {
        "partial".to_string()
    } else if stem.contains("udg") {
        "udg".to_string()
    } else {
        "non_udg".to_string()
    }
}

pub(crate) fn terminal_damage_profile(path: &std::path::Path) -> Result<serde_json::Value> {
    let mut ct_events = 0_u64;
    let mut ga_events = 0_u64;
    let mut seen = 0_u64;
    let mut five_prime: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut three_prime: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    let mut lines = open_fastq_lines(path)?;
    while let (Some(_h), Some(seq), Some(_plus), Some(_qual)) =
        (lines.next(), lines.next(), lines.next(), lines.next())
    {
        let seq = seq.trim().to_ascii_uppercase();
        if seq.len() < 2 {
            continue;
        }
        let first = seq.chars().next().unwrap_or('N');
        let last = seq.chars().next_back().unwrap_or('N');
        *five_prime.entry(first.to_string()).or_insert(0) += 1;
        *three_prime.entry(last.to_string()).or_insert(0) += 1;
        if seq.starts_with("CT") {
            ct_events += 1;
        }
        if seq.ends_with("GA") {
            ga_events += 1;
        }
        seen += 1;
        if seen >= 200_000 {
            break;
        }
    }
    let denom = (ct_events + ga_events)
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0);
    let asymmetry = if denom > 0.0 {
        (ct_events.to_string().parse::<f64>().unwrap_or(0.0)
            - ga_events.to_string().parse::<f64>().unwrap_or(0.0))
            / denom
    } else {
        0.0
    };
    Ok(serde_json::json!({
        "reads_profiled": seen,
        "terminal_base_composition_5p": five_prime,
        "terminal_base_composition_3p": three_prime,
        "ct_events": ct_events,
        "ga_events": ga_events,
        "ct_ga_asymmetry": asymmetry,
    }))
}
