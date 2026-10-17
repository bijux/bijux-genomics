//! Deterministic kinship projection from an already acquired miniature alignment document.
use std::collections::HashMap;
use std::path::Path;

use crate::artifacts::alignment::{TinySamDocument, TinySamRecord};
use crate::artifacts::round_metric;
use crate::types::artifact_reports::{
    BamKinshipPairResultV1, BamKinshipSummaryV1, BAM_KINSHIP_SUMMARY_SCHEMA_VERSION,
};

#[derive(Clone, Copy)]
pub(in crate::artifacts) struct KinshipContext<'a> {
    pub(in crate::artifacts) input_bam: &'a Path,
    pub(in crate::artifacts) method: &'a str,
    pub(in crate::artifacts) reference_panel: &'a str,
    pub(in crate::artifacts) reference_build: &'a str,
    pub(in crate::artifacts) population_scope: &'a str,
    pub(in crate::artifacts) min_overlap_snps: u32,
    pub(in crate::artifacts) requires_cohort_context: bool,
}

// Paths and panel names are report metadata. Acquisition remains with the compatibility facade.
pub(in crate::artifacts) fn project_kinship(
    document: &TinySamDocument,
    context: KinshipContext<'_>,
) -> BamKinshipSummaryV1 {
    let KinshipContext {
        input_bam,
        method,
        reference_panel,
        reference_build,
        population_scope,
        min_overlap_snps,
        requires_cohort_context,
    } = context;
    let sample_site_calls = tiny_kinship_site_calls(document);
    let mut sample_ids = sample_site_calls.keys().cloned().collect::<Vec<_>>();
    sample_ids.sort();
    sample_ids.dedup();

    let mut observed_max_overlap_snps = 0_u32;
    let mut pairwise_results = Vec::new();
    for (left_index, sample_a) in sample_ids.iter().enumerate() {
        for sample_b in sample_ids.iter().skip(left_index + 1) {
            let Some(calls_a) = sample_site_calls.get(sample_a) else {
                continue;
            };
            let Some(calls_b) = sample_site_calls.get(sample_b) else {
                continue;
            };
            let (overlap_snps, result) = build_tiny_kinship_pair_result(
                sample_a,
                sample_b,
                calls_a,
                calls_b,
                min_overlap_snps,
            );
            observed_max_overlap_snps = observed_max_overlap_snps.max(overlap_snps);
            if let Some(result) = result {
                pairwise_results.push(result);
            }
        }
    }

    let insufficiency_reason = if pairwise_results.is_empty() {
        if sample_ids.len() < 2 {
            Some("insufficient_pair_manifest".to_string())
        } else if observed_max_overlap_snps < min_overlap_snps {
            Some("insufficient_overlap_snps".to_string())
        } else {
            Some("no_pairwise_results".to_string())
        }
    } else {
        None
    };

    BamKinshipSummaryV1 {
        schema_version: BAM_KINSHIP_SUMMARY_SCHEMA_VERSION.to_string(),
        stage_id: "bam.kinship".to_string(),
        method: method.to_string(),
        input_bam: input_bam.to_path_buf(),
        reference_panel: reference_panel.to_string(),
        reference_build: reference_build.to_string(),
        population_scope: population_scope.to_string(),
        min_overlap_snps,
        requires_cohort_context,
        sample_count: sample_ids.len() as u32,
        observed_max_overlap_snps,
        pair_count: pairwise_results.len() as u32,
        status: if pairwise_results.is_empty() {
            "insufficient".to_string()
        } else {
            "ok".to_string()
        },
        insufficiency_reason,
        pairwise_results,
    }
}

fn tiny_kinship_site_calls(
    document: &TinySamDocument,
) -> HashMap<String, HashMap<(String, u64), Option<char>>> {
    let mut calls = HashMap::<String, HashMap<(String, u64), Option<char>>>::new();
    for record in document
        .records
        .iter()
        .filter(|record| record.is_mapped() && record.pos > 0 && record.seq != "*")
    {
        let sample_id = kinship_sample_id_for_record(document, record);
        let sample_calls = calls.entry(sample_id).or_default();
        for (offset, base) in record.seq.chars().enumerate() {
            let normalized = normalize_called_base(base);
            let site = (record.rname.clone(), record.pos + offset as u64);
            match sample_calls.get(&site).copied() {
                Some(Some(existing)) if existing != normalized => {
                    sample_calls.insert(site, None);
                }
                Some(_) => {}
                None => {
                    sample_calls.insert(site, Some(normalized));
                }
            }
        }
    }
    calls
}

fn kinship_sample_id_for_record(document: &TinySamDocument, record: &TinySamRecord) -> String {
    record
        .read_group_id
        .as_ref()
        .and_then(|read_group_id| document.read_group_sample_map.get(read_group_id))
        .cloned()
        .or_else(|| document.read_group_samples.first().cloned())
        .unwrap_or_else(|| "unknown".to_string())
}

fn normalize_called_base(base: char) -> char {
    match base.to_ascii_uppercase() {
        'A' | 'C' | 'G' | 'T' => base.to_ascii_uppercase(),
        _ => 'N',
    }
}

fn build_tiny_kinship_pair_result(
    sample_a: &str,
    sample_b: &str,
    calls_a: &HashMap<(String, u64), Option<char>>,
    calls_b: &HashMap<(String, u64), Option<char>>,
    min_overlap_snps: u32,
) -> (u32, Option<BamKinshipPairResultV1>) {
    let mut overlap_snps = 0_u32;
    let mut matching_sites = 0_u32;
    for (site, base_a) in calls_a {
        let Some(base_a) = *base_a else {
            continue;
        };
        if base_a == 'N' {
            continue;
        }
        let Some(Some(base_b)) = calls_b.get(site).copied() else {
            continue;
        };
        if base_b == 'N' {
            continue;
        }
        overlap_snps = overlap_snps.saturating_add(1);
        if base_a == base_b {
            matching_sites = matching_sites.saturating_add(1);
        }
    }

    if overlap_snps < min_overlap_snps {
        return (overlap_snps, None);
    }

    let mismatch_sites = overlap_snps.saturating_sub(matching_sites);
    let concordance = round_metric(matching_sites as f64 / overlap_snps as f64);
    let kinship_coefficient = round_metric(concordance / 2.0);
    let relationship_label = classify_tiny_kinship_relationship(kinship_coefficient).to_string();

    (
        overlap_snps,
        Some(BamKinshipPairResultV1 {
            sample_a: sample_a.to_string(),
            sample_b: sample_b.to_string(),
            overlap_snps,
            matching_sites,
            mismatch_sites,
            concordance,
            kinship_coefficient,
            relationship_label,
        }),
    )
}

fn classify_tiny_kinship_relationship(kinship_coefficient: f64) -> &'static str {
    if kinship_coefficient >= 0.45 {
        "duplicate_or_monozygotic"
    } else if kinship_coefficient >= 0.177 {
        "first_degree"
    } else if kinship_coefficient >= 0.0884 {
        "second_degree"
    } else {
        "unrelated"
    }
}

#[cfg(test)]
#[path = "kinship/contracts.rs"]
mod contracts;
