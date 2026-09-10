use anyhow::Result;
use bijux_dna_bench_model::{
    assemble_hpc_code_submission, assemble_hpc_results_submission, encode_redacted_hpc_bundle,
    HpcBundleFoundationLocks, HpcBundleSubmission, HpcCodeFreezeObservation,
    HpcCodeSubmissionBundle, HpcResultsSubmissionBundle,
};
use serde_json::{json, Value};

use crate::hpc_campaign_fixture::campaign;

#[test]
fn hpc_submission_bundles_preserve_identity_settings_and_pending_execution() -> Result<()> {
    let campaign = campaign();
    let job = &campaign.planned_jobs[0];
    let (results, code) = payloads()?;
    assert_eq!(results["schema_version"], "bijux.hpc.results_bundle.v1");
    assert_eq!(results["metrics"], json!({"status":"pending_execution","submission_mode":"slurm"}));
    assert_eq!(
        results["job"],
        json!({"planned_job_id":"planned-1","scheduler_job_id":"42","submitted_at":"1700000000","job_name":"validate-one","stage":"fastq.validate_reads","tool":"checker","sample":"one","resource_template":"small","resources":job.resources,"depends_on":["prepare"]})
    );
    assert_eq!(
        results["campaign"],
        json!({"id":"mini","domain":"fastq","config_path":"campaign.toml","env_file_path":"private.env","user_policy_path":"user.toml"})
    );
    assert_eq!(
        results["artifacts"],
        json!({"inventory":[{"kind":"log","path":"logs/one.log"},{"kind":"out","path":"logs/one.out"},{"kind":"err","path":"logs/one.err"}],"encrypted_targets":[{"kind":"results","path":"results/one.bundle"},{"kind":"code","path":"code/one.bundle"}]})
    );
    assert_eq!(results["inventories"]["recipient_fingerprints"], json!(["2bd806c97f0e00af"]));
    assert_eq!(results["traces"], json!({"script_path":"logs/one.sbatch","scheduler_job_id":"42"}));
    assert_eq!(results["appraiser_outputs"][0]["status"], "pending");
    assert_eq!(code["schema_version"], "bijux.hpc.code_bundle.v1");
    assert_eq!(
        code["code_freeze"]["effective_settings"],
        json!({"slurm":campaign.resolved_slurm,"resources":job.resources})
    );
    assert_eq!(
        code["code_freeze"]["repository_state"],
        json!({"git_head":null,"git_branch":"main","git_status_porcelain":" M owned.rs"})
    );
    assert_eq!(code["code_freeze"]["dvc_state"]["available"], false);
    assert_eq!(code["plan"], json!({"depends_on":job.depends_on,"outputs":job.outputs}));
    assert_eq!(
        code["locks"],
        json!({"corpus_lock":"corpora/prepare.lock","database_lock":"databases/prepare.lock","image_lock":"images/prepare.lock","tool_lock":"deferred_to_runtime_registry_capture"})
    );
    Ok(())
}

#[test]
fn hpc_submission_bundle_wire_rejects_unknown_fields_and_versions() -> Result<()> {
    let (results, code) = payloads()?;
    let encoded = encode_redacted_hpc_bundle(code.clone(), &["quoted\"secret".into()])?;
    let redacted: Value = serde_json::from_slice(&encoded)?;
    assert_eq!(redacted["code_freeze"]["slurm_script"], "run <redacted>\n");
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<HpcResultsSubmissionBundle>(
            results.clone()
        )?)?,
        results
    );
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<HpcCodeSubmissionBundle>(code.clone())?)?,
        code
    );
    for pointer in ["/schema_version", "/job/unknown", "/job/resources/unknown"] {
        let mut invalid = results.clone();
        if pointer == "/schema_version" {
            invalid["schema_version"] = json!("bijux.hpc.results_bundle.v9");
        } else if pointer == "/job/unknown" {
            invalid["job"]["unknown"] = json!(true);
        } else {
            invalid["job"]["resources"]["unknown"] = json!(true);
        }
        assert!(serde_json::from_value::<HpcResultsSubmissionBundle>(invalid).is_err());
    }
    let mut invalid = code.clone();
    invalid["code_freeze"]["repository_state"]["unknown"] = json!(true);
    assert!(serde_json::from_value::<HpcCodeSubmissionBundle>(invalid).is_err());
    let mut invalid = code;
    invalid["schema_version"] = json!("bijux.hpc.code_bundle.v9");
    assert!(serde_json::from_value::<HpcCodeSubmissionBundle>(invalid).is_err());
    Ok(())
}

fn payloads() -> Result<(Value, Value)> {
    let campaign = campaign();
    let job = &campaign.planned_jobs[0];
    let input = HpcBundleSubmission {
        campaign: &campaign,
        job,
        scheduler_job_id: "42",
        submitted_at: "1700000000",
        script_path: "logs/one.sbatch",
    };
    let results = serde_json::to_value(assemble_hpc_results_submission(&input))?;
    let observation = HpcCodeFreezeObservation {
        slurm_script: "run quoted\"secret\n".into(),
        git_head: None,
        git_branch: Some("main".into()),
        git_status_porcelain: Some(" M owned.rs".into()),
        dvc_available: false,
    };
    let locks = HpcBundleFoundationLocks {
        corpus_lock: "corpora/prepare.lock".into(),
        database_lock: "databases/prepare.lock".into(),
        image_lock: "images/prepare.lock".into(),
    };
    let code = serde_json::to_value(assemble_hpc_code_submission(&input, observation, locks))?;
    Ok((results, code))
}
