#[test]
fn postprocess_stage_merges_and_emits_contract_artifacts() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let species = SpeciesContext {
        species_id: "Homo sapiens".to_string(),
        build_id: "GRCh38".to_string(),
        contig_set_digest: "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
            .to_string(),
        contigs: vec![
            ContigSpec { name: "1".to_string(), length_bp: 248956422 },
            ContigSpec { name: "2".to_string(), length_bp: 242193529 },
            ContigSpec { name: "chr1".to_string(), length_bp: 248956422 },
            ContigSpec { name: "chr2".to_string(), length_bp: 242193529 },
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
            split_multiallelic: true,
            run_level_checksums_path: Some(dir.path().join("run_level_artifact_checksums.json")),
        },
    )
    .unwrap_or_else(|err| panic!("postprocess stage: {err}"));
    assert!(out.merged_vcf.exists());
    assert!(out.merged_tbi.exists());
    assert!(out.merged_bcf.is_some());
    assert!(out.artifact_checksums_json.exists());
    assert!(out.normalization_contract_json.exists());
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
        contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 248_956_422 }],
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
            split_multiallelic: true,
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
    assert!(manifest
        .get("normalization")
        .and_then(|v| v.get("split_multiallelic_enabled"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false));
    let normalization_raw = std::fs::read_to_string(&out.normalization_contract_json)
        .unwrap_or_else(|err| panic!("read normalization contract: {err}"));
    let normalization: serde_json::Value = serde_json::from_str(&normalization_raw)
        .unwrap_or_else(|err| panic!("parse normalization contract: {err}"));
    assert_eq!(
        normalization.get("schema_version").and_then(|v| v.as_str()),
        Some("bijux.vcf.normalization_contract.v1")
    );
}

#[test]
fn postprocess_splits_multiallelic_and_normalizes_variant_identity() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let input = dir.path().join("multiallelic.vcf");
    std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts2\ts1\n1\t100\t.\ta\tg,t\t60\t.\tMQ=50\tGT\t0/1\t1/2\n",
        )
        .unwrap_or_else(|err| panic!("write multiallelic fixture: {err}"));
    let species = SpeciesContext {
        species_id: "Homo sapiens".to_string(),
        build_id: "GRCh38".to_string(),
        contig_set_digest: "x".repeat(64),
        contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 248_956_422 }],
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
            remove_info_fields: vec![],
            compression_level: 6,
            compression_threads: 2,
            emit_bcf: false,
            normalize_indels: true,
            split_multiallelic: true,
            run_level_checksums_path: None,
        },
    )
    .unwrap_or_else(|err| panic!("postprocess multiallelic fixture: {err}"));
    let merged = bijux_dna_stages_vcf::vcf_io::read_vcf_text(&out.merged_vcf)
        .unwrap_or_else(|err| panic!("read merged VCF {}: {err}", out.merged_vcf.display()));
    let record_lines = merged.lines().filter(|line| !line.starts_with('#')).collect::<Vec<_>>();
    assert_eq!(record_lines.len(), 2, "multiallelic record must split into biallelic rows");
    assert!(record_lines.iter().all(|line| line.contains("\t1:100:A:")));
    let chrom = merged.lines().find(|line| line.starts_with("#CHROM\t")).unwrap_or_default();
    assert!(chrom.ends_with("\ts1\ts2"), "sample columns should be deterministically ordered");
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
    let manifest_raw = std::fs::read_to_string(&out.pca_manifest_json)
        .unwrap_or_else(|err| panic!("read pca manifest: {err}"));
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)
        .unwrap_or_else(|err| panic!("parse pca manifest: {err}"));
    assert_eq!(manifest.get("sample_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        manifest
            .get("sample_ids")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["sample1"])
    );
    assert!(
        manifest
            .get("eigenvalues")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|values| !values.is_empty()),
        "expected PCA manifest to keep explicit eigenvalues"
    );
}

