#![allow(non_snake_case)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn tsv_records(path: &str) -> Vec<BTreeMap<String, String>> {
    let root = repo_root();
    let raw =
        std::fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"));
    let mut lines = raw.lines();
    let header = lines
        .next()
        .unwrap_or_else(|| panic!("{path} must not be empty"))
        .split('\t')
        .map(str::to_string)
        .collect::<Vec<_>>();
    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let record = line.split('\t').map(str::to_string).collect::<Vec<_>>();
            assert_eq!(
                record.len(),
                header.len(),
                "{path} row has {} columns but header has {}",
                record.len(),
                header.len()
            );
            header.iter().cloned().zip(record).collect()
        })
        .collect()
}

fn checksum_paths(path: &str) -> BTreeSet<String> {
    let root = repo_root();
    std::fs::read_to_string(root.join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
        .lines()
        .filter_map(|line| line.split_whitespace().nth(1))
        .map(str::to_string)
        .collect()
}

#[test]
fn policy__contracts__assets_governance_policy__assets_root_uses_taxonomy_dirs_only() {
    let root = repo_root();
    let assets = root.join("assets");
    let mut offenders = Vec::new();
    let allowed_dirs = ["publications", "golden", "toy", "reference"];
    for entry in
        std::fs::read_dir(&assets).unwrap_or_else(|err| panic!("read {}: {err}", assets.display()))
    {
        let entry =
            entry.unwrap_or_else(|err| panic!("read entry under {}: {err}", assets.display()));
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if !allowed_dirs.contains(&name.as_str()) {
                offenders.push(format!("unexpected directory: assets/{name}"));
            }
        } else if name != "index.md" && name != "CONTRACT.md" && name != "LARGE_FILE_ALLOWLIST.txt"
        {
            offenders.push(format!("unexpected file: assets/{name}"));
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "assets root must contain only taxonomy dirs + index.md:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__assets_docs_use_current_dev_crate_name() {
    let root = repo_root();
    let assets = root.join("assets");
    let mut offenders = Vec::new();
    for entry in
        WalkDir::new(&assets).into_iter().filter_map(Result::ok).filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let rel = path.strip_prefix(&root).unwrap_or(path).display().to_string();
        let raw = std::fs::read_to_string(path).unwrap_or_else(|err| panic!("read {rel}: {err}"));
        if raw.contains("bijux-dev-dna") {
            offenders.push(rel);
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "assets markdown must use the current bijux-dna-dev crate name:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__contaminant_references_publish_status_rows() {
    let rows = tsv_records("assets/reference/contaminants/references/REFERENCE_STATUS.tsv");
    let by_asset =
        rows.iter().map(|row| (row["asset_id"].as_str(), row)).collect::<BTreeMap<_, _>>();
    let checksums = checksum_paths("assets/reference/contaminants/references/CHECKSUMS.sha256");
    let mut offenders = Vec::new();

    for asset_id in ["phix174", "univec"] {
        let Some(row) = by_asset.get(asset_id) else {
            offenders.push(format!("missing contaminant reference status row for {asset_id}"));
            continue;
        };
        if row["current_status"] != "sentinel" {
            offenders.push(format!(
                "{asset_id} must stay marked sentinel until replaced, found {}",
                row["current_status"]
            ));
        }
        if row["production_use"] != "blocked_for_production" {
            offenders.push(format!(
                "{asset_id} must block production use, found {}",
                row["production_use"]
            ));
        }
        if row["source_locator"].trim().is_empty() || row["expected_replacement"].trim().is_empty()
        {
            offenders.push(format!("{asset_id} status row must name source and replacement"));
        }
        let rel_path = row["path"].strip_prefix("assets/reference/contaminants/references/");
        if rel_path.is_none_or(|path| !checksums.contains(path)) {
            offenders.push(format!("{} missing from CHECKSUMS.sha256", row["path"]));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "contaminant reference status violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__sentinel_contaminants_stay_tiny_and_blocked() {
    let root = repo_root();
    let rows = tsv_records("assets/reference/contaminants/references/REFERENCE_STATUS.tsv");
    let blocked_paths = rows
        .iter()
        .filter(|row| row["production_use"] == "blocked_for_production")
        .map(|row| row["path"].clone())
        .collect::<BTreeSet<_>>();
    let mut offenders = Vec::new();

    for (path, max_len) in [
        ("assets/reference/contaminants/references/phix174.fasta", 128_u64),
        ("assets/reference/contaminants/references/univec.fasta", 128_u64),
    ] {
        if !blocked_paths.contains(path) {
            offenders.push(format!("{path} must be blocked_for_production in status table"));
        }
        let len = std::fs::metadata(root.join(path))
            .unwrap_or_else(|err| panic!("stat {path}: {err}"))
            .len();
        if len > max_len {
            offenders.push(format!(
                "{path} is no longer sentinel-sized ({len} bytes); update status before release"
            ));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "sentinel contaminant reference policy violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__primer_evidence_covers_primer_bank() {
    let rows = tsv_records("assets/reference/primers/PRIMER_EVIDENCE.tsv");
    let evidence_ids = rows.iter().map(|row| row["primer_set"].clone()).collect::<BTreeSet<_>>();
    let checksums = checksum_paths("assets/reference/primers/CHECKSUMS.sha256");
    let expected_ids = ["16S_universal_v1", "COI_folmer_v1", "ITS2_plant_v1"]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let mut offenders = Vec::new();

    if evidence_ids != expected_ids {
        offenders.push(format!("primer evidence ids {evidence_ids:?} != {expected_ids:?}"));
    }
    if !checksums.contains("assets/reference/primers/PRIMER_EVIDENCE.tsv") {
        offenders.push("PRIMER_EVIDENCE.tsv missing from primer checksums".to_string());
    }
    for row in rows {
        if row["primary_locator"].trim().is_empty()
            || row["doi_status"].trim().is_empty()
            || row["review_note"].trim().is_empty()
        {
            offenders.push(format!("{} has incomplete evidence metadata", row["primer_set"]));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "primer evidence coverage violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__primer_doi_status_matches_authority() {
    let rows = tsv_records("assets/reference/primers/PRIMER_EVIDENCE.tsv");
    let by_set =
        rows.iter().map(|row| (row["primer_set"].as_str(), row)).collect::<BTreeMap<_, _>>();
    let mut offenders = Vec::new();

    for (primer_set, locator) in [
        ("16S_universal_v1", "https://doi.org/10.1128/jb.173.2.697-703.1991"),
        ("ITS2_plant_v1", "https://doi.org/10.1371/journal.pone.0008613"),
    ] {
        let row = by_set.get(primer_set).unwrap_or_else(|| panic!("missing {primer_set}"));
        if row["doi_status"] != "doi_verified" || row["primary_locator"] != locator {
            offenders.push(format!("{primer_set} must use verified DOI locator {locator}"));
        }
    }

    let folmer = by_set.get("COI_folmer_v1").expect("missing COI_folmer_v1");
    if folmer["doi_status"] != "doi_unverified"
        || !folmer["primary_locator"].contains("pubmed.ncbi.nlm.nih.gov/7881515")
    {
        offenders
            .push("COI_folmer_v1 must stay PubMed-backed with doi_unverified status".to_string());
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "primer DOI authority violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__publication_dirs_require_manifest_toml() {
    let root = repo_root();
    let publications = root.join("assets/publications");
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&publications).expect("read publications dir") {
        let entry = entry.expect("read publication entry");
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if id == "index.md" {
            continue;
        }
        let manifest = path.join("MANIFEST.toml");
        if !manifest.is_file() {
            offenders.push(format!("assets/publications/{id}/MANIFEST.toml missing"));
            continue;
        }
        let raw = std::fs::read_to_string(&manifest).unwrap_or_default();
        let parsed: toml::Value = match toml::from_str(&raw) {
            Ok(v) => v,
            Err(err) => {
                offenders
                    .push(format!("assets/publications/{id}/MANIFEST.toml invalid TOML: {err}"));
                continue;
            }
        };
        for required in ["title", "authors", "year", "provenance_notes", "license"] {
            if parsed.get(required).is_none() {
                offenders.push(format!(
                    "assets/publications/{id}/MANIFEST.toml missing field `{required}`"
                ));
            }
        }
        for required in ["title", "provenance_notes", "license"] {
            if parsed
                .get(required)
                .and_then(toml::Value::as_str)
                .is_none_or(|value| value.trim().is_empty())
            {
                offenders.push(format!(
                    "assets/publications/{id}/MANIFEST.toml field `{required}` must be a non-empty string"
                ));
            }
        }
        if parsed.get("authors").and_then(toml::Value::as_array).is_none_or(std::vec::Vec::is_empty)
        {
            offenders.push(format!(
                "assets/publications/{id}/MANIFEST.toml field `authors` must be a non-empty array"
            ));
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "publication asset dirs manifest violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__golden_files_require_generate_metadata() {
    let root = repo_root();
    let golden = root.join("assets/golden");
    let mut offenders = Vec::new();
    for entry in
        WalkDir::new(&golden).into_iter().filter_map(Result::ok).filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or_default();
        if name == "GENERATE.md"
            || std::path::Path::new(name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            continue;
        }
        let Some(dir) = path.parent() else {
            continue;
        };
        if !dir.join("GENERATE.md").is_file() {
            offenders.push(path.strip_prefix(&root).unwrap_or(path).display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "golden files must declare deterministic generation metadata:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__toy_datasets_require_checksums() {
    let root = repo_root();
    let toy = root.join("assets/toy");
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&toy).expect("read toy dir") {
        let entry = entry.expect("read toy entry");
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if !path.join("CHECKSUMS.sha256").is_file() {
            offenders.push(format!("assets/toy/{id}/CHECKSUMS.sha256 missing"));
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "toy datasets must include CHECKSUMS.sha256:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__assets_forbid_local_machine_paths() {
    let root = repo_root();
    let assets = root.join("assets");
    let mut offenders = Vec::new();
    let banned = ["/Users/", "/home/", "C:\\\\Users\\\\", "\\\\Users\\\\"];
    for entry in
        WalkDir::new(&assets).into_iter().filter_map(Result::ok).filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !matches!(
            ext,
            "yaml" | "yml" | "json" | "jsonl" | "toml" | "txt" | "md" | "vcf" | "sam" | "fastq"
        ) {
            continue;
        }
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        if banned.iter().any(|needle| raw.contains(needle)) {
            offenders.push(path.strip_prefix(&root).unwrap_or(path).display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "assets must not embed local-machine/PII path literals:\\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__tests_must_not_write_into_assets() {
    let root = repo_root();
    let mut offenders = Vec::new();
    let write_into_assets_patterns = [
        regex::Regex::new(r#"(?s)\bwrite\s*\(\s*[^,\n)]*assets/"#).expect("write regex"),
        regex::Regex::new(r#"(?s)\bcreate_dir(?:_all)?\s*\(\s*[^)\n]*assets/"#)
            .expect("create dir regex"),
        regex::Regex::new(r#"(?s)\bremove_(?:file|dir(?:_all)?)\s*\(\s*[^)\n]*assets/"#)
            .expect("remove regex"),
        regex::Regex::new(r#"(?s)\brename\s*\(\s*[^,\n]*assets/"#).expect("rename from regex"),
        regex::Regex::new(r#"(?s)\brename\s*\([^,\n]+,\s*[^)\n]*assets/"#)
            .expect("rename into regex"),
        regex::Regex::new(r#"(?s)\bcopy\s*\([^,\n]+,\s*[^)\n]*assets/"#).expect("copy regex"),
        regex::Regex::new(r#"(?s)\bOpenOptions\b[\s\S]{0,200}\.open\s*\(\s*[^)\n]*assets/"#)
            .expect("open options regex"),
    ];
    for dir in ["crates", "scripts", "makes"] {
        for entry in WalkDir::new(root.join(dir)).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let rel = path.strip_prefix(&root).unwrap_or(path);
            let rel_s = rel.to_string_lossy();
            if rel_s
                == "crates/bijux-dna-policies/tests/contracts/tooling/assets/assets_governance_policy.rs"
            {
                continue;
            }
            let is_testish = rel_s.contains("/tests/") || rel_s.ends_with(".mk");
            if !is_testish {
                continue;
            }
            let raw = std::fs::read_to_string(path).unwrap_or_default();
            if write_into_assets_patterns.iter().any(|pattern| pattern.is_match(&raw)) {
                offenders.push(rel.display().to_string());
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tests/tooling must not write into assets/ (assets are read-only):\\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__assets_governance_policy__docs_publication_refs_use_publications_prefix() {
    let root = repo_root();
    let docs = root.join("docs");
    let mut offenders = Vec::new();
    for entry in
        WalkDir::new(&docs).into_iter().filter_map(Result::ok).filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        for token in raw.split_whitespace() {
            if let Some(idx) = token.find("assets/publications/") {
                let frag = &token[idx..];
                if !frag.contains("/index.md") {
                    offenders.push(format!(
                        "{}: publication refs must target assets/publications/<pub-id>/index.md",
                        path.strip_prefix(&root).unwrap_or(path).display()
                    ));
                    break;
                }
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "docs asset references must use assets/publications/<pub-id>/...:\\n{}",
        offenders.join("\n")
    );
}
