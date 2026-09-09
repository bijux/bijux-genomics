//! Owner: bijux-dna-bench
//! Corpus admission regressions.

use super::*;
use std::io::Write;

fn source_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/corpora/human-short-read-diversity.toml")
}

fn corpus() -> Result<(tempfile::TempDir, CorpusSpec, BTreeMap<String, String>)> {
    let root = tempfile::tempdir()?;
    fs::create_dir(root.path().join("normalized"))?;
    let mut spec = load_fastq_corpus_source_spec(&source_path())?;
    spec.target_total = 4;
    spec.target_ancient_se = Some(1);
    spec.target_ancient_pe = Some(1);
    spec.target_modern_se = Some(1);
    spec.target_modern_pe = Some(1);
    let template = spec.samples[0].clone();
    spec.samples.clear();
    let mut files = BTreeMap::new();
    for (sample, era, layout) in [
        ("d", "modern", CorpusAcquisitionLayout::Pe),
        ("b", "ancient", CorpusAcquisitionLayout::Pe),
        ("c", "modern", CorpusAcquisitionLayout::Se),
        ("a", "ancient", CorpusAcquisitionLayout::Se),
    ] {
        let accession = format!("accession_{sample}");
        let mut metadata = template.clone();
        metadata.accession = accession.clone();
        metadata.era = era.to_string();
        metadata.layout = layout;
        spec.samples.push(metadata);
        fs::create_dir_all(root.path().join("raw").join(&accession))?;
        let mates = if layout == CorpusAcquisitionLayout::Pe { 2 } else { 1 };
        for mate in 1..=mates {
            let name = format!("{sample}_R{mate}.fastq.gz");
            let mut gzip =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            write!(gzip, "@{sample}/{mate}\nACGT\n+\nIIII\n")?;
            let bytes = gzip.finish()?;
            let digest = bijux_dna_infra::hash_bytes_sha256(&bytes);
            for relative in [format!("normalized/{name}"), format!("raw/{accession}/{name}")] {
                fs::write(root.path().join(&relative), &bytes)?;
                files.insert(relative, digest.clone());
            }
        }
    }
    write_manifest(root.path(), &files)?;
    Ok((root, spec, files))
}

fn write_manifest(root: &Path, files: &BTreeMap<String, String>) -> Result<()> {
    fs::write(
        root.join("MANIFEST.json"),
        serde_json::to_vec(&serde_json::json!({"files": files}))?,
    )?;
    Ok(())
}

#[test]
fn discovers_ordered_reads_matches_accessions_and_selects_both_paired_cohorts() -> Result<()> {
    let (root, spec, _) = corpus()?;
    let samples = discover_normalized_fastq_samples(
        root.path(),
        &spec.corpus_id,
        normalized_fastq_expected_sample_total(&spec),
    )?;
    assert_eq!(
        samples.iter().map(|row| row.sample_id.as_str()).collect::<Vec<_>>(),
        ["a", "b", "c", "d"]
    );
    assert_eq!(
        samples.iter().map(|row| row.layout.as_str()).collect::<Vec<_>>(),
        ["se", "pe", "se", "pe"]
    );
    let metadata = match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)?;
    for sample in &samples {
        assert_eq!(
            metadata[&sample.sample_id].accession,
            format!("accession_{}", sample.sample_id)
        );
    }
    let paired = select_normalized_fastq_paired_samples(&spec, &samples, &metadata)?;
    assert_eq!(paired.iter().map(|row| row.sample_id.as_str()).collect::<Vec<_>>(), ["b", "d"]);
    Ok(())
}

#[test]
fn rejects_missing_read_roots_orphan_mates_and_sample_count_drift() -> Result<()> {
    let empty = tempfile::tempdir()?;
    assert!(discover_normalized_fastq_samples(empty.path(), "corpus", 0).is_err());
    let (root, spec, _) = corpus()?;
    assert!(discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 3).is_err());
    fs::remove_file(root.path().join("normalized/b_R1.fastq.gz"))?;
    assert!(discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)
        .unwrap_err()
        .to_string()
        .contains("missing R1"));
    Ok(())
}

