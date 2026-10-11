//! Miniature SAM text decoding and encoding. Unrepresented SAM fields retain legacy behavior.
use super::{TinySamDocument, TinySamRecord};
use crate::params::ReadGroupSpec;
use anyhow::{anyhow, Result};
use std::path::Path;

fn parse_tag_value(field: &str, key: &str) -> Option<String> {
    if !field.starts_with(key) {
        return None;
    }
    let mut iter = field.splitn(3, ':');
    let first = iter.next()?;
    let second = iter.next()?;
    let value = iter.next()?;
    if first == key && second == "Z" {
        Some(value.to_string())
    } else {
        None
    }
}

pub(in crate::artifacts) fn parse_tiny_sam(path: &Path) -> Result<TinySamDocument> {
    let mut document = TinySamDocument::default();
    let payload = std::fs::read_to_string(path)?;
    for (line_index, raw_line) in payload.lines().enumerate() {
        let line = raw_line.trim_end();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('@') {
            let fields = line.split('\t').collect::<Vec<_>>();
            match fields.first().copied() {
                Some("@HD") => {
                    for field in &fields[1..] {
                        if let Some(sort_order) = field.strip_prefix("SO:") {
                            document.sort_order = Some(sort_order.to_string());
                        }
                    }
                }
                Some("@SQ") => {
                    let mut reference_name = None::<String>;
                    let mut reference_length = None::<u64>;
                    for field in &fields[1..] {
                        if let Some(name) = field.strip_prefix("SN:") {
                            reference_name = Some(name.to_string());
                        } else if let Some(length) = field.strip_prefix("LN:") {
                            reference_length = length.parse::<u64>().ok();
                        }
                    }
                    if let Some(name) = reference_name {
                        document.references.push(name.clone());
                        if let Some(length) = reference_length {
                            document.reference_lengths.insert(name, length);
                        }
                    }
                }
                Some("@RG") => {
                    let mut read_group_id_value = None::<String>;
                    let mut sample_id_value = None::<String>;
                    for field in &fields[1..] {
                        if let Some(read_group_id) = field.strip_prefix("ID:") {
                            let read_group_id = read_group_id.to_string();
                            document.read_groups.push(read_group_id.clone());
                            read_group_id_value = Some(read_group_id);
                        } else if let Some(sample_id) = field.strip_prefix("SM:") {
                            let sample_id = sample_id.to_string();
                            document.read_group_samples.push(sample_id.clone());
                            sample_id_value = Some(sample_id);
                        }
                    }
                    if let (Some(read_group_id), Some(sample_id)) =
                        (read_group_id_value, sample_id_value)
                    {
                        document.read_group_sample_map.insert(read_group_id, sample_id);
                    }
                }
                _ => {}
            }
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 11 {
            return Err(anyhow!(
                "malformed SAM record at line {}: expected at least 11 fields",
                line_index + 1
            ));
        }
        let flag = fields[1].parse::<u16>().map_err(|error| {
            anyhow!(
                "malformed SAM record at line {}: invalid flag {} ({error})",
                line_index + 1,
                fields[1]
            )
        })?;
        let pos = fields[3].parse::<u64>().map_err(|error| {
            anyhow!(
                "malformed SAM record at line {}: invalid position {} ({error})",
                line_index + 1,
                fields[3]
            )
        })?;
        let mapq = fields[4].parse::<u8>().map_err(|error| {
            anyhow!(
                "malformed SAM record at line {}: invalid MAPQ {} ({error})",
                line_index + 1,
                fields[4]
            )
        })?;
        let mut read_group_id = None;
        for tag in &fields[11..] {
            if let Some(value) = parse_tag_value(tag, "RG") {
                read_group_id = Some(value);
                break;
            }
        }
        let template_length = fields[8].parse::<i64>().map_err(|error| {
            anyhow!(
                "malformed SAM record at line {}: invalid template length {} ({error})",
                line_index + 1,
                fields[8]
            )
        })?;
        document.records.push(TinySamRecord {
            qname: fields[0].to_string(),
            flag,
            rname: fields[2].to_string(),
            pos,
            mapq,
            cigar: fields[5].to_string(),
            template_length,
            seq: fields[9].to_string(),
            read_group_id,
        });
    }
    Ok(document)
}

pub(in crate::artifacts) fn write_tiny_sam_document<'a>(
    output_bam: &Path,
    references: impl IntoIterator<Item = (&'a str, usize)>,
    read_group: &ReadGroupSpec,
    records: &[TinySamRecord],
) -> Result<()> {
    let mut payload = String::new();
    payload.push_str("@HD\tVN:1.6\tSO:unsorted\n");
    for (name, length) in references {
        payload.push_str(&format!("@SQ\tSN:{name}\tLN:{length}\n"));
    }
    payload.push_str(&format!(
        "@RG\tID:{}\tSM:{}\tPL:{}\tLB:{}",
        read_group.id, read_group.sample, read_group.platform, read_group.library
    ));
    if let Some(unit) = &read_group.platform_unit {
        payload.push_str(&format!("\tPU:{unit}"));
    }
    payload.push('\n');
    for record in records {
        payload.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t*\t0\t{}\t{}\t*\tRG:Z:{}\n",
            record.qname,
            record.flag,
            record.rname,
            record.pos,
            record.mapq,
            record.cigar,
            record.template_length,
            record.seq,
            record.read_group_id.as_deref().unwrap_or(&read_group.id)
        ));
    }
    bijux_dna_infra::write_bytes(output_bam, payload.as_bytes())?;
    Ok(())
}

pub(in crate::artifacts) fn write_tiny_sam_from_document(
    output_bam: &Path,
    document: &TinySamDocument,
    sort_order: &str,
) -> Result<()> {
    let mut payload = super::header::render(document, sort_order)?;
    for record in &document.records {
        payload.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t*\t0\t{}\t{}\t*\tRG:Z:{}\n",
            record.qname,
            record.flag,
            record.rname,
            record.pos,
            record.mapq,
            record.cigar,
            record.template_length,
            record.seq,
            record.read_group_id.as_deref().unwrap_or("unknown"),
        ));
    }
    bijux_dna_infra::write_bytes(output_bam, payload.as_bytes())?;
    Ok(())
}
