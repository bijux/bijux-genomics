//! Ordering for the miniature native alignment fixture converter.
use noodles_sam as sam;
use std::collections::HashMap;

pub(super) fn sam_record_sort_key(
    record: &sam::Record,
    header: &sam::Header,
    reference_order: &HashMap<String, usize>,
) -> (usize, usize, usize, String) {
    let reference_rank = record
        .reference_sequence_id(header)
        .transpose()
        .ok()
        .flatten()
        .and_then(|reference_id| {
            reference_order
                .iter()
                .find_map(|(name, rank)| (*rank == reference_id).then_some((name.clone(), *rank)))
        })
        .map_or(usize::MAX, |(_, rank)| rank);
    let alignment_start =
        record.alignment_start().transpose().ok().flatten().map_or(usize::MAX, usize::from);
    let unmapped_rank = usize::from(
        record.flags().ok().is_none_or(noodles_sam::alignment::record::Flags::is_unmapped),
    );
    let name = record
        .name()
        .map(|name| String::from_utf8_lossy(name.as_ref()).into_owned())
        .unwrap_or_default();
    (unmapped_rank, reference_rank, alignment_start, name)
}
