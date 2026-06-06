#![allow(clippy::expect_used)]

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
fn bench_local_pipeline_dag_validates_fastq_core_preprocess_contract() {
    let payload = run_cli_json(&["bench", "local", "validate-pipeline-dag", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_pipeline_dag_validation.v1")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/fastq-core-preprocess.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/fastq-core-preprocess.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("fastq-core-preprocess")
    );
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-mini")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(payload.get("acyclic").and_then(serde_json::Value::as_bool), Some(true));

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.trim_reads")
                && node.get("depends_on").and_then(serde_json::Value::as_array).is_some_and(
                    |deps| {
                        deps.iter().any(|dep| dep.as_str() == Some("fastq.validate_reads"))
                            && deps.iter().any(|dep| dep.as_str() == Some("fastq.detect_adapters"))
                    },
                )
        }),
        "trim_reads must depend on validation and adapter detection"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.report_qc")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|input| input.as_str() == Some("validation_report"))
                            && inputs.iter().any(|input| input.as_str() == Some("filtered_profile"))
                    },
                )
        }),
        "report_qc must collate governed upstream preprocessing metrics"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_fastq_paired_merge_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/fastq-paired-merge.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/fastq-paired-merge.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/fastq-paired-merge.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("fastq-paired-merge")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(12));

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.merge_pairs")
                && node.get("outputs").and_then(serde_json::Value::as_array).is_some_and(
                    |outputs| {
                        outputs.iter().any(|value| value.as_str() == Some("merged_reads"))
                            && outputs
                                .iter()
                                .any(|value| value.as_str() == Some("unmerged_r1_reads"))
                            && outputs
                                .iter()
                                .any(|value| value.as_str() == Some("unmerged_r2_reads"))
                    },
                )
        }),
        "merge_pairs must expose merged and unmerged outputs in the CLI validation report"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.filter_reads")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("merged_reads"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("unmerged_r1_reads"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("unmerged_r2_reads"))
                    },
                )
        }),
        "filter_reads must consume the merged and unmerged handoff in the CLI validation report"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_fastq_edna_taxonomy_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/fastq-edna-taxonomy.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/fastq-edna-taxonomy.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/fastq-edna-taxonomy.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("fastq-edna-taxonomy")
    );
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-02-edna-mini")
    );

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.screen_taxonomy")
                && node
                    .get("external_inputs")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("taxonomy_database.root"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("taxonomy_expected_truth_table"))
                    })
                && node
                    .get("outputs")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|outputs| {
                        outputs
                            .iter()
                            .any(|value| value.as_str() == Some("taxonomy_classification"))
                            && outputs
                                .iter()
                                .any(|value| value.as_str() == Some("unclassified_reads"))
                    })
        }),
        "screen_taxonomy must expose governed taxonomy assets plus classification and unclassified outputs"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_fastq_amplicon_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/fastq-amplicon.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/fastq-amplicon.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/fastq-amplicon.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("fastq-amplicon")
    );
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-03-amplicon-mini")
    );

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.cluster_otus")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs
                            .iter()
                            .any(|value| value.as_str() == Some("normalized_amplicon_reads"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("non_chimeric_representatives"))
                    },
                )
        }),
        "cluster_otus must show normalized-read and non-chimeric representative handoffs"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.normalize_abundance")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| inputs.iter().any(|value| value.as_str() == Some("otu_table")),
                )
                && node.get("outputs").and_then(serde_json::Value::as_array).is_some_and(
                    |outputs| {
                        outputs
                            .iter()
                            .any(|value| value.as_str() == Some("normalized_abundance_table"))
                    },
                )
        }),
        "normalize_abundance must expose the OTU-to-abundance handoff"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_fastq_umi_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/fastq-umi.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/fastq-umi.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/fastq-umi.json")
    );
    assert_eq!(payload.get("pipeline_id").and_then(serde_json::Value::as_str), Some("fastq-umi"));
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-mini")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(13));

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.extract_umis")
                && node.get("outputs").and_then(serde_json::Value::as_array).is_some_and(
                    |outputs| {
                        outputs.iter().any(|value| value.as_str() == Some("umi_tagged_reads_r1"))
                            && outputs
                                .iter()
                                .any(|value| value.as_str() == Some("umi_tagged_reads_r2"))
                            && outputs
                                .iter()
                                .any(|value| value.as_str() == Some("umi_extraction_report"))
                    },
                )
        }),
        "extract_umis must expose UMI-tagged read outputs in the CLI validation report"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.remove_duplicates")
                && node
                    .get("upstream_inputs")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|inputs| {
                        inputs
                            .iter()
                            .any(|value| value.as_str() == Some("filtered_umi_reads_r1"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("filtered_umi_reads_r2"))
                    })
                && node
                    .get("outputs")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|outputs| {
                        outputs.iter().any(|value| {
                            value.as_str() == Some("deduplicated_umi_reads_r1")
                        }) && outputs.iter().any(|value| {
                            value.as_str() == Some("deduplicated_umi_reads_r2")
                        }) && outputs
                            .iter()
                            .any(|value| value.as_str() == Some("deduplication_report"))
                    })
        }),
        "remove_duplicates must show the duplicate-aware UMI read handoff in the CLI validation report"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_fastq_to_bam_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/fastq-to-bam.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/fastq-to-bam.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/fastq-to-bam.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("fastq-to-bam")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("cross"));
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-mini")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(7));

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.align")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("trimmed_reads_r1_path"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("trimmed_reads_r2_path"))
                    },
                )
                && node.get("external_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| {
                            value.as_str() == Some("alignment_reference_fasta_contract")
                        }) && inputs.iter().any(|value| {
                            value.as_str() == Some("alignment_reference_index_contract")
                        }) && inputs
                            .iter()
                            .any(|value| value.as_str() == Some("alignment_read_group_contract"))
                    },
                )
        }),
        "bam.align must consume trimmed FASTQ path outputs plus the governed alignment contracts"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.mapping_summary")
                && node.get("depends_on").and_then(serde_json::Value::as_array).is_some_and(
                    |deps| {
                        deps.iter().any(|value| value.as_str() == Some("bam.align"))
                            && deps.iter().any(|value| value.as_str() == Some("bam.qc_pre"))
                    },
                )
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("align_bam"))
                            && inputs.iter().any(|value| value.as_str() == Some("qc_pre_flagstat"))
                    },
                )
        }),
        "bam.mapping_summary must stay downstream of alignment and governed BAM pre-QC artifacts"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_core_germline_fastq_bam_vcf_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/core-germline-fastq-bam-vcf.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/core-germline-fastq-bam-vcf.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/core-germline-fastq-bam-vcf.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("core-germline-fastq-bam-vcf")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("cross"));
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-mini")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(15));

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call")
                && node.get("depends_on").and_then(serde_json::Value::as_array).is_some_and(
                    |deps| {
                        deps.iter().any(|value| value.as_str() == Some("bam.align"))
                            && deps.iter().any(|value| value.as_str() == Some("bam.coverage"))
                    },
                )
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("aligned_bam"))
                            && inputs.iter().any(|value| value.as_str() == Some("aligned_bai"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("coverage_report_json"))
                    },
                )
        }),
        "vcf.call must stay downstream of BAM alignment and governed coverage readiness"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.qc")
                && node.get("depends_on").and_then(serde_json::Value::as_array).is_some_and(
                    |deps| {
                        deps.iter().any(|value| value.as_str() == Some("vcf.filter"))
                            && deps.iter().any(|value| value.as_str() == Some("vcf.stats"))
                    },
                )
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("filtered_vcf"))
                            && inputs.iter().any(|value| value.as_str() == Some("filtered_vcf_tbi"))
                            && inputs.iter().any(|value| value.as_str() == Some("stats_json"))
                    },
                )
        }),
        "vcf.qc must consume the filtered VCF handoff plus explicit stats evidence"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_bam_core_qc_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/bam-core-qc.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/bam-core-qc.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/bam-core-qc.json")
    );
    assert_eq!(payload.get("pipeline_id").and_then(serde_json::Value::as_str), Some("bam-core-qc"));
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-bam-mini")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(5));

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.qc_pre")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| inputs.iter().any(|value| value.as_str() == Some("validation_report")),
                )
        }),
        "bam.qc_pre must consume the validation report in the CLI validation report"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.filter")
                && node.get("readiness_kind").and_then(serde_json::Value::as_str)
                    == Some("dry_or_smoke")
                && node.get("outputs").and_then(serde_json::Value::as_array).is_some_and(
                    |outputs| {
                        outputs.iter().any(|value| value.as_str() == Some("filtered_bam"))
                            && outputs.iter().any(|value| value.as_str() == Some("filtered_bai"))
                            && outputs
                                .iter()
                                .any(|value| value.as_str() == Some("filter_report_json"))
                    },
                )
        }),
        "bam.filter must expose the filtered BAM handoff in the CLI validation report"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.coverage")
                && node
                    .get("upstream_inputs")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|inputs| {
                        inputs
                            .iter()
                            .any(|value| value.as_str() == Some("mapping_summary_report_json"))
                            && inputs.iter().any(|value| value.as_str() == Some("filtered_bam"))
                            && inputs.iter().any(|value| value.as_str() == Some("filtered_bai"))
                    })
        }),
        "bam.coverage must show the mapping-summary and filtered-BAM handoff in the CLI validation report"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_bam_authenticity_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/bam-authenticity.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/bam-authenticity.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/bam-authenticity.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("bam-authenticity")
    );
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-adna-damage-mini")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(7));

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.sex")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("coverage_report_json"))
                    },
                )
        }),
        "bam.sex must remain visible as a coverage-driven branch in the CLI validation report"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.authenticity")
                && node
                    .get("upstream_inputs")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|inputs| {
                        inputs
                            .iter()
                            .any(|value| value.as_str() == Some("mapping_summary_report_json"))
                            && inputs.iter().any(|value| value.as_str() == Some("coverage_report_json"))
                            && inputs.iter().any(|value| value.as_str() == Some("damage_report_json"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("contamination_report_json"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("complexity_report_json"))
                            && !inputs.iter().any(|value| value.as_str() == Some("sex_report_json"))
                    })
        }),
        "bam.authenticity must expose only the required upstream evidence in the CLI validation report"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_adna_pseudohaploid_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/adna-pseudohaploid-fastq-bam-vcf.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/adna-pseudohaploid-fastq-bam-vcf.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/adna-pseudohaploid-fastq-bam-vcf.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("adna-pseudohaploid-fastq-bam-vcf")
    );
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-mini")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(24));

    let profiles = payload
        .get("validation_profiles")
        .and_then(serde_json::Value::as_array)
        .expect("validation profiles");
    assert!(
        profiles.iter().any(|profile| {
            profile.get("profile_id").and_then(serde_json::Value::as_str)
                == Some("ancient_dna_pseudohaploid")
                && profile.get("check_count").and_then(serde_json::Value::as_u64) == Some(8)
        }),
        "aDNA pseudohaploid pipeline must emit the ancient_dna_pseudohaploid validation profile"
    );

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.remove_duplicates")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| {
                            value.as_str() == Some("terminal_damage_trimmed_reads_r1_path")
                        }) && inputs.iter().any(|value| {
                            value.as_str() == Some("terminal_damage_trimmed_reads_r2_path")
                        })
                    },
                )
        }),
        "duplicate handling must stay downstream of terminal-damage trimming"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.call_pseudohaploid")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("damage_report_json"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("authenticity_report_json"))
                    },
                )
        }),
        "pseudohaploid calling must consume BAM damage and authenticity evidence"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.damage_filter")
                && node
                    .get("depends_on")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|deps| {
                        deps.iter().any(|value| value.as_str() == Some("vcf.call_pseudohaploid"))
                            && deps.iter().any(|value| value.as_str() == Some("bam.damage"))
                    })
        }),
        "damage-aware filtering must stay downstream of pseudohaploid calling and BAM damage evidence"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_adna_gl_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/adna-gl-fastq-bam-vcf.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/adna-gl-fastq-bam-vcf.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/adna-gl-fastq-bam-vcf.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("adna-gl-fastq-bam-vcf")
    );
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-mini")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(23));

    let profiles = payload
        .get("validation_profiles")
        .and_then(serde_json::Value::as_array)
        .expect("validation profiles");
    assert!(
        profiles.iter().any(|profile| {
            profile.get("profile_id").and_then(serde_json::Value::as_str) == Some("ancient_dna_gl")
                && profile.get("check_count").and_then(serde_json::Value::as_u64) == Some(8)
        }),
        "aDNA genotype-likelihood pipeline must emit the ancient_dna_gl validation profile"
    );

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call_gl")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("damage_report_json"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("authenticity_report_json"))
                    },
                )
        }),
        "genotype-likelihood calling must consume BAM damage and authenticity evidence"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.gl_propagation")
                && node
                    .get("depends_on")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|deps| {
                        deps.iter().any(|value| value.as_str() == Some("vcf.call_gl"))
                    })
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("gl_sites_vcf"))
                    },
                )
        }),
        "GL propagation must stay downstream of likelihood calling and consume the explicit GL VCF handoff"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.qc")
                && node
                    .get("depends_on")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|deps| {
                        deps.iter().any(|value| value.as_str() == Some("vcf.gl_propagation"))
                    })
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("gl_propagated_vcf"))
                            && inputs.iter().any(|value| {
                                value.as_str() == Some("gl_propagation_report_json")
                            })
                    },
                )
        }),
        "vcf.qc must stay downstream of propagated genotype-likelihood outputs rather than a hard-call branch"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_diploid_small_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/diploid-small-fastq-bam-vcf.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/diploid-small-fastq-bam-vcf.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/diploid-small-fastq-bam-vcf.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("diploid-small-fastq-bam-vcf")
    );
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-mini")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(16));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(24));

    let profiles = payload
        .get("validation_profiles")
        .and_then(serde_json::Value::as_array)
        .expect("validation profiles");
    assert!(
        profiles.iter().any(|profile| {
            profile.get("profile_id").and_then(serde_json::Value::as_str)
                == Some("diploid_small_sample")
                && profile.get("check_count").and_then(serde_json::Value::as_u64) == Some(8)
        }),
        "small-sample diploid pipeline must emit the diploid_small_sample validation profile"
    );

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.call_diploid")
                && node
                    .get("upstream_inputs")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("filtered_bam"))
                            && inputs.iter().any(|value| value.as_str() == Some("recalibrated_bam"))
                            && inputs.iter().any(|value| {
                                value.as_str() == Some("recalibration_summary_json")
                            })
                            && inputs.iter().any(|value| value.as_str() == Some("coverage_report_json"))
                    })
        }),
        "diploid calling must expose both the filtered-BAM fallback path and recalibrated-BAM run path"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.qc")
                && node
                    .get("depends_on")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|deps| {
                        deps.iter().any(|value| value.as_str() == Some("vcf.filter"))
                            && deps.iter().any(|value| value.as_str() == Some("vcf.stats"))
                            && !deps.iter().any(|value| value.as_str() == Some("vcf.phasing"))
                    })
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("filtered_vcf"))
                            && inputs.iter().any(|value| value.as_str() == Some("stats_json"))
                    },
                )
        }),
        "vcf.qc must remain independent of optional phasing and consume filtered VCF plus stats evidence"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.phasing")
                && node
                    .get("depends_on")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|deps| {
                        deps.iter().any(|value| value.as_str() == Some("vcf.filter"))
                            && deps.iter().any(|value| value.as_str() == Some("vcf.qc"))
                    })
                && node
                    .get("external_inputs")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("genetic_map_contract"))
                            && inputs.iter().any(|value| {
                                value.as_str() == Some("reference_panel_lock_contract")
                            })
                    })
        }),
        "optional phasing must stay downstream of completed QC with explicit map and panel contracts"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_reference_panel_imputation_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/reference-panel-imputation.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/reference-panel-imputation.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/reference-panel-imputation.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("reference-panel-imputation")
    );
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(7));

    let profiles = payload
        .get("validation_profiles")
        .and_then(serde_json::Value::as_array)
        .expect("validation profiles");
    assert!(
        profiles.iter().any(|profile| {
            profile.get("profile_id").and_then(serde_json::Value::as_str)
                == Some("reference_panel_imputation")
                && profile.get("check_count").and_then(serde_json::Value::as_u64) == Some(8)
        }),
        "reference-panel imputation pipeline must emit the reference_panel_imputation validation profile"
    );

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.prepare_reference_panel")
                && node.get("external_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs
                            .iter()
                            .any(|value| value.as_str() == Some("reference_panel_id_contract"))
                            && inputs.iter().any(|value| {
                                value.as_str() == Some("reference_panel_lock_contract")
                            })
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("genetic_map_contract"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("reference_fasta_contract"))
                    },
                )
                && node.get("outputs").and_then(serde_json::Value::as_array).is_some_and(
                    |outputs| {
                        outputs.iter().any(|value| value.as_str() == Some("prepared_panel_vcf"))
                            && outputs
                                .iter()
                                .any(|value| value.as_str() == Some("prepared_panel_panel_id"))
                    },
                )
        }),
        "panel preparation must keep panel identity plus map and reference contracts explicit"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.impute")
                && node
                    .get("depends_on")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|deps| {
                        deps.iter().any(|value| {
                            value.as_str() == Some("vcf.prepare_reference_panel")
                        }) && deps.iter().any(|value| value.as_str() == Some("vcf.qc"))
                            && deps.iter().any(|value| value.as_str() == Some("vcf.phasing"))
                    })
                && node
                    .get("external_inputs")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|inputs| {
                        inputs
                            .iter()
                            .any(|value| value.as_str() == Some("reference_panel_id_contract"))
                            && inputs.iter().any(|value| {
                                value.as_str() == Some("reference_panel_lock_contract")
                            })
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("genetic_map_contract"))
                    })
                && node
                    .get("upstream_inputs")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("qc_target_vcf"))
                            && inputs.iter().any(|value| value.as_str() == Some("phased_vcf"))
                            && inputs.iter().any(|value| {
                                value.as_str() == Some("phasing_requirement_decision_json")
                            })
                            && inputs.iter().any(|value| {
                                value.as_str() == Some("prepared_panel_panel_id")
                            })
                    })
        }),
        "imputation must keep both the qc-target fallback path and phased run path behind explicit panel identity"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.imputation")
                && node
                    .get("depends_on")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|deps| {
                        deps.iter().any(|value| {
                            value.as_str() == Some("vcf.prepare_reference_panel")
                        }) && deps.iter().any(|value| value.as_str() == Some("vcf.impute"))
                    })
                && node
                    .get("upstream_inputs")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|inputs| {
                        inputs.iter().any(|value| {
                            value.as_str() == Some("prepared_panel_panel_id")
                        }) && inputs.iter().any(|value| {
                            value.as_str() == Some("imputation_manifest_json")
                        }) && inputs.iter().any(|value| {
                            value.as_str() == Some("imputation_qc_json")
                        })
                    })
        }),
        "imputation metrics must stay downstream of imputation execution with explicit panel and qc evidence"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_popgen_structure_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/popgen-structure-vcf.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/popgen-structure-vcf.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/popgen-structure-vcf.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("popgen-structure-vcf")
    );
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(6));

    let profiles = payload
        .get("validation_profiles")
        .and_then(serde_json::Value::as_array)
        .expect("validation profiles");
    assert!(
        profiles.iter().any(|profile| {
            profile.get("profile_id").and_then(serde_json::Value::as_str)
                == Some("population_structure_vcf")
                && profile.get("check_count").and_then(serde_json::Value::as_u64) == Some(8)
        }),
        "population-structure pipeline must emit the population_structure_vcf validation profile"
    );

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca")
                && node.get("external_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| {
                            value.as_str() == Some("sample_metadata_manifest_contract")
                        }) && inputs.iter().any(|value| {
                            value.as_str() == Some("population_metadata_manifest_contract")
                        }) && inputs
                            .iter()
                            .any(|value| value.as_str() == Some("population_labels_contract"))
                    },
                )
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("qc_cohort_vcf"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("pruned_variants_tsv"))
                    },
                )
                && node.get("outputs").and_then(serde_json::Value::as_array).is_some_and(
                    |outputs| {
                        outputs.iter().any(|value| value.as_str() == Some("pca_report"))
                            && outputs
                                .iter()
                                .any(|value| value.as_str() == Some("pca_metadata_join_tsv"))
                    },
                )
        }),
        "pca must require metadata inputs and emit a metadata-join handoff"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.admixture")
                && node.get("depends_on").and_then(serde_json::Value::as_array).is_some_and(
                    |deps| {
                        deps.iter().any(|value| value.as_str() == Some("vcf.qc"))
                            && deps.iter().any(|value| value.as_str() == Some("vcf.pca"))
                    },
                )
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("pruned_variants_tsv"))
                            && inputs.iter().any(|value| value.as_str() == Some("pca_report"))
                    },
                )
                && node.get("outputs").and_then(serde_json::Value::as_array).is_some_and(
                    |outputs| {
                        outputs.iter().any(|value| value.as_str() == Some("admixture_report"))
                            && outputs
                                .iter()
                                .any(|value| value.as_str() == Some("admixture_metadata_join_tsv"))
                    },
                )
        }),
        "admixture must stay downstream of pca and emit a metadata-join handoff"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.population_structure")
                && node.get("external_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| {
                            value.as_str() == Some("sample_metadata_manifest_contract")
                        }) && inputs.iter().any(|value| {
                            value.as_str() == Some("population_metadata_manifest_contract")
                        }) && inputs
                            .iter()
                            .any(|value| value.as_str() == Some("population_labels_contract"))
                    },
                )
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("pca_report"))
                            && inputs.iter().any(|value| value.as_str() == Some("admixture_report"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("pca_metadata_join_tsv"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("admixture_metadata_join_tsv"))
                    },
                )
        }),
        "population-structure summary must consume pca and admixture metadata-join handoffs"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_bam_genotyping_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/bam-genotyping.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/bam-genotyping.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/bam-genotyping.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("bam-genotyping")
    );
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-bam-mini")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(5));

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.recalibration")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("filtered_bam"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("coverage_report_json"))
                    },
                )
                && node.get("outputs").and_then(serde_json::Value::as_array).is_some_and(
                    |outputs| {
                        outputs
                            .iter()
                            .any(|value| value.as_str() == Some("recalibrated_bam"))
                            && outputs.iter().any(|value| {
                                value.as_str() == Some("recalibration_summary_json")
                            })
                    },
                )
        }),
        "bam.recalibration must expose the coverage-gated summary and recalibrated BAM outputs in the CLI validation report"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.genotyping")
                && node.get("readiness_kind").and_then(serde_json::Value::as_str)
                    == Some("dry_or_smoke")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("filtered_bam"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("filtered_bai"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("recalibrated_bam"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("recalibrated_bai"))
                            && inputs.iter().any(|value| {
                                value.as_str() == Some("recalibration_summary_json")
                            })
                    },
                )
                && node
                    .get("external_inputs")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|inputs| {
                        inputs
                            .iter()
                            .any(|value| value.as_str() == Some("genotyping_reference_contract"))
                            && inputs.iter().any(|value| {
                                value.as_str() == Some("candidate_sites_vcf_contract")
                            })
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("target_regions_contract"))
                    })
        }),
        "bam.genotyping must expose the filtered fallback path, recalibrated run path, and governed genotyping contracts"
    );
}

