#![cfg(test)]

//! Collected current-result reuse must retain the same complete payload authority as publication admission.
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use super::{
    artifact, execute_or_resume_tool, write_raw_manifest, ContainerToolContract,
    SecurityProbePublication, SecurityProbeStatus, SecurityRuntimeProbe, SecurityRuntimeReport,
    SecurityStructuralReport, SecurityVerificationRequest, ToolSecurityProbe, ToolTask,
    SECURITY_PROBE_PUBLICATION_SCHEMA_V1,
};
use anyhow::{ensure, Context as _, Result};
use bijux_dna_containers::api::{
    admit_security_probe_publication, ContainerArtifactDelivery, ContainerProductSupport,
    SecurityProbeAuthority,
};

#[test]
fn current_collected_resume_requires_every_declared_payload() -> Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("security-current-result-admission");
    let root = temporary.path();
    let request = request(root);
    let tool_root = request.evidence_root.join("tools/samtools");
    std::fs::create_dir_all(&tool_root)?;
    let image = root.join("samtools.sif");
    std::fs::write(&image, b"private image identity fixture")?;
    let task = task(&image)?;
    let complete = complete_record(&request, &task, &tool_root)?;
    let authority = SecurityProbeAuthority {
        repository_commit: task.contract.repository_commit.clone(),
        catalog_sha256: "2".repeat(64),
        inventory_sha256: "9".repeat(64),
        scanner_completion_sha256: "c".repeat(64),
        database_completion_sha256: "d".repeat(64),
    };
    let publication = SecurityProbePublication {
        schema_version: SECURITY_PROBE_PUBLICATION_SCHEMA_V1.to_string(),
        repository_commit: authority.repository_commit.clone(),
        catalog_sha256: authority.catalog_sha256.clone(),
        inventory_sha256: authority.inventory_sha256.clone(),
        scanner_completion_sha256: authority.scanner_completion_sha256.clone(),
        database_completion_sha256: authority.database_completion_sha256.clone(),
        reused_publication_sha256: None,
        tools: BTreeMap::from([(task.tool_id.clone(), complete.clone())]),
    };
    let publication_path = root.join("complete-publication.json");
    std::fs::write(&publication_path, serde_json::to_vec(&publication)?)?;
    admit_security_probe_publication(&publication_path, root, &authority)?;
    let result_path = tool_root.join("result.json");
    let original = serde_json::to_vec_pretty(&complete)?;
    std::fs::write(&result_path, &original)?;
    let permissions = std::fs::metadata(&result_path)?.permissions();
    let (current, reused) = execute_or_resume_tool(&request, &task, None)?;
    ensure!(current == complete && !reused, "actual complete current resume control");
    let mut admitted = Vec::new();
    for field in [
        "sbom",
        "vulnerability_report",
        "secret_report",
        "signature_stdout",
        "signature_stderr",
        "structural_report",
        "runtime_report",
    ] {
        let mut value = serde_json::to_value(&complete)?;
        *value.get_mut(field).context("declared payload field exists")? = serde_json::Value::Null;
        let changed: ToolSecurityProbe = serde_json::from_value(value)?;
        let mut candidate = publication.clone();
        candidate.tools.insert(task.tool_id.clone(), changed.clone());
        let path = root.join(format!("missing-{field}-publication.json"));
        std::fs::write(&path, serde_json::to_vec(&candidate)?)?;
        let publication_error = admit_security_probe_publication(&path, root, &authority)
            .err()
            .context("real publication owner must refuse incomplete Collected state")?;
        ensure!(
            format!("{publication_error:#}").contains("collected security probe is incomplete"),
            "{publication_error:#}"
        );
        std::fs::write(&result_path, serde_json::to_vec(&changed)?)?;
        let resumed = execute_or_resume_tool(&request, &task, None);
        if resumed.as_ref().is_ok_and(|(row, _)| row.status == SecurityProbeStatus::Collected) {
            admitted.push(field);
        }
        eprintln!("SECURITY_CURRENT_RESULT_OBSERVED: missing={field}; actual_current={resumed:?}");
        std::fs::write(&result_path, &original)?;
    }
    ensure!(
        std::fs::read(&result_path)? == original
            && std::fs::metadata(&result_path)?.permissions() == permissions,
        "current result bytes and mode restored before causal comparison"
    );
    let (restored, reused) = execute_or_resume_tool(&request, &task, None)?;
    ensure!(restored == complete && !reused, "restored complete current control");
    admit_security_probe_publication(&publication_path, root, &authority)?;
    eprintln!("SECURITY_CURRENT_RESULT_CONTROLS_PASS: actual public complete admission, actual current reuse, seven public shape refusals and restored current; admitted={admitted:?}");
    ensure!(
        admitted.is_empty(),
        "current resume bypassed required Collected payload membership: {admitted:?}"
    );
    Ok(())
}

