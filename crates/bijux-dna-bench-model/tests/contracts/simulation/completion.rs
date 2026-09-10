use super::*;
use crate::contract::{admit_all_domain_completion_check, validate_all_domain_completion_check};
use anyhow::Result;

type CompletionCase<'a> = (&'a str, &'a str, &'a str, &'a str, &'a str, &'a str, &'a str);

const CASES: [CompletionCase<'static>; 5] = [
    (
        MUTATION_MISSING_DECLARED_OUTPUT,
        "fastq",
        "corpus-01-mini",
        "fastq.profile_reads",
        "corpus_only",
        "seqkit_stats",
        REASON_MISSING_DECLARED_OUTPUTS,
    ),
    (
        MUTATION_MISSING_NORMALIZED_METRICS,
        "fastq",
        "corpus-02-edna-mini",
        "fastq.screen_taxonomy",
        "database_artifact_id+taxonomy_database_root",
        "kraken2",
        REASON_MISSING_NORMALIZED_METRICS,
    ),
    (
        MUTATION_MISSING_MANIFEST,
        "vcf",
        "vcf_production_regression",
        "vcf.call",
        "bam_bundle",
        "bcftools",
        REASON_MISSING_MANIFEST,
    ),
    (
        MUTATION_REQUIRED_FILE_EMPTY,
        "bam",
        "corpus-01-bam-mini",
        "bam.coverage",
        "corpus_only",
        "samtools",
        REASON_REQUIRED_FILES_INCOMPLETE,
    ),
    (
        MUTATION_EXECUTION_NOT_SUCCESSFUL,
        "bam",
        "corpus-01-bam-mini",
        "bam.qc_pre",
        "corpus_only",
        "multiqc",
        REASON_EXECUTION_NOT_SUCCESSFUL,
    ),
];

fn declared_completion_case(
    root: &str,
    (mutation, domain, corpus, stage, profile, tool, reason): CompletionCase<'_>,
) -> (AllDomainCompletionCheckRow, AllDomainCompletionSeededMutation) {
    let run = format!("{root}/{domain}/{corpus}/{stage}/{profile}/{tool}");
    let scope = if domain == "vcf" { profile } else { "sample-set" };
    let result_id = format!("{domain}:{corpus}:{stage}:{scope}:{tool}");
    let script = format!("{run}/command.sh");
    let stdout = format!("{run}/stdout.log");
    let stderr = format!("{run}/stderr.log");
    let metrics = format!("{run}/metrics.json");
    let manifest = format!("{run}/stage-result.json");
    let manifest_exists = mutation != MUTATION_MISSING_MANIFEST;
    let failed = mutation == MUTATION_EXECUTION_NOT_SUCCESSFUL;
    let empty_script = mutation == MUTATION_REQUIRED_FILE_EMPTY;
    let required_files = [
        ("command_script", &script),
        ("stdout", &stdout),
        ("stderr", &stderr),
        ("metrics", &metrics),
    ]
    .into_iter()
    .map(|(id, path)| AllDomainCompletionRequiredFile {
        file_id: id.to_string(),
        path: path.clone(),
        exists: true,
        non_empty: !(id == "command_script" && empty_script),
    })
    .collect();
    let output_missing = mutation == MUTATION_MISSING_DECLARED_OUTPUT;
    let metric_missing = mutation == MUTATION_MISSING_NORMALIZED_METRICS;
    let row = AllDomainCompletionCheckRow {
        result_id: result_id.clone(),
        domain: domain.to_string(),
        stage_id: stage.to_string(),
        tool_id: tool.to_string(),
        corpus_id: corpus.to_string(),
        asset_profile_id: profile.to_string(),
        command_script_path: script.clone(),
        stdout_path: stdout,
        stderr_path: stderr,
        metrics_path: metrics,
        stage_result_path: manifest.clone(),
        manifest_exists,
        manifest_valid: manifest_exists,
        manifest_output_match: manifest_exists,
        exit_code_zero: manifest_exists && !failed,
        runtime_status: manifest_exists
            .then(|| if failed { "failed" } else { "succeeded" }.to_string()),
        exit_code: manifest_exists.then_some(if failed { 23 } else { 0 }),
        declared_output_count: 1,
        present_declared_output_count: usize::from(!output_missing),
        missing_declared_output_ids: if output_missing {
            vec!["qc_tsv".to_string()]
        } else {
            Vec::new()
        },
        normalized_metrics_count: 1,
        present_normalized_metrics_count: usize::from(!metric_missing),
        missing_normalized_metric_ids: if metric_missing {
            vec!["classification_report_json".to_string()]
        } else {
            Vec::new()
        },
        required_file_count: 4,
        ready_required_file_count: if empty_script { 3 } else { 4 },
        required_files,
        completion_status: AllDomainCompletionStatus::Incomplete,
        failure_reasons: vec![reason.to_string()],
    };
    let evidence_path = if mutation == MUTATION_MISSING_MANIFEST || failed {
        manifest
    } else if empty_script {
        script
    } else {
        format!("{run}/removed-artifact")
    };
    let seeded_mutation = AllDomainCompletionSeededMutation {
        mutation_id: mutation.to_string(),
        result_id,
        evidence_path,
    };
    (row, seeded_mutation)
}