#[test]
fn pca_stage_manifest_keeps_sample_population_labels_when_metadata_is_supplied() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let metadata = dir.path().join("population_labels.json");
    std::fs::write(&metadata, r#"{"samples":[{"sample":"sample1","population":"POP_A"}]}"#)
        .unwrap_or_else(|err| panic!("write metadata: {err}"));
    let out = run_pca_stage(
        Path::new("tests/fixtures/vcf/default/input.vcf"),
        dir.path(),
        &PcaStageParams {
            sample_metadata_manifest: Some(metadata.clone()),
            components: 2,
            ..PcaStageParams::default()
        },
    )
    .unwrap_or_else(|err| panic!("run pca stage: {err}"));
    let manifest_raw = std::fs::read_to_string(&out.pca_manifest_json)
        .unwrap_or_else(|err| panic!("read pca manifest: {err}"));
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)
        .unwrap_or_else(|err| panic!("parse pca manifest: {err}"));
    assert_eq!(
        manifest.get("sample_metadata_manifest").and_then(serde_json::Value::as_str),
        Some(metadata.to_string_lossy().as_ref())
    );
    let sample_population_labels = manifest
        .get("sample_population_labels")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("sample_population_labels missing"));
    assert_eq!(sample_population_labels.len(), 1);
    assert_eq!(
        sample_population_labels[0].get("sample_id").and_then(serde_json::Value::as_str),
        Some("sample1")
    );
    assert_eq!(
        sample_population_labels[0].get("population_id").and_then(serde_json::Value::as_str),
        Some("POP_A")
    );
}

#[test]
fn pca_stage_refuses_without_ld_pruning_policy() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let err = run_pca_stage(
        Path::new("tests/fixtures/vcf/default/input.vcf"),
        dir.path(),
        &PcaStageParams {
            preprocessing: PopulationPreprocessingParams {
                ld_pruning_policy: None,
                ..PopulationPreprocessingParams::default()
            },
            ..PcaStageParams::default()
        },
    )
    .expect_err("pca should refuse without explicit ld pruning policy");
    assert!(err.to_string().contains("LD pruning policy"));
}

#[test]
fn population_structure_stage_emits_structured_outputs() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let input = dir.path().join("population_structure_input.vcf");
    std::fs::write(
        &input,
        "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample1\tsample2\n1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\t1/1\n1\t200\t.\tC\tT\t60\tPASS\t.\tGT\t0/0\t0/1\n",
    )
    .unwrap_or_else(|err| panic!("write vcf input: {err}"));
    let metadata = dir.path().join("population_labels.json");
    std::fs::write(
            &metadata,
            r#"{"samples":[{"sample":"sample1","population":"POP_A"},{"sample":"sample2","population":"POP_B"}]}"#,
        )
        .unwrap_or_else(|err| panic!("write metadata: {err}"));
    let out = run_population_structure_stage(
        &input,
        dir.path(),
        &PopulationStructureStageParams {
            run_admixture: true,
            sample_metadata_manifest: Some(metadata),
            ..PopulationStructureStageParams::default()
        },
    )
    .unwrap_or_else(|err| panic!("run population_structure stage: {err}"));
    assert!(out.pruned_variants_tsv.exists());
    assert!(out.population_structure_json.exists());
    let report_raw = std::fs::read_to_string(&out.population_structure_json)
        .unwrap_or_else(|err| panic!("read population structure json: {err}"));
    let report: serde_json::Value = serde_json::from_str(&report_raw)
        .unwrap_or_else(|err| panic!("parse population structure json: {err}"));
    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.vcf.population_structure.v1")
    );
    assert_eq!(report.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert_eq!(report.get("variants_passing").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        report
            .get("sample_ids")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["sample1", "sample2"])
    );
    assert_eq!(
        report
            .pointer("/sample_labels/rows")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.len()),
        Some(2)
    );
    assert_eq!(report.pointer("/pca/sample_count").and_then(serde_json::Value::as_u64), Some(2));
    assert!(
        report.pointer("/pca/execution_mode").and_then(serde_json::Value::as_str).is_some(),
        "expected nested PCA execution evidence"
    );
    assert!(
        report
            .pointer("/pca/manifest_json")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|path| !path.trim().is_empty()),
        "expected consumed PCA manifest path"
    );
    assert_eq!(
        report.pointer("/admixture/sample_count").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        report.pointer("/admixture/selected_k").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        report.pointer("/admixture/status").and_then(serde_json::Value::as_str),
        Some("complete")
    );
    assert!(
        report
            .pointer("/admixture/k_selection_json")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|path| !path.trim().is_empty()),
        "expected consumed admixture manifest path"
    );
}