fn complete_record(
    request: &SecurityVerificationRequest,
    task: &ToolTask,
    tool_root: &Path,
) -> Result<ToolSecurityProbe> {
    for (name, bytes) in [
        ("sbom.json", &b"{}"[..]),
        ("vulnerabilities.json", &b"{}"[..]),
        ("secrets.json", &b"{}"[..]),
        ("signature.stdout", &b"fixture observation"[..]),
        ("signature.stderr", &b""[..]),
    ] {
        std::fs::write(tool_root.join(name), bytes)?;
    }
    let structural = SecurityStructuralReport {
        suid_sgid_paths: Vec::new(),
        world_writable_paths: Vec::new(),
        suid_sgid_observations: Vec::new(),
        world_writable_observations: Vec::new(),
    };
    std::fs::write(tool_root.join("structural.json"), serde_json::to_vec(&structural)?)?;
    let probe = || -> Result<SecurityRuntimeProbe> {
        Ok(SecurityRuntimeProbe {
            exit_code: 0,
            stdout: artifact(request, &tool_root.join("signature.stdout"))?,
            stderr: artifact(request, &tool_root.join("signature.stderr"))?,
        })
    };
    let runtime = SecurityRuntimeReport {
        network_isolated: probe()?,
        non_root_uid: probe()?,
        system_read_only: probe()?,
    };
    std::fs::write(tool_root.join("runtime.json"), serde_json::to_vec(&runtime)?)?;
    Ok(ToolSecurityProbe {
        tool_id: task.tool_id.clone(),
        version: task.version.clone(),
        image_sha256: task.image_sha256.clone(),
        tool_contract_sha256: bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(
            &task.contract,
        )?),
        build_evidence_id: task.contract.build_evidence_id.clone(),
        status: SecurityProbeStatus::Collected,
        sbom: Some(artifact(request, &tool_root.join("sbom.json"))?),
        vulnerability_report: Some(artifact(request, &tool_root.join("vulnerabilities.json"))?),
        secret_report: Some(artifact(request, &tool_root.join("secrets.json"))?),
        signature_stdout: Some(artifact(request, &tool_root.join("signature.stdout"))?),
        signature_stderr: Some(artifact(request, &tool_root.join("signature.stderr"))?),
        signature_verified: false,
        structural_report: Some(artifact(request, &tool_root.join("structural.json"))?),
        runtime_report: Some(artifact(request, &tool_root.join("runtime.json"))?),
        raw_manifest: write_raw_manifest(request, tool_root)?,
        error: None,
    })
}

fn request(root: &Path) -> SecurityVerificationRequest {
    SecurityVerificationRequest {
        repository_root: root.to_path_buf(),
        study_root: root.to_path_buf(),
        inventory_path: root.join("inventory.json"),
        apptainer_executable: root.join("unavailable-provider"),
        scanner_contract_path: root.join("scanner.toml"),
        scanner_root: root.join("scanner"),
        database_contract_path: root.join("database.toml"),
        database_root: root.join("database"),
        reused_publication_path: None,
        evidence_root: root.join("evidence"),
        worker_count: 1,
        timeout: Duration::from_secs(1),
    }
}

fn task(image: &Path) -> Result<ToolTask> {
    let contract = ContainerToolContract {
        schema_version: "bijux.container.tool_contract.v3".to_string(),
        tool_id: "samtools".to_string(),
        version: "1.20".to_string(),
        product_support: ContainerProductSupport::Supported,
        artifact_delivery: ContainerArtifactDelivery::RedistributableImage,
        redistribution_permitted: true,
        catalog_entry_sha256: "1".repeat(64),
        catalog_sha256: "2".repeat(64),
        recipe_path: "containers/tools/samtools/samtools".to_string(),
        recipe_sha256: "3".repeat(64),
        source_sha256: None,
        pinned_commit: None,
        repository_commit: "4".repeat(40),
        architecture: "linux/amd64".to_string(),
        build_evidence_id: "5".repeat(64),
        expected_binary: "samtools".to_string(),
        version_command: "samtools --version".to_string(),
        expected_version_regex: "1\\.20".to_string(),
        interface_sha256: "6".repeat(64),
    };
    Ok(ToolTask {
        tool_id: contract.tool_id.clone(),
        version: contract.version.clone(),
        image_path: image.to_path_buf(),
        image_sha256: bijux_dna_infra::hash_file_sha256(image)?,
        contract,
    })
}
