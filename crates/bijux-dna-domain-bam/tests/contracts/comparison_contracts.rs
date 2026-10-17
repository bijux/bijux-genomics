use std::collections::BTreeSet;

use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_bam::{
    comparable_benchmark_stage_contracts, comparable_benchmark_stage_ids,
    comparable_tool_ids_for_stage, stage_comparable_metric_contracts_for_stage,
    stage_comparable_metric_fields_for_stage, BamScientificInsufficiencyPolicy,
    BamScientificPassDirection, BamScientificToleranceKind,
};

fn assert_f64_eq(left: f64, right: f64) {
    assert!((left - right).abs() < f64::EPSILON, "left=`{left}`, right=`{right}`");
}

#[test]
fn multi_tool_bam_comparable_stage_slice_stays_explicit() {
    let stage_ids = comparable_benchmark_stage_ids()
        .into_iter()
        .map(|stage_id| stage_id.to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        stage_ids,
        BTreeSet::from([
            "bam.align".to_string(),
            "bam.authenticity".to_string(),
            "bam.contamination".to_string(),
            "bam.coverage".to_string(),
            "bam.damage".to_string(),
            "bam.duplication_metrics".to_string(),
            "bam.filter".to_string(),
            "bam.kinship".to_string(),
            "bam.length_filter".to_string(),
            "bam.mapping_summary".to_string(),
            "bam.mapq_filter".to_string(),
            "bam.markdup".to_string(),
            "bam.qc_pre".to_string(),
            "bam.sex".to_string(),
            "bam.validate".to_string(),
        ]),
        "the governed BAM multi-tool comparable slice must stay explicit"
    );
}

#[test]
fn multi_tool_bam_comparable_stages_publish_shared_metrics() {
    let contracts = comparable_benchmark_stage_contracts();
    assert_eq!(contracts.len(), 15);
    for contract in &contracts {
        assert!(
            contract.compatible_tool_ids.len() >= 2,
            "multi-tool comparable BAM stage `{}` must admit at least two tools",
            contract.stage_id
        );
        assert!(
            !contract.shared_metrics.is_empty(),
            "multi-tool comparable BAM stage `{}` must publish governed shared metrics",
            contract.stage_id
        );
        assert!(
            contract.shared_metrics.iter().all(|metric| metric.scientific_threshold.is_some()),
            "multi-tool comparable BAM stage `{}` must publish scientific threshold semantics for every shared metric",
            contract.stage_id
        );
    }
}

#[test]
fn bam_comparable_contracts_retain_shared_metric_fields_for_real_comparison_surfaces() {
    assert_eq!(
        comparable_tool_ids_for_stage(&StageId::from_static("bam.coverage")),
        vec![
            ToolId::from_static("bedtools"),
            ToolId::from_static("mosdepth"),
            ToolId::from_static("samtools"),
        ]
    );
    assert_eq!(
        stage_comparable_metric_fields_for_stage(&StageId::from_static("bam.validate")),
        vec![
            "validation_status".to_string(),
            "validation_errors".to_string(),
            "validation_warnings".to_string(),
        ]
    );
    assert_eq!(
        stage_comparable_metric_fields_for_stage(&StageId::from_static("bam.mapping_summary")),
        vec![
            "mapping_fraction".to_string(),
            "mapped_reads".to_string(),
            "unmapped_reads".to_string(),
            "secondary_reads".to_string(),
            "supplementary_reads".to_string(),
        ]
    );
    assert_eq!(
        stage_comparable_metric_fields_for_stage(&StageId::from_static("bam.coverage")),
        vec!["mean_depth".to_string(), "breadth_1x".to_string(), "covered_bases".to_string(),]
    );
    assert_eq!(
        stage_comparable_metric_fields_for_stage(&StageId::from_static("bam.authenticity")),
        vec![
            "score".to_string(),
            "confidence".to_string(),
            "status".to_string(),
            "pmd_like_signal_present".to_string(),
        ]
    );
    assert_eq!(
        stage_comparable_metric_fields_for_stage(&StageId::from_static("bam.contamination")),
        vec!["estimate".to_string(), "ci_low".to_string(), "ci_high".to_string(),]
    );
    assert_eq!(
        stage_comparable_metric_fields_for_stage(&StageId::from_static("bam.sex")),
        vec![
            "x_coverage".to_string(),
            "y_coverage".to_string(),
            "autosomal_coverage".to_string(),
            "call".to_string(),
            "confidence".to_string(),
            "status".to_string(),
        ]
    );
    assert_eq!(
        stage_comparable_metric_fields_for_stage(&StageId::from_static("bam.kinship")),
        vec![
            "observed_max_overlap_snps".to_string(),
            "pair_count".to_string(),
            "status".to_string(),
            "pairwise_results".to_string(),
        ]
    );
}

