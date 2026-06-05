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
fn bench_local_validate_bam_corpus_fixture_json_reports_governed_corpus_01_adna_bam_mini_contract()
{
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
            "tests/fixtures/corpora/corpus-01-adna-bam-mini/manifest.toml",
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
        Some("tests/fixtures/corpora/corpus-01-adna-bam-mini/manifest.toml")
    );
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-adna-bam-mini")
    );
    assert_eq!(payload.get("udg_model").and_then(serde_json::Value::as_str), Some("non_udg"));
    assert_eq!(payload.get("damage_signal").and_then(serde_json::Value::as_str), Some("moderate"));
    assert_eq!(
        payload.get("expected_terminal_pattern_class").and_then(serde_json::Value::as_str),
        Some("ct5p_dominant")
    );
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(3));
    assert!(payload.get("valid").and_then(serde_json::Value::as_bool) == Some(true));
    assert!(payload.get("samples").and_then(serde_json::Value::as_array).is_some_and(|samples| {
        samples.iter().any(|sample| {
            sample.get("sample_id").and_then(serde_json::Value::as_str)
                == Some("adna_contamination_panel_screen")
                && sample
                    .get("observed_read_group_ids")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|read_groups| {
                        read_groups == &vec![serde_json::json!("rg-contamination-adna")]
                    })
        }) && samples.iter().any(|sample| {
            sample.get("sample_id").and_then(serde_json::Value::as_str)
                == Some("adna_xy_autosome_coverage")
                && sample.get("observed_contigs").and_then(serde_json::Value::as_array).is_some_and(
                    |contigs| {
                        contigs
                            == &vec![
                                serde_json::json!("chr1"),
                                serde_json::json!("chrX"),
                                serde_json::json!("chrY"),
                            ]
                    },
                )
        }) && samples.iter().any(|sample| {
            sample.get("sample_id").and_then(serde_json::Value::as_str)
                == Some("adna_y_haplogroup_panel")
                && sample
                    .get("observed_read_group_ids")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|read_groups| {
                        read_groups == &vec![serde_json::json!("rg-haplogroups-adna")]
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
    let contract = payload
        .get("genotyping_contract")
        .unwrap_or_else(|| panic!("genotyping_contract must be present"));
    assert_eq!(
        contract.get("sample_id").and_then(serde_json::Value::as_str),
        Some("human_like_genotyping_candidate_panel")
    );
    assert_eq!(
        contract.get("sites_vcf").and_then(serde_json::Value::as_str),
        Some(
            "tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_genotyping_candidate_sites.vcf"
        )
    );
    assert_eq!(
        contract.get("regions").and_then(serde_json::Value::as_str),
        Some(
            "tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_genotyping_target_regions.txt"
        )
    );
    assert_eq!(contract.get("min_posterior").and_then(serde_json::Value::as_f64), Some(0.9));
    assert_eq!(contract.get("min_call_rate").and_then(serde_json::Value::as_f64), Some(0.5));
    assert_eq!(
        contract
            .get("expected_outputs")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec![
            "genotyping_bcf",
            "genotyping_vcf",
            "genotyping_vcf_tbi",
            "genotyping_gl",
            "summary",
            "stage_metrics",
        ])
    );
    assert!(payload.get("samples").and_then(serde_json::Value::as_array).is_some_and(|samples| {
        samples.len() == 1
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_genotyping_candidate_panel")
                    && sample
                        .get("observed_read_group_ids")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|read_groups| {
                            read_groups == &vec![serde_json::json!("rg-genotyping-human-like")]
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
    let contract = payload
        .get("kinship_contract")
        .unwrap_or_else(|| panic!("kinship_contract must be present"));
    assert_eq!(
        contract.get("reference_panel").and_then(serde_json::Value::as_str),
        Some("human_like_relatedness_panel")
    );
    assert_eq!(
        contract.get("reference_panel_path").and_then(serde_json::Value::as_str),
        Some(
            "tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_relatedness_panel.tsv"
        )
    );
    assert_eq!(contract.get("reference_build").and_then(serde_json::Value::as_str), Some("grch38"));
    assert_eq!(
        contract.get("population_scope").and_then(serde_json::Value::as_str),
        Some("human_diploid_panel")
    );
    assert_eq!(
        contract
            .get("expected_outputs")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["kinship_report", "summary", "kinship_segments", "stage_metrics"])
    );
    assert!(contract.get("cases").and_then(serde_json::Value::as_array).is_some_and(|cases| {
        cases.len() == 2
            && cases.iter().any(|case| {
                case.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_kinship_low_overlap_pair")
                    && case.get("min_overlap_snps").and_then(serde_json::Value::as_u64) == Some(5)
                    && case.get("expected_status").and_then(serde_json::Value::as_str)
                        == Some("insufficient")
                    && case
                        .get("expected_observed_max_overlap_snps")
                        .and_then(serde_json::Value::as_u64)
                        == Some(4)
                    && case.get("skip_behavior").and_then(serde_json::Value::as_str)
                        == Some("stop_without_pairwise_results")
            })
            && cases.iter().any(|case| {
                case.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_kinship_related_pair")
                    && case.get("min_overlap_snps").and_then(serde_json::Value::as_u64) == Some(6)
                    && case.get("expected_status").and_then(serde_json::Value::as_str) == Some("ok")
                    && case
                        .get("expected_observed_max_overlap_snps")
                        .and_then(serde_json::Value::as_u64)
                        == Some(6)
                    && case
                        .get("expected_relationship_labels")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|labels| labels == &vec![serde_json::json!("first_degree")])
                    && case.get("skip_behavior").and_then(serde_json::Value::as_str)
                        == Some("emit_pairwise_results")
            })
    }));
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
