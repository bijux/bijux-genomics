use super::{count_fastq_stats, observe_records};
use anyhow::Result;
use flate2::{write::GzEncoder, Compression};
use std::{
    io::{self, BufRead, Cursor, Read, Write},
    path::Path,
};
#[test]
fn plain_and_concatenated_gzip_records_preserve_read_and_base_counts() -> Result<()> {
    let root = tempfile::tempdir()?;
    let first = b"@one\nACGT\n+\nIIII\n";
    let second = b"@two\nAC\n+two\nII\n";
    let plain = root.path().join("reads.fastq");
    std::fs::write(&plain, [first.as_slice(), second.as_slice()].concat())?;
    let mut compressed = Vec::new();
    for record in [first.as_slice(), second.as_slice()] {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(record)?;
        compressed.extend(encoder.finish()?);
    }
    let gzip = root.path().join("reads.fastq.gz");
    std::fs::write(&gzip, compressed)?;
    for path in [&plain, &gzip] {
        let stats = count_fastq_stats(path)?;
        assert_eq!(stats.reads, 2);
        assert_eq!(stats.bases, 6);
    }
    Ok(())
}
#[test]
fn malformed_headers_separators_lengths_and_truncation_are_rejected() {
    for raw in ["one\nAC\n+\nII\n", "@one\nAC\nminus\nII\n", "@one\nAC\n+\nI\n", "@one\nAC\n+\n"] {
        assert!(observe_records(Cursor::new(raw), Path::new("fixture.fastq")).is_err());
    }
}
struct HeaderFailure {
    failed: bool,
    rest: Cursor<Vec<u8>>,
}
impl Read for HeaderFailure {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.rest.read(buf)
    }
}
impl BufRead for HeaderFailure {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if !self.failed {
            self.failed = true;
            return Err(io::Error::other("header read failed"));
        }
        self.rest.fill_buf()
    }
    fn consume(&mut self, count: usize) {
        self.rest.consume(count);
    }
}
#[test]
fn a_header_io_failure_cannot_be_discarded_as_a_valid_record() {
    let reader = HeaderFailure { failed: false, rest: Cursor::new(b"AC\n+\nII\n".to_vec()) };
    assert!(observe_records(reader, Path::new("fixture.fastq")).is_err());
}
