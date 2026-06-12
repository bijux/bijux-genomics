#![allow(clippy::expect_used)]

use std::collections::BTreeSet;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
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
        .args(args)
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn bench_readiness_all_domain_rendered_commands_tracks_governed_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-all-domain-commands", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_rendered_commands.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/rendered-commands-all-domains.sh")
    );
    assert_eq!(
        payload.get("argv_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/rendered-commands-all-domains.argv.jsonl")
    );
    let row_count = support::json_u64(&payload, "row_count").expect("row_count");
    assert_eq!(support::json_u64(&payload, "result_id_count"), Some(row_count));

    let domain_counts = support::json_object(&payload, "domain_counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(63));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(19));
    assert_eq!(support::object_u64_sum(domain_counts), row_count);

    let command_source_counts = support::json_object(&payload, "command_source_counts");
    assert_eq!(
        command_source_counts.get("fastq_bam_command_adapter").and_then(serde_json::Value::as_u64),
        Some(112)
    );
    assert_eq!(
        command_source_counts.get("vcf_bcftools_adapter").and_then(serde_json::Value::as_u64),
        Some(11)
    );
    assert_eq!(
        command_source_counts
            .get("vcf_eigensoft_adapter")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        command_source_counts.get("vcf_plink_family_adapter").and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        command_source_counts
            .get("vcf_imputation_family_adapter")
            .and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        command_source_counts
            .get("vcf_phasing_family_adapter")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let rows = support::json_array(&payload, "rows");
    assert_eq!(rows.len() as u64, row_count);
    let result_ids = rows
        .iter()
        .filter_map(|row| row.get("result_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(result_ids.len() as u64, row_count);

    let bias_mitigation = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("bam:corpus-01-bam-mini:bam.bias_mitigation:sample-set:mapdamage2")
        })
        .expect("bias mitigation row");
    assert_eq!(
        bias_mitigation.get("command_source").and_then(serde_json::Value::as_str),
        Some("fastq_bam_command_adapter")
    );
    assert!(bias_mitigation
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|steps| {
            steps.iter().any(|step| {
                step.get("argv").and_then(serde_json::Value::as_array).is_some_and(|argv| {
                    argv.iter().any(|token| token.as_str() == Some("bam_downstream"))
                })
            })
        }));

    let taxonomy = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2")
        })
        .expect("taxonomy row");
    assert_eq!(
        taxonomy.get("asset_profile_id").and_then(serde_json::Value::as_str),
        Some("database_artifact_id+taxonomy_database_root")
    );
    assert!(taxonomy.get("command_steps").and_then(serde_json::Value::as_array).is_some_and(
        |steps| {
            steps.iter().any(|step| {
                step.get("argv").and_then(serde_json::Value::as_array).is_some_and(|argv| {
                    argv.iter()
                        .filter_map(serde_json::Value::as_str)
                        .collect::<Vec<_>>()
                        .join(" ")
                        .contains("kraken2 --db")
                })
            })
        }
    ));

    let vcf_call = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")
        })
        .expect("VCF call row");
    assert_eq!(
        vcf_call.get("command_source").and_then(serde_json::Value::as_str),
        Some("vcf_bcftools_adapter")
    );
    assert!(vcf_call.get("command_steps").and_then(serde_json::Value::as_array).is_some_and(
        |steps| {
            steps.len() == 3
                && steps.iter().any(|step| {
                    step.get("argv").and_then(serde_json::Value::as_array).is_some_and(|argv| {
                        argv.first().and_then(serde_json::Value::as_str) == Some("bcftools")
                    })
                })
        }
    ));

    let vcf_postprocess = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools")
        })
        .expect("VCF postprocess row");
    assert_eq!(
        vcf_postprocess.get("command_source").and_then(serde_json::Value::as_str),
        Some("vcf_bcftools_adapter")
    );
    assert!(vcf_postprocess
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|steps| {
            steps.len() == 2
                && steps.iter().any(|step| {
                    step.get("argv").and_then(serde_json::Value::as_array).is_some_and(|argv| {
                        argv.iter()
                            .filter_map(serde_json::Value::as_str)
                            .collect::<Vec<_>>()
                            .join(" ")
                            .contains("bcftools +fill-tags")
                    })
                })
        }));

    let imputation_metrics = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some(
                    "vcf:vcf_production_regression:vcf.imputation_metrics:vcf_cohort_with_panel:beagle",
                )
        })
        .expect("VCF imputation metrics row");
    assert_eq!(
        imputation_metrics.get("command_source").and_then(serde_json::Value::as_str),
        Some("vcf_imputation_family_adapter")
    );
}