#[test]
fn population_structure_refuses_without_metadata_manifest() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let err = run_population_structure_stage(
        Path::new("tests/fixtures/vcf/default/input.vcf"),
        dir.path(),
        &PopulationStructureStageParams::default(),
    )
    .expect_err("population_structure should refuse without sample metadata manifest");
    assert!(err.to_string().contains("sample metadata manifest"));
}

#[test]
fn population_structure_refuses_without_consumed_admixture() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let metadata = dir.path().join("population_labels.json");
    std::fs::write(
        &metadata,
        r#"{"samples":[{"sample":"sample1","population":"POP_A"}]}"#,
    )
    .unwrap_or_else(|err| panic!("write metadata: {err}"));
    let err = run_population_structure_stage(
        Path::new("tests/fixtures/vcf/default/input.vcf"),
        dir.path(),
        &PopulationStructureStageParams {
            run_admixture: false,
            sample_metadata_manifest: Some(metadata),
            ..PopulationStructureStageParams::default()
        },
    )
    .expect_err("population_structure should refuse without consumed admixture output");
    assert!(err.to_string().contains("consumed admixture output"));
}

#[test]
fn admixture_stage_emits_q_matrix_and_selection_artifacts() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let input = dir.path().join("admixture_input.vcf");
    std::fs::write(
        &input,
        "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample1\tsample2\n1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\t1/1\n",
    )
    .unwrap_or_else(|err| panic!("write vcf input: {err}"));
    let metadata = dir.path().join("population_labels.json");
    std::fs::write(
            &metadata,
            r#"{"samples":[{"sample":"sample1","population":"POP_A"},{"sample":"sample2","population":"POP_B"}]}"#,
        )
        .unwrap_or_else(|err| panic!("write metadata: {err}"));
    let out = run_admixture_stage(
        &input,
        dir.path(),
        &AdmixtureStageParams {
            sample_metadata_manifest: Some(metadata),
            ..AdmixtureStageParams::default()
        },
    )
    .unwrap_or_else(|err| panic!("run admixture stage: {err}"));
    assert!(out.q_matrix_tsv.exists());
    assert!(out.k_selection_json.exists());
    assert!(out.logs_txt.exists());
    let q_matrix_raw = std::fs::read_to_string(&out.q_matrix_tsv)
        .unwrap_or_else(|err| panic!("read q matrix: {err}"));
    assert_eq!(q_matrix_raw.lines().next(), Some("sample\tcluster_1\tcluster_2"));
    for row in q_matrix_raw.lines().skip(1) {
        let total = row
            .split('\t')
            .skip(1)
            .map(|value| value.parse::<f64>().unwrap_or_else(|err| panic!("parse cluster value: {err}")))
            .sum::<f64>();
        assert!((total - 1.0).abs() <= 1e-6, "expected admixture row `{row}` to sum to 1.0");
    }
    let manifest_raw = std::fs::read_to_string(&out.k_selection_json)
        .unwrap_or_else(|err| panic!("read admixture manifest: {err}"));
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)
        .unwrap_or_else(|err| panic!("parse admixture manifest: {err}"));
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.vcf.admixture.v1")
    );
    assert_eq!(manifest.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert_eq!(manifest.get("insufficient_data_reason").and_then(serde_json::Value::as_str), None);
    assert_eq!(manifest.get("sample_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(manifest.get("population_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(manifest.get("cluster_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        manifest
            .get("sample_ids")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["sample1", "sample2"])
    );
    assert_eq!(
        manifest
            .get("cluster_headers")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["cluster_1", "cluster_2"])
    );
    assert_eq!(
        manifest
            .get("cluster_population_labels")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.len()),
        Some(2)
    );
}

