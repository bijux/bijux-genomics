//! Owner: bijux-dna-bench-corpus
//! AMPLICON corpus truth ownership.
use super::{
    anyhow, fs, path_relative_to_repo, resolve_manifest_relative_path,
    AmpliconCorpusFixtureManifest, AmpliconCorpusSampleKind, AmpliconExpectedAsvRow,
    AmpliconExpectedAsvTruthRow, AmpliconExpectedChimeraRow, AmpliconExpectedChimeraTruthRow,
    AmpliconExpectedPresence, AmpliconPrimerTableRow, AmpliconPrimerTruthRow, BTreeMap, BTreeSet,
    BufReader, Path, Result,
};
use anyhow::Context;
use std::io::BufRead;

/// Load declared rows and validate their identities, values and coverage.
///
/// # Errors
/// Rejects unreadable or malformed tables and rows that contradict the corpus declarations.
pub fn load_validated_amplicon_primer_rows(
    repo_root: &Path,
    manifest_path: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
) -> Result<Vec<AmpliconPrimerTruthRow>> {
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    let primer_fasta_path = resolve_manifest_relative_path(manifest_dir, &manifest.primer_fasta);
    let primers_tsv_path = resolve_manifest_relative_path(manifest_dir, &manifest.primers_tsv_path);
    let primer_fasta_records = validate_primer_fasta_headers(manifest, &primer_fasta_path)?;
    let rows = validate_primer_table_contract(
        repo_root,
        manifest,
        &primers_tsv_path,
        &primer_fasta_records,
    )?;
    Ok(rows
        .into_iter()
        .map(|row| AmpliconPrimerTruthRow {
            primer_id: row.primer_id,
            forward_sequence: row.forward_sequence,
            reverse_sequence: row.reverse_sequence,
            target: row.target,
            orientation: row.orientation,
        })
        .collect())
}

/// Load declared rows and validate their identities, values and coverage.
///
/// # Errors
/// Rejects unreadable or malformed tables and rows that contradict the corpus declarations.
pub fn load_validated_amplicon_expected_asv_rows(
    repo_root: &Path,
    manifest_path: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
) -> Result<Vec<AmpliconExpectedAsvTruthRow>> {
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    let expected_asvs_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.expected_asvs_path);
    let rows = validate_expected_asvs_contract(repo_root, manifest, &expected_asvs_path)?;
    Ok(rows
        .into_iter()
        .map(|row| AmpliconExpectedAsvTruthRow {
            asv_id: row.asv_id,
            sequence: row.sequence,
            sample_id: row.sample_id,
            expected_presence: match row.expected_presence {
                AmpliconExpectedPresence::Present => "present".to_string(),
                AmpliconExpectedPresence::Absent => "absent".to_string(),
            },
        })
        .collect())
}

/// Load declared rows and validate their identities, values and coverage.
///
/// # Errors
/// Rejects unreadable or malformed tables and rows that contradict the corpus declarations.
pub fn load_validated_amplicon_expected_chimera_rows(
    repo_root: &Path,
    manifest_path: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
) -> Result<Vec<AmpliconExpectedChimeraTruthRow>> {
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    let chimera_controls_fasta_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.chimera_controls_fasta_path);
    let chimera_expectations_path =
        resolve_manifest_relative_path(manifest_dir, &manifest.chimera_expectations_path);
    let chimera_control_records = load_fasta_records(&chimera_controls_fasta_path)?;
    let rows = validate_chimera_expectations_contract(
        repo_root,
        manifest,
        &chimera_controls_fasta_path,
        &chimera_expectations_path,
        &chimera_control_records,
    )?;
    Ok(rows
        .into_iter()
        .map(|row| AmpliconExpectedChimeraTruthRow {
            chimera_id: row.chimera_id,
            sequence: row.sequence,
            sample_id: row.sample_id,
            expected_presence: match row.expected_presence {
                AmpliconExpectedPresence::Present => "present".to_string(),
                AmpliconExpectedPresence::Absent => "absent".to_string(),
            },
        })
        .collect())
}

