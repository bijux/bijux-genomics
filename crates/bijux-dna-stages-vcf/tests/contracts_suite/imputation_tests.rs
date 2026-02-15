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
                chunk_window_bp: None,
                chunk_overlap_bp: 0,
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
                chunk_window_bp: None,
                chunk_overlap_bp: 0,
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
                chunk_window_bp: None,
                chunk_overlap_bp: 0,
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
                map_id: None,
                threads: 2,
                seed: 42,
                emit_ds: true,
                emit_gp: true,
                truth_vcf: None,
                imputation_accept_mode: ImputationAcceptMode::MarkNonProduction,
                chunk_window_bp: None,
                chunk_overlap_bp: 0,
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
    fn impute_manifest_contains_chunk_plan_mode_and_chunk_manifests() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "x".repeat(64),
            contigs: vec![
                ContigSpec { name: "chr1".to_string(), length_bp: 1000 },
                ContigSpec { name: "chr2".to_string(), length_bp: 1000 },
            ],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let out = run_impute_stage(
            input,
            dir.path(),
            &species,
            &ImputeStageParams {
                species_id: species.species_id.clone(),
                build_id: species.build_id.clone(),
                backend: ImputeBackend::Beagle,
                panel_id: Some("hsapiens_grch38_mini".to_string()),
                map_id: None,
                threads: 2,
                seed: 42,
                emit_ds: true,
                emit_gp: true,
                truth_vcf: None,
                imputation_accept_mode: ImputationAcceptMode::MarkNonProduction,
                chunk_window_bp: Some(500),
                chunk_overlap_bp: 50,
            },
        )
        .unwrap_or_else(|err| panic!("run impute with chunk planning: {err}"));
        let manifest_raw = std::fs::read_to_string(&out.imputation_manifest_json)
            .unwrap_or_else(|err| panic!("read imputation manifest: {err}"));
        let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)
            .unwrap_or_else(|err| panic!("parse imputation manifest: {err}"));
        assert_eq!(
            manifest
                .get("chunk_plan")
                .and_then(|v| v.get("mode"))
                .and_then(|v| v.as_str())
                .unwrap_or_default(),
            "fixed_windows_overlap"
        );
        assert!(manifest
            .get("chunk_manifests")
            .and_then(|v| v.as_array())
            .is_some_and(|arr| !arr.is_empty()));
        assert!(manifest
            .get("chunk_logs")
            .and_then(|v| v.as_array())
            .is_some_and(|arr| !arr.is_empty()));
    }

    #[test]
    fn phased_path_then_minimac_impute_emits_output_on_mini_dataset() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("phased_input.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0|1\n1\t150\t.\tC\tT\t60\tPASS\t.\tGT\t0|0\n",
        )
        .unwrap_or_else(|err| panic!("write phased input: {err}"));
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "x".repeat(64),
            contigs: vec![ContigSpec {
                name: "1".to_string(),
                length_bp: 248956422,
            }],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let out = run_impute_stage(
            &input,
            dir.path(),
            &species,
            &ImputeStageParams {
                species_id: species.species_id.clone(),
                build_id: species.build_id.clone(),
                backend: ImputeBackend::Minimac4,
                panel_id: Some("hsapiens_grch38_mini".to_string()),
                map_id: Some("hsapiens_grch38_chr_map".to_string()),
                threads: 2,
                seed: 42,
                emit_ds: true,
                emit_gp: true,
                truth_vcf: None,
                imputation_accept_mode: ImputationAcceptMode::MarkNonProduction,
                chunk_window_bp: Some(1000),
                chunk_overlap_bp: 50,
            },
        )
        .unwrap_or_else(|err| panic!("run minimac phased path: {err}"));
        assert!(out.imputed_vcf.exists());
        assert!(out.imputed_tbi.exists());
        assert!(out.imputation_manifest_json.exists());
    }

    #[test]
    fn imputation_wrapper_runs_orchestration_and_emits_wrapper_manifest() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "x".repeat(64),
            contigs: vec![
                ContigSpec { name: "chr1".to_string(), length_bp: 1000 },
                ContigSpec { name: "chr2".to_string(), length_bp: 1000 },
            ],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let out = run_imputation_orchestration_stage(
            input,
            dir.path(),
            &species,
            &ImputeStageParams {
                species_id: species.species_id.clone(),
                build_id: species.build_id.clone(),
                backend: ImputeBackend::Beagle,
                panel_id: Some("hsapiens_grch38_mini".to_string()),
                map_id: None,
                threads: 2,
                seed: 42,
                emit_ds: true,
                emit_gp: true,
                truth_vcf: None,
                imputation_accept_mode: ImputationAcceptMode::MarkNonProduction,
                chunk_window_bp: None,
                chunk_overlap_bp: 0,
            },
        )
        .unwrap_or_else(|err| panic!("run imputation wrapper: {err}"));
        assert!(out.orchestration_manifest_json.exists());
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
                chunk_window_bp: None,
                chunk_overlap_bp: 0,
            },
        )
        .expect_err("triploid must be refused");
        assert!(err.to_string().contains("unsupported ploidy model"));
    }
