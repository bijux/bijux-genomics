#![cfg(test)]

//! Real acquired VCF observations must preserve sample and variant truth identities.
use super::{
    execute_vcf_stats_workflow, summarize_vcf_imputation_output_truth,
    summarize_vcf_phasing_output_truth,
};
use anyhow::Result;

fn vcf(header_samples: &[&str], payloads: &[&str], reference: &str, alternate: &str) -> String {
    format!(
        "##fileformat=VCFv4.3\n##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n##FORMAT=<ID=PS,Number=1,Type=Integer,Description=\"Phase set\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t{}\n1\t10\t.\t{reference}\t{alternate}\t50\tPASS\t.\tGT:PS\t{}\n",
        header_samples.join("\t"), payloads.join("\t")
    )
}

#[test]
fn acquired_sample_payload_count_must_match_declared_header_before_truth_aggregation() -> Result<()>
{
    let root = bijux_dna_testkit::tempdir_for("vcf-output-sample-cardinality");
    let input = root.path().join("observed.vcf");
    std::fs::write(&input, vcf(&["s1"], &["0|1:101"], "A", "G"))?;
    let stats = execute_vcf_stats_workflow(&input)?;
    let phased = summarize_vcf_phasing_output_truth(&input, "vcf.phasing", "shapeit5")?;
    assert_eq!(stats.sample_count, 1);
    assert_eq!(phased.called_genotype_count, 1);
    std::fs::write(&input,
        "##fileformat=VCFv4.3\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n1\t10\t.\tA\tG\t50\tPASS\t.\n")?;
    assert_eq!(execute_vcf_stats_workflow(&input)?.sample_count, 0);
    assert_eq!(
        summarize_vcf_phasing_output_truth(&input, "vcf.phasing", "shapeit5")?.sample_count,
        0
    );
    let mut observations = Vec::new();
    for (label, headers, payloads) in [
        ("extra", vec!["s1"], vec!["0|1:101", "0|1:101"]),
        ("missing", vec!["s1", "s2"], vec!["0|1:101"]),
    ] {
        std::fs::write(&input, vcf(&headers, &payloads, "A", "G"))?;
        let stats = std::panic::catch_unwind(|| execute_vcf_stats_workflow(&input))
            .is_ok_and(|result| result.is_err());
        let phased = std::panic::catch_unwind(|| {
            summarize_vcf_phasing_output_truth(&input, "vcf.phasing", "shapeit5")
        })
        .is_ok_and(|result| result.is_err());
        observations.push((label, stats, phased));
    }
    for (label, stats, phased) in observations {
        assert!(stats, "stats must return Err without panic for {label} sample payloads");
        assert!(phased, "phasing truth must return Err without panic for {label} sample payloads");
    }
    Ok(())
}

#[test]
fn imputed_truth_joins_full_variant_identity_and_compares_unphased_alleles() -> Result<()> {
    let root = bijux_dna_testkit::tempdir_for("vcf-imputed-variant-identity");
    let truth = root.path().join("truth.vcf");
    let output = root.path().join("output.vcf");
    std::fs::write(&truth, vcf(&["sample"], &["0/1:101"], "A", "G"))?;
    let mut observations = Vec::new();
    for (label, reference, alternate, genotype, comparable) in [
        ("same", "A", "G", "0/1:101", true),
        ("unphased-order", "A", "G", "1/0:101", true),
        ("phased-order", "A", "G", "1|0:101", true),
        ("foreign-reference", "C", "G", "0/1:101", false),
        ("foreign-alternate", "A", "C", "0/1:101", false),
    ] {
        std::fs::write(&output, vcf(&["sample"], &[genotype], reference, alternate))?;
        let summary =
            summarize_vcf_imputation_output_truth(&output, Some(&truth), "vcf.impute", "beagle")?;
        observations.push((label, comparable, summary));
    }
    for (label, comparable, summary) in observations {
        assert_eq!(summary.masked_truth_match_count, u64::from(comparable), "{label}");
        assert_eq!(summary.masked_truth_mismatch_count, 0, "{label}");
        assert_eq!(summary.unresolved_count, u64::from(!comparable), "{label}");
        if comparable {
            assert!(
                summary
                    .genotype_concordance
                    .is_some_and(|value| (value - 1.0).abs() < f64::EPSILON),
                "{label}"
            );
        } else {
            assert!(summary.genotype_concordance.is_none(), "foreign variant is not comparable");
        }
    }
    Ok(())
}
