mod contracts {
    use std::path::Path;

    use bijux_dna_runtime::recording::{prepare_tool_run_dirs, write_run_manifest, RunArtifactInput};
    use bijux_dna_runtime::RunProvenanceV1;
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
        let call =
            parse_vcf_call_summary(Path::new("tests/fixtures/vcf/default/input.vcf"), "sample1")
                .unwrap_or_else(|err| panic!("parse call fixture: {err}"));
        assert_eq!(call.variants_called, 4);
        assert_eq!(call.snps, 3);

        let filter = parse_vcf_filter_breakdown(
            Path::new("tests/fixtures/vcf/default/input.vcf"),
            "sample1",
        )
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
        let err = match assert_bgzip_tabix_artifacts(&vcf, &tbi) {
            Ok(()) => panic!("missing tbi must fail artifact correctness"),
            Err(err) => err,
        };
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

    #[test]
    fn vcf_toy_downstream_manifest_is_deterministic_and_hashed() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let out_dir = dir.path().join("pipeline_out");
        std::fs::create_dir_all(&out_dir).unwrap_or_else(|err| panic!("mkdir out: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let (called, filtered, stats, _metrics) = run_toy_vcf_pipeline(input, &out_dir, "sample1")
            .unwrap_or_else(|err| panic!("toy vcf pipeline: {err}"));

        // Downstream placeholders to validate artifact layout/checksum support for new stage kinds.
        let phased = out_dir.join("phased.vcf.gz");
        let imputed = out_dir.join("imputed.vcf.gz");
        let ibd = out_dir.join("ibd_segments.tsv");
        let roh = out_dir.join("roh_segments.tsv");
        std::fs::write(&phased, b"phased\n").unwrap_or_else(|err| panic!("write phased: {err}"));
        std::fs::write(&imputed, b"imputed\n")
            .unwrap_or_else(|err| panic!("write imputed: {err}"));
        std::fs::write(&ibd, b"sample_a\tsample_b\tcm\n")
            .unwrap_or_else(|err| panic!("write ibd: {err}"));
        std::fs::write(&roh, b"sample\tchr\tstart\tend\n")
            .unwrap_or_else(|err| panic!("write roh: {err}"));

        let tools_root = dir.path().join("tools");
        let run_dirs = prepare_tool_run_dirs(&tools_root, "bcftools", "deterministic-run")
            .unwrap_or_else(|err| panic!("prepare run dirs: {err}"));
        std::fs::write(&run_dirs.manifest_path, b"{}\n")
            .unwrap_or_else(|err| panic!("write execution manifest: {err}"));
        std::fs::write(&run_dirs.metrics_path, b"{}\n")
            .unwrap_or_else(|err| panic!("write metrics: {err}"));

        let tbi = out_dir.join("filtered.vcf.gz.tbi");
        let provenance = RunProvenanceV1 {
            schema_version: "bijux.run_provenance.v1".to_string(),
            pipeline_id: "vcf-to-vcf__downstream_toy__v2".to_string(),
            tool_version: "1.20".to_string(),
            tool_image_digest: Some("sha256:toy".to_string()),
            params_hash: "sha256:params".to_string(),
            input_hashes: vec!["sha256:input".to_string()],
            git_commit: "dev".to_string(),
            build_profile: "test".to_string(),
            reference_genome: Some("GRCh37".to_string()),
            plan_hash: Some("sha256:plan".to_string()),
        };
        let extra_artifacts = vec![
            RunArtifactInput {
                name: "vcf.called",
                path: called,
            },
            RunArtifactInput {
                name: "vcf.filtered",
                path: filtered,
            },
            RunArtifactInput {
                name: "vcf.filtered_index",
                path: tbi,
            },
            RunArtifactInput {
                name: "vcf.stats",
                path: stats,
            },
            RunArtifactInput {
                name: "vcf.phased",
                path: phased,
            },
            RunArtifactInput {
                name: "vcf.imputed",
                path: imputed,
            },
            RunArtifactInput {
                name: "vcf.ibd_segments",
                path: ibd,
            },
            RunArtifactInput {
                name: "vcf.roh_segments",
                path: roh,
            },
        ];

        write_run_manifest(
            &run_dirs,
            "vcf.postprocess",
            "bcftools",
            &provenance,
            None,
            &extra_artifacts,
        )
        .unwrap_or_else(|err| panic!("write run manifest: {err}"));
        let first_hash = {
            use sha2::Digest;
            let bytes = std::fs::read(&run_dirs.run_manifest_path)
                .unwrap_or_else(|err| panic!("read run manifest first: {err}"));
            let mut hasher = sha2::Sha256::new();
            hasher.update(bytes);
            format!("{:x}", hasher.finalize())
        };
        write_run_manifest(
            &run_dirs,
            "vcf.postprocess",
            "bcftools",
            &provenance,
            None,
            &extra_artifacts,
        )
        .unwrap_or_else(|err| panic!("rewrite run manifest: {err}"));
        let second_hash = {
            use sha2::Digest;
            let bytes = std::fs::read(&run_dirs.run_manifest_path)
                .unwrap_or_else(|err| panic!("read run manifest second: {err}"));
            let mut hasher = sha2::Sha256::new();
            hasher.update(bytes);
            format!("{:x}", hasher.finalize())
        };
        assert_eq!(first_hash, second_hash, "manifest hash must be deterministic");

        let raw = std::fs::read_to_string(&run_dirs.run_manifest_path)
            .unwrap_or_else(|err| panic!("read run manifest: {err}"));
        let manifest: serde_json::Value =
            serde_json::from_str(&raw).unwrap_or_else(|err| panic!("parse run manifest: {err}"));
        let outputs = manifest
            .get("output_artifacts")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("missing output_artifacts"));
        for required in [
            "vcf.phased",
            "vcf.imputed",
            "vcf.ibd_segments",
            "vcf.roh_segments",
        ] {
            assert!(
                outputs.iter().any(|entry| {
                    entry.get("name").and_then(serde_json::Value::as_str) == Some(required)
                        && entry
                            .get("sha256")
                            .and_then(serde_json::Value::as_str)
                            .is_some_and(|sha| sha.len() == 64)
                }),
                "missing hashed output artifact record for {required}"
            );
        }
    }
}
