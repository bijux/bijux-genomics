//! Owner: bijux-dna-domain-fastq
//! FASTQ record values and header pairing semantics shared by domain transformations.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FastqRecord {
    pub header: String,
    pub sequence: String,
    pub plus: String,
    pub quality: String,
}

impl FastqRecord {
    #[must_use]
    pub(crate) fn base_name_and_mate(&self) -> (String, Option<u8>) {
        parse_header_pairing(self.header.trim_start_matches('@'))
    }
}

#[must_use]
pub(crate) fn parse_header_pairing(header: &str) -> (String, Option<u8>) {
    let token = header.split_whitespace().next().unwrap_or(header);
    if let Some(base) = token.strip_suffix("/1") {
        return (base.to_string(), Some(1));
    }
    if let Some(base) = token.strip_suffix("/2") {
        return (base.to_string(), Some(2));
    }

    let mut fields = header.split_whitespace();
    let first = fields.next().unwrap_or(header);
    if let Some(descriptor) = fields.next() {
        if descriptor.starts_with("1:") {
            return (first.to_string(), Some(1));
        }
        if descriptor.starts_with("2:") {
            return (first.to_string(), Some(2));
        }
    }

    (first.to_string(), None)
}
