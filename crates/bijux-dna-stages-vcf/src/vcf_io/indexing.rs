//! Bounded VCF/BGZF encoding and publication of its paired tabix index.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use noodles_bgzf as bgzf;
use noodles_core::Position;
use noodles_csi::{self as csi, binning_index::index::reference_sequence::bin::Chunk};
use noodles_tabix as tabix;

// Readiness fixtures are small. Large scientific inputs retain streaming publication.
const BUFFERED_INPUT_LIMIT: u64 = 1024 * 1024;

struct EncodedVcfPair {
    bgzf: Vec<u8>,
    tabix: Vec<u8>,
}

/// # Errors
/// Returns an error if bgzip/tabix indexing fails.
///
/// Inputs up to 1 MiB are recomputed in memory. Existing regular output files are
/// retained only when both encoded payloads match exactly; no path or mtime cache is used.
pub fn vcf_index_bgzip_tabix(input_vcf: &Path, output_vcfgz: &Path) -> Result<PathBuf> {
    let parent = output_vcfgz.parent().ok_or_else(|| {
        anyhow!("vcf_index_bgzip_tabix: output has no parent: {}", output_vcfgz.display())
    })?;
    std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    let output_tbi = append_suffix(output_vcfgz, ".tbi");
    let encoded = encode_bounded_input(input_vcf)?;
    if encoded.as_ref().is_some_and(|pair| {
        file_matches(output_vcfgz, &pair.bgzf) && file_matches(&output_tbi, &pair.tabix)
    }) {
        return Ok(output_tbi);
    }
    let nonce = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("read system clock for bgzip temp path")?
            .as_nanos()
    );
    let tmp_vcfgz = append_suffix(output_vcfgz, &format!(".{nonce}.tmp"));
    let tmp_tbi = append_suffix(&tmp_vcfgz, ".tbi");

    let result = if let Some(pair) = encoded {
        bijux_dna_infra::atomic_write_bytes(&tmp_vcfgz, &pair.bgzf)
            .and_then(|()| bijux_dna_infra::atomic_write_bytes(&tmp_tbi, &pair.tabix))
            .map_err(anyhow::Error::from)
    } else {
        write_bgzf_vcf_and_tabix(input_vcf, &tmp_vcfgz, &tmp_tbi)
    };
    if let Err(err) = result {
        let _ = std::fs::remove_file(&tmp_vcfgz);
        let _ = std::fs::remove_file(&tmp_tbi);
        return Err(err);
    }

    std::fs::rename(&tmp_vcfgz, output_vcfgz)?;
    std::fs::rename(&tmp_tbi, &output_tbi)?;
    Ok(output_tbi)
}

fn write_bgzf_vcf_and_tabix(
    input_vcf: &Path,
    output_vcfgz: &Path,
    output_tbi: &Path,
) -> Result<()> {
    let input = File::open(input_vcf).with_context(|| format!("open {}", input_vcf.display()))?;
    let mut indexer = vcf_indexer();
    bijux_dna_infra::atomic_write_with(output_vcfgz, |output| {
        encode_bgzf_vcf(BufReader::new(input), output, &mut indexer)
    })
    .with_context(|| format!("write indexed BGZF payload {}", output_vcfgz.display()))?;
    let index = indexer.build();
    bijux_dna_infra::atomic_write_with(output_tbi, |output| {
        tabix::io::Writer::new(output).write_index(&index)
    })
    .with_context(|| format!("write {}", output_tbi.display()))?;
    Ok(())
}

fn vcf_indexer() -> tabix::index::Indexer {
    let mut indexer = tabix::index::Indexer::default();
    indexer.set_header(csi::binning_index::index::header::Builder::vcf().build());
    indexer
}

fn encode_bgzf_vcf(
    input: impl BufRead,
    output: impl Write,
    indexer: &mut tabix::index::Indexer,
) -> io::Result<()> {
    let mut writer = bgzf::io::Writer::new(output);
    let mut chunk_start = writer.virtual_position();

    for line in input.lines() {
        let line = line?;
        writeln!(writer, "{line}")?;
        let chunk_end = writer.virtual_position();

        if !line.starts_with('#') && !line.is_empty() {
            let (reference_sequence_name, start, end) = vcf_record_interval(&line)
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
            indexer
                .add_record(reference_sequence_name, start, end, Chunk::new(chunk_start, chunk_end))
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
        }

        chunk_start = chunk_end;
    }

    writer.finish()?;
    Ok(())
}

