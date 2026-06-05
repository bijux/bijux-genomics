#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_validate_bam_corpus_fixture_json_reports_governed_corpus_01_bam_mini_contract() {
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
        .args([
            "bench",
            "local",
            "validate-corpus-fixture",
            "--manifest",
            "tests/fixtures/corpora/corpus-01-bam-mini/manifest.toml",
            "--json",
        ])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse stdout as json");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.bam_corpus_fixture_validation.v1")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("tests/fixtures/corpora/corpus-01-bam-mini/manifest.toml")
    );
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-bam-mini")
    );
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(21));
    assert_eq!(
        payload.get("reference_contigs").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(6)
    );
    assert!(payload.get("valid").and_then(serde_json::Value::as_bool) == Some(true));
    assert!(payload.get("samples").and_then(serde_json::Value::as_array).is_some_and(|samples| {
        samples.len() == 21
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_duplicate_flagged_multicontig")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs == &vec![serde_json::json!("chr1"), serde_json::json!("chr2")]
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids
                                == &vec![serde_json::json!(
                                    "human_like_duplicate_flagged_multicontig"
                                )]
                        })
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_kinship_low_overlap_pair")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chr1")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids
                                == &vec![
                                    serde_json::json!("sample_a"),
                                    serde_json::json!("sample_b"),
                                ]
                        })
                    && sample
                        .get("observed_read_group_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|read_groups| {
                            read_groups
                                == &vec![
                                    serde_json::json!("rg-kinship-low-overlap-a"),
                                    serde_json::json!("rg-kinship-low-overlap-b"),
                                ]
                        })
                    && sample.get("observed_record_count").and_then(serde_json::Value::as_u64)
                        == Some(2)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_kinship_related_pair")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chr1")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids
                                == &vec![
                                    serde_json::json!("sample_a"),
                                    serde_json::json!("sample_b"),
                                ]
                        })
                    && sample
                        .get("observed_read_group_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|read_groups| {
                            read_groups
                                == &vec![
                                    serde_json::json!("rg-kinship-related-a"),
                                    serde_json::json!("rg-kinship-related-b"),
                                ]
                        })
                    && sample.get("observed_record_count").and_then(serde_json::Value::as_u64)
                        == Some(2)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_partial_mapping")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chr1")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_partial_mapping")
                        })
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_recalibration_low_coverage")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chr1")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_recalibration_low_coverage")
                        })
                    && sample
                        .get("observed_read_group_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|read_groups| {
                            read_groups.len() == 1
                                && read_groups.first().and_then(serde_json::Value::as_str)
                                    == Some("rg-recalibration-human-like")
                        })
                    && sample.get("observed_record_count").and_then(serde_json::Value::as_u64)
                        == Some(2)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_endogenous_partial_mapping")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chr1")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_endogenous_partial_mapping")
                        })
                    && sample
                        .get("observed_read_group_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|read_groups| {
                            read_groups.len() == 1
                                && read_groups.first().and_then(serde_json::Value::as_str)
                                    == Some("rg-endogenous-content-human-like")
                        })
                    && sample.get("observed_record_count").and_then(serde_json::Value::as_u64)
                        == Some(5)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_contamination_panel_screen")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chr1")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_contamination_panel_screen")
                        })
                    && sample
                        .get("observed_read_group_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|read_groups| {
                            read_groups.len() == 1
                                && read_groups.first().and_then(serde_json::Value::as_str)
                                    == Some("rg-contamination-human-like")
                        })
                    && sample.get("observed_record_count").and_then(serde_json::Value::as_u64)
                        == Some(3)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_paired_overlap_control")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chr1")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_paired_overlap_control")
                        })
                    && sample
                        .get("observed_read_group_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|read_groups| {
                            read_groups.len() == 1
                                && read_groups.first().and_then(serde_json::Value::as_str)
                                    == Some("rg-overlap-correction-human-like")
                        })
                    && sample.get("observed_record_count").and_then(serde_json::Value::as_u64)
                        == Some(4)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_mapq_threshold_ladder")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chr1")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_mapq_threshold_ladder")
                        })
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_length_threshold_ladder")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chr1")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_length_threshold_ladder")
                        })
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_mixed_filter_constraints")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chr1")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_mixed_filter_constraints")
                        })
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_duplicate_cluster")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chr1")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_duplicate_cluster")
                        })
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_complexity_projection")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chr1")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_complexity_projection")
                        })
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_target_window_coverage")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs == &vec![serde_json::json!("chr1"), serde_json::json!("chr2")]
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_target_window_coverage")
                        })
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_insert_size_triplet")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chr1")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_insert_size_triplet")
                        })
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_gc_window_ladder")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chrgc")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_gc_window_ladder")
                        })
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_xy_autosome_coverage")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs
                                == &vec![
                                    serde_json::json!("chr1"),
                                    serde_json::json!("chrX"),
                                    serde_json::json!("chrY"),
                                ]
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_xy_autosome_coverage")
                        })
                    && sample
                        .get("observed_read_group_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|read_groups| {
                            read_groups.len() == 1
                                && read_groups.first().and_then(serde_json::Value::as_str)
                                    == Some("rg-sex-human-like")
                        })
                    && sample.get("observed_record_count").and_then(serde_json::Value::as_u64)
                        == Some(5)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_y_haplogroup_panel")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chrY")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_y_haplogroup_panel")
                        })
                    && sample
                        .get("observed_read_group_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|read_groups| {
                            read_groups.len() == 1
                                && read_groups.first().and_then(serde_json::Value::as_str)
                                    == Some("rg-haplogroups-human-like")
                        })
                    && sample.get("observed_record_count").and_then(serde_json::Value::as_u64)
                        == Some(4)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_genotyping_candidate_panel")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chr1")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("human_like_genotyping_candidate_panel")
                        })
                    && sample
                        .get("observed_read_group_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|read_groups| {
                            read_groups.len() == 1
                                && read_groups.first().and_then(serde_json::Value::as_str)
                                    == Some("rg-genotyping-human-like")
                        })
                    && sample.get("observed_record_count").and_then(serde_json::Value::as_u64)
                        == Some(4)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("adna_like_damage")
                    && sample
                        .get("observed_contigs")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|contigs| {
                            contigs.len() == 1
                                && contigs.first().and_then(serde_json::Value::as_str)
                                    == Some("chranc")
                        })
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids.len() == 1
                                && sample_ids.first().and_then(serde_json::Value::as_str)
                                    == Some("adna_like_damage")
                        })
            })
    }));
}