#[test]
fn bam_comparable_metrics_carry_governed_scientific_threshold_contracts() {
    let damage_metrics =
        stage_comparable_metric_contracts_for_stage(&StageId::from_static("bam.damage"));
    assert_eq!(damage_metrics.len(), 3);

    let Some(damage_signal) = damage_metrics.iter().find(|metric| metric.name == "damage_signal")
    else {
        panic!("damage signal metric");
    };
    let Some(threshold) = damage_signal.scientific_threshold.as_ref() else {
        panic!("damage threshold");
    };
    assert_eq!(threshold.pass_direction, BamScientificPassDirection::ExactMatch);
    assert_eq!(threshold.tolerance_kind, BamScientificToleranceKind::ExactMatch);
    assert_f64_eq(threshold.tolerance_value, 0.0);
    assert_eq!(
        threshold.insufficiency_policy,
        BamScientificInsufficiencyPolicy::WarnAndExcludeStage
    );

    let Some(validation_errors) =
        stage_comparable_metric_contracts_for_stage(&StageId::from_static("bam.validate"))
            .into_iter()
            .find(|metric| metric.name == "validation_errors")
    else {
        panic!("validation error metric");
    };
    let Some(threshold) = validation_errors.scientific_threshold.as_ref() else {
        panic!("validation threshold");
    };
    assert_eq!(threshold.pass_direction, BamScientificPassDirection::StructuredMatch);
    assert_eq!(threshold.tolerance_kind, BamScientificToleranceKind::NormalizedSetOverlap);
    assert_f64_eq(threshold.tolerance_value, 1.0);
    assert_eq!(
        threshold.insufficiency_policy,
        BamScientificInsufficiencyPolicy::RefuseStageComparison
    );
}

fn write_observation_admission_sam(
    root: &std::path::Path,
    name: &str,
    records: &[&str],
) -> anyhow::Result<std::path::PathBuf> {
    let path = root.join(name);
    let mut content = "@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:100\n@RG\tID:a\tSM:sample_a\n@RG\tID:b\tSM:sample_b\n".to_string();
    for record in records {
        content.push_str(record);
        content.push('\n');
    }
    std::fs::write(&path, content)?;
    Ok(path)
}

#[test]
fn acquired_overlap_outputs_refuse_incomplete_or_foreign_records() -> anyhow::Result<()> {
    use bijux_dna_domain_bam::{
        correct_tiny_bam_overlaps, summarize_tiny_bam_overlap_correction_outputs,
    };
    let root = bijux_dna_infra::temp_dir("overlap-record-admission")?;
    let left = "pair\t99\tchr1\t10\t60\t12M\t=\t15\t17\tACGTACGTACGT\tFFFFFFFFFFFF\tRG:Z:a";
    let right = "pair\t147\tchr1\t15\t60\t12M\t=\t10\t-17\tTTTTCCCCAAAA\tFFFFFFFFFFFF\tRG:Z:a";
    let input = write_observation_admission_sam(root.path(), "input.sam", &[left, right])?;
    let output = root.path().join("corrected.sam");
    let positive = correct_tiny_bam_overlaps(&input, &output, "bamutil")?;
    assert_eq!(positive.pair_count, Some(1));
    assert_eq!(positive.corrected_overlap_bases, Some(7));
    let observed = summarize_tiny_bam_overlap_correction_outputs(&input, &output, "bamutil")?;
    assert_eq!(observed.corrected_overlap_bases, Some(7));
    let before = std::fs::read(&input)?;
    let missing = write_observation_admission_sam(root.path(), "missing.sam", &[left])?;
    let foreign_left = left.replacen("pair", "foreign", 1);
    let foreign_right = right.replacen("pair", "foreign", 1);
    let foreign = write_observation_admission_sam(
        root.path(),
        "foreign.sam",
        &[&foreign_left, &foreign_right],
    )?;
    let mut refusals = Vec::new();
    for path in [&missing, &foreign] {
        let result = std::panic::catch_unwind(|| {
            summarize_tiny_bam_overlap_correction_outputs(&input, path, "bamutil")
        });
        eprintln!("overlap_output_admission {}: {result:?}", path.display());
        refusals.push(matches!(result, Ok(Err(_))));
    }
    assert_eq!(std::fs::read(input)?, before);
    assert!(refusals.into_iter().all(|refused| refused),
        "incomplete or foreign output membership must return an error without panic or invented pair truth");
    Ok(())
}

