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
        contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 248956422 }],
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
            backend: ImputeBackend::Impute5,
            panel_id: Some("hsapiens_grch38_mini".to_string()),
            map_id: None,
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
    let digest = manifest.get("tool_digest").and_then(serde_json::Value::as_str).unwrap_or("");
    assert!(digest.starts_with("sha256:"), "imputation manifest missing tool_digest sha256");
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
        contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 248956422 }],
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
            map_id: None,
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
        contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 248956422 }],
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
            backend: ImputeBackend::Impute5,
            panel_id: Some("hsapiens_grch38_mini".to_string()),
            map_id: None,
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
    assert_eq!(a_keys, b_keys, "qc schema keys must remain cross-run stable");
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
            ContigSpec { name: "1".to_string(), length_bp: 1000 },
            ContigSpec { name: "2".to_string(), length_bp: 1000 },
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
    assert!(manifest.get("chunk_regions_artifact").is_some());
}

#[test]
fn impute_backend_selector_prefers_glimpse_for_gl_regime_when_requested_beagle() {
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
        contig_set_digest: "x".repeat(64),
        contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 248_956_422 }],
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
    .unwrap_or_else(|err| panic!("run impute selector test: {err}"));
    let manifest_raw = std::fs::read_to_string(&out.imputation_manifest_json)
        .unwrap_or_else(|err| panic!("read imputation manifest: {err}"));
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)
        .unwrap_or_else(|err| panic!("parse imputation manifest: {err}"));
    assert_eq!(manifest.get("backend").and_then(|v| v.as_str()).unwrap_or_default(), "glimpse");
    assert!(manifest.get("glimpse_site_list").is_some());
}

#[test]
fn impute_backend_selector_prefers_minimac4_for_phased_panel_when_requested_beagle() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let input = dir.path().join("phased_gt_input.vcf");
    std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n##reference=GRCh38\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0|1\n",
        )
        .unwrap_or_else(|err| panic!("write input: {err}"));
    let species = SpeciesContext {
        species_id: "Homo sapiens".to_string(),
        build_id: "GRCh38".to_string(),
        contig_set_digest: "x".repeat(64),
        contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 248_956_422 }],
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
    .unwrap_or_else(|err| panic!("run minimac selector test: {err}"));
    let manifest_raw = std::fs::read_to_string(&out.imputation_manifest_json)
        .unwrap_or_else(|err| panic!("read imputation manifest: {err}"));
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)
        .unwrap_or_else(|err| panic!("parse imputation manifest: {err}"));
    assert_eq!(manifest.get("backend").and_then(|v| v.as_str()).unwrap_or_default(), "minimac4");
    assert!(manifest.get("minimac_reference_conversion_cache").is_some());
}

#[test]
fn impute_backend_selector_prefers_impute5_when_minimac_not_supported() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let input = dir.path().join("phased_gt_input.vcf");
    std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n##reference=GRCh38\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0|1\n",
        )
        .unwrap_or_else(|err| panic!("write input: {err}"));
    let species = SpeciesContext {
        species_id: "Homo sapiens".to_string(),
        build_id: "GRCh38".to_string(),
        contig_set_digest: "x".repeat(64),
        contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 248_956_422 }],
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
            backend: ImputeBackend::Beagle,
            panel_id: Some("hsapiens_grch38_full".to_string()),
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
    .unwrap_or_else(|err| panic!("run impute5 selector test: {err}"));
    let manifest_raw = std::fs::read_to_string(&out.imputation_manifest_json)
        .unwrap_or_else(|err| panic!("read imputation manifest: {err}"));
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)
        .unwrap_or_else(|err| panic!("parse imputation manifest: {err}"));
    assert_eq!(manifest.get("backend").and_then(|v| v.as_str()).unwrap_or_default(), "impute5");
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
            ContigSpec { name: "1".to_string(), length_bp: 1000 },
            ContigSpec { name: "2".to_string(), length_bp: 1000 },
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
        contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 248956422 }],
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

#[test]
fn minimac_path_emits_reference_conversion_cache_marker() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let input = dir.path().join("phased_input.vcf");
    std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0|1\n",
        )
        .unwrap_or_else(|err| panic!("write phased input: {err}"));
    let species = SpeciesContext {
        species_id: "Homo sapiens".to_string(),
        build_id: "GRCh38".to_string(),
        contig_set_digest: "x".repeat(64),
        contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 248_956_422 }],
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
            chunk_window_bp: None,
            chunk_overlap_bp: 0,
        },
    )
    .unwrap_or_else(|err| panic!("run minimac path: {err}"));
    let manifest_raw = std::fs::read_to_string(&out.imputation_manifest_json)
        .unwrap_or_else(|err| panic!("read imputation manifest: {err}"));
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)
        .unwrap_or_else(|err| panic!("parse imputation manifest: {err}"));
    let cache = manifest
        .get("minimac_reference_conversion_cache")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert!(!cache.is_empty(), "missing minimac reference conversion cache marker");
    assert!(std::path::Path::new(cache).exists(), "cache marker path does not exist");
}

