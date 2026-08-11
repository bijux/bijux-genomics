//! Owner: bijux-dna-bench
//! Rust-owned materialization of immutable benchmark reference bundles.

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read as _, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::GovernedDataCatalogSnapshotPublication;
use bijux_dna_db_ref::ReferenceBankEntry;
use chrono::{DateTime, FixedOffset};
use flate2::read::GzDecoder;
use serde_json::{json, Value};

use crate::repo::{publish_governed_data_catalog_snapshots, GovernedDataCatalogSnapshotRequest};

const GOVERNED_DATA_CATALOG_SCHEMA_V1: &str = "bijux.data_catalog.v1";

/// Inputs for one checksum-bound reference materialization and catalog snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceBundleMaterializationRequest {
    pub data_root: PathBuf,
    pub source_archive: PathBuf,
    pub reference_id: String,
    pub species_id: String,
    pub build_id: String,
    pub source_commit: String,
    pub previous_registry_path: Option<PathBuf>,
    pub admitted_at_utc: String,
}

#[derive(Debug)]
struct ReferencePaths {
    root: PathBuf,
    fasta: PathBuf,
    fasta_index: PathBuf,
    sequence_dictionary: PathBuf,
    catalog: PathBuf,
}

struct SequenceState {
    name: String,
    length: u64,
    offset: u64,
    line_bases: Option<u64>,
    line_width: Option<u64>,
    saw_short_line: bool,
    md5: md5::Context,
}

/// Materialize one checked-in reference-bank entry and publish its immutable catalog snapshot.
///
/// The source archive is never inferred from its filename: size, SHA-256, and MD5 are checked
/// against `bijux-dna-db-ref`. FASTA normalization, FAI construction, sequence-dictionary
/// construction, catalog publication, and catalog-registry composition are all crate-owned.
/// Existing complete output is revalidated; partial or conflicting output fails closed.
///
/// # Errors
/// Returns an error for stale source authority, path escape, malformed FASTA, identity drift,
/// incomplete prior output, or conflicting immutable publication.
pub fn materialize_checked_in_reference_bundle(
    request: &ReferenceBundleMaterializationRequest,
) -> Result<GovernedDataCatalogSnapshotPublication> {
    let authority =
        bijux_dna_db_ref::resolve_reference_bank(&request.species_id, &request.build_id)
            .context("resolve checked-in reference source authority")?;
    materialize_reference_bundle(request, &authority)
}

fn materialize_reference_bundle(
    request: &ReferenceBundleMaterializationRequest,
    authority: &ReferenceBankEntry,
) -> Result<GovernedDataCatalogSnapshotPublication> {
    validate_request(request, authority)?;
    let data_root = canonical_directory(&request.data_root, "governed data root")?;
    let source_archive =
        governed_regular_file(&request.source_archive, &data_root, "reference source archive")?;
    verify_source_archive(&source_archive, authority)?;
    let paths = reference_paths(&data_root, request, authority);
    let materialized = [
        paths.fasta.as_path(),
        paths.fasta_index.as_path(),
        paths.sequence_dictionary.as_path(),
        paths.catalog.as_path(),
    ];
    let existing = materialized.iter().filter(|path| path.exists() || path.is_symlink()).count();
    if existing == 0 {
        create_reference_files(&source_archive, &paths, authority)?;
        publish_or_revalidate_flat_catalog(request, authority, &source_archive, &paths)?;
    } else if existing == materialized.len() {
        revalidate_existing_reference_files(request, &paths)?;
        publish_or_revalidate_flat_catalog(request, authority, &source_archive, &paths)?;
    } else {
        bail!("reference materialization is incomplete and cannot be inferred or overwritten");
    }

    publish_governed_data_catalog_snapshots(&GovernedDataCatalogSnapshotRequest {
        data_root,
        source_commit: request.source_commit.clone(),
        preserved_at_utc: request.admitted_at_utc.clone(),
        previous_registry_path: request.previous_registry_path.clone(),
    })
}

