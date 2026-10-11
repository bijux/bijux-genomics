//! Reference-sequence authorities derived from BAM headers.

use std::fs::File;
use std::path::Path;

use anyhow::{bail, Context, Result};
use noodles_bam as bam;

/// Return the exact sum of all reference-sequence lengths declared by a BAM header.
///
/// This reads only the header. References with no aligned records remain part of the span, which
/// makes the value suitable as a common coverage denominator across tool implementations.
///
/// # Errors
/// Returns an error when the BAM header cannot be read, contains no references, or its lengths
/// overflow a `u64`.
pub fn bam_header_reference_span(path: &Path) -> Result<u64> {
    let mut reader = File::open(path)
        .map(bam::io::Reader::new)
        .with_context(|| format!("open BAM header authority {}", path.display()))?;
    let header = reader
        .read_header()
        .with_context(|| format!("read BAM header authority {}", path.display()))?;
    let span = header.reference_sequences().values().try_fold(0_u64, |total, reference| {
        total
            .checked_add(reference.length().get() as u64)
            .context("BAM header reference span overflow")
    })?;
    if span == 0 {
        bail!("BAM header declares no positive reference span: {}", path.display());
    }
    Ok(span)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::bam_header_reference_span;

    #[test]
    fn header_span_includes_the_complete_binary_bam_reference_dictionary() -> anyhow::Result<()> {
        let bam = workspace_root().join("assets/toy/core-v1/bam/validation_pass.bam");
        assert_eq!(bam_header_reference_span(&bam)?, 50);
        Ok(())
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf()
    }
}