fn encode_bounded_input(path: &Path) -> Result<Option<EncodedVcfPair>> {
    let input = File::open(path).with_context(|| format!("open {}", path.display()))?;
    if input.metadata()?.len() > BUFFERED_INPUT_LIMIT {
        return Ok(None);
    }
    let mut raw = Vec::new();
    input.take(BUFFERED_INPUT_LIMIT + 1).read_to_end(&mut raw)?;
    if raw.len() as u64 > BUFFERED_INPUT_LIMIT {
        return Ok(None);
    }
    let mut indexer = vcf_indexer();
    let mut bgzf = Vec::new();
    encode_bgzf_vcf(Cursor::new(raw), &mut bgzf, &mut indexer)?;
    let mut tabix = Vec::new();
    tabix::io::Writer::new(&mut tabix).write_index(&indexer.build())?;
    Ok(Some(EncodedVcfPair { bgzf, tabix }))
}

fn file_matches(path: &Path, expected: &[u8]) -> bool {
    let Ok(metadata) = path.symlink_metadata() else {
        return false;
    };
    if !metadata.is_file() || metadata.len() != expected.len() as u64 {
        return false;
    }
    let Ok(input) = File::open(path) else {
        return false;
    };
    let mut observed = Vec::with_capacity(expected.len());
    input.take(expected.len() as u64 + 1).read_to_end(&mut observed).is_ok() && observed == expected
}