fn validate_request(
    request: &ReferenceBundleMaterializationRequest,
    authority: &ReferenceBankEntry,
) -> Result<()> {
    validate_identifier(&request.reference_id, "reference")?;
    if request.species_id != authority.species_id || request.build_id != authority.build_id {
        bail!("requested reference species/build differs from its source authority");
    }
    if request.source_commit.len() != 40
        || !request
            .source_commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("reference materialization source commit is malformed");
    }
    let admitted_at = DateTime::<FixedOffset>::parse_from_rfc3339(&request.admitted_at_utc)
        .context("reference admission timestamp is not RFC 3339")?;
    if admitted_at.offset().local_minus_utc() != 0 {
        bail!("reference admission timestamp must use UTC");
    }
    if authority.fasta_size_bytes == 0
        || authority.fasta_sha256.len() != 64
        || !authority
            .fasta_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || authority.fasta_md5.len() != 32
        || !authority.fasta_md5.bytes().all(|byte| byte.is_ascii_hexdigit())
        || !authority.fasta_url.starts_with("https://")
        || !authority.license_url.starts_with("https://")
    {
        bail!("checked-in reference source authority is incomplete or malformed");
    }
    Ok(())
}

fn verify_source_archive(path: &Path, authority: &ReferenceBankEntry) -> Result<()> {
    let metadata = fs::metadata(path).context("inspect reference source archive")?;
    if metadata.len() != authority.fasta_size_bytes {
        bail!("reference source archive size differs from the checked-in authority");
    }
    let sha256 = bijux_dna_infra::hash_file_sha256(path)
        .context("hash reference source archive with SHA-256")?;
    if sha256 != authority.fasta_sha256 {
        bail!("reference source archive SHA-256 differs from the checked-in authority");
    }
    let mut input = File::open(path).context("open reference source archive for MD5")?;
    let mut digest = md5::Context::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let count = input.read(&mut buffer).context("read reference source archive for MD5")?;
        if count == 0 {
            break;
        }
        digest.consume(&buffer[..count]);
    }
    if format!("{:x}", digest.finalize()) != authority.fasta_md5.to_ascii_lowercase() {
        bail!("reference source archive MD5 differs from the checked-in authority");
    }
    Ok(())
}

fn reference_paths(
    data_root: &Path,
    request: &ReferenceBundleMaterializationRequest,
    authority: &ReferenceBankEntry,
) -> ReferencePaths {
    let root = data_root
        .join("references")
        .join(&request.reference_id)
        .join(format!("source-{}", authority.fasta_sha256));
    ReferencePaths {
        fasta: root.join("reference.fasta"),
        fasta_index: root.join("reference.fasta.fai"),
        sequence_dictionary: root.join("reference.dict"),
        catalog: data_root.join("catalog").join(format!("{}.json", request.reference_id)),
        root,
    }
}

