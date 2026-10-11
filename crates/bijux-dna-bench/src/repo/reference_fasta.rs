//! Owner: bijux-dna-bench
//! Admit and publish paired FASTA/index inputs for benchmark adapters.

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Materialize a benchmark reference and its admitted FAI without changing exact matches.
///
/// # Errors
/// Rejects unreadable or non-indexable FASTA before publication, or reports output IO failures.
pub fn materialize_benchmark_reference_fasta(
    source: &Path,
    output_root: &Path,
) -> Result<(PathBuf, PathBuf)> {
    let name = source.file_name().context("reference FASTA has no file name")?;
    let fasta = output_root.join(name);
    let raw = fs::read(source).with_context(|| format!("read {}", source.display()))?;
    let index_payload = bijux_dna_db_ref::build_reference_fasta_index(&raw)
        .with_context(|| format!("index governed reference {}", source.display()))?;
    let mut index_name = fasta.as_os_str().to_os_string();
    index_name.push(".fai");
    let index = PathBuf::from(index_name);
    if regular_file_matches(&fasta, &raw) && regular_file_matches(&index, index_payload.as_bytes())
    {
        return Ok((fasta, index));
    }
    bijux_dna_infra::ensure_dir(output_root)?;
    bijux_dna_infra::atomic_write_bytes(&fasta, &raw).with_context(|| {
        format!("materialize governed reference {} into {}", source.display(), fasta.display())
    })?;
    bijux_dna_infra::atomic_write_bytes(&index, index_payload.as_bytes())?;
    Ok((fasta, index))
}

fn regular_file_matches(path: &Path, expected: &[u8]) -> bool {
    let Ok(metadata) = path.symlink_metadata() else { return false };
    if !metadata.is_file() || metadata.len() != expected.len() as u64 {
        return false;
    }
    let Ok(mut file) = File::open(path) else { return false };
    let mut buffer = [0; 8192];
    for chunk in expected.chunks(buffer.len()) {
        if file.read_exact(&mut buffer[..chunk.len()]).is_err() || &buffer[..chunk.len()] != chunk {
            return false;
        }
    }
    matches!(file.read(&mut buffer[..1]), Ok(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_pair_reuses_exact_bytes_and_rebuilds_missing_or_corrupt_members() -> Result<()> {
        let root = tempfile::tempdir()?;
        let source = root.path().join("input.fa");
        fs::write(&source, b">chr1\r\nACGT\r\nAC\r\n")?;
        let out = root.path().join("staging");
        let (fasta, index) = materialize_benchmark_reference_fasta(&source, &out)?;
        assert_eq!(fs::read(&fasta)?, fs::read(&source)?);
        assert_eq!(fs::read_to_string(&index)?, "chr1\t6\t7\t4\t6\n");
        let historical = std::time::UNIX_EPOCH + std::time::Duration::from_secs(10000);
        for path in [&fasta, &index] {
            File::options()
                .write(true)
                .open(path)?
                .set_times(fs::FileTimes::new().set_modified(historical))?;
        }
        let times = [fs::metadata(&fasta)?.modified()?, fs::metadata(&index)?.modified()?];
        materialize_benchmark_reference_fasta(&source, &out)?;
        assert_eq!(times, [fs::metadata(&fasta)?.modified()?, fs::metadata(&index)?.modified()?]);
        for path in [&fasta, &index] {
            fs::write(path, "corrupt")?;
            materialize_benchmark_reference_fasta(&source, &out)?;
            assert_eq!(fs::read(&fasta)?, fs::read(&source)?);
            assert_eq!(fs::read_to_string(&index)?, "chr1\t6\t7\t4\t6\n");
            fs::remove_file(path)?;
            materialize_benchmark_reference_fasta(&source, &out)?;
            assert!(fasta.is_file() && index.is_file());
        }
        Ok(())
    }

    #[test]
    fn invalid_reference_preserves_both_previous_outputs() -> Result<()> {
        let root = tempfile::tempdir()?;
        let source = root.path().join("input.fa");
        fs::write(&source, ">chr1\nACGT\n")?;
        let out = root.path().join("staging");
        let (fasta, index) = materialize_benchmark_reference_fasta(&source, &out)?;
        let previous = (fs::read(&fasta)?, fs::read(&index)?);
        fs::write(&source, ">chr1\nACGT\nAC\nGT\n")?;
        assert!(materialize_benchmark_reference_fasta(&source, &out).is_err());
        assert_eq!(previous, (fs::read(&fasta)?, fs::read(&index)?));
        Ok(())
    }
}
