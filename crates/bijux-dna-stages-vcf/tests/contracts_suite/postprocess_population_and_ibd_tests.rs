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
        assert!(out.final_manifest_json.exists());
        assert!(out.logs_txt.exists());
    }

    #[test]
    fn postprocess_removes_invalid_records_and_records_normalization_summary() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("invalid_records.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t1\t.\tA\t.\t60\t.\tMQ=50\tGT\t0/1\n1\t2\t.\tAA\tA\t60\t.\tMQ=50\tGT\t0/1\n",
        )
        .unwrap_or_else(|err| panic!("write invalid fixture: {err}"));
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "x".repeat(64),
            contigs: vec![ContigSpec {
                name: "1".to_string(),
                length_bp: 248_956_422,
            }],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let out = run_postprocess_stage(
            &input,
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
                emit_bcf: false,
                normalize_indels: true,
                run_level_checksums_path: None,
            },
        )
        .unwrap_or_else(|err| panic!("postprocess invalid fixture: {err}"));
        let manifest_raw = std::fs::read_to_string(&out.final_manifest_json)
            .unwrap_or_else(|err| panic!("read final manifest: {err}"));
        let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)
            .unwrap_or_else(|err| panic!("parse final manifest: {err}"));
        assert_eq!(
            manifest
                .get("normalization")
                .and_then(|v| v.get("invalid_records_removed"))
                .and_then(|v| v.as_u64())
                .unwrap_or_default(),
            1
        );
    }

    #[test]
    fn pca_stage_emits_eigen_artifacts_with_preprocessing_contract() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let out = run_pca_stage(
            Path::new("tests/fixtures/vcf/default/input.vcf"),
            dir.path(),
            &PcaStageParams::default(),
        )
        .unwrap_or_else(|err| panic!("run pca stage: {err}"));
        assert!(out.eigenvec_tsv.exists());
        assert!(out.eigenval_tsv.exists());
        assert!(out.pca_manifest_json.exists());
    }

    #[test]
    fn population_structure_stage_emits_structured_outputs() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let out = run_population_structure_stage(
            Path::new("tests/fixtures/vcf/default/input.vcf"),
            dir.path(),
            &PopulationStructureStageParams::default(),
        )
        .unwrap_or_else(|err| panic!("run population_structure stage: {err}"));
        assert!(out.pruned_variants_tsv.exists());
        assert!(out.population_structure_json.exists());
    }

    #[test]
    fn admixture_stage_refuses_when_runtime_not_available() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let err = run_admixture_stage(
            Path::new("tests/fixtures/vcf/default/input.vcf"),
            dir.path(),
            &AdmixtureStageParams::default(),
        )
        .expect_err("admixture should refuse until container/runtime is enabled");
        assert!(err.to_string().contains("refusal"));
    }

    #[test]
    fn roh_stage_emits_segments_and_metrics() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let out = run_roh_stage(
            Path::new("tests/fixtures/vcf/default/input.vcf"),
            dir.path(),
            &RohStageParams {
                min_snp_density_per_mb: 0.00001,
                min_segment_kb: 0,
                max_gap_bp: 10_000_000,
            },
        )
        .unwrap_or_else(|err| panic!("run roh stage: {err}"));
        assert!(out.roh_segments_tsv.exists());
        assert!(out.roh_summary_json.exists());
        assert!(out.roh_metrics_json.exists());
    }

    #[test]
    fn roh_stage_refuses_when_density_below_threshold() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let err = run_roh_stage(
            Path::new("tests/fixtures/vcf/default/input.vcf"),
            dir.path(),
            &RohStageParams {
                min_snp_density_per_mb: 1_000_000.0,
                min_segment_kb: 500,
                max_gap_bp: 1_000_000,
            },
        )
        .expect_err("roh should refuse under impossible density requirement");
        assert!(err.to_string().contains("density"));
    }

    #[test]
    fn ibd_stage_emits_segments_filtered_and_metrics() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let out = run_ibd_stage(
            Path::new("tests/fixtures/vcf/default/input.vcf"),
            dir.path(),
            &IbdStageParams {
                min_variant_density_per_mb: 0.00001,
                max_missingness: 1.0,
                min_samples: 1,
                min_segment_cm: 1.0,
            },
        )
        .unwrap_or_else(|err| panic!("run ibd stage: {err}"));
        assert!(out.ibd_segments_tsv.exists());
        assert!(out.ibd_filtered_segments_tsv.exists());
        assert!(out.ibd_summary_json.exists());
        assert!(out.ibd_metrics_json.exists());
    }

    #[test]
    fn ibd_stage_refuses_when_readiness_fails() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let err = run_ibd_stage(
            Path::new("tests/fixtures/vcf/default/input.vcf"),
            dir.path(),
            &IbdStageParams {
                min_variant_density_per_mb: 1_000_000.0,
                max_missingness: 0.0,
                min_samples: 100,
                min_segment_cm: 2.0,
            },
        )
        .expect_err("ibd must refuse when readiness constraints fail");
        assert!(err.to_string().contains("refusal"));
    }

    #[test]
    fn demography_stage_consumes_ibd_segments_and_emits_ne_metrics() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let ibd_segments = dir.path().join("ibd_segments.tsv");
        std::fs::write(
            &ibd_segments,
            "sample_a\tsample_b\tcontig\tstart\tend\tlength_cm\ns1\ts2\tchr1\t1000\t5000\t3.50\n",
        )
        .unwrap_or_else(|err| panic!("write ibd segments fixture: {err}"));
        let out = run_demography_stage(
            &ibd_segments,
            &dir.path().join("demography"),
            &DemographyStageParams { min_segments: 1 },
        )
        .unwrap_or_else(|err| panic!("run demography: {err}"));
        assert!(out.ne_trajectory_tsv.exists());
        assert!(out.demography_metrics_json.exists());
    }
