//! Owner: bijux-dna-bench-corpus
//! EDNA corpus reads ownership.
use super::{
    anyhow, count_fastq_gz_reads, path_relative_to_repo, resolve_manifest_relative_path,
    validate_fastq_fixture_path, EdnaCorpusFixtureManifest, EdnaCorpusFixtureSample,
    EdnaCorpusFixtureSampleValidationReport, Path, Result,
};

pub(super) fn validate_edna_corpus_fixture_sample(
    repo_root: &Path,
    manifest_dir: &Path,
    manifest: &EdnaCorpusFixtureManifest,
    sample: &EdnaCorpusFixtureSample,
) -> Result<EdnaCorpusFixtureSampleValidationReport> {
    let fastq_path = resolve_manifest_relative_path(manifest_dir, &sample.fastq_path);
    validate_fastq_fixture_path(
        &fastq_path,
        manifest.compression,
        &sample.sample_id,
        "fastq_path",
    )?;
    let observed_read_count = count_fastq_gz_reads(&fastq_path)?;
    if observed_read_count != sample.expected_read_count {
        return Err(anyhow!(
            "eDNA corpus fixture sample `{}` expected {} reads but observed {}",
            sample.sample_id,
            sample.expected_read_count,
            observed_read_count
        ));
    }

    let source_paths = sample
        .source_paths
        .iter()
        .map(|path| {
            let absolute = if path.is_absolute() { path.clone() } else { repo_root.join(path) };
            if !absolute.is_file() {
                return Err(anyhow!(
                    "eDNA corpus fixture sample `{}` source path is missing: {}",
                    sample.sample_id,
                    absolute.display()
                ));
            }
            Ok(path_relative_to_repo(repo_root, &absolute))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(EdnaCorpusFixtureSampleValidationReport {
        sample_id: sample.sample_id.clone(),
        community_label: sample.community_label.clone(),
        fastq_path: path_relative_to_repo(repo_root, &fastq_path),
        source_paths,
        observed_read_count,
        valid: true,
    })
}