fn create_reference_files(
    source_archive: &Path,
    paths: &ReferencePaths,
    authority: &ReferenceBankEntry,
) -> Result<()> {
    let parent = paths.root.parent().context("reference root lacks a parent")?;
    bijux_dna_infra::ensure_dir(parent)
        .with_context(|| format!("create reference parent {}", parent.display()))?;
    if paths.root.exists() || paths.root.is_symlink() {
        bail!("reference root already exists without a complete materialization");
    }
    let name = paths
        .root
        .file_name()
        .and_then(|value| value.to_str())
        .context("invalid reference root")?;
    let staging = parent.join(format!(".{name}.partial-{}", std::process::id()));
    fs::create_dir(&staging)
        .with_context(|| format!("create reference staging root {}", staging.display()))?;
    let fasta_partial = staging.join("reference.fasta");
    let fai_partial = staging.join("reference.fasta.fai");
    let dict_partial = staging.join("reference.dict");
    let result = (|| -> Result<()> {
        let source = File::open(source_archive).context("open compressed reference source")?;
        let decoded = BufReader::new(GzDecoder::new(source));
        let fasta_file = create_new(&fasta_partial, "normalized FASTA")?;
        let fai_file = create_new(&fai_partial, "FASTA index")?;
        let dict_file = create_new(&dict_partial, "sequence dictionary")?;
        normalize_fasta(
            decoded,
            BufWriter::new(fasta_file),
            BufWriter::new(fai_file),
            BufWriter::new(dict_file),
            &authority.source_accession,
        )?;
        for path in [&fasta_partial, &fai_partial, &dict_partial] {
            make_read_only(path)?;
        }
        fs::rename(&staging, &paths.root).context("publish immutable reference directory")?;
        Ok(())
    })();
    if result.is_err() {
        for path in [&fasta_partial, &fai_partial, &dict_partial] {
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_dir(&staging);
    }
    result
}

fn normalize_fasta<R: BufRead, W: Write>(
    mut input: R,
    mut fasta: W,
    mut fai: W,
    mut dictionary: W,
    source_accession: &str,
) -> Result<()> {
    writeln!(dictionary, "@HD\tVN:1.6\tSO:unsorted")?;
    let mut line = Vec::new();
    let mut output_offset = 0_u64;
    let mut state: Option<SequenceState> = None;
    let mut names = BTreeSet::new();
    let mut sequence_count = 0_u64;
    loop {
        line.clear();
        let read = input.read_until(b'\n', &mut line).context("read compressed FASTA")?;
        if read == 0 {
            break;
        }
        while matches!(line.last(), Some(b'\n' | b'\r')) {
            line.pop();
        }
        if line.first() == Some(&b'>') {
            if let Some(previous) = state.take() {
                finish_sequence(previous, &mut fai, &mut dictionary, source_accession)?;
            }
            let header = std::str::from_utf8(&line).context("FASTA header is not UTF-8")?;
            let name = header[1..]
                .split_ascii_whitespace()
                .next()
                .filter(|value| !value.is_empty())
                .context("FASTA header lacks a sequence name")?
                .to_string();
            if name.contains('\t') || !names.insert(name.clone()) {
                bail!("FASTA sequence name is duplicated or unsafe: {name}");
            }
            fasta.write_all(&line)?;
            fasta.write_all(b"\n")?;
            output_offset = output_offset
                .checked_add(u64::try_from(line.len() + 1)?)
                .context("normalized FASTA offset overflow")?;
            state = Some(SequenceState {
                name,
                length: 0,
                offset: output_offset,
                line_bases: None,
                line_width: None,
                saw_short_line: false,
                md5: md5::Context::new(),
            });
            sequence_count = sequence_count.checked_add(1).context("sequence count overflow")?;
            continue;
        }
        if line.is_empty() {
            bail!("FASTA contains an empty sequence line");
        }
        let current = state.as_mut().context("FASTA sequence data precedes the first header")?;
        if current.saw_short_line {
            bail!("FASTA contains sequence data after a short terminal line");
        }
        if !line.iter().all(|byte| byte.is_ascii_alphabetic()) {
            bail!("FASTA sequence contains non-alphabetic bytes");
        }
        let bases = u64::try_from(line.len()).context("FASTA line length overflow")?;
        match current.line_bases {
            None => {
                current.line_bases = Some(bases);
                current.line_width = Some(bases.checked_add(1).context("FASTA width overflow")?);
            }
            Some(width) if bases > width => {
                bail!("FASTA sequence line exceeds the first-line width")
            }
            Some(width) if bases < width => current.saw_short_line = true,
            Some(_) => {}
        }
        current.length = current.length.checked_add(bases).context("sequence length overflow")?;
        let mut uppercase = line.clone();
        uppercase.make_ascii_uppercase();
        current.md5.consume(&uppercase);
        fasta.write_all(&line)?;
        fasta.write_all(b"\n")?;
        output_offset = output_offset
            .checked_add(bases.checked_add(1).context("normalized FASTA offset overflow")?)
            .context("normalized FASTA offset overflow")?;
    }
    if let Some(previous) = state.take() {
        finish_sequence(previous, &mut fai, &mut dictionary, source_accession)?;
    }
    if sequence_count == 0 {
        bail!("reference FASTA contains no sequences");
    }
    fasta.flush().context("flush normalized FASTA")?;
    fai.flush().context("flush FASTA index")?;
    dictionary.flush().context("flush sequence dictionary")?;
    Ok(())
}

fn finish_sequence<W: Write>(
    state: SequenceState,
    fai: &mut W,
    dictionary: &mut W,
    source_accession: &str,
) -> Result<()> {
    if state.length == 0 {
        bail!("FASTA sequence contains no bases: {}", state.name);
    }
    let line_bases = state.line_bases.context("FASTA sequence lacks a line width")?;
    let line_width = state.line_width.context("FASTA sequence lacks a byte width")?;
    writeln!(
        fai,
        "{}\t{}\t{}\t{}\t{}",
        state.name, state.length, state.offset, line_bases, line_width
    )?;
    writeln!(
        dictionary,
        "@SQ\tSN:{}\tLN:{}\tM5:{:x}\tAS:{}",
        state.name,
        state.length,
        state.md5.finalize(),
        source_accession
    )?;
    Ok(())
}

fn publish_or_revalidate_flat_catalog(
    request: &ReferenceBundleMaterializationRequest,
    authority: &ReferenceBankEntry,
    source_archive: &Path,
    paths: &ReferencePaths,
) -> Result<()> {
    let catalog = expected_catalog(request, authority, source_archive, paths)?;
    if paths.catalog.exists() || paths.catalog.is_symlink() {
        let catalog_path = governed_regular_file(
            &paths.catalog,
            &request.data_root,
            "existing reference catalog",
        )?;
        let observed: Value = serde_json::from_slice(&fs::read(catalog_path)?)
            .context("parse existing reference catalog")?;
        if observed != catalog {
            bail!("existing reference catalog differs from revalidated materialization");
        }
        return Ok(());
    }
    bijux_dna_infra::atomic_create_json(&paths.catalog, &catalog)
        .with_context(|| format!("publish reference catalog {}", paths.catalog.display()))?;
    make_read_only(&paths.catalog)
}

fn expected_catalog(
    request: &ReferenceBundleMaterializationRequest,
    authority: &ReferenceBankEntry,
    source_archive: &Path,
    paths: &ReferencePaths,
) -> Result<Value> {
    Ok(json!({
        "schema_version": GOVERNED_DATA_CATALOG_SCHEMA_V1,
        "id": request.reference_id,
        "purpose": "immutable public human reference assembly for alignment and reference-dependent benchmark campaigns",
        "source_identities": [
            format!("authority:NCBI Genome Reference Consortium {}", authority.source_accession),
            format!("source_url:{}", authority.fasta_url),
            format!("source_sha256:{}", authority.fasta_sha256),
        ],
        "source_url": authority.fasta_url,
        "source_record": format!("NCBI:{}", authority.source_accession),
        "source_accession": authority.source_accession,
        "source_md5": authority.fasta_md5,
        "source_size_bytes": authority.fasta_size_bytes,
        "access_classification": "public",
        "redistribution_classification": "public-reference-assembly",
        "scientific_name": authority.species_id,
        "reference_assembly": format!("{} / {}", authority.build_id, authority.source_accession),
        "admitted_at_utc": request.admitted_at_utc,
        "license_id": authority.license_id,
        "license_url": authority.license_url,
        "allowed_use": "reference-dependent scientific benchmarking and derived index construction",
        "files": [
            file_record(source_archive, "reference-source-archive")?,
            file_record(&paths.fasta, "reference-fasta")?,
            file_record(&paths.fasta_index, "reference-fasta-index")?,
            file_record(&paths.sequence_dictionary, "reference-sequence-dictionary")?,
        ],
        "known_limitations": [
            "This linear assembly does not represent every alternate haplotype or population-specific sequence.",
            "Aligner-specific BWA and Bowtie2 indexes remain derived attempt inputs and must retain this FASTA identity.",
        ],
    }))
}

fn file_record(path: &Path, kind: &str) -> Result<Value> {
    let metadata = fs::metadata(path).with_context(|| format!("inspect {}", path.display()))?;
    Ok(json!({
        "path": path_string(path)?,
        "kind": kind,
        "sha256": bijux_dna_infra::hash_file_sha256(path)?,
        "size_bytes": metadata.len(),
    }))
}

fn revalidate_existing_reference_files(
    request: &ReferenceBundleMaterializationRequest,
    paths: &ReferencePaths,
) -> Result<()> {
    for path in [&paths.fasta, &paths.fasta_index, &paths.sequence_dictionary] {
        let _ = governed_regular_file(path, &request.data_root, "existing reference material")?;
    }
    Ok(())
}

fn create_new(path: &Path, label: &str) -> Result<File> {
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .with_context(|| format!("create {label} {}", path.display()))
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("{label} is not a regular non-symlink directory");
    }
    path.canonicalize().with_context(|| format!("canonicalize {label}"))
}

