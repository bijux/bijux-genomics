use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PrimerBankV1 {
    pub schema_version: String,
    pub bank_id: String,
    pub version: String,
    pub provenance_status: String,
    pub license: String,
    pub source_document: String,
    pub source_checksum_sha256: String,
    pub applicable_assays: Vec<String>,
    pub selection_logic: String,
    pub primer_sets: Vec<PrimerSetV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PrimerSetV1 {
    pub id: String,
    pub marker: String,
    pub primer_fasta: String,
    pub primer_sha256: String,
    pub applicable_assays: Vec<String>,
    pub expected_amplicon_min_bp: u32,
    pub expected_amplicon_max_bp: u32,
    pub primary_locator: String,
    pub doi_status: String,
    pub review_note: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AmpliconGovernanceFile {
    bank: PrimerBankMetadata,
    markers: BTreeMap<String, MarkerGovernanceEntry>,
    #[serde(default)]
    #[serde(rename = "taxonomy")]
    _taxonomy: Option<toml::Value>,
    #[serde(default)]
    #[serde(rename = "merge_policy")]
    _merge_policy: Option<toml::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrimerBankMetadata {
    schema_version: String,
    bank_id: String,
    version: String,
    provenance_status: String,
    license: String,
    source_document: String,
    source_checksum_sha256: String,
    applicable_assays: Vec<String>,
    selection_logic: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MarkerGovernanceEntry {
    primer_set_id: String,
    primer_fasta: String,
    primer_sha256: String,
    applicable_assays: Vec<String>,
    expected_amplicon_min_bp: u32,
    expected_amplicon_max_bp: u32,
}

#[derive(Debug)]
struct PrimerEvidenceEntry {
    primer_set_id: String,
    primary_locator: String,
    doi_status: String,
    review_note: String,
}

#[must_use]
pub fn amplicon_governance_path() -> PathBuf {
    PathBuf::from("assets/reference/amplicon_governance.toml")
}

#[must_use]
pub fn primer_checksums_path() -> PathBuf {
    PathBuf::from("assets/reference/primers/CHECKSUMS.sha256")
}

#[must_use]
pub fn primer_evidence_path() -> PathBuf {
    PathBuf::from("assets/reference/primers/PRIMER_EVIDENCE.tsv")
}

/// Load the governed primer bank and validate the linked checksum and evidence registries.
///
/// # Errors
/// Returns an error if the TOML file cannot be read, parsed, or fails validation.
pub fn load_primer_bank(path: &Path) -> Result<PrimerBankV1> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read amplicon governance {}", path.display()))?;
    let governance: AmpliconGovernanceFile =
        toml::from_str(&raw).context("parse amplicon governance toml")?;

    let repo_root = path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .context("derive repository root from amplicon governance path")?;
    let evidence = load_primer_evidence(&repo_root.join(primer_evidence_path()))?;
    let checksums = load_checksum_manifest(&repo_root.join(primer_checksums_path()))?;

    let mut primer_sets = Vec::new();
    for (marker, entry) in &governance.markers {
        let evidence_entry = evidence
            .get(&entry.primer_set_id)
            .ok_or_else(|| anyhow!("missing primer evidence for {}", entry.primer_set_id))?;
        let checksum = checksums.get(&entry.primer_fasta).ok_or_else(|| {
            anyhow!("missing checksum entry for primer fasta {}", entry.primer_fasta)
        })?;
        if checksum != &entry.primer_sha256 {
            return Err(anyhow!("primer fasta checksum mismatch for {}", entry.primer_set_id));
        }
        let fasta_path = repo_root.join(&entry.primer_fasta);
        let actual_fasta_sha = bijux_dna_infra::hash_file_sha256(&fasta_path)
            .with_context(|| format!("hash primer fasta {}", fasta_path.display()))?;
        if actual_fasta_sha != entry.primer_sha256 {
            return Err(anyhow!("primer fasta content hash mismatch for {}", entry.primer_set_id));
        }
        primer_sets.push(PrimerSetV1 {
            id: entry.primer_set_id.clone(),
            marker: marker.clone(),
            primer_fasta: entry.primer_fasta.clone(),
            primer_sha256: entry.primer_sha256.clone(),
            applicable_assays: entry.applicable_assays.clone(),
            expected_amplicon_min_bp: entry.expected_amplicon_min_bp,
            expected_amplicon_max_bp: entry.expected_amplicon_max_bp,
            primary_locator: evidence_entry.primary_locator.clone(),
            doi_status: evidence_entry.doi_status.clone(),
            review_note: evidence_entry.review_note.clone(),
        });
    }
    primer_sets.sort_by(|left, right| left.id.cmp(&right.id));

    let bank = PrimerBankV1 {
        schema_version: governance.bank.schema_version,
        bank_id: governance.bank.bank_id,
        version: governance.bank.version,
        provenance_status: governance.bank.provenance_status,
        license: governance.bank.license,
        source_document: governance.bank.source_document,
        source_checksum_sha256: governance.bank.source_checksum_sha256,
        applicable_assays: governance.bank.applicable_assays,
        selection_logic: governance.bank.selection_logic,
        primer_sets,
    };
    validate_primer_bank(&bank)?;
    Ok(bank)
}

fn load_primer_evidence(path: &Path) -> Result<BTreeMap<String, PrimerEvidenceEntry>> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read primer evidence {}", path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| anyhow!("primer evidence is empty"))?;
    if header.trim() != "primer_set\tmarker\tprimary_locator\tdoi_status\treview_note" {
        return Err(anyhow!("primer evidence header is invalid"));
    }
    let mut rows = BTreeMap::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 5 {
            return Err(anyhow!("primer evidence row must have five columns"));
        }
        let entry = PrimerEvidenceEntry {
            primer_set_id: fields[0].to_string(),
            primary_locator: fields[2].to_string(),
            doi_status: fields[3].to_string(),
            review_note: fields[4].to_string(),
        };
        rows.insert(entry.primer_set_id.clone(), entry);
    }
    Ok(rows)
}

fn load_checksum_manifest(path: &Path) -> Result<BTreeMap<String, String>> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read checksum manifest {}", path.display()))?;
    let mut entries = BTreeMap::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (hash, rel_path) = line
            .split_once("  ")
            .ok_or_else(|| anyhow!("checksum line must use double-space separator"))?;
        entries.insert(rel_path.to_string(), hash.to_string());
    }
    Ok(entries)
}

