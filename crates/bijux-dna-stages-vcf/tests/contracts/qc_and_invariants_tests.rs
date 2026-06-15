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
        assert!(out.filter_explain_json.exists());
    }

    #[test]
    fn filter_stage_applies_mq_dp_and_strand_bias_tags() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("filter_thresholds.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\nchr1\t1\t.\tA\tG\t60\tPASS\tDP=5;MQ=20;FS=80\tGT\t0/1\n",
        )
        .unwrap_or_else(|err| panic!("write fixture: {err}"));
        let out = run_filter_stage_real(
            &input,
            dir.path(),
            &bijux_dna_domain_vcf::params::VcfFilterParams {
                require_pass: false,
                ..bijux_dna_domain_vcf::params::VcfFilterParams::default()
            },
        )
        .unwrap_or_else(|err| panic!("run filter stage: {err}"));
        let raw = std::fs::read_to_string(&out.filter_breakdown_json)
            .unwrap_or_else(|err| panic!("read filter_breakdown.json: {err}"));
        let json: serde_json::Value =
            serde_json::from_str(&raw).unwrap_or_else(|err| panic!("parse json: {err}"));
        assert!(json.to_string().contains("LOW_DP"));
        assert!(json.to_string().contains("LOW_MQ"));
        assert!(json.to_string().contains("STRAND_BIAS"));
        let explain_raw = std::fs::read_to_string(&out.filter_explain_json)
            .unwrap_or_else(|err| panic!("read filter_explain.json: {err}"));
        let explain: serde_json::Value = serde_json::from_str(&explain_raw)
            .unwrap_or_else(|err| panic!("parse filter_explain json: {err}"));
        assert_eq!(explain["schema_version"], serde_json::json!("bijux.vcf.filter_explain.v1"));
        assert_eq!(explain["filter_scope"]["output_subset"], serde_json::json!("retain_tagged_records"));
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
        assert!(out.imputation_qc_tsv.exists());
        assert!(out.warnings_json.exists());
        assert!(out.qc_histograms_json.exists());
        let summary_raw = std::fs::read_to_string(&out.qc_summary_json)
            .unwrap_or_else(|err| panic!("read qc_summary.json: {err}"));
        let summary: serde_json::Value =
            serde_json::from_str(&summary_raw).unwrap_or_else(|err| panic!("parse qc summary: {err}"));
        assert!(summary.get("ti_tv").is_some());
        assert!(summary.get("het_hom_ratio").is_some());
    }

    #[test]
    fn qc_stage_allows_hwe_when_explicitly_enabled_for_ancient() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = Path::new("tests/fixtures/vcf/default/input.vcf");
        let out = run_qc_stage(
            input,
            dir.path(),
            &QcStageParams {
                sample_name: "sample1".to_string(),
                is_ancient_dna: true,
                allow_hwe_for_ancient: true,
                production_profile: false,
                pre_filter_vcf: None,
            },
        )
        .unwrap_or_else(|err| panic!("run qc stage with explicit ancient HWE enablement: {err}"));
        assert!(out.qc_summary_json.exists());
        let summary_raw = std::fs::read_to_string(&out.qc_summary_json)
            .unwrap_or_else(|err| panic!("read qc_summary.json: {err}"));
        let summary: serde_json::Value = serde_json::from_str(&summary_raw)
            .unwrap_or_else(|err| panic!("parse qc_summary json: {err}"));
        assert_eq!(
            summary.get("hwe_status").and_then(serde_json::Value::as_str),
            Some("computed_modern")
        );
    }

    #[test]
    fn qc_stage_persists_named_missingness_and_exclusion_rows() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("qc_missingness_input.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n\
##contig=<ID=chr1,length=1000>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Read depth\">\n\
##INFO=<ID=INFO,Number=1,Type=Float,Description=\"Imputation info\">\n\
##INFO=<ID=R2,Number=1,Type=Float,Description=\"Imputation R2\">\n\
##INFO=<ID=AF,Number=1,Type=Float,Description=\"Allele frequency\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tqc_ref\tqc_sparse\tqc_balanced\n\
chr1\t10\t.\tA\tG\t60\tPASS\tDP=20;INFO=0.95;R2=0.90;AF=0.10\tGT\t0/1\t./.\t0/0\n\
chr1\t20\t.\tC\tT\t62\tPASS\tDP=18;INFO=0.90;R2=0.88;AF=0.25\tGT\t1/1\t./.\t0/1\n\
chr1\t30\t.\tG\tA\t59\tLOWQUAL\tDP=16;INFO=0.82;R2=0.80;AF=0.05\tGT\t0/0\t./.\t./.\n\
chr1\t40\t.\tT\tC\t65\tPASS\tDP=22;INFO=0.93;R2=0.91;AF=0.40\tGT\t0/1\t1/1\t0/1\n",
        )
        .unwrap_or_else(|err| panic!("write QC fixture: {err}"));
        let out = run_qc_stage(
            &input,
            dir.path(),
            &QcStageParams {
                sample_name: "qc_cohort".to_string(),
                is_ancient_dna: false,
                allow_hwe_for_ancient: false,
                production_profile: false,
                pre_filter_vcf: None,
            },
        )
        .unwrap_or_else(|err| panic!("run qc stage for exclusion evidence: {err}"));

        let summary_raw = std::fs::read_to_string(&out.qc_summary_json)
            .unwrap_or_else(|err| panic!("read qc_summary.json: {err}"));
        let summary: serde_json::Value = serde_json::from_str(&summary_raw)
            .unwrap_or_else(|err| panic!("parse qc_summary json: {err}"));

        let sample_rows = summary
            .get("sample_missingness")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("sample_missingness rows missing"));
        assert!(sample_rows.iter().any(|row| {
            row.get("sample_id").and_then(serde_json::Value::as_str) == Some("qc_sparse")
                && row.get("missingness").and_then(serde_json::Value::as_f64) == Some(0.75)
        }));
        assert!(sample_rows.iter().any(|row| {
            row.get("sample_id").and_then(serde_json::Value::as_str) == Some("qc_balanced")
                && row.get("missingness").and_then(serde_json::Value::as_f64) == Some(0.25)
        }));

        let variant_rows = summary
            .get("variant_missingness")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("variant_missingness rows missing"));
        assert!(variant_rows.iter().any(|row| {
            row.get("variant_id").and_then(serde_json::Value::as_str)
                == Some("chr1:30:G:A")
                && row.get("missingness").and_then(serde_json::Value::as_f64)
                    == Some(2.0 / 3.0)
        }));

        let excluded_samples = summary
            .get("excluded_samples")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("excluded_samples missing"));
        assert_eq!(excluded_samples.len(), 1);
        assert_eq!(
            excluded_samples[0].get("sample_id").and_then(serde_json::Value::as_str),
            Some("qc_sparse")
        );
        assert_eq!(
            summary
                .get("sample_missingness_exclusion_threshold")
                .and_then(serde_json::Value::as_f64),
            Some(0.5)
        );

        let excluded_variants = summary
            .get("excluded_variants")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("excluded_variants missing"));
        assert_eq!(excluded_variants.len(), 1);
        assert_eq!(
            excluded_variants[0].get("variant_id").and_then(serde_json::Value::as_str),
            Some("chr1:30:G:A")
        );
        assert_eq!(
            summary
                .get("variant_missingness_exclusion_threshold")
                .and_then(serde_json::Value::as_f64),
            Some(0.5)
        );
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
        assert_eq!(out.metrics.sample_count, 1);
        assert_eq!(out.metrics.missingness_post, Some(0.0));
        assert_eq!(out.metrics.annotation_coverage, Some(1.0));
    }

    #[test]
    fn stats_stage_enriches_ti_tv_from_plain_vcf() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("stats_input.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n\
##contig=<ID=chr1,length=24>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Read depth\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\ts2\n\
chr1\t3\t.\tA\tG\t60\tPASS\tDP=12\tGT\t0/1\t0/0\n\
chr1\t5\t.\tC\tT\t62\tPASS\tDP=14\tGT\t1/1\t0/1\n\
chr1\t7\t.\tA\tT\t64\tPASS\tDP=16\tGT\t0/1\t1/1\n\
chr1\t9\t.\tAT\tA\t58\tPASS\tDP=18\tGT\t0/1\t0/0\n",
        )
        .unwrap_or_else(|err| panic!("write input fixture: {err}"));
        let out = run_stats_stage_real(
            &input,
            dir.path(),
            &bijux_dna_domain_vcf::params::VcfStatsParams {
                sample_name: "cohort_stats".to_string(),
                ..bijux_dna_domain_vcf::params::VcfStatsParams::default()
            },
        )
        .unwrap_or_else(|err| panic!("run stats stage for ti/tv fallback: {err}"));
        assert_eq!(out.metrics.sample_count, 2);
        assert_eq!(out.metrics.variants_total, 4);
        assert_eq!(out.metrics.snps, 3);
        assert_eq!(out.metrics.indels, 1);
        assert_eq!(out.metrics.ti_tv, Some(2.0));
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
        assert!(stage.artifact_dir.join("imputation_qc.tsv").exists());
        assert!(stage.artifact_dir.join("warnings.json").exists());
        let chunk_logs = stage
            .artifact_dir
            .join("logs")
            .join("vcf.qc")
            .join("chunk-000");
        assert!(chunk_logs.join("stdout.log").exists());
        assert!(chunk_logs.join("stderr.log").exists());
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
        assert!(out.index_path.ends_with("vcf.gz.tbi"));
        assert!(out.invariants_json.ends_with("vcf_invariants.json"));
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
            "##fileformat=VCFv4.2\n##reference=GRCh38\n##contig=<ID=chr1,length=1000000>\n##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\nchr1\t1\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\n",
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
    fn vcf_preflight_refuses_missing_contig_and_format_definitions() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("missing_definitions.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t1\t.\tA\tG\t60\tPASS\tDP=10\tGT\t0/1\n",
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
            .expect_err("missing contig and FORMAT declarations must refuse");
        assert!(
            err.to_string().contains("missing ##contig headers")
                || err.to_string().contains("FORMAT field GT is used without header declaration"),
            "unexpected refusal: {err}"
        );
    }

    #[test]
    fn vcf_preflight_refuses_unsorted_records_instead_of_reordering_them() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("unsorted.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n##contig=<ID=1,length=1000000>\n##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">\n##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t2\t.\tA\tG\t60\tPASS\tDP=10\tGT\t0/1\n1\t1\t.\tC\tT\t60\tPASS\tDP=12\tGT\t0/1\n",
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
        let err = run_vcf_preflight(
            &input,
            &dir.path().join("out"),
            &species,
            &InvariantConfig {
                require_sex_metadata_for_sex_chr: false,
                ..InvariantConfig::default()
            },
        )
        .expect_err("unsorted records must refuse");
        assert!(err.to_string().contains("not sorted"), "unexpected refusal: {err}");
    }

    #[test]
    fn vcf_preflight_preserves_sample_column_order() {
        let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
        let input = dir.path().join("sample_order.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n##contig=<ID=1,length=1000000>\n##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">\n##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts2\ts1\n1\t1\t.\tA\tG\t60\tPASS\tDP=10\tGT\t0/1\t1/1\n",
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
        let out = run_vcf_preflight(
            &input,
            &dir.path().join("out"),
            &species,
            &InvariantConfig {
                require_sex_metadata_for_sex_chr: false,
                ..InvariantConfig::default()
            },
        )
        .unwrap_or_else(|err| panic!("run_vcf_preflight with sample order fixture: {err}"));
        let normalized = bijux_dna_stages_vcf::vcf_io::read_vcf_text(&out.normalized_input)
            .unwrap_or_else(|err| panic!("read normalized vcf: {err}"));
        let chrom = normalized
            .lines()
            .find(|line| line.starts_with("#CHROM\t"))
            .unwrap_or_default();
        assert!(chrom.ends_with("\ts2\ts1"), "sample order must be preserved");
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