pub(super) fn validate_primer_fasta_headers(
    manifest: &AmpliconCorpusFixtureManifest,
    primer_fasta_path: &Path,
) -> Result<BTreeMap<String, String>> {
    let sequences = load_fasta_records(primer_fasta_path)?;
    if !sequences.contains_key(&manifest.forward_primer_id) {
        return Err(anyhow!(
            "amplicon corpus fixture forward primer `{}` is missing from {}",
            manifest.forward_primer_id,
            primer_fasta_path.display()
        ));
    }
    if !sequences.contains_key(&manifest.reverse_primer_id) {
        return Err(anyhow!(
            "amplicon corpus fixture reverse primer `{}` is missing from {}",
            manifest.reverse_primer_id,
            primer_fasta_path.display()
        ));
    }
    Ok(sequences)
}

pub(super) fn load_fasta_records(path: &Path) -> Result<BTreeMap<String, String>> {
    if !path.is_file() {
        return Err(anyhow!("FASTA fixture is missing: {}", path.display()));
    }
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut sequences = BTreeMap::new();
    let mut current_header: Option<String> = None;
    let mut current_sequence = String::new();
    for line in reader.lines() {
        let line = line.with_context(|| format!("read {}", path.display()))?;
        if let Some(header) = line.strip_prefix('>') {
            if let Some(previous_header) = current_header.take() {
                sequences.insert(previous_header, current_sequence.clone());
                current_sequence.clear();
            }
            current_header = Some(header.trim().to_string());
        } else if !line.trim().is_empty() {
            current_sequence.push_str(line.trim());
        }
    }
    if let Some(previous_header) = current_header.take() {
        sequences.insert(previous_header, current_sequence);
    }
    if sequences.is_empty() {
        return Err(anyhow!("FASTA fixture must declare at least one record: {}", path.display()));
    }
    Ok(sequences)
}

pub(super) fn validate_primer_table_contract(
    repo_root: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
    primers_tsv_path: &Path,
    primer_fasta_records: &BTreeMap<String, String>,
) -> Result<Vec<AmpliconPrimerTableRow>> {
    if !primers_tsv_path.is_file() {
        return Err(anyhow!(
            "amplicon corpus fixture primer table is missing: {}",
            primers_tsv_path.display()
        ));
    }
    let raw = fs::read_to_string(primers_tsv_path)
        .with_context(|| format!("read {}", primers_tsv_path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        anyhow!("amplicon corpus fixture primer table is empty: {}", primers_tsv_path.display())
    })?;
    if header != "primer_id\tforward_sequence\treverse_sequence\ttarget\torientation" {
        return Err(anyhow!(
            "amplicon corpus fixture primer table header is unexpected in {}",
            primers_tsv_path.display()
        ));
    }
    let rows = lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| parse_primer_row(line, primers_tsv_path))
        .collect::<Result<Vec<_>>>()?;
    if rows.is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture primer table must declare at least one row in {}",
            primers_tsv_path.display()
        ));
    }
    let manifest_relative_path = path_relative_to_repo(repo_root, primers_tsv_path);
    let mut primer_ids = BTreeSet::new();
    for row in &rows {
        if row.primer_id.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture primer table rows must declare a non-empty `primer_id` in {manifest_relative_path}"
            ));
        }
        if !primer_ids.insert(row.primer_id.as_str()) {
            return Err(anyhow!(
                "amplicon corpus fixture primer table repeats primer_id `{}` in {manifest_relative_path}",
                row.primer_id
            ));
        }
        if row.primer_id != manifest.primer_set_id {
            return Err(anyhow!(
                "amplicon corpus fixture primer table primer_id `{}` does not match manifest primer_set_id `{}` in {manifest_relative_path}",
                row.primer_id,
                manifest.primer_set_id
            ));
        }
        if row.target != manifest.target_region {
            return Err(anyhow!(
                "amplicon corpus fixture primer table target `{}` does not match manifest target_region `{}` in {manifest_relative_path}",
                row.target,
                manifest.target_region
            ));
        }
        if row.orientation != "normalize_to_forward_primer" {
            return Err(anyhow!(
                "amplicon corpus fixture primer table orientation must be `normalize_to_forward_primer` in {manifest_relative_path}"
            ));
        }
        let governed_forward =
            primer_fasta_records.get(&manifest.forward_primer_id).ok_or_else(|| {
                anyhow!(
                    "amplicon corpus fixture primer FASTA missing governed forward primer `{}`",
                    manifest.forward_primer_id
                )
            })?;
        let governed_reverse =
            primer_fasta_records.get(&manifest.reverse_primer_id).ok_or_else(|| {
                anyhow!(
                    "amplicon corpus fixture primer FASTA missing governed reverse primer `{}`",
                    manifest.reverse_primer_id
                )
            })?;
        if &row.forward_sequence != governed_forward {
            return Err(anyhow!(
                "amplicon corpus fixture primer table forward sequence does not match `{}` in {manifest_relative_path}",
                manifest.forward_primer_id
            ));
        }
        if &row.reverse_sequence != governed_reverse {
            return Err(anyhow!(
                "amplicon corpus fixture primer table reverse sequence does not match `{}` in {manifest_relative_path}",
                manifest.reverse_primer_id
            ));
        }
    }
    Ok(rows)
}

