fn repo_root() -> std::path::PathBuf {
    crate::support::repo_root().unwrap_or_else(|err| panic!("repo root: {err}"))
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
        .unwrap_or_else(|| panic!("markers table missing in {}", cfg.display()));
    assert!(!markers.is_empty(), "markers table must not be empty");
    for (marker, row) in markers {
        let primer_rel = row
            .get("primer_fasta")
            .and_then(toml::Value::as_str)
            .unwrap_or_else(|| panic!("marker {marker} missing primer_fasta"));
        let primer_sha256_locked = row
            .get("primer_sha256")
            .and_then(toml::Value::as_str)
            .unwrap_or_else(|| panic!("marker {marker} missing primer_sha256"));
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
        let actual_sha = bijux_dna_infra::hash_file_sha256(&primer_path)
            .unwrap_or_else(|e| panic!("hash {}: {e}", primer_path.display()));
        assert_eq!(
            actual_sha, primer_sha256_locked,
            "marker {marker} primer sha256 lock mismatch"
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
        .unwrap_or_else(|| panic!("taxonomy table missing in {}", cfg.display()));
    let db_path = taxonomy
        .get("db_path")
        .and_then(toml::Value::as_str)
        .unwrap_or_else(|| panic!("taxonomy.db_path missing in {}", cfg.display()));
    let db_sha256 = taxonomy
        .get("db_sha256")
        .and_then(toml::Value::as_str)
        .unwrap_or_else(|| panic!("taxonomy.db_sha256 missing in {}", cfg.display()));
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
    let actual_sha = bijux_dna_infra::hash_file_sha256(&root.join(db_path))
        .unwrap_or_else(|e| panic!("hash {}: {e}", root.join(db_path).display()));
    assert_eq!(
        actual_sha, db_sha256,
        "taxonomy db_sha256 must lock current db_path content"
    );
}

#[test]
fn fastq_amplicon_tables_define_expected_schema_headers() {
    let root = repo_root();
    let runtime_src =
        root.join("crates/bijux-dna-api/src/internal/fastq/stages/preprocess/amplicon_runtime.rs");
    let normalize_src =
        root.join("crates/bijux-dna-api/src/internal/fastq/stages/normalize_abundance.rs");
    let runtime_raw = std::fs::read_to_string(&runtime_src)
        .unwrap_or_else(|e| panic!("read {}: {e}", runtime_src.display()));
    let normalize_raw = std::fs::read_to_string(&normalize_src)
        .unwrap_or_else(|e| panic!("read {}: {e}", normalize_src.display()));
    assert!(
        runtime_raw.contains("sample_id\\tfeature_id\\tabundance"),
        "ASV/OTU tables must include sample_id/feature_id/abundance header contract"
    );
    assert!(
        normalize_raw.contains("\"normalized_abundance\".to_string()"),
        "relative abundance normalization must lock the normalized_abundance output column"
    );
}

#[test]
fn fastq_amplicon_runtime_invokes_real_tool_paths() {
    let root = repo_root();
    let runtime_src =
        root.join("crates/bijux-dna-api/src/internal/fastq/stages/preprocess/amplicon_runtime.rs");
    let governance_src = root
        .join("crates/bijux-dna-api/src/internal/fastq/stages/preprocess/amplicon_governance.rs");
    let raw_runtime = std::fs::read_to_string(&runtime_src)
        .unwrap_or_else(|e| panic!("read {}: {e}", runtime_src.display()));
    let raw_governance = std::fs::read_to_string(&governance_src)
        .unwrap_or_else(|e| panic!("read {}: {e}", governance_src.display()));
    let raw = format!("{raw_runtime}\n{raw_governance}");
    for needle in [
        "cutadapt_normalize_primers",
        "vsearch_uchime_denovo",
        "vsearch_cluster_fast",
        "dada2_rscript",
    ] {
        assert!(
            raw.contains(needle),
            "expected amplicon tool execution path marker `{needle}` in preprocess runtime"
        );
    }
}
