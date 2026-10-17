#![allow(clippy::expect_used)]

use std::process::Command;

use crate::support;

#[test]
fn dev_crates_planner_no_parser_writes_the_governed_audit_report() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let temp = tempfile::tempdir().expect("tempdir");
    let out = temp.path().join("planner-no-parser.json");

    let output = Command::new(support::cli_binary_path())
        .current_dir(&repo_root)
        .arg("dev")
        .arg("crates")
        .arg("planner-no-parser")
        .arg("--output")
        .arg(&out)
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
        serde_json::from_slice(&output.stdout).expect("stdout json payload");
    let expected_output_path = support::path_relative_to_repo(&repo_root, &out);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.crates.planner_no_parser.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some(expected_output_path.as_str())
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("audited_crate_count").and_then(serde_json::Value::as_u64), Some(3));

    let crates = payload.get("crates").and_then(serde_json::Value::as_array).expect("crates array");
    for crate_name in ["bijux-dna-planner-bam", "bijux-dna-planner-fastq", "bijux-dna-planner-vcf"]
    {
        let report = crates
            .iter()
            .find(|crate_report| {
                crate_report.get("crate_name").and_then(serde_json::Value::as_str)
                    == Some(crate_name)
            })
            .unwrap_or_else(|| panic!("missing crate report for {crate_name}"));
        assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(true));
        let allowed = report
            .get("allowed_input_read_paths")
            .and_then(serde_json::Value::as_array)
            .expect("allowed_input_read_paths");
        assert!(
            !allowed.is_empty(),
            "{crate_name} must declare the governed planner-input read allowlist"
        );
        for key in ["unexpected_input_read_refs", "forbidden_parser_api_refs"] {
            let rows = report.get(key).and_then(serde_json::Value::as_array).expect(key);
            assert!(rows.is_empty(), "{crate_name} must have no {key}");
        }
    }

    assert_bam_input_readers(crates);
    assert_fastq_input_readers(crates);

    let vcf_report = crates
        .iter()
        .find(|crate_report| {
            crate_report.get("crate_name").and_then(serde_json::Value::as_str)
                == Some("bijux-dna-planner-vcf")
        })
        .expect("vcf planner report");
    let vcf_allowed = vcf_report
        .get("allowed_input_read_paths")
        .and_then(serde_json::Value::as_array)
        .expect("allowed_input_read_paths");
    assert!(
        vcf_allowed.iter().any(|row| {
            row.get("path").and_then(serde_json::Value::as_str)
                == Some("crates/bijux-dna-planner-vcf/src/workspace_config.rs")
        }),
        "VCF planner audit must keep governed workspace config reads explicit"
    );

    assert!(out.is_file(), "audit must write the governed report file");
}

fn assert_bam_input_readers(crates: &[serde_json::Value]) {
    let bam_report = crates
        .iter()
        .find(|report| {
            report.get("crate_name").and_then(serde_json::Value::as_str)
                == Some("bijux-dna-planner-bam")
        })
        .expect("bam planner report");
    let bam_case_paths = bam_report
        .get("planner_input_read_refs")
        .and_then(serde_json::Value::as_array)
        .expect("bam input reads")
        .iter()
        .filter_map(|row| row.get("path").and_then(serde_json::Value::as_str))
        .filter(|path| path.starts_with("crates/bijux-dna-planner-bam/src/benchmark/cases/"))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        bam_case_paths,
        std::collections::BTreeSet::from([
            "crates/bijux-dna-planner-bam/src/benchmark/cases/ancient/authenticity.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/ancient/damage.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/ancient/sex.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/downstream/bias_mitigation.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/downstream/kinship.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/pre/filter.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/pre/length_filter.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/pre/mapping_summary.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/pre/mapq_filter.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/pre/overlap_correction.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/pre/qc_pre.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/pre/validate.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/complexity.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/coverage.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/duplication_metrics.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/endogenous_content.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/gc_bias.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/insert_size.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/markdup.rs",
            "crates/bijux-dna-planner-bam/src/benchmark/cases/qc/recalibration.rs",
        ]),
        "every governed BAM case input reader must remain explicit"
    );
}

fn assert_fastq_input_readers(crates: &[serde_json::Value]) {
    let fastq_report = crates
        .iter()
        .find(|crate_report| {
            crate_report.get("crate_name").and_then(serde_json::Value::as_str)
                == Some("bijux-dna-planner-fastq")
        })
        .expect("fastq planner report");
    let fastq_read_refs = fastq_report
        .get("planner_input_read_refs")
        .and_then(serde_json::Value::as_array)
        .expect("planner_input_read_refs");
    let case_input_paths = fastq_read_refs
        .iter()
        .filter_map(|row| row.get("path").and_then(serde_json::Value::as_str))
        .filter(|path| {
            path.starts_with("crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/")
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(case_input_paths, std::collections::BTreeSet::from([
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/amplicon/cluster_otus.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/amplicon/infer_asvs.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/amplicon/normalize_abundance.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/amplicon/normalize_primers.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/amplicon/remove_chimeras.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/pre/detect_adapters.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/pre/detect_duplicates_premerge.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/pre/estimate_library_complexity_prealign.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/pre/profile_overrepresented_sequences.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/pre/profile_read_lengths.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/pre/validate_reads.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/qc/profile_reads.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/qc/report_qc.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/correct_errors.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/extract_umis.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/filter_low_complexity.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/filter_reads.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/merge_pairs.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/remove_duplicates.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/trim_polyg_tails.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/trim_reads.rs",
        "crates/bijux-dna-planner-fastq/src/planner/benchmark/cases/transform/trim_terminal_damage.rs",
    ]), "every governed FASTQ case input reader must remain explicit in the planner audit");
}