pub(super) fn validate_expected_asvs_contract(
    repo_root: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
    expected_asvs_path: &Path,
) -> Result<Vec<AmpliconExpectedAsvRow>> {
    if !expected_asvs_path.is_file() {
        return Err(anyhow!(
            "amplicon corpus fixture expected ASV table is missing: {}",
            expected_asvs_path.display()
        ));
    }
    let raw = fs::read_to_string(expected_asvs_path)
        .with_context(|| format!("read {}", expected_asvs_path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        anyhow!(
            "amplicon corpus fixture expected ASV table is empty: {}",
            expected_asvs_path.display()
        )
    })?;
    if header != "asv_id\tsequence\tsample_id\texpected_presence" {
        return Err(anyhow!(
            "amplicon corpus fixture expected ASV table header is unexpected in {}",
            expected_asvs_path.display()
        ));
    }
    let rows = lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| parse_expected_asv_row(line, expected_asvs_path))
        .collect::<Result<Vec<_>>>()?;
    if rows.is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture expected ASV table must declare at least one row in {}",
            expected_asvs_path.display()
        ));
    }
    let biological_sample_ids = manifest
        .samples
        .iter()
        .filter(|sample| matches!(sample.sample_kind, AmpliconCorpusSampleKind::Biological))
        .map(|sample| sample.sample_id.as_str())
        .collect::<BTreeSet<_>>();
    let manifest_relative_path = path_relative_to_repo(repo_root, expected_asvs_path);
    let mut asv_sample_pairs = BTreeSet::new();
    let mut present_rows = 0_usize;
    for row in &rows {
        if row.asv_id.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture expected ASV rows must declare a non-empty `asv_id` in {manifest_relative_path}"
            ));
        }
        if row.sequence.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture expected ASV rows must declare a non-empty `sequence` in {manifest_relative_path}"
            ));
        }
        if !row.sequence.chars().all(|ch| matches!(ch, 'A' | 'C' | 'G' | 'T' | 'N')) {
            return Err(anyhow!(
                "amplicon corpus fixture expected ASV sequence for `{}` must be uppercase DNA in {manifest_relative_path}",
                row.asv_id
            ));
        }
        if !biological_sample_ids.contains(row.sample_id.as_str()) {
            return Err(anyhow!(
                "amplicon corpus fixture expected ASV row `{}` references undeclared biological sample `{}` in {manifest_relative_path}",
                row.asv_id,
                row.sample_id
            ));
        }
        if !asv_sample_pairs.insert((row.asv_id.as_str(), row.sample_id.as_str())) {
            return Err(anyhow!(
                "amplicon corpus fixture expected ASV table repeats (`{}`, `{}`) in {manifest_relative_path}",
                row.asv_id,
                row.sample_id
            ));
        }
        if row.expected_presence == AmpliconExpectedPresence::Present {
            present_rows = present_rows.saturating_add(1);
        }
    }
    if present_rows == 0 {
        return Err(anyhow!(
            "amplicon corpus fixture expected ASV table must declare at least one `present` row in {manifest_relative_path}"
        ));
    }
    if !rows.iter().any(|row| row.sample_id == "corpus-03-amplicon-se") {
        return Err(anyhow!(
            "amplicon corpus fixture expected ASV table must include sample `corpus-03-amplicon-se` in {manifest_relative_path}"
        ));
    }
    Ok(rows)
}

