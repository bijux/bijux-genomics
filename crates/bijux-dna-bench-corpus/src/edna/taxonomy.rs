//! Owner: bijux-dna-bench-corpus
//! EDNA corpus taxonomy ownership.
use super::{
    anyhow, fs, resolve_manifest_relative_path, BTreeSet, EdnaCorpusFixtureManifest,
    EdnaExpectedPresence, EdnaExpectedTaxonRow, Path, PathBuf, Result,
};
use anyhow::Context;

/// Resolve the declared corpus resource against the manifest directory.
///
/// # Errors
/// Rejects a manifest path without a parent directory.
pub fn resolve_edna_expected_taxa_path(
    manifest_path: &Path,
    manifest: &EdnaCorpusFixtureManifest,
) -> Result<PathBuf> {
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    Ok(resolve_manifest_relative_path(manifest_dir, &manifest.expected_taxa_path))
}

/// Load declared rows and validate their identities, values and coverage.
///
/// # Errors
/// Rejects unreadable or malformed tables and rows that contradict the corpus declarations.
pub fn load_validated_edna_expected_taxa_rows(
    manifest: &EdnaCorpusFixtureManifest,
    expected_taxa_path: &Path,
) -> Result<Vec<EdnaExpectedTaxonRow>> {
    if !expected_taxa_path.is_file() {
        return Err(anyhow!(
            "eDNA corpus fixture expected taxonomy output is missing: {}",
            expected_taxa_path.display()
        ));
    }
    let raw = fs::read_to_string(expected_taxa_path)
        .with_context(|| format!("read {}", expected_taxa_path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        anyhow!(
            "eDNA corpus fixture expected taxonomy output is empty: {}",
            expected_taxa_path.display()
        )
    })?;
    if header != "sample_id\ttaxon_id\tname\trank\texpected_presence" {
        return Err(anyhow!(
            "eDNA corpus fixture expected taxonomy output header is unexpected in {}",
            expected_taxa_path.display()
        ));
    }

    let manifest_samples =
        manifest.samples.iter().map(|sample| sample.sample_id.as_str()).collect::<BTreeSet<_>>();
    let manifest_taxa = manifest
        .expected_taxa
        .iter()
        .map(|taxon| (taxon.taxon_id, taxon.name.as_str(), taxon.rank.as_str()))
        .collect::<BTreeSet<_>>();

    let rows = lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| parse_expected_taxon_row(line, expected_taxa_path))
        .collect::<Result<Vec<_>>>()?;

    let expected_row_count = manifest.samples.len().saturating_mul(manifest.expected_taxa.len());
    if rows.len() != expected_row_count {
        return Err(anyhow!(
            "eDNA expected taxonomy output must declare {} sample/taxon rows, observed {}",
            expected_row_count,
            rows.len()
        ));
    }

    let mut observed_pairs = BTreeSet::new();
    for row in &rows {
        if !manifest_samples.contains(row.sample_id.as_str()) {
            return Err(anyhow!(
                "eDNA expected taxonomy output sample_id `{}` is not declared by the manifest",
                row.sample_id
            ));
        }
        if !manifest_taxa.contains(&(row.taxon_id, row.name.as_str(), row.rank.as_str())) {
            return Err(anyhow!(
                "eDNA expected taxonomy output taxon `{}` / `{}` / `{}` is not declared by the manifest",
                row.taxon_id,
                row.name,
                row.rank
            ));
        }
        if !observed_pairs.insert((row.sample_id.as_str(), row.taxon_id)) {
            return Err(anyhow!(
                "eDNA expected taxonomy output repeats sample_id `{}` and taxon_id `{}`",
                row.sample_id,
                row.taxon_id
            ));
        }
    }

    for sample_id in &manifest_samples {
        for taxon in &manifest.expected_taxa {
            if !observed_pairs.contains(&(sample_id, taxon.taxon_id)) {
                return Err(anyhow!(
                    "eDNA expected taxonomy output is missing sample_id `{}` and taxon_id `{}`",
                    sample_id,
                    taxon.taxon_id
                ));
            }
        }
    }
    Ok(rows)
}

fn parse_expected_taxon_row(line: &str, expected_taxa_path: &Path) -> Result<EdnaExpectedTaxonRow> {
    let mut fields = line.split('\t');
    let sample_id = fields
        .next()
        .ok_or_else(|| anyhow!("missing sample_id field in {}", expected_taxa_path.display()))?
        .to_string();
    let taxon_id = fields
        .next()
        .ok_or_else(|| anyhow!("missing taxon_id field in {}", expected_taxa_path.display()))?
        .parse::<u64>()
        .with_context(|| format!("parse taxon_id in {}", expected_taxa_path.display()))?;
    let name = fields
        .next()
        .ok_or_else(|| anyhow!("missing name field in {}", expected_taxa_path.display()))?
        .to_string();
    let rank = fields
        .next()
        .ok_or_else(|| anyhow!("missing rank field in {}", expected_taxa_path.display()))?
        .to_string();
    let expected_presence = EdnaExpectedPresence::parse(fields.next().ok_or_else(|| {
        anyhow!("missing expected_presence field in {}", expected_taxa_path.display())
    })?)?;
    if fields.next().is_some() {
        return Err(anyhow!(
            "eDNA expected taxonomy output row has too many columns in {}",
            expected_taxa_path.display()
        ));
    }
    Ok(EdnaExpectedTaxonRow { sample_id, taxon_id, name, rank, expected_presence })
}
