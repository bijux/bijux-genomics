//! Reference-length and read-group/sample preservation for miniature alignment transformations.
use super::TinySamDocument;
use anyhow::{ensure, Context, Result};
use std::{collections::BTreeSet, fmt::Write};

fn admit(document: &TinySamDocument) -> Result<()> {
    for (label, names) in
        [("reference", &document.references), ("read group", &document.read_groups)]
    {
        ensure!(names.iter().collect::<BTreeSet<_>>().len() == names.len(), "duplicate {label}");
        for name in names {
            ensure!(
                !name.is_empty() && !name.contains(['\t', '\n', '\r']),
                "invalid alignment {label} name"
            );
        }
    }
    for reference in &document.references {
        ensure!(
            document.reference_lengths.get(reference).is_some_and(|length| *length > 0),
            "alignment reference {reference} has no positive declared length"
        );
    }
    for sample in document.read_group_sample_map.values() {
        ensure!(
            !sample.is_empty() && !sample.contains(['\t', '\n', '\r']),
            "invalid alignment sample name"
        );
    }
    Ok(())
}

pub(super) fn render(document: &TinySamDocument, sort_order: &str) -> Result<String> {
    admit(document)?;
    let mut payload = format!("@HD\tVN:1.6\tSO:{sort_order}\n");
    for reference in &document.references {
        let length = document.reference_lengths.get(reference).context("reference length")?;
        writeln!(payload, "@SQ\tSN:{reference}\tLN:{length}")?;
    }
    for read_group in &document.read_groups {
        write!(payload, "@RG\tID:{read_group}")?;
        if let Some(sample) = document.read_group_sample_map.get(read_group) {
            write!(payload, "\tSM:{sample}")?;
        }
        payload.push('\n');
    }
    Ok(payload)
}

pub(super) fn merge(target: &mut TinySamDocument, source: &TinySamDocument) -> Result<()> {
    admit(source)?;
    for reference in &source.references {
        let length = *source.reference_lengths.get(reference).context("reference length")?;
        if let Some(previous) = target.reference_lengths.get(reference) {
            ensure!(*previous == length, "merge reference length conflict for {reference}");
        } else {
            target.references.push(reference.clone());
            target.reference_lengths.insert(reference.clone(), length);
        }
    }
    for read_group in &source.read_groups {
        let sample = source.read_group_sample_map.get(read_group);
        if target.read_groups.contains(read_group) {
            ensure!(
                target.read_group_sample_map.get(read_group) == sample,
                "merge sample identity conflict for read group {read_group}"
            );
        } else {
            target.read_groups.push(read_group.clone());
            if let Some(sample) = sample {
                target.read_group_sample_map.insert(read_group.clone(), sample.clone());
                if !target.read_group_samples.contains(sample) {
                    target.read_group_samples.push(sample.clone());
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/contracts/parsers/alignment_header.rs"]
mod tests;
