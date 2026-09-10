//! Native BAM decoding into the shared miniature alignment representation.
use super::{TinySamDocument, TinySamRecord};
use anyhow::{anyhow, Result};
use noodles_bam as bam;
use noodles_sam::alignment::record::data::field::{Tag, Value};
use noodles_sam::header::record::value::map::header::{sort_order::COORDINATE, tag::SORT_ORDER};
use noodles_sam::header::record::value::map::read_group::tag as read_group_tag;
use std::{fmt::Write as _, fs::File, path::Path};

pub(super) fn parse_tiny_bam(path: &Path) -> Result<TinySamDocument> {
    let mut document = TinySamDocument::default();
    let mut reader = File::open(path).map(bam::io::Reader::new)?;
    let header = reader.read_header()?;

    document.sort_order = header
        .header()
        .and_then(|header_record| header_record.other_fields().get(&SORT_ORDER))
        .map(|sort_order| String::from_utf8_lossy(sort_order.as_ref()).into_owned())
        .or_else(|| Some(String::from_utf8_lossy(COORDINATE).into_owned()));

    for (name, reference_sequence) in header.reference_sequences() {
        let reference_name = String::from_utf8_lossy(name.as_ref()).into_owned();
        document.references.push(reference_name.clone());
        document.reference_lengths.insert(reference_name, reference_sequence.length().get() as u64);
    }

    for (read_group_id, read_group) in header.read_groups() {
        let read_group_id = String::from_utf8_lossy(read_group_id.as_ref()).into_owned();
        document.read_groups.push(read_group_id.clone());
        if let Some(sample_id) = read_group.other_fields().get(&read_group_tag::SAMPLE) {
            let sample_id = String::from_utf8_lossy(sample_id.as_ref()).into_owned();
            document.read_group_samples.push(sample_id.clone());
            document.read_group_sample_map.insert(read_group_id, sample_id);
        }
    }

    for result in reader.records() {
        let record = result?;
        let rname = match record.reference_sequence_id().transpose()? {
            Some(reference_sequence_id) => header
                .reference_sequences()
                .get_index(reference_sequence_id)
                .map(|(name, _)| String::from_utf8_lossy(name.as_ref()).into_owned())
                .ok_or_else(|| {
                    anyhow!("BAM record reference_sequence_id `{reference_sequence_id}` missing from header")
                })?,
            None => "*".to_string(),
        };
        let read_group_id = record
            .data()
            .get(&Tag::READ_GROUP)
            .transpose()?
            .map(|value| match value {
                Value::String(read_group) => {
                    Ok(String::from_utf8_lossy(read_group.as_ref()).into_owned())
                }
                _ => Err(anyhow!("BAM READ_GROUP tag is not a string")),
            })
            .transpose()?;

        document.records.push(TinySamRecord {
            qname: record
                .name()
                .map(|name| String::from_utf8_lossy(name.as_ref()).into_owned())
                .unwrap_or_default(),
            flag: u16::from(record.flags()),
            rname,
            pos: record
                .alignment_start()
                .transpose()?
                .map(|position| position.get() as u64)
                .unwrap_or(0),
            mapq: record.mapping_quality().map(u8::from).unwrap_or(0),
            cigar: record.cigar().iter().collect::<std::io::Result<Vec<_>>>()?.into_iter().fold(
                String::new(),
                |mut cigar, op| {
                    let _ = write!(cigar, "{}{}", op.len(), render_cigar_kind(op.kind()));
                    cigar
                },
            ),
            template_length: i64::from(record.template_length()),
            seq: record.sequence().iter().map(char::from).collect::<String>(),
            read_group_id,
        });
    }

    Ok(document)
}

fn render_cigar_kind(kind: noodles_sam::alignment::record::cigar::op::Kind) -> char {
    use noodles_sam::alignment::record::cigar::op::Kind;

    match kind {
        Kind::Match => 'M',
        Kind::Insertion => 'I',
        Kind::Deletion => 'D',
        Kind::Skip => 'N',
        Kind::SoftClip => 'S',
        Kind::HardClip => 'H',
        Kind::Pad => 'P',
        Kind::SequenceMatch => '=',
        Kind::SequenceMismatch => 'X',
    }
}
