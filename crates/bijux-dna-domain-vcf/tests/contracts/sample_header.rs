use bijux_dna_domain_vcf::artifacts::sample_count_from_vcf_text;

#[test]
fn sample_cardinality_distinguishes_sites_only_and_sample_columns() -> anyhow::Result<()> {
    let header = "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO";
    assert_eq!(sample_count_from_vcf_text(header)?, 0);
    assert_eq!(sample_count_from_vcf_text(&format!("{header}\tFORMAT"))?, 0);
    assert_eq!(sample_count_from_vcf_text(&format!("{header}\tFORMAT\ta\tb\n"))?, 2);
    Ok(())
}

#[test]
fn sample_counting_does_not_claim_valid_or_unique_sample_identity() -> anyhow::Result<()> {
    assert_eq!(
        sample_count_from_vcf_text(
            "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ta\ta\t\n"
        )?,
        3
    );
    Ok(())
}

#[test]
fn sample_count_requires_the_vcf_column_header() {
    assert!(
        sample_count_from_vcf_text("##fileformat=VCFv4.2\n1\t1\t.\tA\tC\t30\tPASS\t.\n").is_err()
    );
}
