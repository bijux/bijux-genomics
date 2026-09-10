//! Owner: bijux-dna-bench-corpus
//! FASTQ corpus reads ownership.
use super::{
    anyhow, fs, path_relative_to_repo, resolve_manifest_relative_path, BufReader,
    FastqCorpusFixtureCompression, FastqCorpusFixtureLayout, FastqCorpusFixtureManifest,
    FastqCorpusFixtureSample, FastqCorpusFixtureSampleValidationReport, MultiGzDecoder, Path,
    Result,
};
use anyhow::Context;
use std::io::BufRead;

pub(super) fn validate_fastq_corpus_fixture_sample(
    repo_root: &Path,
    manifest_dir: &Path,
    manifest: &FastqCorpusFixtureManifest,
    sample: &FastqCorpusFixtureSample,
) -> Result<FastqCorpusFixtureSampleValidationReport> {
    let r1_path = resolve_manifest_relative_path(manifest_dir, &sample.r1_path);
    validate_fastq_fixture_path(&r1_path, manifest.compression, &sample.sample_id, "r1_path")?;
    let observed_read_count_r1 = count_fastq_gz_reads(&r1_path)?;
    if observed_read_count_r1 != sample.expected_read_count_r1 {
        return Err(anyhow!(
            "FASTQ corpus fixture sample `{}` expected {} reads in R1 but observed {}",
            sample.sample_id,
            sample.expected_read_count_r1,
            observed_read_count_r1
        ));
    }

    let (r2_path, observed_read_count_r2) = match (&sample.layout, &sample.r2_path) {
        (FastqCorpusFixtureLayout::Pe, Some(r2_relative_path)) => {
            let r2_path = resolve_manifest_relative_path(manifest_dir, r2_relative_path);
            validate_fastq_fixture_path(
                &r2_path,
                manifest.compression,
                &sample.sample_id,
                "r2_path",
            )?;
            let observed_read_count_r2 = count_fastq_gz_reads(&r2_path)?;
            let expected_read_count_r2 = sample.expected_read_count_r2.ok_or_else(|| {
                anyhow!("paired sample `{}` has no R2 count expectation", sample.sample_id)
            })?;
            if observed_read_count_r2 != expected_read_count_r2 {
                return Err(anyhow!(
                    "FASTQ corpus fixture sample `{}` expected {} reads in R2 but observed {}",
                    sample.sample_id,
                    expected_read_count_r2,
                    observed_read_count_r2
                ));
            }
            (Some(r2_path), Some(observed_read_count_r2))
        }
        _ => (None, None),
    };

    let observed_read_count_total =
        observed_read_count_r1.saturating_add(observed_read_count_r2.unwrap_or(0));
    if observed_read_count_total != sample.expected_read_count_total {
        return Err(anyhow!(
            "FASTQ corpus fixture sample `{}` expected {} total reads but observed {}",
            sample.sample_id,
            sample.expected_read_count_total,
            observed_read_count_total
        ));
    }

    let source_paths = sample
        .source_paths
        .iter()
        .map(|path| {
            let absolute = if path.is_absolute() { path.clone() } else { repo_root.join(path) };
            if !absolute.is_file() {
                return Err(anyhow!(
                    "FASTQ corpus fixture sample `{}` source path is missing: {}",
                    sample.sample_id,
                    absolute.display()
                ));
            }
            Ok(path_relative_to_repo(repo_root, &absolute))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(FastqCorpusFixtureSampleValidationReport {
        sample_id: sample.sample_id.clone(),
        cohort: sample.cohort.clone(),
        layout: sample.layout.as_str().to_string(),
        r1_path: path_relative_to_repo(repo_root, &r1_path),
        r2_path: r2_path.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        source_paths,
        observed_read_count_r1,
        observed_read_count_r2,
        observed_read_count_total,
        valid: true,
    })
}

pub(crate) fn validate_fastq_fixture_path(
    path: &Path,
    compression: FastqCorpusFixtureCompression,
    sample_id: &str,
    field_name: &str,
) -> Result<()> {
    if !path.is_file() {
        return Err(anyhow!(
            "FASTQ corpus fixture sample `{sample_id}` is missing {} file {}",
            field_name,
            path.display()
        ));
    }
    if !path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(compression.expected_suffix()))
    {
        return Err(anyhow!(
            "FASTQ corpus fixture sample `{sample_id}` {} must end with `{}`",
            field_name,
            compression.expected_suffix()
        ));
    }
    Ok(())
}

pub(crate) fn count_fastq_gz_reads(path: &Path) -> Result<u64> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let decoder = MultiGzDecoder::new(file);
    let reader = BufReader::new(decoder);
    let mut lines = reader.lines();
    let mut count = 0_u64;
    while let Some(header) = lines.next() {
        let header = header.with_context(|| format!("read {}", path.display()))?;
        let Some(sequence) = lines.next() else {
            return Err(anyhow!("truncated FASTQ record in {}", path.display()));
        };
        let sequence = sequence.with_context(|| format!("read {}", path.display()))?;
        let Some(separator) = lines.next() else {
            return Err(anyhow!("truncated FASTQ record in {}", path.display()));
        };
        let separator = separator.with_context(|| format!("read {}", path.display()))?;
        let Some(quality) = lines.next() else {
            return Err(anyhow!("truncated FASTQ record in {}", path.display()));
        };
        let quality = quality.with_context(|| format!("read {}", path.display()))?;

        if !header.starts_with('@') {
            return Err(anyhow!("invalid FASTQ header in {}: `{header}`", path.display()));
        }
        if !separator.starts_with('+') {
            return Err(anyhow!("invalid FASTQ separator in {}: `{separator}`", path.display()));
        }
        if sequence.len() != quality.len() {
            return Err(anyhow!("FASTQ sequence/quality length mismatch in {}", path.display()));
        }
        count = count.saturating_add(1);
    }
    Ok(count)
}
