//! Owner: bijux-dna-bench-corpus
//! Amplicon truth-bundle observations.
use super::{
    anyhow, fs, load_amplicon_corpus_fixture_manifest_path, load_validated_amplicon_abundance_rows,
    load_validated_amplicon_expected_asv_rows, load_validated_amplicon_expected_chimera_rows,
    load_validated_amplicon_primer_rows, resolve_repo_relative_path,
    validate_amplicon_corpus_fixture_manifest_contract,
    validate_amplicon_corpus_fixture_manifest_path, AmpliconTruthBundle, AmpliconTruthManifest,
    BufRead, BufReader, Context, FastaSequenceTruth, NormalizedAbundanceTruthRow, Path, Result,
    AMPLICON_TRUTH_BUNDLE_SCHEMA_VERSION, OTU_ABUNDANCE_TABLE_KIND,
};
pub(crate) fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &AmpliconTruthManifest,
) -> Result<AmpliconTruthBundle> {
    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    let corpus_manifest = load_amplicon_corpus_fixture_manifest_path(&corpus_manifest_path)?;
    validate_amplicon_corpus_fixture_manifest_contract(&corpus_manifest)?;
    validate_amplicon_corpus_fixture_manifest_path(repo_root, &corpus_manifest_path)?;

    let primer_truths =
        load_validated_amplicon_primer_rows(repo_root, &corpus_manifest_path, &corpus_manifest)?;
    let asv_truths = load_validated_amplicon_expected_asv_rows(
        repo_root,
        &corpus_manifest_path,
        &corpus_manifest,
    )?;
    let chimera_truths = load_validated_amplicon_expected_chimera_rows(
        repo_root,
        &corpus_manifest_path,
        &corpus_manifest,
    )?;
    let otu_abundances = load_validated_amplicon_abundance_rows(
        repo_root,
        &corpus_manifest_path,
        &corpus_manifest,
        OTU_ABUNDANCE_TABLE_KIND,
    )?;
    let asv_representatives = load_fasta_truth_rows(&resolve_repo_relative_path(
        repo_root,
        &manifest.asv_representatives_path,
    ))?;
    let non_chimeric_representatives = load_fasta_truth_rows(&resolve_repo_relative_path(
        repo_root,
        &manifest.non_chimeric_representatives_path,
    ))?;
    let otu_representatives = load_fasta_truth_rows(&resolve_repo_relative_path(
        repo_root,
        &manifest.otu_representatives_path,
    ))?;
    let normalized_abundances = load_normalized_abundance_rows(&resolve_repo_relative_path(
        repo_root,
        &manifest.normalized_abundance_path,
    ))?;

    Ok(AmpliconTruthBundle {
        schema_version: AMPLICON_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: manifest.fixture_id.clone(),
        primer_truths,
        asv_truths,
        chimera_truths,
        asv_representatives,
        non_chimeric_representatives,
        otu_representatives,
        otu_abundances,
        normalized_abundances,
    })
}

pub(crate) fn load_fasta_truth_rows(path: &Path) -> Result<Vec<FastaSequenceTruth>> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();
    let mut current_id = None::<String>;
    let mut current_sequence = String::new();

    for line in reader.lines() {
        let line = line.with_context(|| format!("read {}", path.display()))?;
        if let Some(header) = line.strip_prefix('>') {
            if let Some(id) = current_id.take() {
                rows.push(FastaSequenceTruth { id, sequence: current_sequence.clone() });
                current_sequence.clear();
            }
            current_id = Some(header.trim().to_string());
        } else if !line.trim().is_empty() {
            current_sequence.push_str(line.trim());
        }
    }
    if let Some(id) = current_id.take() {
        rows.push(FastaSequenceTruth { id, sequence: current_sequence });
    }
    if rows.is_empty() {
        return Err(anyhow!("amplicon truth FASTA is empty: {}", path.display()));
    }
    Ok(rows)
}

pub(crate) fn load_normalized_abundance_rows(
    path: &Path,
) -> Result<Vec<NormalizedAbundanceTruthRow>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        anyhow!("amplicon truth normalized abundance table is empty: {}", path.display())
    })?;
    if header != "sample_id\tfeature_id\tnormalized_abundance" {
        return Err(anyhow!(
            "amplicon truth normalized abundance header is unexpected in {}",
            path.display()
        ));
    }
    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let mut fields = line.split('\t');
        let sample_id = fields
            .next()
            .ok_or_else(|| anyhow!("missing sample_id field in {}", path.display()))?;
        let feature_id = fields
            .next()
            .ok_or_else(|| anyhow!("missing feature_id field in {}", path.display()))?;
        let normalized_abundance = fields
            .next()
            .ok_or_else(|| anyhow!("missing normalized_abundance field in {}", path.display()))?;
        if fields.next().is_some() {
            return Err(anyhow!(
                "amplicon truth normalized abundance row has too many columns in {}",
                path.display()
            ));
        }
        rows.push(NormalizedAbundanceTruthRow {
            sample_id: sample_id.to_string(),
            feature_id: feature_id.to_string(),
            normalized_abundance: normalized_abundance.to_string(),
        });
    }
    if rows.is_empty() {
        return Err(anyhow!(
            "amplicon truth normalized abundance table must declare at least one row in {}",
            path.display()
        ));
    }
    Ok(rows)
}