fn governed_regular_file(path: &Path, root: &Path, label: &str) -> Result<PathBuf> {
    let root = root.canonicalize().with_context(|| format!("canonicalize {label} root"))?;
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} is not a regular non-symlink file: {}", path.display());
    }
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    if !canonical.starts_with(root) {
        bail!("{label} escapes governed data root: {}", path.display());
    }
    Ok(canonical)
}

fn validate_identifier(value: &str, label: &str) -> Result<()> {
    let mut bytes = value.bytes();
    if !bytes.next().is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        || !bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        bail!("{label} identity is malformed");
    }
    Ok(())
}

fn make_read_only(path: &Path) -> Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("make reference material read-only: {}", path.display()))
}

fn path_string(path: &Path) -> Result<String> {
    path.to_str().map(str::to_string).context("governed reference path is not UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::{
        admit_benchmark_reference, build_benchmark_data_index,
        publish_governed_data_catalog_registry, BenchmarkDataIndexRequest,
        GovernedDataCatalogRegistryPublicationRequest,
    };
    use flate2::{write::GzEncoder, Compression};

    #[test]
    fn materialization_builds_admissible_fasta_index_and_dictionary() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let data_root = temporary.path().join("data");
        fs::create_dir_all(data_root.join("catalog"))?;
        fs::create_dir_all(data_root.join("evidence/source-assets"))?;
        let data_root = data_root.canonicalize()?;
        let source = data_root.join("evidence/source-assets/reference.fna.gz");
        let fasta = b">chr1 description\nACGT\nAC\n>chr2\nNN\n";
        let mut encoder = GzEncoder::new(File::create(&source)?, Compression::default());
        encoder.write_all(fasta)?;
        encoder.finish()?;
        let source_bytes = fs::read(&source)?;
        let authority = ReferenceBankEntry {
            species_id: "Test species".to_string(),
            build_id: "TestBuild".to_string(),
            source_accession: "GCF_TEST.1".to_string(),
            fasta_url: "https://example.invalid/reference.fna.gz".to_string(),
            fasta_md5: format!("{:x}", md5::compute(&source_bytes)),
            fasta_sha256: bijux_dna_infra::hash_bytes_sha256(&source_bytes),
            fasta_size_bytes: u64::try_from(source_bytes.len())?,
            license_id: "public-test".to_string(),
            license_url: "https://example.invalid/license".to_string(),
            required_indexes: vec!["samtools_faidx".to_string()],
        };
        let request = ReferenceBundleMaterializationRequest {
            data_root: data_root.clone(),
            source_archive: source,
            reference_id: "test-reference".to_string(),
            species_id: authority.species_id.clone(),
            build_id: authority.build_id.clone(),
            source_commit: "a".repeat(40),
            previous_registry_path: None,
            admitted_at_utc: "2026-08-11T00:00:00Z".to_string(),
        };

        let publication = materialize_reference_bundle(&request, &authority)?;
        let root = data_root
            .join("references/test-reference")
            .join(format!("source-{}", authority.fasta_sha256));
        assert_eq!(
            fs::read_to_string(root.join("reference.fasta.fai"))?,
            "chr1\t6\t18\t4\t5\nchr2\t2\t32\t2\t3\n"
        );
        let dictionary = fs::read_to_string(root.join("reference.dict"))?;
        assert!(
            dictionary.contains(&format!("@SQ\tSN:chr1\tLN:6\tM5:{:x}", md5::compute(b"ACGTAC")))
        );
        assert_eq!(publication.preserved_snapshot_count, 1);
        assert_eq!(materialize_reference_bundle(&request, &authority)?, publication);

        let registry = publish_governed_data_catalog_registry(
            &GovernedDataCatalogRegistryPublicationRequest {
                data_root: data_root.clone(),
                source_commit: request.source_commit.clone(),
            },
        )?;
        let index = build_benchmark_data_index(&BenchmarkDataIndexRequest {
            catalog_registry_path: registry.registry_path,
            data_root: data_root.clone(),
            created_at_utc: "2026-08-11T00:01:00Z".to_string(),
        })?;
        let index_path = data_root.join("data-index.json");
        bijux_dna_infra::atomic_create_json(&index_path, &index)?;
        let admitted = admit_benchmark_reference(
            &temporary.path().canonicalize()?,
            &index_path,
            "test-reference",
        )?;
        assert_eq!(admitted.reference_id, "test-reference");
        assert_eq!(admitted.fasta.size_bytes, u64::try_from(fasta.len())?);
        Ok(())
    }
}
