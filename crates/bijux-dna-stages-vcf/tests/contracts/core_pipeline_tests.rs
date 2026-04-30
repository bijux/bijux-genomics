    #[test]
    fn vcf_stats_parser_fixture_roundtrip() {
        let path = Path::new("tests/fixtures/vcf_stats/default/stats.txt");
        let metrics =
            parse_vcf_stats(path).unwrap_or_else(|err| panic!("parse stats fixture: {err}"));
        assert_eq!(metrics.schema_version, "bijux.vcf.stats.v1");
        assert_eq!(metrics.sample_name, "sample1");
        assert_eq!(metrics.variants_total, 12);
        assert_eq!(metrics.sample_count, 1);
        assert_eq!(metrics.snps, 9);
        assert_eq!(metrics.indels, 3);
        assert_eq!(metrics.ti_tv, Some(2.25));
        assert_eq!(metrics.missingness_post, Some(0.0));
        assert_eq!(metrics.heterozygosity_ratio, Some(1.0));
        assert_eq!(metrics.annotation_coverage, Some(1.0));
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
            damage_filter: None,
            gl_propagation: None,
            qc: None,
            phasing: None,
            impute: None,
            postprocess: None,
            invariants: InvariantConfig {
                allow_contig_aliasing: true,
                require_sex_metadata_for_sex_chr: false,
                ..InvariantConfig::default()
            },
        })
        .unwrap_or_else(|err| panic!("dispatch vcf pipeline: {err}"));
        assert!(out.report_path.exists());
        let report_raw = std::fs::read_to_string(&out.report_path)
            .unwrap_or_else(|err| panic!("read report.json: {err}"));
        let report_json: serde_json::Value = serde_json::from_str(&report_raw)
            .unwrap_or_else(|err| panic!("parse report.json: {err}"));
        let provenance_line = report_json
            .get("vcf_provenance_line")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        assert!(
            provenance_line.contains("tools=")
                && provenance_line.contains("digests=")
                && provenance_line.contains("panel_lock=")
                && provenance_line.contains("reference_lock="),
            "vcf_provenance_line must include tool digest and panel/reference lock fields"
        );
        let explain_path = out.artifact_root.join("explain.json");
        assert!(explain_path.exists(), "missing explain.json");
        let explain: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&explain_path)
                .unwrap_or_else(|err| panic!("read explain.json: {err}")),
        )
        .unwrap_or_else(|err| panic!("parse explain.json: {err}"));
        assert!(explain.get("chosen_regime").is_some(), "chosen_regime missing");
        assert!(explain.get("chosen_backend").is_some(), "chosen_backend missing");
        assert!(explain.get("panel_lock_id").is_some(), "panel_lock_id missing");
        assert!(explain.get("chunk_plan").is_some(), "chunk_plan missing");
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
            damage_filter: None,
            gl_propagation: None,
            qc: None,
            phasing: None,
            impute: None,
            postprocess: None,
            invariants: InvariantConfig {
                allow_contig_aliasing: true,
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
    fn vcf_call_family_enforces_input_contracts_and_outputs_manifests() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let params = bijux_dna_domain_vcf::params::VcfCallParams {
            sample_name: "sample1".to_string(),
            ..bijux_dna_domain_vcf::params::VcfCallParams::default()
        };

        let diploid_outputs = run_call_diploid_stage(input, &dir.path().join("diploid"), &params)
            .unwrap_or_else(|err| panic!("diploid call stage: {err}"));
        assert!(diploid_outputs.called_vcf.exists());
        assert!(diploid_outputs.called_tbi.exists());
        assert!(diploid_outputs.call_metrics_json.exists());
        assert!(diploid_outputs.call_manifest_json.exists());

        let dip_gatk = run_call_diploid_stage(
            input,
            &dir.path().join("diploid_gatk"),
            &bijux_dna_domain_vcf::params::VcfCallParams {
                caller: "gatk".to_string(),
                sample_name: "sample1".to_string(),
                ..bijux_dna_domain_vcf::params::VcfCallParams::default()
            },
        )
        .unwrap_or_else(|err| panic!("diploid call stage with gatk caller contract: {err}"));
        assert!(dip_gatk.called_vcf.exists());
        assert!(dip_gatk.called_tbi.exists());
        assert!(dip_gatk.call_manifest_json.exists());

        let pseudo = run_call_pseudohaploid_stage(input, &dir.path().join("pseudo"), &params)
            .unwrap_or_else(|err| panic!("pseudo call stage: {err}"));
        assert!(pseudo.called_vcf.exists());
        assert!(pseudo.call_metrics_tsv.exists());

        let gl_err = run_call_gl_stage(input, &dir.path().join("gl"), &params)
            .expect_err("gl stage must reject fixture without GL/GP/PL");
        assert!(gl_err.to_string().contains("GL/GP/PL"));
    }

    #[test]
    fn bam_to_gl_to_postprocess_integration_mini() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let bam = dir.path().join("mini.bam");
        let bam_index = dir.path().join("mini.bam.bai");
        std::fs::write(&bam, b"BAM_PLACEHOLDER\n").unwrap_or_else(|err| panic!("write bam: {err}"));
        std::fs::write(&bam_index, b"BAI_PLACEHOLDER\n")
            .unwrap_or_else(|err| panic!("write bai: {err}"));
        let err = run_call_gl_from_bam_stage(
            &bam,
            &dir.path().join("call_gl"),
            &bijux_dna_domain_vcf::params::VcfCallParams {
                caller: "angsd".to_string(),
                sample_name: "sample1".to_string(),
                ..bijux_dna_domain_vcf::params::VcfCallParams::default()
            },
        )
        .expect_err("bam->gl call must require reference_fasta in real backend flow");
        assert!(err.to_string().contains("reference_fasta"));
    }

    #[test]
    fn vcf_real_bgzip_tabix_path_works_on_toy_input() {
        if std::env::var("BIJUX_E2E").is_err() {
            return;
        }
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let normalized = dir.path().join("normalized.vcf");
        let output_vcfgz = dir.path().join("normalized.vcf.gz");
        std::fs::copy(input, &normalized).unwrap_or_else(|err| panic!("copy fixture: {err}"));
        let tbi = bijux_dna_stages_vcf::vcf_io::vcf_index_bgzip_tabix(&normalized, &output_vcfgz)
            .unwrap_or_else(|err| panic!("real bgzip/tabix path: {err}"));
        assert!(output_vcfgz.exists(), "missing bgzip output");
        assert!(tbi.exists(), "missing tabix index");
    }

    #[test]
    fn vcf_call_alias_dispatches_to_regime_specific_stage() {
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
            requested_stages: vec![VcfDomainStage::Call],
            production_profile: false,
            reference_fasta: None,
            prepare_panel: None,
            panel_vcf: None,
            damage_filter: None,
            gl_propagation: None,
            qc: None,
            phasing: None,
            impute: None,
            postprocess: None,
            invariants: InvariantConfig {
                allow_contig_aliasing: true,
                require_sex_metadata_for_sex_chr: false,
                ..InvariantConfig::default()
            },
        })
        .unwrap_or_else(|err| panic!("vcf call alias pipeline: {err}"));
        let call_stage = out
            .stages
            .iter()
            .find(|s| s.stage_id == "vcf.call")
            .unwrap_or_else(|| panic!("call stage missing"));
        let manifest = call_stage.artifact_dir.join("call_manifest.json");
        let payload = std::fs::read_to_string(&manifest)
            .unwrap_or_else(|err| panic!("read call_manifest: {err}"));
        let manifest_json: serde_json::Value = serde_json::from_str(&payload)
            .unwrap_or_else(|err| panic!("parse call_manifest json: {err}"));
        assert_eq!(
            manifest_json
                .get("stage_kind")
                .and_then(|v| v.as_str())
                .unwrap_or_default(),
            "diploid"
        );
    }

    #[test]
    fn damage_filter_refuses_unknown_regime_in_strict_mode() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let err = run_damage_filter_stage(
            input,
            dir.path(),
            &DamageFilterStageParams {
                udg_regime: DamageUdgRegime::Unknown,
                strict_regime: true,
                ..DamageFilterStageParams::recommended()
            },
        )
        .expect_err("strict mode must refuse unknown UDG regime");
        assert!(err
            .to_string()
            .contains("strict mode requires known UDG regime"));
    }

    #[test]
    fn damage_filter_emits_expected_artifacts() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let out = run_damage_filter_stage(
            input,
            dir.path(),
            &DamageFilterStageParams {
                udg_regime: DamageUdgRegime::NonUdg,
                strict_regime: true,
                min_qual: 1.0,
                max_damage_ratio: 1.0,
            },
        )
        .unwrap_or_else(|err| panic!("run damage filter stage: {err}"));
        assert!(out.filtered_vcf.exists());
        assert!(out.filtered_tbi.exists());
        assert!(out.damage_filter_summary_json.exists());
        assert!(out.damage_filter_counts_json.exists());
        assert!(out.damage_genotype_manifest_json.exists());
        let summary_raw = std::fs::read_to_string(&out.damage_filter_summary_json)
            .unwrap_or_else(|err| panic!("read damage_filter_summary.json: {err}"));
        let summary: serde_json::Value = serde_json::from_str(&summary_raw)
            .unwrap_or_else(|err| panic!("parse damage_filter_summary json: {err}"));
        assert!(summary.pointer("/prefilter/ct_ga_asymmetry").is_some());
        assert!(summary.pointer("/filtering/filtered_fraction_by_mutation_type").is_some());
        assert!(summary.pointer("/masking_strategy/mode").is_some());
        assert!(summary.pointer("/thresholds/terminal_damage_threshold").is_some());
        assert!(summary.pointer("/thresholds/pmd_min").is_some());
        let counts_raw = std::fs::read_to_string(&out.damage_filter_counts_json)
            .unwrap_or_else(|err| panic!("read damage_filter_counts.json: {err}"));
        let counts: serde_json::Value = serde_json::from_str(&counts_raw)
            .unwrap_or_else(|err| panic!("parse damage_filter_counts json: {err}"));
        assert!(counts.pointer("/counts/kept").is_some());
        let manifest_raw = std::fs::read_to_string(&out.damage_genotype_manifest_json)
            .unwrap_or_else(|err| panic!("read damage_genotype_manifest.json: {err}"));
        let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)
            .unwrap_or_else(|err| panic!("parse damage_genotype_manifest json: {err}"));
        assert_eq!(
            manifest.get("schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.vcf.damage_genotype_manifest.v1")
        );
        assert!(manifest.pointer("/required_fields_contract").is_some());
        assert!(manifest.pointer("/thresholds/terminal_damage_threshold").is_some());
    }

    #[test]
    fn vcf_pipeline_runs_damage_filter_stage() {
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
            requested_stages: vec![VcfDomainStage::DamageFilter],
            production_profile: false,
            reference_fasta: None,
            prepare_panel: None,
            panel_vcf: None,
            damage_filter: Some(DamageFilterStageParams {
                udg_regime: DamageUdgRegime::NonUdg,
                strict_regime: true,
                min_qual: 1.0,
                max_damage_ratio: 1.0,
            }),
            gl_propagation: None,
            qc: None,
            phasing: None,
            impute: None,
            postprocess: None,
            invariants: InvariantConfig {
                allow_contig_aliasing: true,
                require_sex_metadata_for_sex_chr: false,
                ..InvariantConfig::default()
            },
        })
        .unwrap_or_else(|err| panic!("run damage filter pipeline: {err}"));
        let stage = out
            .stages
            .iter()
            .find(|s| s.stage_id == "vcf.damage_filter")
            .unwrap_or_else(|| panic!("missing damage_filter stage"));
        assert!(stage.artifact_dir.join("damage_filter_summary.json").exists());
        assert!(stage.artifact_dir.join("damage_filter_counts.json").exists());
        assert!(stage.artifact_dir.join("warnings.json").exists());
        assert!(stage
            .artifact_dir
            .join("damage_genotype_manifest.json")
            .exists());
        let warnings_raw = std::fs::read_to_string(stage.artifact_dir.join("warnings.json"))
            .unwrap_or_else(|err| panic!("read warnings.json: {err}"));
        let warnings_json: serde_json::Value = serde_json::from_str(&warnings_raw)
            .unwrap_or_else(|err| panic!("parse warnings json: {err}"));
        assert!(warnings_json
            .to_string()
            .contains("filtered_and_why"));
    }

    #[test]
    fn adna_mode_refuses_pipeline_without_damage_filter_unless_override() {
        let old_mode = std::env::var("BIJUX_ADNA_MODE").ok();
        let old_override = std::env::var("BIJUX_ALLOW_SKIP_DAMAGE_FILTER").ok();
        std::env::set_var("BIJUX_ADNA_MODE", "1");
        std::env::remove_var("BIJUX_ALLOW_SKIP_DAMAGE_FILTER");

        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
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
        let err = run_vcf_pipeline(&VcfPipelineRequest {
            run_root: dir.path().to_path_buf(),
            input_vcf: input.to_path_buf(),
            species_context: species,
            sample_name: "sample1".to_string(),
            requested_stages: vec![VcfDomainStage::CallGl],
            production_profile: false,
            reference_fasta: None,
            prepare_panel: None,
            panel_vcf: None,
            damage_filter: None,
            gl_propagation: None,
            qc: None,
            phasing: None,
            impute: None,
            postprocess: None,
            invariants: InvariantConfig {
                allow_contig_aliasing: true,
                require_sex_metadata_for_sex_chr: false,
                ..InvariantConfig::default()
            },
        })
        .expect_err("aDNA mode should require damage_filter");
        assert!(err.to_string().contains("requires vcf.damage_filter"));

        if let Some(v) = old_mode {
            std::env::set_var("BIJUX_ADNA_MODE", v);
        } else {
            std::env::remove_var("BIJUX_ADNA_MODE");
        }
        if let Some(v) = old_override {
            std::env::set_var("BIJUX_ALLOW_SKIP_DAMAGE_FILTER", v);
        } else {
            std::env::remove_var("BIJUX_ALLOW_SKIP_DAMAGE_FILTER");
        }
    }

    #[test]
    fn gl_propagation_requires_gl_or_pl_when_configured() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let err = run_gl_propagation_stage(
            input,
            dir.path(),
            &GlPropagationStageParams {
                require_gl_or_pl: true,
                expected_ploidy: Some(2),
                emit_bcf: true,
            },
        )
        .expect_err("expected GL/PL requirement to fail on GT-only fixture");
        assert!(err.to_string().contains("requires GL/PL"));
    }

    #[test]
    fn gl_propagation_emits_normalized_outputs_and_report() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("gl_input.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n##reference=GRCh38\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\nchr1\t1\t.\tA\tG\t60\tPASS\t.\tGT:GL\t0/1:0.0,-1.0,-2.0\n",
        )
        .unwrap_or_else(|err| panic!("write gl fixture: {err}"));
        let out = run_gl_propagation_stage(
            &input,
            dir.path(),
            &GlPropagationStageParams::recommended(),
        )
        .unwrap_or_else(|err| panic!("run gl propagation: {err}"));
        assert!(out.normalized_vcf.exists());
        assert!(out.normalized_tbi.exists());
        assert!(out.normalized_bcf.as_ref().is_some_and(|p| p.exists()));
        assert!(out
            .normalized_bcf_csi
            .as_ref()
            .is_some_and(|p| p.exists()));
        assert!(out.gl_propagation_report_json.exists());
    }

    #[test]
    fn pseudohaploid_call_can_be_derived_from_gl_probabilities() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("gl_only.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\nchr1\t1\t.\tA\tG\t50\tPASS\tDP=8\tGP\t0.02,0.08,0.90\n",
        )
        .unwrap_or_else(|err| panic!("write fixture: {err}"));
        let out = run_call_pseudohaploid_stage(
            &input,
            dir.path(),
            &VcfCallParams {
                caller: "angsd".to_string(),
                sample_name: "s1".to_string(),
                ..VcfCallParams::default()
            },
        )
        .unwrap_or_else(|err| panic!("run pseudohaploid from GL fixture: {err}"));
        let raw = bijux_dna_stages_vcf::vcf_io::read_vcf_text(&out.called_vcf)
            .unwrap_or_else(|err| panic!("read called VCF: {err}"));
        assert!(raw.contains("\t1\n"), "expected haploid allele 1 from GP argmax");
    }

    #[test]
    fn vcf_pipeline_runs_gl_propagation_stage() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("gl_pipeline_input.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n##reference=GRCh38\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t1\t.\tA\tG\t60\tPASS\t.\tGT:GL\t0/1:0.0,-1.0,-2.0\n",
        )
        .unwrap_or_else(|err| panic!("write gl pipeline fixture: {err}"));
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
        let out = run_vcf_pipeline(&VcfPipelineRequest {
            run_root: dir.path().to_path_buf(),
            input_vcf: input,
            species_context: species,
            sample_name: "sample1".to_string(),
            requested_stages: vec![VcfDomainStage::GlPropagation],
            production_profile: false,
            reference_fasta: None,
            prepare_panel: None,
            panel_vcf: None,
            damage_filter: None,
            gl_propagation: Some(GlPropagationStageParams::recommended()),
            qc: None,
            phasing: None,
            impute: None,
            postprocess: None,
            invariants: InvariantConfig {
                require_sex_metadata_for_sex_chr: false,
                ..InvariantConfig::default()
            },
        })
        .unwrap_or_else(|err| panic!("run gl_propagation pipeline: {err}"));
        let stage = out
            .stages
            .iter()
            .find(|s| s.stage_id == "vcf.gl_propagation")
            .unwrap_or_else(|| panic!("missing gl_propagation stage"));
        assert!(stage.artifact_dir.join("gl_propagation_report.json").exists());
    }
