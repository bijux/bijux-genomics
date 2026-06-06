use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::commands::benchmark::local_corpus_fixture::vcf::{
    load_vcf_corpus_fixture_manifest_path, DEFAULT_VCF_MINI_MANIFEST_PATH,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfFixtureInputs {
    pub(crate) reference_fasta_path: String,
    pub(crate) reference_fasta_index_path: String,
    pub(crate) raw_vcf_path: String,
    pub(crate) filtered_vcf_path: String,
    pub(crate) multisample_vcf_path: String,
    pub(crate) phased_vcf_path: String,
    pub(crate) panel_vcf_path: String,
    pub(crate) target_sites_bed_path: String,
    pub(crate) sample_metadata_path: String,
    pub(crate) population_metadata_path: String,
}

pub(crate) fn load_governed_vcf_fixture_inputs(
    repo_root: &Path,
) -> Result<GovernedVcfFixtureInputs> {
    let manifest_path = repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH);
    let manifest = load_vcf_corpus_fixture_manifest_path(&manifest_path)
        .with_context(|| format!("load governed VCF fixture from {}", manifest_path.display()))?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("VCF fixture manifest has no parent: {}", manifest_path.display())
    })?;

    Ok(GovernedVcfFixtureInputs {
        reference_fasta_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.reference_fasta_path),
        ),
        reference_fasta_index_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.reference_fasta_index_path),
        ),
        raw_vcf_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.raw_vcf_path),
        ),
        filtered_vcf_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.filtered_vcf_path),
        ),
        multisample_vcf_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.multisample_vcf_path),
        ),
        phased_vcf_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.phased_vcf_path),
        ),
        panel_vcf_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.panel_vcf_path),
        ),
        target_sites_bed_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.target_sites_bed_path),
        ),
        sample_metadata_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.sample_metadata_path),
        ),
        population_metadata_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.population_metadata_path),
        ),
    })
}

pub(crate) fn materialize_indexed_vcf_input(
    repo_root: &Path,
    source_relative_path: &str,
    staging_root: &Path,
    output_name: &str,
) -> Result<(String, String)> {
    let source_vcf = repo_root.join(source_relative_path);
    let output_root = repo_root.join(staging_root);
    fs::create_dir_all(&output_root)
        .with_context(|| format!("create {}", output_root.display()))?;
    let output_vcfgz = output_root.join(output_name);
    let output_tbi =
        bijux_dna_stages_vcf::vcf_io::vcf_index_bgzip_tabix(&source_vcf, &output_vcfgz)
            .with_context(|| {
                format!(
                    "materialize indexed governed VCF {} into {}",
                    source_vcf.display(),
                    output_vcfgz.display()
                )
            })?;
    Ok((
        path_relative_to_repo(repo_root, &output_vcfgz),
        path_relative_to_repo(repo_root, &output_tbi),
    ))
}

pub(crate) fn materialize_reference_fasta_with_index(
    repo_root: &Path,
    source_relative_path: &str,
    staging_root: &Path,
) -> Result<(String, String)> {
    let source_fasta = repo_root.join(source_relative_path);
    let output_root = repo_root.join(staging_root);
    fs::create_dir_all(&output_root).with_context(|| format!("create {}", output_root.display()))?;
    let file_name = source_fasta
        .file_name()
        .ok_or_else(|| anyhow!("reference FASTA has no file name: {}", source_fasta.display()))?;
    let materialized_fasta = output_root.join(file_name);
    fs::copy(&source_fasta, &materialized_fasta).with_context(|| {
        format!(
            "copy governed reference {} to {}",
            source_fasta.display(),
            materialized_fasta.display()
        )
    })?;
    let fai_path = PathBuf::from(format!("{}.fai", materialized_fasta.display()));
    let fai_payload = build_fasta_index_payload(&materialized_fasta)?;
    bijux_dna_infra::atomic_write_bytes(&fai_path, fai_payload.as_bytes())?;
    Ok((
        path_relative_to_repo(repo_root, &materialized_fasta),
        path_relative_to_repo(repo_root, &fai_path),
    ))
}

fn resolve_manifest_relative_path(manifest_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}

pub(crate) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn build_fasta_index_payload(fasta_path: &Path) -> Result<String> {
    let raw = fs::read(fasta_path).with_context(|| format!("read {}", fasta_path.display()))?;
    let mut rows = Vec::new();
    let mut offset = 0usize;
    let mut current_name: Option<String> = None;
    let mut current_length = 0usize;
    let mut current_sequence_offset = 0usize;
    let mut current_line_bases = 0usize;
    let mut current_line_width = 0usize;

    for chunk in raw.split_inclusive(|byte| *byte == b'\n') {
        let line_width = chunk.len();
        let line = if chunk.ends_with(b"\n") { &chunk[..chunk.len() - 1] } else { chunk };
        if let Some(header) = line.strip_prefix(b">") {
            if let Some(name) = current_name.take() {
                rows.push((name, current_length, current_sequence_offset, current_line_bases, current_line_width));
            }
            let header_text = std::str::from_utf8(header)
                .with_context(|| format!("decode FASTA header in {}", fasta_path.display()))?;
            let name = header_text
                .split_whitespace()
                .next()
                .ok_or_else(|| anyhow!("empty FASTA header in {}", fasta_path.display()))?;
            current_name = Some(name.to_string());
            current_length = 0;
            current_line_bases = 0;
            current_line_width = 0;
            current_sequence_offset = offset + line_width;
        } else if !line.is_empty() {
            if current_name.is_none() {
                return Err(anyhow!(
                    "FASTA sequence line appears before any header in {}",
                    fasta_path.display()
                ));
            }
            current_length += line.len();
            if current_line_bases == 0 {
                current_line_bases = line.len();
                current_line_width = line_width;
            }
        }
        offset += line_width;
    }

    if let Some(name) = current_name.take() {
        rows.push((name, current_length, current_sequence_offset, current_line_bases, current_line_width));
    }

    if rows.is_empty() {
        return Err(anyhow!("no FASTA records found in {}", fasta_path.display()));
    }

    let payload = rows
        .into_iter()
        .map(|(name, length, sequence_offset, line_bases, line_width)| {
            format!("{name}\t{length}\t{sequence_offset}\t{line_bases}\t{line_width}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!("{payload}\n"))
}