fn validate_primer_bank(bank: &PrimerBankV1) -> Result<()> {
    if bank.schema_version.trim().is_empty() {
        return Err(anyhow!("primer bank missing schema_version"));
    }
    if bank.bank_id.trim().is_empty() {
        return Err(anyhow!("primer bank missing bank_id"));
    }
    if bank.version.trim().is_empty() {
        return Err(anyhow!("primer bank missing version"));
    }
    if bank.provenance_status != "complete" {
        return Err(anyhow!(
            "primer bank provenance_status must be `complete` for supported scope"
        ));
    }
    if bank.license.trim().is_empty() {
        return Err(anyhow!("primer bank missing license"));
    }
    if bank.source_document.trim().is_empty() {
        return Err(anyhow!("primer bank missing source_document"));
    }
    if bank.source_checksum_sha256.trim().len() != 64 {
        return Err(anyhow!("primer bank missing source_checksum_sha256"));
    }
    if bank.applicable_assays.is_empty() {
        return Err(anyhow!("primer bank missing applicable_assays"));
    }
    if bank.selection_logic.trim().is_empty() {
        return Err(anyhow!("primer bank missing selection_logic"));
    }
    if bank.primer_sets.is_empty() {
        return Err(anyhow!("primer bank contains no primer sets"));
    }

    for primer_set in &bank.primer_sets {
        if primer_set.id.trim().is_empty() {
            return Err(anyhow!("primer set missing id"));
        }
        if primer_set.marker.trim().is_empty() {
            return Err(anyhow!("primer set {} missing marker", primer_set.id));
        }
        if primer_set.primer_fasta.trim().is_empty() {
            return Err(anyhow!("primer set {} missing primer_fasta", primer_set.id));
        }
        if primer_set.primer_sha256.trim().len() != 64 {
            return Err(anyhow!("primer set {} missing primer_sha256", primer_set.id));
        }
        if primer_set.applicable_assays.is_empty() {
            return Err(anyhow!("primer set {} missing applicable_assays", primer_set.id));
        }
        if primer_set.expected_amplicon_min_bp == 0
            || primer_set.expected_amplicon_max_bp < primer_set.expected_amplicon_min_bp
        {
            return Err(anyhow!("primer set {} has invalid amplicon bounds", primer_set.id));
        }
        if primer_set.primary_locator.trim().is_empty() {
            return Err(anyhow!("primer set {} missing primary_locator", primer_set.id));
        }
        if primer_set.doi_status.trim().is_empty() {
            return Err(anyhow!("primer set {} missing doi_status", primer_set.id));
        }
        if primer_set.review_note.trim().is_empty() {
            return Err(anyhow!("primer set {} missing review_note", primer_set.id));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_bank() -> PrimerBankV1 {
        PrimerBankV1 {
            schema_version: "bijux.fastq.primer_bank.v1".to_string(),
            bank_id: "primer-bank".to_string(),
            version: "1".to_string(),
            provenance_status: "complete".to_string(),
            license: "CC-BY-4.0".to_string(),
            source_document: "assets/reference/EVIDENCE.md".to_string(),
            source_checksum_sha256:
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            applicable_assays: vec!["amplicon".to_string()],
            selection_logic: "select explicit marker-specific primer set".to_string(),
            primer_sets: vec![PrimerSetV1 {
                id: "16S_universal_v1".to_string(),
                marker: "16S".to_string(),
                primer_fasta: "assets/reference/primers/16S_universal_v1.fasta".to_string(),
                primer_sha256: "6247ebb9c7729a2561b82c1e443921a372e597ff8b47e68274b84956a964735f"
                    .to_string(),
                applicable_assays: vec!["amplicon_standard".to_string()],
                expected_amplicon_min_bp: 120,
                expected_amplicon_max_bp: 320,
                primary_locator: "https://doi.org/example".to_string(),
                doi_status: "doi_verified".to_string(),
                review_note: "reviewed".to_string(),
            }],
        }
    }

    #[test]
    fn primer_bank_rejects_missing_license() {
        let mut bank = valid_bank();
        bank.license.clear();
        let err = validate_primer_bank(&bank).expect_err("license is required");
        assert!(err.to_string().contains("missing license"));
    }

    #[test]
    fn primer_bank_rejects_invalid_amplicon_bounds() {
        let mut bank = valid_bank();
        bank.primer_sets[0].expected_amplicon_max_bp = 100;
        let err = validate_primer_bank(&bank).expect_err("amplicon bounds must be ordered");
        assert!(err.to_string().contains("invalid amplicon bounds"));
    }
}