#[test]
fn rejects_missing_ambiguous_and_unknown_manifest_accessions() -> Result<()> {
    let (root, spec, files) = corpus()?;
    let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
    let key = "raw/accession_a/a_R1.fastq.gz";
    let mut missing = files.clone();
    let digest = missing.remove(key).context("raw digest")?;
    write_manifest(root.path(), &missing)?;
    assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)
        .unwrap_err()
        .to_string()
        .contains("missing accession"));
    missing.insert("raw/unlisted/a_R1.fastq.gz".to_string(), digest.clone());
    write_manifest(root.path(), &missing)?;
    assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)
        .unwrap_err()
        .to_string()
        .contains("missing curated metadata"));
    missing.insert(key.to_string(), digest);
    write_manifest(root.path(), &missing)?;
    assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)
        .unwrap_err()
        .to_string()
        .contains("expected one accession"));
    Ok(())
}

#[test]
fn rejects_incomplete_cohort_authority_and_count_drift() -> Result<()> {
    let (root, mut spec, _) = corpus()?;
    assert!(expected_fastq_publication_cohort_counts(&spec, "unknown").is_err());
    spec.target_ancient_se = None;
    assert!(expected_fastq_publication_cohort_counts(&spec, "full").is_err());
    spec.target_ancient_se = Some(2);
    let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
    assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
    Ok(())
}

#[test]
fn rejects_requested_corpus_identity_mismatch() -> Result<()> {
    let spec = load_fastq_corpus_source_spec(&source_path())?;
    assert_eq!(
        load_fastq_corpus_source_for_id(&source_path(), &spec.corpus_id)?.corpus_id,
        spec.corpus_id
    );
    assert!(load_fastq_corpus_source_for_id(&source_path(), "different-corpus").is_err());
    Ok(())
}

#[test]
fn normalized_metadata_rejects_conflicting_mate_accessions() -> Result<()> {
    let (root, spec, mut files) = corpus()?;
    let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
    files.insert(
        "normalized/b_R1.fastq.gz".to_string(),
        files["raw/accession_d/d_R1.fastq.gz"].clone(),
    );
    write_manifest(root.path(), &files)?;
    assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
    Ok(())
}

#[test]
fn normalized_metadata_rejects_a_missing_curated_mate() -> Result<()> {
    let (root, spec, mut files) = corpus()?;
    fs::remove_file(root.path().join("normalized/b_R2.fastq.gz"))?;
    files.remove("normalized/b_R2.fastq.gz");
    write_manifest(root.path(), &files)?;
    let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
    assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
    Ok(())
}

#[test]
fn normalized_metadata_rejects_incoherent_sample_layout() -> Result<()> {
    let (root, spec, _) = corpus()?;
    let mut samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
    samples[1].layout = "se".to_string();
    assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
    Ok(())
}

#[test]
fn normalized_metadata_rejects_missing_selection_metadata() -> Result<()> {
    let (root, spec, _) = corpus()?;
    let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
    let mut metadata = match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)?;
    metadata.remove("a");
    assert!(select_normalized_fastq_paired_samples(&spec, &samples, &metadata).is_err());
    Ok(())
}

#[test]
fn normalized_metadata_accepts_zero_target_paired_cohorts() -> Result<()> {
    let (root, mut spec, _) = corpus()?;
    let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
    let mut metadata = match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)?;
    spec.target_total = 1;
    spec.target_ancient_se = Some(0);
    spec.target_ancient_pe = Some(0);
    spec.target_modern_se = Some(0);
    spec.target_modern_pe = Some(1);
    spec.samples.retain(|row| row.accession == "accession_d");
    metadata.retain(|id, _| id == "d");
    let selected = select_normalized_fastq_paired_samples(&spec, &[samples[3].clone()], &metadata)?;
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].sample_id, "d");
    Ok(())
}

#[test]
fn normalized_metadata_groups_equal_mate_digests_by_accession() -> Result<()> {
    let (root, spec, mut files) = corpus()?;
    let mut gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    gzip.write_all(b"@b\nACGT\n+\nIIII\n")?;
    let bytes = gzip.finish()?;
    let digest = bijux_dna_infra::hash_bytes_sha256(&bytes);
    for mate in [1, 2] {
        for relative in [
            format!("normalized/b_R{mate}.fastq.gz"),
            format!("raw/accession_b/b_R{mate}.fastq.gz"),
        ] {
            fs::write(root.path().join(&relative), &bytes)?;
            files.insert(relative, digest.clone());
        }
    }
    write_manifest(root.path(), &files)?;
    let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
    let metadata = match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)?;
    assert_eq!(metadata["b"].accession, "accession_b");
    Ok(())
}

