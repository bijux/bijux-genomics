//! Owner: bijux-dna-bench-corpus
//! BAM corpus alignment ownership.
use super::{
    anyhow, fs, inspect_tiny_alignment, path_relative_to_repo, resolve_manifest_relative_path,
    BTreeSet, BamCorpusFixtureSample, BamCorpusFixtureSampleValidationReport, Path, Result,
};
use anyhow::Context;

pub(super) fn validate_bam_corpus_fixture_sample(
    repo_root: &Path,
    manifest_dir: &Path,
    reference_contigs: &BTreeSet<String>,
    sample: &BamCorpusFixtureSample,
) -> Result<BamCorpusFixtureSampleValidationReport> {
    let (alignment_path, index_path) = resolve_alignment_inputs(manifest_dir, sample)?;

    let document = inspect_tiny_alignment(&alignment_path)?;
    if document.sort_order.as_deref() != Some("coordinate") {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` must be coordinate-sorted",
            sample.sample_id
        ));
    }
    if document.header_contigs.is_empty() {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` has no `@SQ` contigs",
            sample.sample_id
        ));
    }
    if document.header_sample_ids.is_empty() {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` has no `SM` sample IDs in `@RG` records",
            sample.sample_id
        ));
    }
    if document.read_group_ids.is_empty() {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` has no `@RG` read-group IDs",
            sample.sample_id
        ));
    }
    if document.record_count != sample.expected_record_count {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` expected {} records but observed {}",
            sample.sample_id,
            sample.expected_record_count,
            document.record_count
        ));
    }

    let observed_contigs = document.header_contigs.iter().cloned().collect::<BTreeSet<_>>();
    let declared_contigs = sample.expected_contigs.iter().cloned().collect::<BTreeSet<_>>();
    if observed_contigs != declared_contigs {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` expected contigs {:?} but observed {:?}",
            sample.sample_id,
            sample.expected_contigs,
            document.header_contigs
        ));
    }
    if !observed_contigs.iter().all(|contig| reference_contigs.contains(contig)) {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` has contigs absent from the reference FASTA",
            sample.sample_id
        ));
    }

    let observed_header_sample_ids =
        document.header_sample_ids.iter().cloned().collect::<BTreeSet<_>>();
    let declared_header_sample_ids =
        sample.expected_header_sample_ids.iter().cloned().collect::<BTreeSet<_>>();
    if observed_header_sample_ids != declared_header_sample_ids {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` expected header sample IDs {:?} but observed {:?}",
            sample.sample_id,
            sample.expected_header_sample_ids,
            document.header_sample_ids
        ));
    }

    let observed_read_group_ids = document.read_group_ids.iter().cloned().collect::<BTreeSet<_>>();
    let declared_read_group_ids =
        sample.expected_read_group_ids.iter().cloned().collect::<BTreeSet<_>>();
    if observed_read_group_ids != declared_read_group_ids {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` expected read-group IDs {:?} but observed {:?}",
            sample.sample_id,
            sample.expected_read_group_ids,
            document.read_group_ids
        ));
    }

    let mapped_record_contigs =
        document.mapped_record_contigs.iter().cloned().collect::<BTreeSet<_>>();
    if !mapped_record_contigs.iter().all(|contig| observed_contigs.contains(contig)) {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` has mapped records on contigs absent from `@SQ`",
            sample.sample_id
        ));
    }

    let source_paths = crate::paths::validate_source_paths(
        repo_root,
        &sample.source_paths,
        &format!("BAM corpus fixture sample `{}`", sample.sample_id),
    )?;

    Ok(BamCorpusFixtureSampleValidationReport {
        sample_id: sample.sample_id.clone(),
        cohort: sample.cohort.clone(),
        alignment_path: path_relative_to_repo(repo_root, &alignment_path),
        index_path: path_relative_to_repo(repo_root, &index_path),
        source_paths,
        observed_contigs: observed_contigs.into_iter().collect(),
        observed_header_sample_ids: observed_header_sample_ids.into_iter().collect(),
        observed_read_group_ids: observed_read_group_ids.into_iter().collect(),
        observed_record_count: document.record_count,
        valid: true,
    })
}

pub(super) fn parse_reference_contigs(reference_fasta: &Path) -> Result<BTreeSet<String>> {
    let payload = fs::read_to_string(reference_fasta)
        .with_context(|| format!("read {}", reference_fasta.display()))?;
    let mut contigs = BTreeSet::new();
    for line in payload.lines() {
        if let Some(header) = line.strip_prefix('>') {
            let contig = header.split_whitespace().next().unwrap_or_default().trim();
            if !contig.is_empty() {
                contigs.insert(contig.to_string());
            }
        }
    }
    Ok(contigs)
}

#[expect(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "Governed corpus filenames have case-sensitive spelling."
)]
fn resolve_alignment_inputs(
    manifest_dir: &Path,
    sample: &BamCorpusFixtureSample,
) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    let alignment_path = resolve_manifest_relative_path(manifest_dir, &sample.alignment_path);
    if !alignment_path.is_file() {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` is missing alignment file {}",
            sample.sample_id,
            alignment_path.display()
        ));
    }
    if !alignment_path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".sam") || name.ends_with(".bam"))
    {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` alignment path must end with `.sam` or `.bam`",
            sample.sample_id
        ));
    }

    let index_path = resolve_manifest_relative_path(manifest_dir, &sample.index_path);
    if !index_path.is_file() {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` is missing index file {}",
            sample.sample_id,
            index_path.display()
        ));
    }
    if !index_path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".bai"))
    {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` index path must end with `.bai`",
            sample.sample_id
        ));
    }
    if fs::metadata(&index_path).with_context(|| format!("stat {}", index_path.display()))?.len()
        == 0
    {
        return Err(anyhow!(
            "BAM corpus fixture sample `{}` index file must not be empty",
            sample.sample_id
        ));
    }

    Ok((alignment_path, index_path))
}