pub(super) fn validate_chimera_expectations_contract(
    repo_root: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
    chimera_controls_fasta_path: &Path,
    chimera_expectations_path: &Path,
    chimera_control_records: &BTreeMap<String, String>,
) -> Result<Vec<AmpliconExpectedChimeraRow>> {
    if !chimera_expectations_path.is_file() {
        return Err(anyhow!(
            "amplicon corpus fixture chimera expectation table is missing: {}",
            chimera_expectations_path.display()
        ));
    }
    let raw = fs::read_to_string(chimera_expectations_path)
        .with_context(|| format!("read {}", chimera_expectations_path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        anyhow!(
            "amplicon corpus fixture chimera expectation table is empty: {}",
            chimera_expectations_path.display()
        )
    })?;
    if header != "chimera_id\tsequence\tsample_id\texpected_presence" {
        return Err(anyhow!(
            "amplicon corpus fixture chimera expectation table header is unexpected in {}",
            chimera_expectations_path.display()
        ));
    }
    let rows = lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| parse_chimera_row(line, chimera_expectations_path))
        .collect::<Result<Vec<_>>>()?;
    if rows.is_empty() {
        return Err(anyhow!(
            "amplicon corpus fixture chimera expectation table must declare at least one row in {}",
            chimera_expectations_path.display()
        ));
    }
    let control_sample_ids = manifest
        .samples
        .iter()
        .filter(|sample| matches!(sample.sample_kind, AmpliconCorpusSampleKind::Control))
        .map(|sample| sample.sample_id.as_str())
        .collect::<BTreeSet<_>>();
    let manifest_relative_path = path_relative_to_repo(repo_root, chimera_expectations_path);
    let chimera_fasta_relative_path = path_relative_to_repo(repo_root, chimera_controls_fasta_path);
    let mut chimera_sample_pairs = BTreeSet::new();
    let mut present_rows = 0_usize;
    for row in &rows {
        if row.chimera_id.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture chimera expectation rows must declare a non-empty `chimera_id` in {manifest_relative_path}"
            ));
        }
        if row.sequence.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus fixture chimera expectation rows must declare a non-empty `sequence` in {manifest_relative_path}"
            ));
        }
        if !row.sequence.chars().all(|ch| matches!(ch, 'A' | 'C' | 'G' | 'T' | 'N')) {
            return Err(anyhow!(
                "amplicon corpus fixture chimera expectation sequence for `{}` must be uppercase DNA in {manifest_relative_path}",
                row.chimera_id
            ));
        }
        if !control_sample_ids.contains(row.sample_id.as_str()) {
            return Err(anyhow!(
                "amplicon corpus fixture chimera expectation row `{}` references undeclared control sample `{}` in {manifest_relative_path}",
                row.chimera_id,
                row.sample_id
            ));
        }
        if !chimera_sample_pairs.insert((row.chimera_id.as_str(), row.sample_id.as_str())) {
            return Err(anyhow!(
                "amplicon corpus fixture chimera expectation table repeats (`{}`, `{}`) in {manifest_relative_path}",
                row.chimera_id,
                row.sample_id
            ));
        }
        if row.expected_presence == AmpliconExpectedPresence::Present {
            present_rows = present_rows.saturating_add(1);
            let fasta_sequence = chimera_control_records.get(&row.chimera_id).ok_or_else(|| {
                anyhow!(
                    "amplicon corpus fixture expected chimera `{}` is missing from {chimera_fasta_relative_path}",
                    row.chimera_id
                )
            })?;
            if fasta_sequence != &row.sequence {
                return Err(anyhow!(
                    "amplicon corpus fixture chimera expectation sequence for `{}` does not match {chimera_fasta_relative_path}",
                    row.chimera_id
                ));
            }
        }
    }
    if present_rows == 0 {
        return Err(anyhow!(
            "amplicon corpus fixture chimera expectation table must declare at least one `present` row in {manifest_relative_path}"
        ));
    }
    Ok(rows)
}

