    #[test]
    fn filter_stage_emits_breakdown_artifacts() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let out = run_filter_stage_real(
            input,
            dir.path(),
            &bijux_dna_domain_vcf::params::VcfFilterParams::default(),
        )
        .unwrap_or_else(|err| panic!("run filter stage: {err}"));
        assert!(out.filtered_vcf.exists());
        assert!(out.filtered_tbi.exists());
        assert!(out.filter_breakdown_json.exists());
        assert!(out.filter_breakdown_tsv.exists());
    }

    #[test]
    fn qc_stage_computes_outputs_and_skips_hwe_for_ancient_default() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let out = run_qc_stage(
            input,
            dir.path(),
            &QcStageParams {
                sample_name: "sample1".to_string(),
                is_ancient_dna: true,
                allow_hwe_for_ancient: false,
                production_profile: false,
                pre_filter_vcf: None,
            },
        )
        .unwrap_or_else(|err| panic!("run qc stage: {err}"));
        assert!(out.qc_summary_json.exists());
        assert!(out.qc_tables_tsv.exists());
        assert!(out.qc_histograms_json.exists());
    }

    #[test]
    fn stats_stage_emits_bcftools_stats_and_json() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let out = run_stats_stage_real(
            input,
            dir.path(),
            &bijux_dna_domain_vcf::params::VcfStatsParams {
                sample_name: "sample1".to_string(),
                ..bijux_dna_domain_vcf::params::VcfStatsParams::default()
            },
        )
        .unwrap_or_else(|err| panic!("run stats stage: {err}"));
        assert!(out.bcftools_stats_txt.exists());
        assert!(out.stats_json.exists());
    }

    #[test]
    fn vcf_pipeline_runs_qc_stage() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "x".repeat(64),
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
            requested_stages: vec![VcfDomainStage::Qc],
            production_profile: false,
            reference_fasta: None,
            prepare_panel: None,
            panel_vcf: None,
            damage_filter: None,
            gl_propagation: None,
            qc: Some(QcStageParams {
                sample_name: "sample1".to_string(),
                is_ancient_dna: true,
                allow_hwe_for_ancient: false,
                production_profile: false,
                pre_filter_vcf: None,
            }),
            phasing: None,
            impute: None,
            postprocess: None,
            invariants: InvariantConfig {
                allow_contig_aliasing: true,
                require_sex_metadata_for_sex_chr: false,
                ..InvariantConfig::default()
            },
        })
        .unwrap_or_else(|err| panic!("run qc pipeline: {err}"));
        let stage = out
            .stages
            .iter()
            .find(|s| s.stage_id == "vcf.qc")
            .unwrap_or_else(|| panic!("missing qc stage"));
        assert!(stage.artifact_dir.join("qc_summary.json").exists());
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