#[test]
fn admixture_stage_reports_structured_insufficient_data_when_k_exceeds_population_labels() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let metadata = dir.path().join("population_labels.json");
    std::fs::write(
        &metadata,
        r#"{"samples":[{"sample":"sample1","population":"POP_A"},{"sample":"sample2","population":"POP_A"}]}"#,
    )
    .unwrap_or_else(|err| panic!("write metadata: {err}"));
    let out = run_admixture_stage(
        Path::new("tests/fixtures/vcf/default/input.vcf"),
        dir.path(),
        &AdmixtureStageParams {
            sample_metadata_manifest: Some(metadata),
            k_values: vec![2],
            ..AdmixtureStageParams::default()
        },
    )
    .unwrap_or_else(|err| panic!("run admixture stage: {err}"));
    let manifest_raw = std::fs::read_to_string(&out.k_selection_json)
        .unwrap_or_else(|err| panic!("read admixture manifest: {err}"));
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)
        .unwrap_or_else(|err| panic!("parse admixture manifest: {err}"));
    assert_eq!(
        manifest.get("status").and_then(serde_json::Value::as_str),
        Some("insufficient_data")
    );
    assert_eq!(
        manifest.get("insufficient_data_reason").and_then(serde_json::Value::as_str),
        Some("population_label_count_below_selected_k")
    );
    assert_eq!(manifest.get("population_count").and_then(serde_json::Value::as_u64), Some(1));
    let q_matrix_raw = std::fs::read_to_string(&out.q_matrix_tsv)
        .unwrap_or_else(|err| panic!("read q matrix: {err}"));
    let rows = q_matrix_raw.lines().collect::<Vec<_>>();
    assert_eq!(rows.first().copied(), Some("sample\tcluster_1\tcluster_2"));
    assert!(
        rows.iter().skip(1).all(|line| line.ends_with("\t1.000000\t0.000000")),
        "expected deterministic zero-fill for unavailable clusters, got {rows:?}"
    );
}

#[test]
fn roh_stage_emits_segments_and_metrics() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let out = run_roh_stage(
        Path::new("tests/fixtures/vcf/default/input.vcf"),
        dir.path(),
        &RohStageParams {
            min_snp_density_per_mb: 0.00001,
            max_missingness: 1.0,
            min_segment_kb: 0,
            max_gap_bp: 10_000_000,
            ..RohStageParams::default()
        },
    )
    .unwrap_or_else(|err| panic!("run roh stage: {err}"));
    assert!(out.roh_segments_tsv.exists());
    assert!(out.roh_per_sample_tsv.exists());
    assert!(out.roh_json.exists());
    assert!(out.metrics_json.exists());
    assert!(out.roh_summary_json.exists());
    assert!(out.roh_metrics_json.exists());
    assert!(dir.path().join("vcf_ready_for_downstream.json").exists());
    let roh_raw =
        std::fs::read_to_string(&out.roh_json).unwrap_or_else(|err| panic!("read roh json: {err}"));
    let roh_json: serde_json::Value =
        serde_json::from_str(&roh_raw).unwrap_or_else(|err| panic!("parse roh json: {err}"));
    assert!(roh_json.get("execution_mode").is_some());

    let segments_raw = std::fs::read_to_string(&out.roh_segments_tsv)
        .unwrap_or_else(|err| panic!("read roh segments: {err}"));
    let rows = segments_raw.lines().collect::<Vec<_>>();
    assert_eq!(rows.first().copied(), Some("sample\tcontig\tstart\tend\tlength_bp\tn_sites"));
    assert!(rows.len() > 1, "expected at least one ROH segment row");
    assert_eq!(
        roh_json.get("segment_count").and_then(serde_json::Value::as_u64),
        Some((rows.len() - 1) as u64)
    );
    assert!(
        roh_json
            .get("total_length_bp")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|value| value > 0)
    );

    let per_sample_raw = std::fs::read_to_string(&out.roh_per_sample_tsv)
        .unwrap_or_else(|err| panic!("read roh per-sample summary: {err}"));
    let per_sample_rows = per_sample_raw.lines().collect::<Vec<_>>();
    assert_eq!(
        per_sample_rows.first().copied(),
        Some("sample\tsegment_count\ttotal_length_bp\tmean_length_bp")
    );
    assert!(per_sample_rows.len() > 1, "expected at least one per-sample summary row");
}

