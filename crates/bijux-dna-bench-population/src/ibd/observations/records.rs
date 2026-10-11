//! Owner: bijux-dna-bench-population
//! Ibd observations records.
use crate::ibd::model::ParsedIbdSegment;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(crate) fn parse_ibd_filtered_segments(
    path: &Path,
    expected_samples: &BTreeSet<String>,
) -> Result<Vec<ParsedIbdSegment>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut seen_header = false;
    let mut rows = Vec::<ParsedIbdSegment>::new();
    for (index, line) in raw.lines().enumerate() {
        if line.starts_with('#') {
            continue;
        }
        if !seen_header {
            if line.trim() != "sample_a\tsample_b\tcontig\tstart\tend\tlength_cm\tmarker_count" {
                bail!("IBD filtered-segments header drifted in {}: `{line}`", path.display());
            }
            seen_header = true;
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 7 {
            bail!(
                "IBD filtered-segments row {} in {} must have 7 columns",
                index + 1,
                path.display()
            );
        }
        let sample_a = columns[0].trim().to_string();
        let sample_b = columns[1].trim().to_string();
        if !expected_samples.contains(&sample_a) || !expected_samples.contains(&sample_b) {
            bail!("IBD filtered-segments table reported unexpected sample pair `{sample_a}` / `{sample_b}`");
        }
        let contig = columns[2].trim().to_string();
        let start = columns[3]
            .trim()
            .parse::<u64>()
            .with_context(|| format!("parse start from row {} in {}", index + 1, path.display()))?;
        let end = columns[4]
            .trim()
            .parse::<u64>()
            .with_context(|| format!("parse end from row {} in {}", index + 1, path.display()))?;
        let length_cm = columns[5].trim().parse::<f64>().with_context(|| {
            format!("parse length_cm from row {} in {}", index + 1, path.display())
        })?;
        let marker_count = columns[6].trim().parse::<u64>().with_context(|| {
            format!("parse marker_count from row {} in {}", index + 1, path.display())
        })?;
        rows.push(ParsedIbdSegment {
            sample_a,
            sample_b,
            contig,
            start,
            end,
            length_cm,
            marker_count,
        });
    }
    Ok(rows)
}

pub(crate) fn compute_pair_overlap_marker_counts(
    input_vcf: &Path,
    expected_samples: &[String],
) -> Result<BTreeMap<(String, String), u64>> {
    let raw =
        fs::read_to_string(input_vcf).with_context(|| format!("read {}", input_vcf.display()))?;
    let sample_ids = raw
        .lines()
        .find(|line| line.starts_with("#CHROM\t"))
        .map(|line| line.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>())
        .ok_or_else(|| {
            anyhow!("IBD input VCF is missing sample header: {}", input_vcf.display())
        })?;
    if sample_ids != expected_samples {
        bail!(
            "IBD overlap-count derivation expected samples {expected_samples:?}, found {sample_ids:?}"
        );
    }
    let mut counts = BTreeMap::<(String, String), u64>::new();
    for line in raw.lines() {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        let Some(gt_index) = parse_format_index(&fields, "GT") else {
            continue;
        };
        for sample_a_index in 0..sample_ids.len() {
            for sample_b_index in (sample_a_index + 1)..sample_ids.len() {
                let sample_a = fields.get(9 + sample_a_index).copied().unwrap_or_default();
                let sample_b = fields.get(9 + sample_b_index).copied().unwrap_or_default();
                if !genotype_is_called(sample_a, gt_index)
                    || !genotype_is_called(sample_b, gt_index)
                {
                    continue;
                }
                let key = ordered_pair(&sample_ids[sample_a_index], &sample_ids[sample_b_index]);
                *counts.entry(key).or_insert(0) += 1;
            }
        }
    }
    Ok(counts)
}

pub(crate) fn genotype_is_called(sample_field: &str, gt_index: usize) -> bool {
    sample_field
        .split(':')
        .nth(gt_index)
        .is_some_and(|gt| !gt.trim().is_empty() && gt != "." && gt != "./." && gt != ".|.")
}

pub(crate) fn ordered_pair(sample_a: &str, sample_b: &str) -> (String, String) {
    if sample_a <= sample_b {
        (sample_a.to_string(), sample_b.to_string())
    } else {
        (sample_b.to_string(), sample_a.to_string())
    }
}

pub(crate) fn parse_record_fields(line: &str) -> Option<Vec<&str>> {
    if line.trim().is_empty() || line.starts_with('#') {
        None
    } else {
        Some(line.split('\t').collect())
    }
}

pub(crate) fn parse_format_index(fields: &[&str], name: &str) -> Option<usize> {
    fields.get(8)?.split(':').position(|token| token == name)
}
