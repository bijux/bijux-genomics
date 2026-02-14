mod contracts {
    use std::path::Path;

    use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};
    use bijux_dna_domain_vcf::VcfDomainStage;
    use bijux_dna_stages_vcf::engine::{run_vcf_pipeline, VcfPipelineRequest};
    use bijux_dna_stages_vcf::invariants::{run_vcf_preflight, InvariantConfig, InputRegime};
    use bijux_dna_stages_vcf::metrics::{
        parse_vcf_call_summary, parse_vcf_filter_breakdown, parse_vcf_stats,
    };
    use bijux_dna_stages_vcf::pipeline::{
        assert_bgzip_tabix_artifacts, run_chunked_regions, run_impute_stage, run_phasing_stage,
        run_postprocess_stage, run_prepare_reference_panel_stage,
        ChunkFailurePolicy, ChunkingPlanParams, ImputationAcceptMode, ImputeBackend,
        ImputeStageParams, PhasingBackend, PhasingStageParams, PostprocessStageParams,
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
    fn vcf_dispatch_pipeline_runs_end_to_end() {
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
                ContigSpec {
                    name: "chr1".to_string(),
                    length_bp: 248956422,
                },
                ContigSpec {
                    name: "chr2".to_string(),
                    length_bp: 242193529,
                },
            ],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let out = run_vcf_pipeline(&VcfPipelineRequest {
            run_root: dir.path().to_path_buf(),
            input_vcf: input.to_path_buf(),
            species_context: species,
            sample_name: "sample1".to_string(),
            requested_stages: vec![
                VcfDomainStage::Call,
                VcfDomainStage::Filter,
                VcfDomainStage::Stats,
            ],
            production_profile: false,
            reference_fasta: None,
            prepare_panel: None,
            panel_vcf: None,
            phasing: None,
            impute: None,
            postprocess: None,
            invariants: InvariantConfig {
                require_sex_metadata_for_sex_chr: false,
                ..InvariantConfig::default()
            },
        })
        .unwrap_or_else(|err| panic!("dispatch vcf pipeline: {err}"));
        assert!(out.report_path.exists());
        assert!(out.stages.iter().any(|s| s.stage_id == "vcf.call"));
        assert!(out.stages.iter().all(|s| s.stage_manifest.exists()));
        for stage in &out.stages {
            assert!(stage.artifact_dir.join("tool_invocation.json").exists());
            assert!(stage.artifact_dir.join("tool_version.txt").exists());
            assert!(stage.artifact_dir.join("artifact_checksums.json").exists());
        }

        let resumed = run_vcf_pipeline(&VcfPipelineRequest {
            run_root: dir.path().to_path_buf(),
            input_vcf: input.to_path_buf(),
            species_context: SpeciesContext {
                species_id: "Homo sapiens".to_string(),
                build_id: "GRCh38".to_string(),
                contig_set_digest:
                    "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
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
                    ContigSpec {
                        name: "chr1".to_string(),
                        length_bp: 248956422,
                    },
                    ContigSpec {
                        name: "chr2".to_string(),
                        length_bp: 242193529,
                    },
                ],
                sex_system: "xy".to_string(),
                par_policy: "grch38_par".to_string(),
                default_coverage_regime: None,
            },
            sample_name: "sample1".to_string(),
            requested_stages: vec![
                VcfDomainStage::Call,
                VcfDomainStage::Filter,
                VcfDomainStage::Stats,
            ],
            production_profile: false,
            reference_fasta: None,
            prepare_panel: None,
            panel_vcf: None,
            phasing: None,
            impute: None,
            postprocess: None,
            invariants: InvariantConfig {
                require_sex_metadata_for_sex_chr: false,
                ..InvariantConfig::default()
            },
        })
        .unwrap_or_else(|err| panic!("dispatch resume vcf pipeline: {err}"));
        assert!(
            resumed.stages.iter().all(|s| s.runtime.wall_time_ms == 0),
            "resume run should skip stages with matching checksums"
        );
    }

    #[test]
    fn vcf_preflight_emits_invariants_and_normalized_index_artifacts() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "x".repeat(64),
            contigs: vec![
                ContigSpec {
                    name: "1".to_string(),
                    length_bp: 1_000_000,
                },
                ContigSpec {
                    name: "2".to_string(),
                    length_bp: 1_000_000,
                },
                ContigSpec {
                    name: "X".to_string(),
                    length_bp: 1_000_000,
                },
                ContigSpec {
                    name: "Y".to_string(),
                    length_bp: 1_000_000,
                },
            ],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let out = run_vcf_preflight(
            input,
            dir.path(),
            &species,
            &InvariantConfig {
                allow_contig_aliasing: true,
                require_sex_metadata_for_sex_chr: false,
                ..InvariantConfig::default()
            },
        )
        .unwrap_or_else(|err| panic!("run_vcf_preflight: {err}"));
        assert!(out.normalized_input.exists());
        assert!(out.index_path.exists());
        assert!(out.invariants_json.exists());
        assert!(out.overlap_json.exists());
        assert!(matches!(
            out.regime.regime,
            InputRegime::GtOnly | InputRegime::Mixed | InputRegime::GlOnly
        ));
    }

    #[test]
    fn vcf_preflight_refuses_chr_prefix_mismatch_by_default() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("chr_input.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n##reference=GRCh38\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\nchr1\t1\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\n",
        )
        .unwrap_or_else(|err| panic!("write fixture: {err}"));
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "x".repeat(64),
            contigs: vec![ContigSpec {
                name: "1".to_string(),
                length_bp: 1_000_000,
            }],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let err = run_vcf_preflight(&input, &dir.path().join("out"), &species, &InvariantConfig::default())
            .expect_err("chr prefix mismatch must refuse by default");
        assert!(err.to_string().contains("chr prefix mismatch"));
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
                ContigSpec {
                    name: "chr1".to_string(),
                    length_bp: 248956422,
                },
                ContigSpec {
                    name: "chr2".to_string(),
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
        let manifest_raw = std::fs::read_to_string(&outputs.phasing_manifest_json)
            .unwrap_or_else(|err| panic!("read phasing manifest: {err}"));
        let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)
            .unwrap_or_else(|err| panic!("parse phasing manifest: {err}"));
        let digest = manifest
            .get("tool_digest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        assert!(
            digest.starts_with("sha256:"),
            "phasing manifest missing tool_digest sha256"
        );
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
        assert!(err
            .to_string()
            .contains("GL-only/GP-only inputs are refused"));
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
                truth_vcf: Some(input.clone()),
                imputation_accept_mode: ImputationAcceptMode::MarkNonProduction,
            },
        )
        .unwrap_or_else(|err| panic!("run glimpse impute: {err}"));
        assert!(outputs.imputed_vcf.exists());
        assert!(outputs.imputation_manifest_json.exists());
        assert!(outputs.imputation_qc_json.exists());
        assert!(outputs.imputation_qc_tsv.exists());
        assert!(outputs.panel_mismatch_diagnostics_json.exists());
        assert!(outputs.info_hist_json.exists());
        assert!(outputs.warnings_json.exists());
        assert!(outputs.imputation_accept_json.exists());
        let manifest_raw = std::fs::read_to_string(&outputs.imputation_manifest_json)
            .unwrap_or_else(|err| panic!("read imputation manifest: {err}"));
        let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)
            .unwrap_or_else(|err| panic!("parse imputation manifest: {err}"));
        let digest = manifest
            .get("tool_digest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        assert!(
            digest.starts_with("sha256:"),
            "imputation manifest missing tool_digest sha256"
        );
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
                truth_vcf: None,
                imputation_accept_mode: ImputationAcceptMode::Fail,
            },
        )
        .expect_err("unphased GT should fail minimac4");
        let msg = err.to_string();
        assert!(
            msg.contains("phased GT")
                || msg.contains("m3vcf")
                || msg.contains("compatib")
                || msg.contains("requires")
                || msg.contains("contig digest/namespace mismatch"),
            "unexpected minimac refusal message: {msg}"
        );
    }

    #[test]
    fn imputation_qc_schema_is_stable_across_backends() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("qc_schema_input.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t100\t.\tA\tG\t60\tPASS\t.\tGT:GL\t0/1:-0.1,-1.0,-2.0\n1\t140\t.\tC\tT\t60\tPASS\t.\tGT:GL\t0/0:-0.1,-1.0,-2.0\n",
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
        let glimpse = run_impute_stage(
            &input,
            &dir.path().join("glimpse"),
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
                truth_vcf: None,
                imputation_accept_mode: ImputationAcceptMode::MarkNonProduction,
            },
        )
        .unwrap_or_else(|err| panic!("run glimpse impute: {err}"));
        let impute5 = run_impute_stage(
            &input,
            &dir.path().join("impute5"),
            &species,
            &ImputeStageParams {
                species_id: "Homo sapiens".to_string(),
                build_id: "GRCh38".to_string(),
                backend: ImputeBackend::Impute5,
                panel_id: Some("hsapiens_grch38_mini".to_string()),
                map_id: Some("hsapiens_grch38_chr_map".to_string()),
                threads: 2,
                seed: 42,
                emit_ds: true,
                emit_gp: true,
                truth_vcf: None,
                imputation_accept_mode: ImputationAcceptMode::MarkNonProduction,
            },
        )
        .unwrap_or_else(|err| panic!("run impute5 impute: {err}"));
        let a = std::fs::read_to_string(glimpse.imputation_qc_json)
            .unwrap_or_else(|err| panic!("read glimpse qc: {err}"));
        let b = std::fs::read_to_string(impute5.imputation_qc_json)
            .unwrap_or_else(|err| panic!("read impute5 qc: {err}"));
        let av: serde_json::Value =
            serde_json::from_str(&a).unwrap_or_else(|err| panic!("parse glimpse qc: {err}"));
        let bv: serde_json::Value =
            serde_json::from_str(&b).unwrap_or_else(|err| panic!("parse beagle qc: {err}"));
        let mut a_keys = av
            .as_object()
            .unwrap_or_else(|| panic!("glimpse qc must be object"))
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let mut b_keys = bv
            .as_object()
            .unwrap_or_else(|| panic!("impute5 qc must be object"))
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        a_keys.sort();
        b_keys.sort();
        assert_eq!(
            a_keys, b_keys,
            "qc schema keys must remain cross-run stable"
        );
    }

    #[test]
    fn impute_stage_refuses_unsupported_ploidy_models() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("triploid.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0/0/1\n",
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
        let err = run_impute_stage(
            &input,
            dir.path(),
            &species,
            &ImputeStageParams {
                species_id: "Homo sapiens".to_string(),
                build_id: "GRCh38".to_string(),
                backend: ImputeBackend::Impute5,
                panel_id: Some("hsapiens_grch38_mini".to_string()),
                map_id: Some("hsapiens_grch38_chr_map".to_string()),
                threads: 2,
                seed: 42,
                emit_ds: true,
                emit_gp: true,
                truth_vcf: None,
                imputation_accept_mode: ImputationAcceptMode::Fail,
            },
        )
        .expect_err("triploid must be refused");
        assert!(err.to_string().contains("unsupported ploidy model"));
    }

    #[test]
    fn postprocess_stage_merges_and_emits_contract_artifacts() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
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
                ContigSpec {
                    name: "chr1".to_string(),
                    length_bp: 248956422,
                },
                ContigSpec {
                    name: "chr2".to_string(),
                    length_bp: 242193529,
                },
            ],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let out = run_postprocess_stage(
            Path::new("tests/fixtures/vcf/default/input.vcf"),
            dir.path(),
            &species,
            &PostprocessStageParams {
                species_id: "Homo sapiens".to_string(),
                build_id: "GRCh38".to_string(),
                per_chr_inputs: vec![],
                retain_info_fields: vec![],
                remove_info_fields: vec!["MQ".to_string()],
                compression_level: 6,
                compression_threads: 2,
                emit_bcf: true,
                normalize_indels: true,
                run_level_checksums_path: Some(
                    dir.path().join("run_level_artifact_checksums.json"),
                ),
            },
        )
        .unwrap_or_else(|err| panic!("postprocess stage: {err}"));
        assert!(out.merged_vcf.exists());
        assert!(out.merged_tbi.exists());
        assert!(out.merged_bcf.is_some());
        assert!(out.artifact_checksums_json.exists());
        assert!(out.validate_outputs_json.exists());
        assert!(out.logs_txt.exists());
    }
}
