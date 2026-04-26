#[test]
fn vcf_io_real_tools_split_extract_concat_and_overlap() {
    if std::env::var("BIJUX_E2E").is_err() {
        return;
    }
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let input = std::path::Path::new("tests/fixtures/vcf/default/input.vcf");
    let normalized = dir.path().join("normalized.vcf");
    let input_vcfgz = dir.path().join("normalized.vcf.gz");
    std::fs::copy(input, &normalized).unwrap_or_else(|err| panic!("copy fixture: {err}"));
    bijux_dna_stages_vcf::vcf_io::vcf_index_bgzip_tabix(&normalized, &input_vcfgz)
        .unwrap_or_else(|err| panic!("index fixture: {err}"));

    let split_dir = dir.path().join("split");
    let parts = bijux_dna_stages_vcf::vcf_io::vcf_split_by_chrom(&input_vcfgz, &split_dir)
        .unwrap_or_else(|err| panic!("split by chrom: {err}"));
    assert!(!parts.is_empty(), "split must produce per-contig files");

    let regions = dir.path().join("regions.txt");
    std::fs::write(&regions, b"1:100-220\n").unwrap_or_else(|err| panic!("write regions: {err}"));
    let extracted = dir.path().join("region.vcf.gz");
    bijux_dna_stages_vcf::vcf_io::vcf_region_extract(&input_vcfgz, &regions, &extracted)
        .unwrap_or_else(|err| panic!("region extract: {err}"));
    assert!(extracted.exists(), "region output missing");
    assert!(
        std::path::PathBuf::from(format!("{}.tbi", extracted.display())).exists(),
        "region output index missing"
    );

    let reconcat = dir.path().join("concat.vcf.gz");
    bijux_dna_stages_vcf::vcf_io::vcf_concat(&parts, &reconcat)
        .unwrap_or_else(|err| panic!("concat: {err}"));
    assert!(reconcat.exists(), "concat output missing");

    let overlap = bijux_dna_stages_vcf::vcf_io::vcf_panel_overlap(&input_vcfgz, &reconcat)
        .unwrap_or_else(|err| panic!("overlap check: {err}"));
    let shared = overlap
        .get("shared_variants_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    assert!(shared > 0, "expected non-zero overlap against self-concat");
}

#[test]
fn vcf_ref_match_check_refuses_build_mismatch() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let input = dir.path().join("x.vcf");
    std::fs::write(
        &input,
        "##fileformat=VCFv4.2\n##reference=GRCh37\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t1\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\n",
    )
    .unwrap_or_else(|err| panic!("write vcf: {err}"));
    let species = bijux_dna_domain_vcf::contracts::SpeciesContext {
        species_id: "Homo sapiens".to_string(),
        build_id: "GRCh38".to_string(),
        contig_set_digest: "x".repeat(64),
        contigs: vec![bijux_dna_domain_vcf::contracts::ContigSpec {
            name: "1".to_string(),
            length_bp: 1_000,
        }],
        sex_system: "xy".to_string(),
        par_policy: "grch38_par".to_string(),
        default_coverage_regime: None,
    };
    let err = bijux_dna_stages_vcf::vcf_io::vcf_ref_match_check(&input, &species)
        .expect_err("build mismatch must refuse");
    assert!(err.to_string().contains("build mismatch"));
}
