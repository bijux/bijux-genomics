//! Coordinate ordering for miniature native alignment fixtures.
use noodles_sam as sam;
use std::io;

/// SAM coordinate order follows the dictionary and POS, including placed unmapped reads.
/// Missing RNAME sorts last; equal coordinates use the read name as a deterministic tie break.
pub(super) fn sam_record_sort_key(
    record: &sam::Record,
    header: &sam::Header,
) -> io::Result<(usize, usize, String)> {
    // Flags do not determine coordinate rank, but malformed flags must reject before writing.
    record.flags()?;
    let reference_rank = record.reference_sequence_id(header).transpose()?.unwrap_or(usize::MAX);
    let position = record.alignment_start().transpose()?.map_or(0, usize::from);
    let name = record
        .name()
        .map(|name| String::from_utf8_lossy(name.as_ref()).into_owned())
        .unwrap_or_default();
    Ok((reference_rank, position, name))
}