#[test]
fn duplicate_action_refuses_before_replacing_output_for_every_input_shape() -> anyhow::Result<()> {
    use bijux_dna_domain_bam::apply_duplicate_policy_tiny_bam;
    let root = bijux_dna_infra::temp_dir("duplicate-action-admission")?;
    let first = "first\t0\tchr1\t1\t60\t4M\t*\t0\t0\tACGT\tFFFF\tRG:Z:a";
    let duplicate = "second\t0\tchr1\t1\t60\t4M\t*\t0\t0\tACGT\tFFFF\tRG:Z:a";
    let unmapped = "unmapped\t4\t*\t0\t0\t*\t*\t0\t0\tACGT\tFFFF\tRG:Z:a";
    let unique = write_observation_admission_sam(root.path(), "unique.sam", &[first])?;
    let output = root.path().join("output.sam");
    for action in ["mark", "remove"] {
        apply_duplicate_policy_tiny_bam(&unique, &output, action, None)?;
        assert!(std::fs::read_to_string(&output)?.contains("first\t0\tchr1"));
    }
    let mut observations = Vec::new();
    for (name, rows) in [
        ("duplicate.sam", vec![first, duplicate]),
        ("unique.sam", vec![first]),
        ("empty.sam", Vec::new()),
        ("unmapped.sam", vec![unmapped]),
    ] {
        let input = write_observation_admission_sam(root.path(), name, &rows)?;
        let input_before = std::fs::read(&input)?;
        std::fs::write(&output, b"retained previous output\n")?;
        let before = std::fs::metadata(&output)?.permissions();
        let result = apply_duplicate_policy_tiny_bam(&input, &output, "unsupported-action", None);
        let retained = std::fs::read(&output)? == b"retained previous output\n"
            && std::fs::metadata(&output)?.permissions() == before;
        assert_eq!(std::fs::read(&input)?, input_before);
        eprintln!(
            "duplicate_action_admission {name}: refused={} output_retained={retained}",
            result.is_err()
        );
        observations.push(result.is_err() && retained);
    }
    assert!(
        observations.into_iter().all(|refused| refused),
        "unsupported duplicate action must refuse before publication independently of record shape"
    );
    Ok(())
}

#[test]
fn conflicting_kinship_site_calls_do_not_reappear_under_row_permutation() -> anyhow::Result<()> {
    use bijux_dna_domain_bam::summarize_tiny_bam_kinship;
    let root = bijux_dna_infra::temp_dir("kinship-conflict-admission")?;
    let a = "a\t0\tchr1\t1\t60\t1M\t*\t0\t0\tA\tF\tRG:Z:a";
    let conflict = "c\t0\tchr1\t1\t60\t1M\t*\t0\t0\tC\tF\tRG:Z:a";
    let b = "b\t0\tchr1\t1\t60\t1M\t*\t0\t0\tA\tF\tRG:Z:b";
    let positive_path = write_observation_admission_sam(root.path(), "positive.sam", &[a, b])?;
    let positive =
        summarize_tiny_bam_kinship(&positive_path, "king", "panel", "build", "diploid", 1, true)?;
    assert_eq!(positive.pair_count, 1);
    assert_eq!(positive.observed_max_overlap_snps, 1);
    let mut observations = Vec::new();
    for (name, rows) in
        [("conflict-repeated.sam", [a, conflict, a, b]), ("conflict-last.sam", [a, a, conflict, b])]
    {
        let path = write_observation_admission_sam(root.path(), name, &rows)?;
        let report =
            summarize_tiny_bam_kinship(&path, "king", "panel", "build", "diploid", 1, true)?;
        eprintln!("kinship_conflict_admission {name}: {report:?}");
        observations.push((report.pair_count, report.observed_max_overlap_snps, report.status));
    }
    assert!(
        observations.iter().all(|(pairs, overlap, status)| *pairs == 0
            && *overlap == 0
            && status == "insufficient"),
        "a conflicting site stays unavailable independently of later duplicate observations"
    );
    Ok(())
}
