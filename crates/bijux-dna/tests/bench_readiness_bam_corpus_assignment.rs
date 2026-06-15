#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

use serde_json::Value;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_bam_corpus_assignment_prints_governed_output_path() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-bam-corpus-assignment"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "benchmarks/readiness/bam-corpus-assignment.tsv"
    );
}

#[test]
fn bench_readiness_bam_corpus_assignment_json_reports_governed_stage_contracts() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-bam-corpus-assignment", "--json"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value =
        serde_json::from_slice(&output.stdout).expect("parse BAM corpus assignment report");
    assert_eq!(
        payload.get("schema_version").and_then(Value::as_str),
        Some("bijux.bench.readiness.bam_corpus_assignment.v3")
    );
    assert_eq!(payload.get("row_count").and_then(Value::as_u64), Some(49));

    let rows = payload.get("rows").and_then(Value::as_array).expect("rows array");

    let find_row = |tool_id: &str, stage_id: &str| {
        rows.iter()
            .find(|row| {
                row.get("tool_id").and_then(Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(Value::as_str) == Some(stage_id)
            })
            .unwrap_or_else(|| panic!("missing {stage_id} / {tool_id} row"))
    };

    let authenticity = find_row("authenticct", "bam.authenticity");
    assert_eq!(authenticity.get("sample_id").and_then(Value::as_str), Some("adna_damage_non_udg"));
    assert_eq!(
        authenticity.get("input_contract").and_then(Value::as_str),
        Some(
            "ct5p=0.18;ga3p=0.11;short_frag=1;signal=moderate;strict_profile_upgraded=false;terminal=ct5p_dominant;udg=non_udg"
        )
    );
    assert_eq!(
        authenticity.get("benchmark_limits").and_then(Value::as_str),
        Some("complexity_min_reads=3;coverage_depth_thresholds=1,5,10")
    );
    assert_eq!(
        authenticity.get("required_assets").and_then(Value::as_str),
        Some("expected_damage=expected_damage.json;reference_fasta=adna_damage_reference.fasta")
    );
    assert_eq!(
        authenticity.get("expected_outputs").and_then(Value::as_str),
        Some("authenticity_report,summary,stage_metrics")
    );
    assert_eq!(authenticity.get("skip_behavior").and_then(Value::as_str), Some("not_applicable"));

    let contamination = find_row("verifybamid2", "bam.contamination");
    assert_eq!(
        contamination.get("sample_id").and_then(Value::as_str),
        Some("adna_contamination_panel_screen")
    );
    assert_eq!(
        contamination.get("input_contract").and_then(Value::as_str),
        Some("signal=moderate;terminal=ct5p_dominant;udg=non_udg")
    );
    assert_eq!(
        contamination.get("benchmark_limits").and_then(Value::as_str),
        Some("minimum_mean_coverage=0.5")
    );
    assert_eq!(
        contamination.get("required_assets").and_then(Value::as_str),
        Some("reference_fasta=adna_bam_reference;reference_panel=adna_contamination_panel")
    );
    assert_eq!(
        contamination.get("expected_outputs").and_then(Value::as_str),
        Some("contamination_report,summary,stage_metrics")
    );

    let sex = find_row("rxy", "bam.sex");
    assert_eq!(sex.get("sample_id").and_then(Value::as_str), Some("adna_xy_autosome_coverage"));
    assert_eq!(
        sex.get("benchmark_limits").and_then(Value::as_str),
        Some(
            "expected_autosomal_coverage=1;expected_x_coverage=0.5;expected_y_coverage=0.5;minimum_y_sites=5"
        )
    );
    assert_eq!(
        sex.get("required_assets").and_then(Value::as_str),
        Some("reference_fasta=adna_bam_reference")
    );
    assert_eq!(
        sex.get("expected_outputs").and_then(Value::as_str),
        Some("sex_report,summary,stage_metrics")
    );

    let haplogroups = find_row("yleaf", "bam.haplogroups");
    assert_eq!(
        haplogroups.get("sample_id").and_then(Value::as_str),
        Some("adna_y_haplogroup_panel")
    );
    assert_eq!(haplogroups.get("benchmark_limits").and_then(Value::as_str), Some("min_coverage=2"));
    assert_eq!(
        haplogroups.get("required_assets").and_then(Value::as_str),
        Some("reference_fasta=adna_bam_reference;reference_panel=adna-y-hg38-mini")
    );
    assert_eq!(
        haplogroups.get("expected_outputs").and_then(Value::as_str),
        Some("haplogroups,summary,stage_metrics")
    );

    let genotyping = find_row("angsd", "bam.genotyping");
    assert_eq!(
        genotyping.get("sample_id").and_then(Value::as_str),
        Some("human_like_genotyping_candidate_panel")
    );
    assert_eq!(
        genotyping.get("input_contract").and_then(Value::as_str),
        Some(
            "reference=corpus_01_bam_reference;regions=human_like_genotyping_target_regions;sites=human_like_genotyping_candidate_sites"
        )
    );
    assert_eq!(
        genotyping.get("benchmark_limits").and_then(Value::as_str),
        Some("min_call_rate=0.5;min_posterior=0.9")
    );
    assert_eq!(
        genotyping.get("required_assets").and_then(Value::as_str),
        Some(
            "reference_fasta=corpus_01_bam_reference;regions=human_like_genotyping_target_regions;sites_vcf=human_like_genotyping_candidate_sites"
        )
    );
    assert_eq!(
        genotyping.get("expected_outputs").and_then(Value::as_str),
        Some(
            "genotyping_bcf,genotyping_vcf,genotyping_vcf_tbi,genotyping_gl,summary,stage_metrics"
        )
    );
    assert_eq!(genotyping.get("skip_behavior").and_then(Value::as_str), Some("not_applicable"));

    let kinship = find_row("king", "bam.kinship");
    assert_eq!(
        kinship.get("sample_id").and_then(Value::as_str),
        Some("human_like_kinship_low_overlap_pair,human_like_kinship_related_pair")
    );
    assert_eq!(
        kinship.get("input_contract").and_then(Value::as_str),
        Some(
            "population_scope=human_diploid_panel;reference=corpus_01_bam_reference;reference_build=grch38;reference_panel=human_like_relatedness_panel"
        )
    );
    assert_eq!(
        kinship.get("benchmark_limits").and_then(Value::as_str),
        Some(
            "human_like_kinship_low_overlap_pair.min_overlap_snps=5;human_like_kinship_low_overlap_pair.observed_max_overlap_snps=4;human_like_kinship_related_pair.min_overlap_snps=6;human_like_kinship_related_pair.observed_max_overlap_snps=6"
        )
    );
    assert_eq!(
        kinship.get("required_assets").and_then(Value::as_str),
        Some(
            "reference_fasta=corpus_01_bam_reference;reference_panel=human_like_relatedness_panel"
        )
    );
    assert_eq!(
        kinship.get("expected_outputs").and_then(Value::as_str),
        Some("kinship_report,summary,kinship_segments,stage_metrics")
    );
    assert_eq!(
        kinship.get("skip_behavior").and_then(Value::as_str),
        Some(
            "human_like_kinship_low_overlap_pair=stop_without_pairwise_results;human_like_kinship_related_pair=emit_pairwise_results"
        )
    );

    let kinship_angsd = find_row("angsd", "bam.kinship");
    assert_eq!(
        kinship_angsd.get("sample_id").and_then(Value::as_str),
        Some("human_like_kinship_low_overlap_pair,human_like_kinship_related_pair")
    );
    assert_eq!(
        kinship_angsd.get("expected_outputs").and_then(Value::as_str),
        Some("kinship_report,summary,kinship_segments,stage_metrics")
    );
    assert_eq!(
        kinship_angsd.get("skip_behavior").and_then(Value::as_str),
        Some(
            "human_like_kinship_low_overlap_pair=stop_without_pairwise_results;human_like_kinship_related_pair=emit_pairwise_results"
        )
    );

    let general_alignment = find_row("bwa", "bam.align");
    assert_eq!(general_alignment.get("sample_id").and_then(Value::as_str), Some("not_applicable"));
    assert_eq!(
        general_alignment.get("input_contract").and_then(Value::as_str),
        Some("not_applicable")
    );
    assert_eq!(
        general_alignment.get("benchmark_limits").and_then(Value::as_str),
        Some("not_applicable")
    );
    assert_eq!(
        general_alignment.get("required_assets").and_then(Value::as_str),
        Some("not_applicable")
    );
    assert_eq!(
        general_alignment.get("expected_outputs").and_then(Value::as_str),
        Some("not_applicable")
    );
    assert_eq!(
        general_alignment.get("skip_behavior").and_then(Value::as_str),
        Some("not_applicable")
    );
}
