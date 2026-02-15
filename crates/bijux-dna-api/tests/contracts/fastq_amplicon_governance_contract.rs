use std::path::Path;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn fastq_amplicon_governance_has_marker_ranges_and_primer_files() {
    let root = repo_root();
    let cfg = root.join("assets/reference/amplicon_governance.toml");
    let raw =
        std::fs::read_to_string(&cfg).unwrap_or_else(|e| panic!("read {}: {e}", cfg.display()));
    let doc: toml::Value =
        toml::from_str(&raw).unwrap_or_else(|e| panic!("parse {}: {e}", cfg.display()));
    let markers = doc
        .get("markers")
        .and_then(toml::Value::as_table)
        .cloned()
        .unwrap_or_default();
    assert!(!markers.is_empty(), "markers table must not be empty");
    for (marker, row) in markers {
        let primer_rel = row
            .get("primer_fasta")
            .and_then(toml::Value::as_str)
            .unwrap_or_else(|| panic!("marker {marker} missing primer_fasta"));
        let min_bp = row
            .get("expected_amplicon_min_bp")
            .and_then(toml::Value::as_integer)
            .unwrap_or_else(|| panic!("marker {marker} missing expected_amplicon_min_bp"));
        let max_bp = row
            .get("expected_amplicon_max_bp")
            .and_then(toml::Value::as_integer)
            .unwrap_or_else(|| panic!("marker {marker} missing expected_amplicon_max_bp"));
        assert!(
            min_bp < max_bp,
            "marker {marker} must have min_bp < max_bp ({min_bp} < {max_bp})"
        );
        let primer_path = root.join(primer_rel);
        assert!(
            primer_path.exists(),
            "marker {marker} primer fasta missing: {}",
            primer_path.display()
        );
    }
}

#[test]
fn fastq_amplicon_governance_taxonomy_lock_fields_present() {
    let root = repo_root();
    let cfg = root.join("assets/reference/amplicon_governance.toml");
    let raw =
        std::fs::read_to_string(&cfg).unwrap_or_else(|e| panic!("read {}: {e}", cfg.display()));
    let doc: toml::Value =
        toml::from_str(&raw).unwrap_or_else(|e| panic!("parse {}: {e}", cfg.display()));
    let taxonomy = doc
        .get("taxonomy")
        .and_then(toml::Value::as_table)
        .cloned()
        .unwrap_or_default();
    let db_path = taxonomy
        .get("db_path")
        .and_then(toml::Value::as_str)
        .unwrap_or_else(|| panic!("taxonomy.db_path missing in {}", cfg.display()));
    let license = taxonomy
        .get("license")
        .and_then(toml::Value::as_str)
        .unwrap_or_else(|| panic!("taxonomy.license missing in {}", cfg.display()));
    assert!(
        !license.trim().is_empty() && license != "unspecified",
        "taxonomy license must be explicit"
    );
    assert!(
        root.join(db_path).exists(),
        "taxonomy db_path must exist: {}",
        root.join(db_path).display()
    );
}

#[test]
fn fastq_amplicon_tables_define_expected_schema_headers() {
    let root = repo_root();
    let src = root.join("crates/bijux-dna-api/src/internal/fastq/preprocess.rs");
    let raw =
        std::fs::read_to_string(&src).unwrap_or_else(|e| panic!("read {}: {e}", src.display()));
    assert!(
        raw.contains("sample_id\\tfeature_id\\tabundance"),
        "ASV/OTU tables must include sample_id/feature_id/abundance header contract"
    );
    assert!(
        raw.contains("sample_id\\tfeature_id\\tnormalized_abundance"),
        "abundance normalization table must include normalized_abundance header contract"
    );
}
