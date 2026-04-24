use super::{open_fastq_lines, Result};

pub(super) fn parse_primer_trimmed_fraction_from_stats(
    primer_stats: &std::path::Path,
) -> Option<f64> {
    let raw = std::fs::read_to_string(primer_stats).ok()?;
    let json = serde_json::from_str::<serde_json::Value>(&raw).ok()?;
    let read_counts = json.get("read_counts")?;
    let reads_in = read_counts.get("input")?.as_f64()?;
    if reads_in <= 0.0 {
        return Some(0.0);
    }
    let trimmed = read_counts
        .get("read1_with_adapter")
        .and_then(serde_json::Value::as_f64)
        .or_else(|| read_counts.get("with_adapter").and_then(serde_json::Value::as_f64))?;
    Some(trimmed / reads_in)
}

pub(super) fn parse_orientation_forward_fraction(
    orientation_report: &std::path::Path,
) -> Option<f64> {
    let raw = std::fs::read_to_string(orientation_report).ok()?;
    let mut total = 0_u64;
    let mut forward = 0_u64;
    for line in raw.lines().skip(1) {
        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() < 2 {
            continue;
        }
        let orientation = cols[0].trim();
        let count = cols[1].trim().parse::<u64>().ok()?;
        total = total.saturating_add(count);
        if orientation.eq_ignore_ascii_case("forward") {
            forward = forward.saturating_add(count);
        }
    }
    if total == 0 {
        return None;
    }
    Some(amplicon_u64_to_f64(forward) / amplicon_u64_to_f64(total))
}

pub(super) fn terminal_damage_reads_profiled(profile: &serde_json::Value) -> Option<u64> {
    profile.get("reads_profiled").and_then(serde_json::Value::as_u64)
}

pub(super) fn terminal_damage_asymmetry(profile: &serde_json::Value) -> Option<f64> {
    profile.get("ct_ga_asymmetry").and_then(serde_json::Value::as_f64)
}

pub(super) fn combined_terminal_damage_asymmetry(
    primary: &serde_json::Value,
    secondary: Option<&serde_json::Value>,
) -> Option<f64> {
    let primary_reads = terminal_damage_reads_profiled(primary)?;
    let primary_asymmetry = terminal_damage_asymmetry(primary)?;
    let secondary_reads = secondary.and_then(terminal_damage_reads_profiled).unwrap_or(0);
    let secondary_asymmetry = secondary.and_then(terminal_damage_asymmetry).unwrap_or(0.0);
    let total_reads = primary_reads + secondary_reads;
    if total_reads == 0 {
        return None;
    }
    Some(
        ((primary_asymmetry * amplicon_u64_to_f64(primary_reads))
            + (secondary_asymmetry * amplicon_u64_to_f64(secondary_reads)))
            / amplicon_u64_to_f64(total_reads),
    )
}

pub(super) fn terminal_damage_base_composition(
    profile: &serde_json::Value,
    key: &str,
) -> Option<std::collections::BTreeMap<String, u64>> {
    serde_json::from_value(profile.get(key)?.clone()).ok()
}

pub(super) fn count_fastq_reads(path: &std::path::Path) -> Result<u64> {
    let mut lines = open_fastq_lines(path)?;
    let mut reads = 0_u64;
    while let (Some(_h), Some(_seq), Some(_plus), Some(_qual)) =
        (lines.next(), lines.next(), lines.next(), lines.next())
    {
        reads += 1;
    }
    Ok(reads)
}

pub(super) fn parse_uchime_fraction(path: &std::path::Path) -> Option<f64> {
    let raw = std::fs::read_to_string(path).ok()?;
    let parsed_records = raw.lines().filter(|line| !line.trim().is_empty()).count() as u64;
    if parsed_records == 0 {
        return Some(0.0);
    }
    let flagged_records = raw
        .lines()
        .filter(|line| line.split('\t').next_back().is_some_and(|flag| flag == "Y"))
        .count() as u64;
    Some(amplicon_u64_to_f64(flagged_records) / amplicon_u64_to_f64(parsed_records))
}

pub(super) fn u64_to_u32(value: u64) -> Option<u32> {
    u32::try_from(value).ok()
}

pub(super) fn rounded_fraction_count(fraction: f64, total: u64) -> Option<u64> {
    if !fraction.is_finite() || fraction <= 0.0 {
        return Some(0);
    }
    let rounded = (fraction * amplicon_u64_to_f64(total)).round();
    if !rounded.is_finite() || rounded < 0.0 {
        return None;
    }
    format!("{rounded:.0}").parse::<u64>().ok()
}

pub(super) fn amplicon_u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}
