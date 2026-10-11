//! Shared miniature alignment representation; independent of format IO and scientific summaries.
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(in crate::artifacts) struct TinySamRecord {
    pub(in crate::artifacts) qname: String,
    pub(in crate::artifacts) flag: u16,
    pub(in crate::artifacts) rname: String,
    pub(in crate::artifacts) pos: u64,
    pub(in crate::artifacts) mapq: u8,
    pub(in crate::artifacts) cigar: String,
    pub(in crate::artifacts) template_length: i64,
    pub(in crate::artifacts) seq: String,
    pub(in crate::artifacts) read_group_id: Option<String>,
}

impl TinySamRecord {
    pub(in crate::artifacts) fn is_mapped(&self) -> bool {
        self.rname != "*" && (self.flag & 0x4) == 0
    }

    pub(in crate::artifacts) fn is_duplicate(&self) -> bool {
        (self.flag & 0x400) != 0
    }
}

#[derive(Debug, Clone, Default)]
pub(in crate::artifacts) struct TinySamDocument {
    pub(in crate::artifacts) sort_order: Option<String>,
    pub(in crate::artifacts) references: Vec<String>,
    pub(in crate::artifacts) reference_lengths: HashMap<String, u64>,
    pub(in crate::artifacts) read_groups: Vec<String>,
    pub(in crate::artifacts) read_group_samples: Vec<String>,
    pub(in crate::artifacts) read_group_sample_map: HashMap<String, String>,
    pub(in crate::artifacts) records: Vec<TinySamRecord>,
}