#[test]
fn bench_local_pipeline_dag_validates_bam_kinship_contract() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "validate-pipeline-dag",
        "--config",
        "configs/pipelines/local/bam-kinship.toml",
        "--json",
    ]);

    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/bam-kinship.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/bam-kinship.json")
    );
    assert_eq!(payload.get("pipeline_id").and_then(serde_json::Value::as_str), Some("bam-kinship"));
    assert_eq!(
        payload.get("default_corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-bam-mini")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(4));

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.overlap_correction")
                && node.get("readiness_kind").and_then(serde_json::Value::as_str)
                    == Some("dry_or_smoke")
                && node.get("outputs").and_then(serde_json::Value::as_array).is_some_and(
                    |outputs| {
                        outputs.iter().any(|value| {
                            value.as_str() == Some("overlap_correction_summary_json")
                        }) && outputs
                            .iter()
                            .any(|value| value.as_str() == Some("overlap_corrected_bam"))
                    },
                )
        }),
        "bam.overlap_correction must expose the overlap-sufficiency summary and corrected BAM outputs in the CLI validation report"
    );
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.kinship")
                && node.get("readiness_kind").and_then(serde_json::Value::as_str)
                    == Some("smoke")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| {
                            value.as_str() == Some("overlap_corrected_bam")
                        }) && inputs
                            .iter()
                            .any(|value| value.as_str() == Some("overlap_corrected_bai"))
                            && inputs.iter().any(|value| {
                                value.as_str() == Some("overlap_correction_summary_json")
                            })
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("genotyping_report_json"))
                    },
                )
                && node
                    .get("external_inputs")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|inputs| {
                        inputs.iter().any(|value| {
                            value.as_str() == Some("kinship_reference_panel_contract")
                        }) && inputs.iter().any(|value| {
                            value.as_str() == Some("kinship_population_scope_contract")
                        }) && inputs
                            .iter()
                            .any(|value| value.as_str() == Some("kinship_pairing_contract"))
                    })
        }),
        "bam.kinship must expose governed overlap and genotype-readiness requirements in the CLI validation report"
    );
    assert!(
        nodes.iter().all(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.kinship")
                || !node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs
                            .iter()
                            .any(|value| value.as_str() == Some("overlap_correction_summary_json"))
                    },
                )
        }),
        "overlap insufficiency must stay local to bam.kinship in the CLI validation report"
    );
}