#[test]
fn roh_stage_refuses_when_density_below_threshold() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let err = run_roh_stage(
        Path::new("tests/fixtures/vcf/default/input.vcf"),
        dir.path(),
        &RohStageParams {
            min_snp_density_per_mb: 1_000_000.0,
            max_missingness: 0.2,
            min_segment_kb: 500,
            max_gap_bp: 1_000_000,
            ..RohStageParams::default()
        },
    )
    .expect_err("roh should refuse under impossible density requirement");
    assert!(err.to_string().contains("density"));
}

#[test]
fn roh_stage_refuses_low_coverage_without_pseudohaploid_support() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let input = dir.path().join("lowcov.vcf");
    std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t./.\n1\t200\t.\tC\tT\t60\tPASS\t.\tGT\t./.\n1\t300\t.\tG\tA\t60\tPASS\t.\tGT\t0/1\n",
        )
        .unwrap_or_else(|err| panic!("write lowcov fixture: {err}"));
    let err = run_roh_stage(
        &input,
        dir.path(),
        &RohStageParams {
            min_snp_density_per_mb: 0.00001,
            max_missingness: 1.0,
            low_coverage_missingness_threshold: 0.20,
            allow_pseudohaploid_low_coverage: false,
            min_segment_kb: 0,
            max_gap_bp: 10_000_000,
            ..RohStageParams::default()
        },
    )
    .expect_err("low coverage without pseudo-haploid support must refuse");
    assert!(err.to_string().contains("low-coverage"));
}

#[test]
fn ibd_stage_emits_segments_filtered_and_metrics() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let input = dir.path().join("ibd_success.vcf");
    std::fs::write(
        &input,
        "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\ts2\nchr1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\t0/0\nchr1\t200\t.\tC\tT\t60\tPASS\t.\tGT\t0/1\t0/1\nchr1\t300\t.\tG\tA\t60\tPASS\t.\tGT\t0/0\t0/1\n",
    )
    .unwrap_or_else(|err| panic!("write ibd success fixture: {err}"));
    let out = run_ibd_stage(
        &input,
        dir.path(),
        &IbdStageParams {
            min_variant_density_per_mb: 0.00001,
            max_missingness: 1.0,
            min_samples: 1,
            min_segment_cm: 1.0,
            min_markers_per_segment: 1,
            ..IbdStageParams::default()
        },
    )
    .unwrap_or_else(|err| panic!("run ibd stage: {err}"));
    assert!(out.ibd_input_tsv.exists());
    assert!(out.ibd_segments_tsv.exists());
    assert!(out.ibd_merged_segments_tsv.exists());
    assert!(out.ibd_filtered_segments_tsv.exists());
    assert!(out.ibd_summary_json.exists());
    assert!(out.ibd_metrics_json.exists());
    assert!(dir.path().join("vcf_ready_for_downstream.json").exists());
    let ibd_summary_raw = std::fs::read_to_string(&out.ibd_summary_json)
        .unwrap_or_else(|err| panic!("read ibd summary: {err}"));
    let ibd_summary_json: serde_json::Value = serde_json::from_str(&ibd_summary_raw)
        .unwrap_or_else(|err| panic!("parse ibd summary: {err}"));
    assert!(ibd_summary_json.get("execution_mode").is_some());
    assert_eq!(ibd_summary_json.get("status").and_then(|value| value.as_str()), Some("complete"));
    assert!(ibd_summary_json.pointer("/tool_attempts/germline").is_some());
    assert!(ibd_summary_json.pointer("/tool_attempts/ibdseq").is_some());
    assert!(ibd_summary_json.pointer("/tool_attempts/ibdhap").is_some());

    let filtered_raw = std::fs::read_to_string(&out.ibd_filtered_segments_tsv)
        .unwrap_or_else(|err| panic!("read filtered IBD segments: {err}"));
    let filtered_rows = filtered_raw.lines().collect::<Vec<_>>();
    assert_eq!(
        filtered_rows.first().copied(),
        Some("sample_a\tsample_b\tcontig\tstart\tend\tlength_cm\tmarker_count")
    );
    assert!(filtered_rows.len() > 1, "expected at least one filtered IBD segment row");
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
            ..IbdStageParams::default()
        },
    )
    .expect_err("ibd must refuse when readiness constraints fail");
    assert!(err.to_string().contains("refusal"));
}