#[test]
fn bench_local_validate_bam_corpus_fixture_json_reports_governed_genotyping_contract() {
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
        .args([
            "bench",
            "local",
            "validate-corpus-fixture",
            "--manifest",
            "tests/fixtures/corpora/corpus-01-genotyping-mini/manifest.toml",
            "--json",
        ])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse stdout as json");
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-genotyping-mini")
    );
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(1));
    assert!(payload.get("samples").and_then(serde_json::Value::as_array).is_some_and(|samples| {
        samples.len() == 1
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_genotyping_candidate_panel")
                    && sample
                        .get("observed_read_group_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|read_groups| {
                            read_groups
                                == &vec![serde_json::json!("rg-genotyping-human-like")]
                        })
            })
    }));
}

#[test]
fn bench_local_validate_bam_corpus_fixture_json_reports_governed_kinship_contract() {
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
        .args([
            "bench",
            "local",
            "validate-corpus-fixture",
            "--manifest",
            "tests/fixtures/corpora/corpus-01-kinship-mini/manifest.toml",
            "--json",
        ])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse stdout as json");
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-kinship-mini")
    );
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(2));
    assert!(payload.get("samples").and_then(serde_json::Value::as_array).is_some_and(|samples| {
        samples.len() == 2
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_kinship_low_overlap_pair")
                    && sample
                        .get("observed_header_sample_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|sample_ids| {
                            sample_ids
                                == &vec![
                                    serde_json::json!("sample_a"),
                                    serde_json::json!("sample_b"),
                                ]
                        })
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_kinship_related_pair")
                    && sample
                        .get("observed_read_group_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|read_groups| {
                            read_groups
                                == &vec![
                                    serde_json::json!("rg-kinship-related-a"),
                                    serde_json::json!("rg-kinship-related-b"),
                                ]
                        })
            })
    }));
}
