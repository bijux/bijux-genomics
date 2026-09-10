//! Native alignment files for local benchmark fixtures, independent of CLI routing.
mod sorting;
use anyhow::{bail, Context, Result};
use noodles_bam as bam;
use noodles_sam as sam;
use sorting::sam_record_sort_key;
use std::{fs, io::BufReader, path::Path};

/// Input encoding is explicit; filenames do not establish format compatibility.
#[derive(Debug, Clone, Copy)]
pub enum AlignmentInputFormat {
    Sam,
    Bam,
}

/// Sort a small SAM fixture and encode native BAM and BAI files.
/// The complete record collection is held in memory.
///
/// # Errors
/// Rejects malformed sort fields before opening outputs. Later encoding, writing
/// or indexing failures can leave a partially published BAM/BAI pair.
pub fn convert_coordinate_sam_to_bam(
    input_sam: &Path,
    output_bam_path: &Path,
    output_bam_index_path: &Path,
) -> Result<()> {
    use noodles_sam::alignment::io::Write as _;
    use noodles_sam::header::record::value::map::header::{
        sort_order::COORDINATE,
        tag::{GROUP_ORDER, SORT_ORDER, SUBSORT_ORDER},
    };

    let file = fs::File::open(input_sam)
        .with_context(|| format!("open semantic alignment SAM {}", input_sam.display()))?;
    let mut reader = sam::io::Reader::new(BufReader::new(file));
    let mut header =
        reader.read_header().with_context(|| format!("read {}", input_sam.display()))?;
    let declaration = header.header_mut().get_or_insert_default();
    declaration.other_fields_mut().insert(SORT_ORDER, COORDINATE.into());
    declaration.other_fields_mut().shift_remove(&GROUP_ORDER);
    declaration.other_fields_mut().shift_remove(&SUBSORT_ORDER);

    let mut records = reader
        .records()
        .map(|record| {
            let record = record?;
            let key = sam_record_sort_key(&record, &header)?;
            Ok((key, record))
        })
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| format!("admit SAM coordinates from {}", input_sam.display()))?;
    records.sort_by(|left, right| left.0.cmp(&right.0));

    let bam_file = bijux_dna_infra::create_file(output_bam_path)
        .with_context(|| format!("create {}", output_bam_path.display()))?;
    let mut writer = bam::io::Writer::new(bam_file);
    writer
        .write_header(&header)
        .with_context(|| format!("write BAM header to {}", output_bam_path.display()))?;
    for (_, record) in &records {
        writer
            .write_alignment_record(&header, record)
            .with_context(|| format!("write BAM record to {}", output_bam_path.display()))?;
    }
    writer.try_finish().with_context(|| format!("finish {}", output_bam_path.display()))?;

    let index = bam::fs::index(output_bam_path)
        .with_context(|| format!("index coordinate BAM {}", output_bam_path.display()))?;
    bam::bai::fs::write(output_bam_index_path, &index)
        .with_context(|| format!("write {}", output_bam_index_path.display()))?;
    Ok(())
}

/// Render header reference spans as zero-based BED intervals, retaining declaration order.
///
/// # Errors
/// Rejects unreadable headers, empty reference dictionaries and output failures.
pub fn write_alignment_reference_bed(
    input: &Path,
    format: AlignmentInputFormat,
    output: &Path,
) -> Result<()> {
    let file = fs::File::open(input).with_context(|| format!("open {}", input.display()))?;
    let header = match format {
        AlignmentInputFormat::Sam => sam::io::Reader::new(BufReader::new(file)).read_header(),
        AlignmentInputFormat::Bam => bam::io::Reader::new(file).read_header(),
    }
    .with_context(|| format!("read {}", input.display()))?;
    let mut body = String::new();
    for (name, reference) in header.reference_sequences() {
        let contig = String::from_utf8_lossy(name.as_ref());
        let length = reference.length().get();
        use std::fmt::Write as _;
        writeln!(body, "{contig}\t0\t{length}\t{contig}")?;
    }
    if body.is_empty() {
        bail!("{} has no reference sequences to render as BED", input.display());
    }
    bijux_dna_infra::write_bytes(output, body)?;
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/contracts/native_alignment.rs"]
mod tests;