#[test]
fn ibd_stage_reports_insufficient_marker_overlap_without_abort() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let input = dir.path().join("sparse_overlap.vcf");
    std::fs::write(
        &input,
        "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\ts2\nchr1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\t0/0\nchr1\t200\t.\tC\tT\t60\tPASS\t.\tGT\t0/1\t./.\nchr1\t300\t.\tG\tA\t60\tPASS\t.\tGT\t./.\t0/1\n",
    )
    .unwrap_or_else(|err| panic!("write sparse overlap fixture: {err}"));

    let out = run_ibd_stage(
        &input,
        dir.path(),
        &IbdStageParams {
            min_variant_density_per_mb: 0.00001,
            max_missingness: 1.0,
            min_samples: 2,
            min_segment_cm: 1.0,
            min_markers_per_segment: 50,
            ..IbdStageParams::default()
        },
    )
    .unwrap_or_else(|err| panic!("run sparse ibd stage: {err}"));

    let ibd_summary_raw = std::fs::read_to_string(&out.ibd_summary_json)
        .unwrap_or_else(|err| panic!("read ibd summary: {err}"));
    let ibd_summary_json: serde_json::Value = serde_json::from_str(&ibd_summary_raw)
        .unwrap_or_else(|err| panic!("parse ibd summary: {err}"));
    assert_eq!(
        ibd_summary_json.get("status").and_then(|value| value.as_str()),
        Some("insufficient_marker_overlap")
    );
    assert_eq!(
        ibd_summary_json.get("insufficient_data_reason").and_then(|value| value.as_str()),
        Some("no_pairs_met_min_marker_or_length_threshold")
    );
    assert_eq!(ibd_summary_json.get("segments_filtered").and_then(|value| value.as_u64()), Some(0));

    let filtered = std::fs::read_to_string(&out.ibd_filtered_segments_tsv)
        .unwrap_or_else(|err| panic!("read filtered segments: {err}"));
    assert_eq!(
        filtered.lines().filter(|line| !line.starts_with('#')).count(),
        1,
        "expected header only when overlap is insufficient"
    );
}

#[test]
fn ibd_stage_refuses_on_genome_build_mismatch() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let input = dir.path().join("build_mismatch.vcf");
    std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n##reference=GRCh37\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\ts2\n1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\t0/0\n",
        )
        .unwrap_or_else(|err| panic!("write fixture: {err}"));
    let err = run_ibd_stage(
        &input,
        dir.path(),
        &IbdStageParams {
            expected_build: Some("GRCh38".to_string()),
            min_variant_density_per_mb: 0.00001,
            max_missingness: 1.0,
            min_samples: 2,
            min_segment_cm: 1.0,
            ..IbdStageParams::default()
        },
    )
    .expect_err("genome build mismatch must refuse");
    assert!(err.to_string().contains("genome build mismatch"));
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
        &DemographyStageParams { min_segments: 1, ..DemographyStageParams::default() },
    )
    .unwrap_or_else(|err| panic!("run demography: {err}"));
    assert!(out.ne_trajectory_tsv.exists());
    assert!(out.demography_json.exists());
    assert!(out.demography_metrics_json.exists());
    let demography_raw = std::fs::read_to_string(&out.demography_json)
        .unwrap_or_else(|err| panic!("read demography json: {err}"));
    let demography_json: serde_json::Value = serde_json::from_str(&demography_raw)
        .unwrap_or_else(|err| panic!("parse demography json: {err}"));
    assert_eq!(
        demography_json.get("schema_version").and_then(|v| v.as_str()).unwrap_or_default(),
        "bijux.vcf.demography.contract.v1"
    );
    assert!(demography_json.get("inference_status").is_some());
    assert_eq!(demography_json.get("method").and_then(|value| value.as_str()), Some("ibdne"));
    assert_eq!(demography_json.get("status").and_then(|value| value.as_str()), Some("complete"));
    assert_eq!(
        demography_json.get("insufficient_data_reason").and_then(|value| value.as_str()),
        None
    );
    assert!(demography_json
        .get("ne_estimates")
        .and_then(|value| value.as_array())
        .is_some_and(|rows| !rows.is_empty()));
}