#[test]
fn beagle_impute_fills_masked_truth_site_from_donor_support() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let input = dir.path().join("masked_input.vcf");
    let truth = dir.path().join("masked_truth.vcf");
    std::fs::write(
        &input,
        "##fileformat=VCFv4.2\n\
##contig=<ID=1,length=248956422>\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tmasked_sample\tdonor_sample\n\
1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t./.\t0/1\n\
1\t140\t.\tC\tT\t60\tPASS\t.\tGT\t0/0\t0/0\n",
    )
    .unwrap_or_else(|err| panic!("write masked input: {err}"));
    std::fs::write(
        &truth,
        "##fileformat=VCFv4.2\n\
##contig=<ID=1,length=248956422>\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tmasked_sample\tdonor_sample\n\
1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\t0/1\n\
1\t140\t.\tC\tT\t60\tPASS\t.\tGT\t0/0\t0/0\n",
    )
    .unwrap_or_else(|err| panic!("write masked truth: {err}"));
    let species = SpeciesContext {
        species_id: "Homo sapiens".to_string(),
        build_id: "GRCh38".to_string(),
        contig_set_digest: "x".repeat(64),
        contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 248_956_422 }],
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
            backend: ImputeBackend::Beagle,
            panel_id: Some("hsapiens_grch38_mini".to_string()),
            map_id: None,
            threads: 2,
            seed: 42,
            emit_ds: true,
            emit_gp: true,
            truth_vcf: Some(truth),
            imputation_accept_mode: ImputationAcceptMode::MarkNonProduction,
            chunk_window_bp: None,
            chunk_overlap_bp: 0,
        },
    )
    .unwrap_or_else(|err| panic!("run beagle impute: {err}"));

    let viewed = std::process::Command::new("bcftools")
        .args(["view", &out.imputed_vcf.display().to_string()])
        .output()
        .unwrap_or_else(|err| panic!("bcftools view imputed output: {err}"));
    assert!(
        viewed.status.success(),
        "bcftools view failed: {}",
        String::from_utf8_lossy(&viewed.stderr)
    );
    let imputed_vcf = String::from_utf8_lossy(&viewed.stdout);
    let first_record = imputed_vcf
        .lines()
        .find(|line| !line.starts_with('#'))
        .unwrap_or_else(|| panic!("missing imputed VCF record"));
    let fields = first_record.split('\t').collect::<Vec<_>>();
    assert_eq!(fields[8], "GT:DS:GP");
    assert_eq!(fields[9], "0/1:1:0.2,0.6,0.2");
    assert_eq!(fields[10], "0/1:1:0.05,0.9,0.05");

    let qc_raw = std::fs::read_to_string(&out.imputation_qc_json)
        .unwrap_or_else(|err| panic!("read imputation qc: {err}"));
    let qc: serde_json::Value =
        serde_json::from_str(&qc_raw).unwrap_or_else(|err| panic!("parse imputation qc: {err}"));
    assert_eq!(qc.get("missing_genotypes_before").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(qc.get("missing_genotypes_after").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(qc.get("imputed_genotypes").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(qc.get("low_confidence_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        qc.pointer("/concordance/masked_truth_site_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        qc.pointer("/concordance/imputed_match_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        qc.pointer("/concordance/unresolved_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
}

#[test]
fn beagle_impute_reports_not_imputable_reason_without_donor_support() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let input = dir.path().join("unresolved_input.vcf");
    let truth = dir.path().join("unresolved_truth.vcf");
    std::fs::write(
        &input,
        "##fileformat=VCFv4.2\n\
##contig=<ID=1,length=248956422>\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tmasked_sample\n\
1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t./.\n",
    )
    .unwrap_or_else(|err| panic!("write unresolved input: {err}"));
    std::fs::write(
        &truth,
        "##fileformat=VCFv4.2\n\
##contig=<ID=1,length=248956422>\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tmasked_sample\n\
1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\n",
    )
    .unwrap_or_else(|err| panic!("write unresolved truth: {err}"));
    let species = SpeciesContext {
        species_id: "Homo sapiens".to_string(),
        build_id: "GRCh38".to_string(),
        contig_set_digest: "x".repeat(64),
        contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 248_956_422 }],
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
            backend: ImputeBackend::Beagle,
            panel_id: Some("hsapiens_grch38_mini".to_string()),
            map_id: None,
            threads: 2,
            seed: 42,
            emit_ds: true,
            emit_gp: true,
            truth_vcf: Some(truth),
            imputation_accept_mode: ImputationAcceptMode::MarkNonProduction,
            chunk_window_bp: None,
            chunk_overlap_bp: 0,
        },
    )
    .unwrap_or_else(|err| panic!("run unresolved beagle impute: {err}"));

    let qc_raw = std::fs::read_to_string(&out.imputation_qc_json)
        .unwrap_or_else(|err| panic!("read unresolved imputation qc: {err}"));
    let qc: serde_json::Value =
        serde_json::from_str(&qc_raw).unwrap_or_else(|err| panic!("parse unresolved qc: {err}"));
    assert_eq!(qc.get("missing_genotypes_before").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(qc.get("missing_genotypes_after").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(qc.get("imputed_genotypes").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        qc.pointer("/not_imputable_reasons/insufficient_donor_support")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        qc.pointer("/concordance/unresolved_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        qc.pointer("/concordance/unresolved_reasons/insufficient_donor_support")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
}
