mod contracts {
    use std::path::Path;

    use bijux_dna_stages_vcf::metrics::{
        parse_vcf_call_summary, parse_vcf_filter_breakdown, parse_vcf_stats,
    };
    use bijux_dna_stages_vcf::pipeline::{assert_bgzip_tabix_artifacts, run_toy_vcf_pipeline};
    use bijux_dna_stages_vcf::stage_specs::{supported_vcf_stages, vcf_stage_catalog};
    use bijux_dna_stages_vcf::wrappers::verify_tool_wrapper;

    #[test]
    fn vcf_stats_parser_fixture_roundtrip() {
        let path = Path::new("tests/fixtures/vcf_stats/default/stats.txt");
        let metrics =
            parse_vcf_stats(path).unwrap_or_else(|err| panic!("parse stats fixture: {err}"));
        assert_eq!(metrics.schema_version, "bijux.vcf.stats.v1");
        assert_eq!(metrics.sample_name, "sample1");
        assert_eq!(metrics.variants_total, 12);
        assert_eq!(metrics.snps, 9);
        assert_eq!(metrics.indels, 3);
        assert_eq!(metrics.ti_tv, Some(2.25));
        assert_eq!(metrics.filter_breakdown.get("PASS"), Some(&10));
        assert_eq!(metrics.depth_distribution.get("0-9"), Some(&4));
    }

    #[test]
    fn vcf_call_and_filter_parsers_fixture_roundtrip() {
        let call = parse_vcf_call_summary(
            Path::new("tests/fixtures/vcf/default/input.vcf"),
            "sample1",
        )
        .unwrap_or_else(|err| panic!("parse call fixture: {err}"));
        assert_eq!(call.variants_called, 4);
        assert_eq!(call.snps, 3);

        let filter =
            parse_vcf_filter_breakdown(Path::new("tests/fixtures/vcf/default/input.vcf"), "sample1")
                .unwrap_or_else(|err| panic!("parse filter fixture: {err}"));
        assert_eq!(filter.variants_in, 4);
        assert_eq!(filter.filter_breakdown.get("PASS"), Some(&3));
    }

    #[test]
    fn vcf_toy_pipeline_runs_end_to_end() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let (_call, _filter, stats, metrics) = run_toy_vcf_pipeline(input, dir.path(), "sample1")
            .unwrap_or_else(|err| panic!("toy vcf pipeline: {err}"));
        assert!(stats.exists());
        assert_eq!(metrics.schema_version, "bijux.vcf.stats.v1");
        assert!(metrics.variants_total > 0);
    }

    #[test]
    fn vcf_tool_wrapper_enforces_version_and_help_contracts() {
        let check = verify_tool_wrapper(
            "bcftools",
            "bcftools 1.20\nUsing htslib 1.20",
            "Usage: bcftools [OPTIONS] <command>",
            "bcftools [0-9]+[.][0-9]+",
        )
        .unwrap_or_else(|err| panic!("wrapper check: {err}"));
        assert_eq!(check.tool, "bcftools");
        assert!(check.help_ok);
    }

    #[test]
    fn vcf_artifact_correctness_requires_bgzip_plus_tabix_index() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let vcf = dir.path().join("x.vcf.gz");
        std::fs::write(&vcf, b"##fileformat=VCFv4.2\n").unwrap_or_else(|err| panic!("{err}"));
        let tbi = dir.path().join("x.vcf.gz.tbi");
        let err = assert_bgzip_tabix_artifacts(&vcf, &tbi)
            .expect_err("missing tbi must fail artifact correctness");
        assert!(err.to_string().contains("tabix index missing"));
    }

    #[test]
    fn no_supported_vcf_stage_without_smoke_and_schema() {
        for spec in vcf_stage_catalog() {
            if supported_vcf_stages().contains(&spec.stage_id) {
                assert!(spec.smoke_supported, "{} missing smoke", spec.stage_id);
                assert!(spec.parser_supported, "{} missing parser", spec.stage_id);
                assert!(
                    !spec.metrics_schema.is_empty(),
                    "{} missing schema",
                    spec.stage_id
                );
            }
        }
    }
}