fn declared_completion_report() -> AllDomainCompletionCheckReport {
    let root =
        "artifacts/benchmark-smoke/simulated-runs/readiness-probes/all-domains/completion-check";

    let mut rows = Vec::new();
    let mut mutations = Vec::new();
    let mut domains = BTreeMap::new();
    let mut reasons = BTreeMap::new();
    for case in CASES {
        let (row, mutation) = declared_completion_case(root, case);
        *domains.entry(row.domain.clone()).or_insert(0) += 1;
        for reason in &row.failure_reasons {
            *reasons.entry(reason.clone()).or_insert(0) += 1;
        }
        rows.push(row);
        mutations.push(mutation);
    }
    AllDomainCompletionCheckReport {
        schema_version: ALL_DOMAIN_COMPLETION_CHECK_SCHEMA_VERSION.to_string(),
        output_path: DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_PATH.to_string(),
        fixture_root: root.to_string(),
        row_count: rows.len(),
        complete_row_count: 0,
        incomplete_row_count: rows.len(),
        domain_counts: domains,
        failure_reason_counts: reasons,
        passes_behavior_test: false,
        seeded_mutations: mutations,
        rows,
    }
}

#[test]
fn completion_replay_preserves_fields_and_rejects_unknown_or_unsupported_wire() -> Result<()> {
    let report = admit_all_domain_completion_check(declared_completion_report())?;
    let value = serde_json::to_value(&report)?;
    let replay: AllDomainCompletionCheckReport = serde_json::from_value(value.clone())?;
    validate_all_domain_completion_check(&replay)?;
    assert_eq!(serde_json::to_value(replay)?, value);
    for (path, replacement) in [
        ("/schema_version", serde_json::json!("future")),
        ("/rows/0/completion_status", serde_json::json!("unknown")),
        ("/rows/0/declared_output_count", serde_json::json!(-1)),
    ] {
        let mut invalid = value.clone();
        *invalid
            .pointer_mut(path)
            .ok_or_else(|| anyhow::anyhow!("missing fixture field {path}"))? = replacement;
        assert!(serde_json::from_value::<AllDomainCompletionCheckReport>(invalid).is_err());
    }
    for path in ["", "/rows/0", "/rows/0/required_files/0", "/seeded_mutations/0"] {
        let mut invalid = value.clone();
        invalid
            .pointer_mut(path)
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(|| anyhow::anyhow!("missing fixture object {path}"))?
            .insert("unrecognized".to_string(), serde_json::json!(true));
        assert!(serde_json::from_value::<AllDomainCompletionCheckReport>(invalid).is_err());
    }
    Ok(())
}

#[test]
fn completion_admission_rejects_forged_verdicts_counters_identity_and_evidence() -> Result<()> {
    let report = admit_all_domain_completion_check(declared_completion_report())?;
    let corruptions: &[fn(&mut AllDomainCompletionCheckReport)] = &[
        |r| r.row_count = usize::MAX,
        |r| r.complete_row_count = 1,
        |r| r.domain_counts.clear(),
        |r| r.failure_reason_counts.clear(),
        |r| r.rows[0].result_id = r.rows[1].result_id.clone(),
        |r| r.rows[0].stage_id = "bam.coverage".to_string(),
        |r| r.rows[2].asset_profile_id = "foreign".to_string(),
        |r| {
            r.rows[0].result_id =
                "fastq:corpus-01-mini:fastq.profile_reads::seqkit_stats".to_string();
        },
        |r| r.rows[0].completion_status = AllDomainCompletionStatus::Complete,
        |r| r.rows[0].failure_reasons.clear(),
        |r| r.rows[0].required_files[0].exists = false,
        |r| r.rows[0].ready_required_file_count = 0,
        |r| r.rows[0].required_files[0].path = r.rows[0].stdout_path.clone(),
        |r| r.rows[0].present_declared_output_count = 2,
        |r| r.rows[0].missing_declared_output_ids.push("qc_tsv".to_string()),
        |r| r.rows[1].missing_normalized_metric_ids.clear(),
        |r| r.rows[2].manifest_valid = true,
        |r| r.rows[2].exit_code_zero = true,
        |r| r.rows[4].runtime_status = Some("succeeded".to_string()),
        |r| r.rows[4].runtime_status = Some("unknown".to_string()),
        |r| r.rows[4].exit_code = None,
        |r| r.rows[0].stage_result_path = "artifacts/outside/stage-result.json".to_string(),
        |r| r.rows[0].command_script_path.push_str("/../outside"),
        |r| r.seeded_mutations[0].mutation_id = r.seeded_mutations[1].mutation_id.clone(),
        |r| r.seeded_mutations[0].result_id = r.rows[1].result_id.clone(),
        |r| r.seeded_mutations[0].evidence_path = r.seeded_mutations[1].evidence_path.clone(),
        |r| r.seeded_mutations[2].evidence_path = r.rows[2].stdout_path.clone(),
    ];
    for (index, corrupt) in corruptions.iter().enumerate() {
        let mut invalid = report.clone();
        corrupt(&mut invalid);
        assert!(
            admit_all_domain_completion_check(invalid.clone()).is_err(),
            "accepted corruption {index}"
        );
        assert!(
            validate_all_domain_completion_check(&invalid).is_err(),
            "accepted replay corruption {index}"
        );
    }
    let mut not_passed = report;
    not_passed.passes_behavior_test = false;
    assert!(validate_all_domain_completion_check(&not_passed).is_err());
    Ok(())
}
