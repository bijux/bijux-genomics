//! Derive an uncompressed FASTA byte-offset index without filesystem access.

use std::collections::BTreeSet;
use std::fmt::Write;

use anyhow::{bail, Context, Result};

struct Record {
    name: String,
    length: usize,
    offset: usize,
    line_bases: usize,
    line_width: usize,
    finished: bool,
}

impl Record {
    fn sequence_line(&mut self, line: &[u8], width: usize) -> Result<()> {
        if line.is_empty() {
            if self.length == 0 {
                bail!("FASTA record `{}` has a blank line before its first base", self.name);
            }
            self.finished = true;
            return Ok(());
        }
        if self.finished || !line.iter().all(u8::is_ascii_graphic) {
            bail!("FASTA record `{}` has an irregular or non-ASCII sequence line", self.name);
        }
        if self.length == 0 {
            self.line_bases = line.len();
            self.line_width = width;
        } else {
            if line.len() > self.line_bases {
                bail!("FASTA record `{}` has an overlong sequence line", self.name);
            }
            let terminator_width = width - line.len();
            if terminator_width != 0 && terminator_width != self.line_width - self.line_bases {
                bail!("FASTA record `{}` mixes sequence line terminators", self.name);
            }
            self.finished = line.len() < self.line_bases || terminator_width == 0;
        }
        self.length += line.len();
        Ok(())
    }

    fn write_index(&self, output: &mut String) -> Result<()> {
        if self.length == 0 {
            bail!("FASTA record `{}` has no sequence bases", self.name);
        }
        writeln!(
            output,
            "{}\t{}\t{}\t{}\t{}",
            self.name, self.length, self.offset, self.line_bases, self.line_width
        )?;
        Ok(())
    }
}

/// Derive the five-column FAI contract for admitted uncompressed FASTA bytes.
///
/// Headers use their first whitespace-delimited word. Sequence lines use LF or
/// CRLF consistently within each record; only a final line may be shorter or
/// lack its terminator. Record order and byte offsets are preserved.
///
/// # Errors
/// Rejects absent, duplicate or empty records, sequence before a header, non-ASCII
/// sequence bytes, internal whitespace, and line layouts that cannot be indexed linearly.
pub fn build_reference_fasta_index(raw: &[u8]) -> Result<String> {
    let mut output = String::new();
    let mut names = BTreeSet::new();
    let mut current: Option<Record> = None;
    let mut offset = 0;
    for chunk in raw.split_inclusive(|byte| *byte == b'\n') {
        let line = chunk.strip_suffix(b"\n").unwrap_or(chunk);
        let line =
            if chunk.ends_with(b"\n") { line.strip_suffix(b"\r").unwrap_or(line) } else { line };
        if let Some(header) = line.strip_prefix(b">") {
            if let Some(record) = current.take() {
                record.write_index(&mut output)?;
            }
            let text = std::str::from_utf8(header).context("decode FASTA header")?;
            let name = text.split_ascii_whitespace().next().context("empty FASTA header")?;
            if !names.insert(name.to_string()) {
                bail!("FASTA repeats record `{name}`");
            }
            current = Some(Record {
                name: name.to_string(),
                length: 0,
                offset: offset + chunk.len(),
                line_bases: 0,
                line_width: 0,
                finished: false,
            });
        } else if let Some(record) = &mut current {
            record.sequence_line(line, chunk.len())?;
        } else if !line.is_empty() {
            bail!("FASTA sequence line appears before any header");
        }
        offset += chunk.len();
    }
    if let Some(record) = current {
        record.write_index(&mut output)?;
    }
    if output.is_empty() {
        bail!("no FASTA records found");
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::build_reference_fasta_index;

    #[test]
    fn index_matches_native_lf_crlf_and_final_line_fixtures() -> anyhow::Result<()> {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/fasta_index/native_cases.json"
        ))?;
        let cases = fixture["cases"].as_array().ok_or_else(|| anyhow::anyhow!("native cases"))?;
        assert_eq!(cases.len(), 4);
        for case in cases {
            let input = case["fasta"].as_str().ok_or_else(|| anyhow::anyhow!("FASTA input"))?;
            let expected = case["fai"].as_str().ok_or_else(|| anyhow::anyhow!("native FAI"))?;
            assert_eq!(
                build_reference_fasta_index(input.as_bytes())?,
                expected,
                "{}",
                case["name"]
            );
        }
        Ok(())
    }

    #[test]
    fn index_rejects_ambiguous_identity_and_non_indexable_sequence_layouts() {
        for input in [
            "",
            ">",
            "ACGT\n",
            ">chr1\n",
            ">chr1\nAC\n>chr1\nGT\n",
            ">chr1\nACGT\nAC\nACGT\n",
            ">chr1\nAC\nACGT\n",
            ">chr1\nACGT\r\nACGT\n",
            ">chr1\nA C G T\n",
            ">chr1\nACGT\n\nACGT\n",
            ">chr1\n\nACGT\n",
            ">chr1\nACéT\n",
        ] {
            assert!(build_reference_fasta_index(input.as_bytes()).is_err(), "accepted {input:?}");
        }
    }
}