#[test]
fn rejects_repeated_source_sample_and_accession_identities() -> Result<()> {
    let (root, mut spec, _) = corpus()?;
    let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
    let metadata = match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)?;
    let mut duplicate_samples = samples.clone();
    duplicate_samples[1] = samples[0].clone();
    assert!(select_normalized_fastq_paired_samples(&spec, &duplicate_samples, &metadata).is_err());
    let mut duplicate_accessions = metadata.clone();
    duplicate_accessions.insert("c".to_string(), metadata["a"].clone());
    assert!(select_normalized_fastq_paired_samples(&spec, &samples, &duplicate_accessions).is_err());
    spec.samples[1] = spec.samples[0].clone();
    assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
    Ok(())
}

#[test]
fn rejects_missing_manifest_mates_and_redirected_sample_paths() -> Result<()> {
    let (root, spec, mut files) = corpus()?;
    let mut samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
    samples[1].r1 = samples[3].r1.clone();
    assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
    samples[1].r1 = root.path().join("normalized/b_R1.fastq.gz");
    files.remove("normalized/b_R1.fastq.gz");
    write_manifest(root.path(), &files)?;
    assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
    Ok(())
}

#[test]
fn rejects_stale_normalized_and_raw_content() -> Result<()> {
    for relative in ["normalized/b_R1.fastq.gz", "raw/accession_b/b_R1.fastq.gz"] {
        let (root, spec, _) = corpus()?;
        let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
        fs::write(root.path().join(relative), b"changed bytes")?;
        let err = match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).unwrap_err();
        assert!(err.to_string().contains("checksum mismatch"), "{err:#}");
    }
    Ok(())
}

#[test]
fn rejects_ambiguous_or_unsupported_manifest_documents() -> Result<()> {
    let (root, spec, files) = corpus()?;
    let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
    let map = serde_json::to_string(&files)?;
    let digest = &files["normalized/a_R1.fastq.gz"];
    for document in [
        format!("{{\"files\":{map},\"files\":{map}}}"),
        format!("{{\"files\":{{\"a\":\"{digest}\",\"a\":\"{digest}\"}}}}"),
        format!("{{\"schema_version\":\"unknown\",\"files\":{map}}}"),
        format!("{{\"unknown\":true,\"files\":{map}}}"),
        "{}".to_string(),
        "{\"files\":{}}".to_string(),
    ] {
        fs::write(root.path().join("MANIFEST.json"), document)?;
        assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
    }
    // Both the explicit schema and its supported legacy omission have the same identity contract.
    fs::write(
        root.path().join("MANIFEST.json"),
        format!("{{\"schema_version\":\"bijux.corpus_manifest.v1\",\"files\":{map}}}"),
    )?;
    match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)?;
    Ok(())
}

#[test]
fn rejects_noncanonical_manifest_paths_and_digests() -> Result<()> {
    let (root, spec, files) = corpus()?;
    let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
    for path in
        ["../outside", "/absolute", "raw/../outside", "raw/./file", "raw//file", "raw\\file", ""]
    {
        let mut invalid = files.clone();
        invalid.insert(path.to_string(), "a".repeat(64));
        write_manifest(root.path(), &invalid)?;
        assert!(
            match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err(),
            "{path}"
        );
    }
    for digest in ["", "abc", &"A".repeat(64), &"g".repeat(64)] {
        let mut invalid = files.clone();
        invalid.insert("normalized/a_R1.fastq.gz".to_string(), digest.to_string());
        write_manifest(root.path(), &invalid)?;
        assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
    }
    Ok(())
}

#[test]
fn rejects_oversized_manifest_before_decoding() -> Result<()> {
    let (root, spec, _) = corpus()?;
    let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
    let file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(root.path().join("MANIFEST.json"))?;
    file.set_len(64 * 1024 * 1024 + 1)?;
    let err = match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).unwrap_err();
    assert!(err.to_string().contains("exceeds"), "{err:#}");
    Ok(())
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_manifest_reads_and_raw_directories() -> Result<()> {
    for relative in ["MANIFEST.json", "normalized/a_R1.fastq.gz", "raw/accession_b"] {
        let (root, spec, _) = corpus()?;
        let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
        let original = root.path().join(relative);
        let saved = root.path().join("preserved");
        fs::rename(&original, &saved)?;
        std::os::unix::fs::symlink(&saved, &original)?;
        let err = match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).unwrap_err();
        assert!(err.to_string().contains("symlink"), "{err:#}");
    }
    Ok(())
}