fn vcf_record_interval(line: &str) -> Result<(&str, Position, Position)> {
    let mut fields = line.split('\t');
    let reference_sequence_name =
        fields.next().filter(|value| !value.is_empty()).context("VCF record missing CHROM")?;
    let start = fields
        .next()
        .context("VCF record missing POS")?
        .parse::<usize>()
        .context("VCF record has invalid POS")
        .and_then(|value| Position::try_from(value).context("VCF record POS must be positive"))?;
    let _id = fields.next().context("VCF record missing ID")?;
    let reference = fields.next().context("VCF record missing REF")?;
    let _alternate = fields.next().context("VCF record missing ALT")?;
    let _quality = fields.next().context("VCF record missing QUAL")?;
    let _filter = fields.next().context("VCF record missing FILTER")?;
    let info = fields.next().context("VCF record missing INFO")?;
    let end = info
        .split(';')
        .find_map(|field| field.strip_prefix("END="))
        .map(str::parse::<usize>)
        .transpose()
        .context("VCF record has invalid INFO/END")?
        .unwrap_or_else(|| usize::from(start) + reference.len().saturating_sub(1));
    let end = Position::try_from(end).context("VCF record end must be positive")?;
    Ok((reference_sequence_name, start, end))
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    value.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const VCF: &str = "##fileformat=VCFv4.2\n##contig=<ID=1,length=1000>\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n1\t10\t.\tA\tC\t.\tPASS\t.\n";

    #[test]
    fn bounded_and_streaming_encoders_preserve_identical_bgzf_and_tabix_bytes() -> Result<()> {
        let root = tempfile::tempdir()?;
        let input = root.path().join("input.vcf");
        fs::write(&input, VCF)?;
        let pair = encode_bounded_input(&input)?.context("small input uses bounded encoding")?;
        let output = root.path().join("streamed.vcf.gz");
        let index = append_suffix(&output, ".tbi");
        write_bgzf_vcf_and_tabix(&input, &output, &index)?;
        assert_eq!(fs::read(&output)?, pair.bgzf);
        assert_eq!(fs::read(&index)?, pair.tabix);
        assert_eq!(crate::vcf_io::read_vcf_text(&output)?, VCF);
        tabix::fs::read(&index)?;
        Ok(())
    }

    #[test]
    fn unchanged_pair_is_recomputed_without_replacing_either_file() -> Result<()> {
        let root = tempfile::tempdir()?;
        let input = root.path().join("input.vcf");
        let output = root.path().join("indexed.vcf.gz");
        fs::write(&input, VCF)?;
        let index = vcf_index_bgzip_tabix(&input, &output)?;
        let historical = UNIX_EPOCH + std::time::Duration::from_secs(10000);
        for path in [&output, &index] {
            File::options()
                .write(true)
                .open(path)?
                .set_times(fs::FileTimes::new().set_modified(historical))?;
        }
        let times = [fs::metadata(&output)?.modified()?, fs::metadata(&index)?.modified()?];
        assert_eq!(vcf_index_bgzip_tabix(&input, &output)?, index);
        assert_eq!(times, [fs::metadata(&output)?.modified()?, fs::metadata(&index)?.modified()?]);
        Ok(())
    }

    #[test]
    fn changed_missing_and_corrupt_members_are_rebuilt_from_current_input() -> Result<()> {
        let root = tempfile::tempdir()?;
        let input = root.path().join("input.vcf");
        let output = root.path().join("indexed.vcf.gz");
        fs::write(&input, VCF)?;
        let index = vcf_index_bgzip_tabix(&input, &output)?;
        let original = [fs::read(&output)?, fs::read(&index)?];
        for member in [&output, &index] {
            fs::write(member, b"corrupt")?;
            vcf_index_bgzip_tabix(&input, &output)?;
            assert_eq!(original, [fs::read(&output)?, fs::read(&index)?]);
            fs::remove_file(member)?;
            vcf_index_bgzip_tabix(&input, &output)?;
            assert_eq!(original, [fs::read(&output)?, fs::read(&index)?]);
        }
        let changed = VCF.replace("1\t10\t", "1\t20\t");
        fs::write(&input, &changed)?;
        vcf_index_bgzip_tabix(&input, &output)?;
        assert_eq!(crate::vcf_io::read_vcf_text(&output)?, changed);
        let valid = [fs::read(&output)?, fs::read(&index)?];
        fs::write(&input, VCF.replace("1\t10\t", "1\t0\t"))?;
        assert!(vcf_index_bgzip_tabix(&input, &output).is_err());
        assert_eq!(valid, [fs::read(&output)?, fs::read(&index)?]);
        Ok(())
    }

    #[test]
    fn oversized_inputs_use_the_streaming_encoder() -> Result<()> {
        let root = tempfile::tempdir()?;
        let input = root.path().join("input.vcf");
        let output = root.path().join("indexed.vcf.gz");
        let header = "x".repeat(BUFFERED_INPUT_LIMIT as usize);
        let raw = format!("##description={header}\n{VCF}");
        fs::write(&input, &raw)?;
        assert!(encode_bounded_input(&input)?.is_none());
        let index = vcf_index_bgzip_tabix(&input, &output)?;
        assert_eq!(crate::vcf_io::read_vcf_text(&output)?, raw);
        tabix::fs::read(index)?;
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn matching_symlinks_are_replaced_without_modifying_the_target() -> Result<()> {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir()?;
        let input = root.path().join("input.vcf");
        let output = root.path().join("indexed.vcf.gz");
        fs::write(&input, VCF)?;
        let index = vcf_index_bgzip_tabix(&input, &output)?;
        assert_eq!(index, append_suffix(&output, ".tbi"));
        let saved = root.path().join("saved.vcf.gz");
        let original = fs::read(&output)?;
        fs::rename(&output, &saved)?;
        symlink(&saved, &output)?;
        vcf_index_bgzip_tabix(&input, &output)?;
        assert!(output.symlink_metadata()?.is_file());
        assert_eq!(fs::read(&output)?, original);
        assert_eq!(fs::read(&saved)?, original);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn index_suffix_preserves_native_path_bytes() {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        let output = PathBuf::from(std::ffi::OsString::from_vec(b"indexed-\xff.vcf.gz".to_vec()));
        assert_eq!(
            append_suffix(&output, ".tbi").as_os_str().as_bytes(),
            b"indexed-\xff.vcf.gz.tbi"
        );
    }
}
