#[test]
fn vcf_index_bgzip_tabix_writes_queryable_assets() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err:#}"));
    let input = std::path::Path::new("tests/fixtures/vcf/default/input.vcf");
    let output = dir.path().join("indexed.vcf.gz");

    let index = bijux_dna_stages_vcf::vcf_io::vcf_index_bgzip_tabix(input, &output)
        .unwrap_or_else(|err| panic!("index fixture: {err:#}"));

    assert!(output.exists(), "BGZF output missing");
    assert_eq!(index, std::path::PathBuf::from(format!("{}.tbi", output.display())));
    assert!(index.exists(), "tabix index missing");
    noodles_tabix::fs::read(index).unwrap_or_else(|err| panic!("read tabix index: {err:#}"));
    let expected = std::fs::read_to_string(input)
        .unwrap_or_else(|err| panic!("read source VCF fixture: {err:#}"));
    let observed = bijux_dna_stages_vcf::vcf_io::read_vcf_text(&output)
        .unwrap_or_else(|err| panic!("read indexed BGZF VCF: {err:#}"));
    assert_eq!(observed, expected);
}

#[test]
fn vcf_io_real_tools_split_extract_concat_and_overlap() {
    if std::env::var("BIJUX_E2E").is_err() {
        return;
    }
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err:#}"));
    let input = std::path::Path::new("tests/fixtures/vcf/default/input.vcf");
    let normalized = dir.path().join("normalized.vcf");
    let input_vcfgz = dir.path().join("normalized.vcf.gz");
    std::fs::copy(input, &normalized).unwrap_or_else(|err| panic!("copy fixture: {err:#}"));
    bijux_dna_stages_vcf::vcf_io::vcf_index_bgzip_tabix(&normalized, &input_vcfgz)
        .unwrap_or_else(|err| panic!("index fixture: {err:#}"));

    let split_dir = dir.path().join("split");
    let parts = bijux_dna_stages_vcf::vcf_io::vcf_split_by_chrom(&input_vcfgz, &split_dir)
        .unwrap_or_else(|err| panic!("split by chrom: {err:#}"));
    assert!(!parts.is_empty(), "split must produce per-contig files");

    // The tabular and BED inputs describe the same closed VCF position interval.
    for (name, coordinates) in [("regions.tsv", "1\t105\t111\n"), ("regions.bed", "1\t104\t111\n")]
    {
        let regions = dir.path().join(name);
        std::fs::write(&regions, coordinates)
            .unwrap_or_else(|err| panic!("write regions: {err:#}"));
        let extracted = dir.path().join(format!("{name}.vcf.gz"));
        bijux_dna_stages_vcf::vcf_io::vcf_region_extract(&input_vcfgz, &regions, &extracted)
            .unwrap_or_else(|err| panic!("region extract: {err:#}"));
        assert!(extracted.exists(), "region output missing");
        assert!(
            std::path::PathBuf::from(format!("{}.tbi", extracted.display())).exists(),
            "region output index missing"
        );
        let text = bijux_dna_stages_vcf::vcf_io::read_vcf_text(&extracted)
            .unwrap_or_else(|err| panic!("read extracted VCF: {err:#}"));
        let positions = text
            .lines()
            .filter(|line| !line.starts_with('#'))
            .map(|line| line.split('\t').nth(1).unwrap_or_default())
            .collect::<Vec<_>>();
        assert_eq!(
            positions,
            ["105", "111"],
            "{name} must retain both boundaries and exclude neighboring sites"
        );
    }

    let reconcat = dir.path().join("concat.vcf.gz");
    bijux_dna_stages_vcf::vcf_io::vcf_concat(&parts, &reconcat)
        .unwrap_or_else(|err| panic!("concat: {err:#}"));
    assert!(reconcat.exists(), "concat output missing");

    let overlap = bijux_dna_stages_vcf::vcf_io::vcf_panel_overlap(&input_vcfgz, &reconcat)
        .unwrap_or_else(|err| panic!("overlap check: {err:#}"));
    let shared =
        overlap.get("shared_variants_count").and_then(serde_json::Value::as_u64).unwrap_or(0);
    assert!(shared > 0, "expected non-zero overlap against self-concat");
}

#[test]
fn vcf_ref_match_check_refuses_build_mismatch() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err:#}"));
    let input = dir.path().join("x.vcf");
    std::fs::write(
        &input,
        "##fileformat=VCFv4.2\n##reference=GRCh37\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t1\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\n",
    )
    .unwrap_or_else(|err| panic!("write vcf: {err:#}"));
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

#[test]
fn sample_observation_matches_plain_and_bgzf_vcf_without_weakening_validation() -> anyhow::Result<()>
{
    use bijux_dna_stages_vcf::vcf_io::{
        vcf_index_bgzip_tabix, vcf_sample_count, vcf_validate_input, VcfFieldRequirement,
    };
    let dir = tempfile::tempdir()?;
    let input = std::path::Path::new("tests/fixtures/vcf/default/input.vcf");
    let indexed = dir.path().join("indexed.vcf.gz");
    vcf_index_bgzip_tabix(input, &indexed)?;
    assert_eq!(vcf_sample_count(input)?, 1);
    assert_eq!(vcf_sample_count(&indexed)?, 1);
    let invalid = dir.path().join("duplicate_samples.vcf");
    let raw =
        std::fs::read_to_string(input)?.replace("\tFORMAT\tsample1", "\tFORMAT\tsample1\tsample1");
    std::fs::write(&invalid, raw)?;
    assert_eq!(vcf_sample_count(&invalid)?, 2);
    assert!(vcf_validate_input(
        &invalid,
        VcfFieldRequirement { require_gt: true, require_gl: false }
    )
    .is_err());
    Ok(())
}