#[test]
fn demography_stage_reports_structured_insufficient_data_without_abort() {
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
        &DemographyStageParams { min_segments: 2, ..DemographyStageParams::default() },
    )
    .unwrap_or_else(|err| panic!("run demography: {err}"));
    assert!(out.ne_trajectory_tsv.exists());
    assert!(out.demography_json.exists());
    assert!(out.demography_metrics_json.exists());
    let demography_raw = std::fs::read_to_string(&out.demography_json)
        .unwrap_or_else(|err| panic!("read demography json: {err}"));
    let demography_json: serde_json::Value = serde_json::from_str(&demography_raw)
        .unwrap_or_else(|err| panic!("parse demography json: {err}"));
    assert_eq!(
        demography_json.get("status").and_then(|value| value.as_str()),
        Some("insufficient_data")
    );
    assert_eq!(
        demography_json.get("insufficient_data_reason").and_then(|value| value.as_str()),
        Some("not_enough_ibd_segments")
    );
    assert_eq!(
        demography_json.get("time_bins").and_then(|value| value.as_array()).map(Vec::len),
        Some(0)
    );
    assert_eq!(
        demography_json.get("ne_estimates").and_then(|value| value.as_array()).map(Vec::len),
        Some(0)
    );
    let trajectory_tsv = std::fs::read_to_string(&out.ne_trajectory_tsv)
        .unwrap_or_else(|err| panic!("read trajectory tsv: {err}"));
    assert_eq!(trajectory_tsv, "generation\tne\tci_low\tci_high\n");
}

#[test]
fn imputed_vcf_to_roh_pca_ibd_integration_mini() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir: {err}"));
    let input = dir.path().join("impute_input.vcf");
    std::fs::write(
            &input,
            "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\ts2\n1\t100\t.\tA\tG\t60\tPASS\t.\tGT:GL\t0/1:-0.1,-1.0,-2.0\t0/0:-0.1,-1.0,-2.0\n1\t200\t.\tC\tT\t60\tPASS\t.\tGT:GL\t0/1:-0.1,-1.0,-2.0\t0/1:-0.1,-1.0,-2.0\n",
        )
        .unwrap_or_else(|err| panic!("write impute input: {err}"));
    let species = SpeciesContext {
        species_id: "Homo sapiens".to_string(),
        build_id: "GRCh38".to_string(),
        contig_set_digest: "x".repeat(64),
        contigs: vec![ContigSpec { name: "1".to_string(), length_bp: 248_956_422 }],
        sex_system: "xy".to_string(),
        par_policy: "grch38_par".to_string(),
        default_coverage_regime: None,
    };
    let imputed = run_impute_stage(
        &input,
        &dir.path().join("impute"),
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
    .unwrap_or_else(|err| panic!("run mini impute: {err}"));
    let roh = run_roh_stage(
        &imputed.imputed_vcf,
        &dir.path().join("roh"),
        &RohStageParams {
            min_snp_density_per_mb: 0.00001,
            max_missingness: 1.0,
            min_segment_kb: 0,
            max_gap_bp: 10_000_000,
            ..RohStageParams::default()
        },
    )
    .unwrap_or_else(|err| panic!("run mini roh: {err}"));
    let pca =
        run_pca_stage(&imputed.imputed_vcf, &dir.path().join("pca"), &PcaStageParams::default())
            .unwrap_or_else(|err| panic!("run mini pca: {err}"));
    let ibd = run_ibd_stage(
        &imputed.imputed_vcf,
        &dir.path().join("ibd"),
        &IbdStageParams {
            min_variant_density_per_mb: 0.00001,
            max_missingness: 1.0,
            min_samples: 2,
            min_segment_cm: 1.0,
            ..IbdStageParams::default()
        },
    )
    .unwrap_or_else(|err| panic!("run mini ibd: {err}"));
    assert!(roh.roh_metrics_json.exists());
    assert!(pca.eigenvec_tsv.exists());
    assert!(ibd.ibd_metrics_json.exists());
}
