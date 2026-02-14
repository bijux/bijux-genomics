    #[test]
    fn prepare_reference_panel_stage_writes_manifest_and_overlap_artifacts() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let panel_root = dir.path().join("panel_store/hsapiens_grch38_mini").join("abc123");
        let panel_raw = panel_root.join("raw");
        let panel_normalized = panel_root.join("normalized");
        let panel_derived = panel_root.join("derived");
        std::fs::create_dir_all(&panel_raw).unwrap_or_else(|err| panic!("mkdir raw: {err}"));
        std::fs::create_dir_all(&panel_normalized)
            .unwrap_or_else(|err| panic!("mkdir normalized: {err}"));
        std::fs::create_dir_all(&panel_derived).unwrap_or_else(|err| panic!("mkdir derived: {err}"));
        let panel = panel_raw.join("panel.vcf.gz");
        std::fs::copy("tests/fixtures/vcf/default/input.vcf", &panel)
            .unwrap_or_else(|err| panic!("copy panel fixture: {err}"));
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
            &panel,
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
        assert!(outputs.panel_overlap_json.exists());
        assert!(outputs.panel_files_json.exists());
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
    fn phasing_auto_backend_selects_shapeit5_when_map_present() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "x".repeat(64),
            contigs: vec![
                ContigSpec {
                    name: "chr1".to_string(),
                    length_bp: 248_956_422,
                },
                ContigSpec {
                    name: "chr2".to_string(),
                    length_bp: 242_193_529,
                },
            ],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let out = run_phasing_stage(
            input,
            dir.path(),
            &species,
            &PhasingStageParams {
                species_id: species.species_id.clone(),
                build_id: species.build_id.clone(),
                backend: PhasingBackend::Auto,
                map_id: Some("hsapiens_grch38_chr_map".to_string()),
                threads: 2,
                seed: 42,
                region: None,
                allow_gl_only_input: false,
            },
        )
        .unwrap_or_else(|err| panic!("phasing auto with map: {err}"));
        let manifest = std::fs::read_to_string(&out.phasing_manifest_json)
            .unwrap_or_else(|err| panic!("read phasing manifest: {err}"));
        let payload: serde_json::Value = serde_json::from_str(&manifest)
            .unwrap_or_else(|err| panic!("parse phasing manifest json: {err}"));
        assert_eq!(
            payload
                .get("requested_backend")
                .and_then(|v| v.as_str())
                .unwrap_or_default(),
            "auto"
        );
        assert_eq!(
            payload
                .get("backend")
                .and_then(|v| v.as_str())
                .unwrap_or_default(),
            "shapeit5"
        );
    }

    #[test]
    fn phasing_auto_backend_selects_beagle_for_gl_regime() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("gl_only.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n##reference=GRCh38\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\nchr1\t100\t.\tA\tG\t60\tPASS\t.\tGL\t0.0,-1.0,-2.0\n",
        )
        .unwrap_or_else(|err| panic!("write gl fixture: {err}"));
        let species = SpeciesContext {
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            contig_set_digest: "x".repeat(64),
            contigs: vec![ContigSpec {
                name: "chr1".to_string(),
                length_bp: 248_956_422,
            }],
            sex_system: "xy".to_string(),
            par_policy: "grch38_par".to_string(),
            default_coverage_regime: None,
        };
        let out = run_phasing_stage(
            &input,
            dir.path(),
            &species,
            &PhasingStageParams {
                species_id: species.species_id.clone(),
                build_id: species.build_id.clone(),
                backend: PhasingBackend::Auto,
                map_id: None,
                threads: 2,
                seed: 42,
                region: None,
                allow_gl_only_input: true,
            },
        )
        .unwrap_or_else(|err| panic!("phasing auto for gl: {err}"));
        let manifest = std::fs::read_to_string(&out.phasing_manifest_json)
            .unwrap_or_else(|err| panic!("read phasing manifest: {err}"));
        let payload: serde_json::Value = serde_json::from_str(&manifest)
            .unwrap_or_else(|err| panic!("parse phasing manifest json: {err}"));
        assert_eq!(
            payload
                .get("requested_backend")
                .and_then(|v| v.as_str())
                .unwrap_or_default(),
            "auto"
        );
        assert_eq!(
            payload
                .get("backend")
                .and_then(|v| v.as_str())
                .unwrap_or_default(),
            "beagle"
        );
    }