fn parse_primer_row(line: &str, primers_tsv_path: &Path) -> Result<AmpliconPrimerTableRow> {
    let mut fields = line.split('\t');
    let primer_row = AmpliconPrimerTableRow {
        primer_id: fields
            .next()
            .ok_or_else(|| anyhow!("missing primer_id field in {}", primers_tsv_path.display()))?
            .to_string(),
        forward_sequence: fields
            .next()
            .ok_or_else(|| {
                anyhow!("missing forward_sequence field in {}", primers_tsv_path.display())
            })?
            .to_string(),
        reverse_sequence: fields
            .next()
            .ok_or_else(|| {
                anyhow!("missing reverse_sequence field in {}", primers_tsv_path.display())
            })?
            .to_string(),
        target: fields
            .next()
            .ok_or_else(|| anyhow!("missing target field in {}", primers_tsv_path.display()))?
            .to_string(),
        orientation: fields
            .next()
            .ok_or_else(|| anyhow!("missing orientation field in {}", primers_tsv_path.display()))?
            .to_string(),
    };
    if fields.next().is_some() {
        return Err(anyhow!(
            "amplicon corpus fixture primer table row has too many columns in {}",
            primers_tsv_path.display()
        ));
    }
    Ok(primer_row)
}

fn parse_expected_asv_row(line: &str, expected_asvs_path: &Path) -> Result<AmpliconExpectedAsvRow> {
    let mut fields = line.split('\t');
    let expected_asv_row = AmpliconExpectedAsvRow {
        asv_id: fields
            .next()
            .ok_or_else(|| anyhow!("missing asv_id field in {}", expected_asvs_path.display()))?
            .to_string(),
        sequence: fields
            .next()
            .ok_or_else(|| anyhow!("missing sequence field in {}", expected_asvs_path.display()))?
            .to_string(),
        sample_id: fields
            .next()
            .ok_or_else(|| anyhow!("missing sample_id field in {}", expected_asvs_path.display()))?
            .to_string(),
        expected_presence: AmpliconExpectedPresence::parse(fields.next().ok_or_else(|| {
            anyhow!("missing expected_presence field in {}", expected_asvs_path.display())
        })?)?,
    };
    if fields.next().is_some() {
        return Err(anyhow!(
            "amplicon corpus fixture expected ASV row has too many columns in {}",
            expected_asvs_path.display()
        ));
    }
    Ok(expected_asv_row)
}

fn parse_chimera_row(
    line: &str,
    chimera_expectations_path: &Path,
) -> Result<AmpliconExpectedChimeraRow> {
    let mut fields = line.split('\t');
    let chimera_row = AmpliconExpectedChimeraRow {
        chimera_id: fields
            .next()
            .ok_or_else(|| {
                anyhow!("missing chimera_id field in {}", chimera_expectations_path.display())
            })?
            .to_string(),
        sequence: fields
            .next()
            .ok_or_else(|| {
                anyhow!("missing sequence field in {}", chimera_expectations_path.display())
            })?
            .to_string(),
        sample_id: fields
            .next()
            .ok_or_else(|| {
                anyhow!("missing sample_id field in {}", chimera_expectations_path.display())
            })?
            .to_string(),
        expected_presence: AmpliconExpectedPresence::parse(fields.next().ok_or_else(|| {
            anyhow!("missing expected_presence field in {}", chimera_expectations_path.display())
        })?)?,
    };
    if fields.next().is_some() {
        return Err(anyhow!(
            "amplicon corpus fixture chimera expectation row has too many columns in {}",
            chimera_expectations_path.display()
        ));
    }
    Ok(chimera_row)
}
