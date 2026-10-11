#![allow(clippy::expect_used)]

use bijux_dna_stages_vcf::pipeline::{
    run_damage_filter_stage_with_policy, DamageFilterPolicy, DamageFilterStageParams,
    DamageLibraryType, DamageMaskMode, DamageUdgRegime,
};
use bijux_dna_stages_vcf::vcf_io::read_vcf_text;
use std::path::Path;

fn params() -> DamageFilterStageParams {
    DamageFilterStageParams {
        udg_regime: DamageUdgRegime::NonUdg,
        strict_regime: true,
        min_qual: 30.0,
        max_damage_ratio: 0.35,
    }
}

fn write_input(path: &Path) {
    std::fs::write(
        path,
        concat!(
        "##fileformat=VCFv4.2\n##contig=<ID=1,length=1000>\n",
        "##INFO=<ID=CT_GA_DAMAGE_RATIO,Number=1,Type=Float,Description=\"Damage ratio\">\n",
        "##INFO=<ID=DEAM5P,Number=1,Type=Float,Description=\"Terminal damage\">\n",
        "##INFO=<ID=PMD_SCORE,Number=1,Type=Float,Description=\"PMD log score\">\n",
        "##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n",
        "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample1\n",
        "1\t10\tdamaged\tC\tT\t60\tPASS\tCT_GA_DAMAGE_RATIO=0.1;DEAM5P=0.45;PMD_SCORE=4\tGT\t0/1\n",
        "1\t20\tnegative_pmd\tA\tG\t60\tPASS\tCT_GA_DAMAGE_RATIO=0.1;PMD_SCORE=-1\tGT\t0/1\n",
    ),
    )
    .expect("write damage observations");
}

fn json(path: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).expect("read artifact"))
        .expect("parse artifact")
}

#[test]
fn independent_damage_policies_run_concurrently_without_duplicate_marked_records() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = dir.path().join("input.vcf");
    write_input(&input);
    let mark = DamageFilterPolicy {
        mask_mode: DamageMaskMode::Mark,
        terminal_damage_threshold: 0.4,
        pmd_min: -2.0,
        ..DamageFilterPolicy::recommended(DamageUdgRegime::NonUdg)
    };
    let remove = DamageFilterPolicy { mask_mode: DamageMaskMode::Remove, ..mark };
    let (marked, removed) = std::thread::scope(|scope| {
        let marked = scope.spawn(|| {
            run_damage_filter_stage_with_policy(
                &input,
                &dir.path().join("marked"),
                &params(),
                &mark,
            )
            .expect("mark execution")
        });
        let removed = scope.spawn(|| {
            run_damage_filter_stage_with_policy(
                &input,
                &dir.path().join("removed"),
                &params(),
                &remove,
            )
            .expect("remove execution")
        });
        (marked.join().expect("mark thread"), removed.join().expect("remove thread"))
    });
    let marked_vcf = read_vcf_text(&marked.filtered_vcf).expect("marked VCF");
    let marked_records: Vec<_> = marked_vcf.lines().filter(|line| !line.starts_with('#')).collect();
    assert_eq!(marked_records.len(), 2, "mark retains exactly one row per input record");
    assert_eq!(marked_records[0].split('\t').nth(6), Some("LOWCONF_DAMAGE_TERMINAL"));
    assert_eq!(marked_records[1].split('\t').nth(2), Some("negative_pmd"));
    let removed_vcf = read_vcf_text(&removed.filtered_vcf).expect("removed VCF");
    let removed_records: Vec<_> =
        removed_vcf.lines().filter(|line| !line.starts_with('#')).collect();
    assert_eq!(removed_records.len(), 1);
    assert_eq!(removed_records[0].split('\t').nth(2), Some("negative_pmd"));
    let marked_counts = json(&marked.damage_filter_counts_json);
    assert_eq!(marked_counts.pointer("/counts/kept"), Some(&serde_json::json!(2)));
    assert_eq!(marked_counts.pointer("/counts/lowconf_marked"), Some(&serde_json::json!(1)));
    assert_eq!(
        json(&removed.damage_filter_counts_json).pointer("/counts/kept"),
        Some(&serde_json::json!(1))
    );
    assert_eq!(json(&marked.damage_genotype_manifest_json)["masking_mode"], "mark");
    assert_eq!(json(&removed.damage_genotype_manifest_json)["masking_mode"], "remove");
}

#[test]
fn explicit_library_context_controls_terminal_threshold_and_preserves_negative_pmd_scores() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = dir.path().join("input.vcf");
    write_input(&input);
    for (library_type, name, count, threshold) in [
        (DamageLibraryType::NotDeclared, "not_declared", 2, 0.5),
        (DamageLibraryType::DoubleStranded, "dsdna", 2, 0.5),
        (DamageLibraryType::SingleStranded, "ssdna", 1, 0.4),
    ] {
        let policy = DamageFilterPolicy {
            library_type,
            pmd_min: -2.0,
            ..DamageFilterPolicy::recommended(DamageUdgRegime::NonUdg)
        };
        let outputs =
            run_damage_filter_stage_with_policy(&input, &dir.path().join(name), &params(), &policy)
                .expect("library policy execution");
        let vcf = read_vcf_text(&outputs.filtered_vcf).expect("VCF");
        assert_eq!(vcf.lines().filter(|line| !line.starts_with('#')).count(), count);
        let manifest = json(&outputs.damage_genotype_manifest_json);
        assert_eq!(manifest["library_type"], name);
        assert_eq!(manifest["thresholds"]["terminal_damage_threshold"], threshold);
        assert_eq!(manifest["thresholds"]["pmd_min"], -2.0);
    }
}

#[test]
fn invalid_explicit_damage_policy_refuses_before_creating_outputs_or_reading_inputs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = DamageFilterPolicy::recommended(DamageUdgRegime::NonUdg);
    for (index, policy) in [
        DamageFilterPolicy { terminal_damage_threshold: f64::NAN, ..base },
        DamageFilterPolicy { terminal_damage_threshold: f64::INFINITY, ..base },
        DamageFilterPolicy { terminal_damage_threshold: -0.01, ..base },
        DamageFilterPolicy { terminal_damage_threshold: 1.01, ..base },
        DamageFilterPolicy { pmd_min: f64::NAN, ..base },
        DamageFilterPolicy { pmd_min: f64::INFINITY, ..base },
        DamageFilterPolicy { pmd_min: f64::NEG_INFINITY, ..base },
    ]
    .into_iter()
    .enumerate()
    {
        let output = dir.path().join(index.to_string());
        let error = run_damage_filter_stage_with_policy(
            &dir.path().join("missing.vcf"),
            &output,
            &params(),
            &policy,
        )
        .expect_err("reject invalid policy");
        assert!(error.to_string().contains("vcf.damage_filter refusal:"), "{error:#}");
        assert!(!output.exists());
    }
}
