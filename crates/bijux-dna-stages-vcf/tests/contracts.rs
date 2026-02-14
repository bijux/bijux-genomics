mod contracts {
    use std::path::Path;

    use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};
    use bijux_dna_stages_vcf::metrics::{
        parse_vcf_call_summary, parse_vcf_filter_breakdown, parse_vcf_stats,
    };
    use bijux_dna_stages_vcf::pipeline::{
        assert_bgzip_tabix_artifacts, run_chunked_regions, run_prepare_reference_panel_stage,
        run_impute_stage, run_phasing_stage, run_toy_vcf_pipeline, ChunkFailurePolicy,
        ChunkingPlanParams, ImputeBackend, ImputeStageParams, PhasingBackend, PhasingStageParams,
        PrepareReferencePanelParams,
    };
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
    fn prepare_reference_panel_stage_writes_manifest_and_overlap_artifacts() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let panel = Path::new("tests/fixtures/vcf/default/input.vcf");
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
                .to_string(),
            contigs: vec![
                ContigSpec {
                    name: "1".to_string(),
                    length_bp: 248956422,
                },
                ContigSpec {
                    name: "2".to_string(),
                    length_bp: 242193529,
                },
            ],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let outputs = run_prepare_reference_panel_stage(
            input,
            panel,
            dir.path(),
            &species,
            &PrepareReferencePanelParams {
                species_id: "Homo sapiens".to_string(),
                build_id: "GRCh38".to_string(),
                panel_id: Some("hsapiens_grch38_mini".to_string()),
                map_id: Some("hsapiens_grch38_chr_map".to_string()),
            },
        )
        .unwrap_or_else(|err| panic!("prepare_reference_panel: {err}"));
        assert!(outputs.panel_manifest_json.exists());
        assert!(outputs.overlap_json.exists());
        assert!(outputs.overlap_tsv.exists());
        assert!(outputs.chunks_json.exists());
    }

    #[test]
    fn chunked_regions_emit_chunks_json_and_merged_output() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
                .to_string(),
            contigs: vec![
                ContigSpec {
                    name: "1".to_string(),
                    length_bp: 248956422,
                },
                ContigSpec {
                    name: "2".to_string(),
                    length_bp: 242193529,
                },
            ],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let outputs = run_chunked_regions(
            input,
            input,
            dir.path(),
            &species,
            &ChunkingPlanParams {
                window_size_bp: 10_000_000,
                overlap_bp: 10_000,
                ..ChunkingPlanParams::default()
            },
            ChunkFailurePolicy::FailFast,
            None,
        )
        .unwrap_or_else(|err| panic!("chunk run: {err}"));
        assert!(outputs.merged_vcf.exists());
        assert!(outputs.chunks_json.exists());
    }

    #[test]
    fn phasing_stage_emits_expected_artifacts_for_shapeit5() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
                .to_string(),
            contigs: vec![
                ContigSpec {
                    name: "1".to_string(),
                    length_bp: 248956422,
                },
                ContigSpec {
                    name: "2".to_string(),
                    length_bp: 242193529,
                },
            ],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let outputs = run_phasing_stage(
            input,
            dir.path(),
            &species,
            &PhasingStageParams {
                species_id: "Homo sapiens".to_string(),
                build_id: "GRCh38".to_string(),
                backend: PhasingBackend::Shapeit5,
                map_id: Some("hsapiens_grch38_chr_map".to_string()),
                threads: 2,
                seed: 7,
                region: Some("1:1-1000000".to_string()),
                allow_gl_only_input: false,
            },
        )
        .unwrap_or_else(|err| panic!("phasing stage: {err}"));
        assert!(outputs.phased_vcf.exists());
        assert!(outputs.phased_tbi.exists());
        assert!(outputs.phasing_manifest_json.exists());
        assert!(outputs.phasing_qc_json.exists());
        assert!(outputs.switch_error_proxy_tsv.exists());
    }

    #[test]
    fn phasing_stage_refuses_unknown_species_build_mismatch() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
                .to_string(),
            contigs: vec![ContigSpec {
                name: "1".to_string(),
                length_bp: 248956422,
            }],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let err = run_phasing_stage(
            input,
            dir.path(),
            &species,
            &PhasingStageParams {
                species_id: "Homo sapiens".to_string(),
                build_id: "GRCh37".to_string(),
                backend: PhasingBackend::Beagle,
                map_id: None,
                threads: 1,
                seed: 1,
                region: None,
                allow_gl_only_input: false,
            },
        )
        .expect_err("species/build mismatch must fail");
        assert!(err.to_string().contains("species/build mismatch"));
    }

    #[test]
    fn phasing_stage_refuses_gl_only_without_backend_opt_in() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("gl_only.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t100\t.\tA\tG\t60\tPASS\t.\tGP\t0.1,0.8,0.1\n",
        )
        .unwrap_or_else(|err| panic!("write gl-only fixture: {err}"));
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
                .to_string(),
            contigs: vec![ContigSpec {
                name: "1".to_string(),
                length_bp: 248956422,
            }],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let err = run_phasing_stage(
            &input,
            dir.path(),
            &species,
            &PhasingStageParams {
                species_id: "Homo sapiens".to_string(),
                build_id: "GRCh38".to_string(),
                backend: PhasingBackend::Shapeit5,
                map_id: Some("hsapiens_grch38_chr_map".to_string()),
                threads: 2,
                seed: 11,
                region: None,
                allow_gl_only_input: false,
            },
        )
        .expect_err("GL-only should fail without explicit support");
        assert!(err.to_string().contains("GL-only/GP-only inputs are refused"));
    }

    #[test]
    fn phasing_stage_allows_gl_only_with_backend_opt_in() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("gl_only_allowed.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t100\t.\tA\tG\t60\tPASS\t.\tGL\t-0.1,-1.0,-2.0\n",
        )
        .unwrap_or_else(|err| panic!("write gl-only fixture: {err}"));
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
                .to_string(),
            contigs: vec![ContigSpec {
                name: "1".to_string(),
                length_bp: 248956422,
            }],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let outputs = run_phasing_stage(
            &input,
            dir.path(),
            &species,
            &PhasingStageParams {
                species_id: "Homo sapiens".to_string(),
                build_id: "GRCh38".to_string(),
                backend: PhasingBackend::Beagle,
                map_id: None,
                threads: 2,
                seed: 12,
                region: None,
                allow_gl_only_input: true,
            },
        )
        .unwrap_or_else(|err| panic!("GL-only explicit support should pass: {err}"));
        assert!(outputs.phasing_manifest_json.exists());
    }

    #[test]
    fn impute_stage_runs_glimpse_for_lowcov_gl_input() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("gl_input.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t100\t.\tA\tG\t60\tPASS\t.\tGL\t-0.1,-1.0,-2.0\n",
        )
        .unwrap_or_else(|err| panic!("write input: {err}"));
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
                .to_string(),
            contigs: vec![ContigSpec {
                name: "1".to_string(),
                length_bp: 248956422,
            }],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let outputs = run_impute_stage(
            &input,
            dir.path(),
            &species,
            &ImputeStageParams {
                species_id: "Homo sapiens".to_string(),
                build_id: "GRCh38".to_string(),
                backend: ImputeBackend::Glimpse,
                panel_id: Some("hsapiens_grch38_mini".to_string()),
                map_id: Some("hsapiens_grch38_chr_map".to_string()),
                threads: 2,
                seed: 42,
                emit_ds: false,
                emit_gp: true,
            },
        )
        .unwrap_or_else(|err| panic!("run glimpse impute: {err}"));
        assert!(outputs.imputed_vcf.exists());
        assert!(outputs.imputation_manifest_json.exists());
        assert!(outputs.imputation_qc_json.exists());
    }

    #[test]
    fn impute_stage_refuses_minimac_without_phased_gt() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
                .to_string(),
            contigs: vec![ContigSpec {
                name: "1".to_string(),
                length_bp: 248956422,
            }],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let err = run_impute_stage(
            input,
            dir.path(),
            &species,
            &ImputeStageParams {
                species_id: "Homo sapiens".to_string(),
                build_id: "GRCh38".to_string(),
                backend: ImputeBackend::Minimac4,
                panel_id: Some("hsapiens_grch38_mini".to_string()),
                map_id: Some("hsapiens_grch38_chr_map".to_string()),
                threads: 2,
                seed: 42,
                emit_ds: true,
                emit_gp: true,
            },
        )
        .expect_err("unphased GT should fail minimac4");
        assert!(err.to_string().contains("phased GT"));
    }
}
