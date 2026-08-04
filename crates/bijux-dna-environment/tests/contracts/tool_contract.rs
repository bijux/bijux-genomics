use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_environment::evidence::tool_contract::{
    apptainer_recipe_identity_at_commit, catalog_identity_at_commit,
    container_build_evidence_matches_recipe, load_container_catalog,
    prepare_apptainer_tool_contract, retained_apptainer_tool_ids, tool_contract_labels,
    validate_container_catalog, validate_tool_contract, validate_tool_contract_labels,
    verify_apptainer_tool_contract, ToolContractBuildRequest,
};

#[test]
fn historical_catalog_identity_rejects_untrusted_commit_names() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("environment crate must be below the repository root"));

    for commit in ["HEAD", "../../catalog", &"0".repeat(40), &"g".repeat(40)] {
        assert!(catalog_identity_at_commit(root, commit)
            .is_err_and(|error| error.to_string().contains("invalid repository commit identity")));
    }
}
use sha2::{Digest, Sha256};

fn repository_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
        .context("environment crate is outside the repository workspace")
}

fn prepare(
    tool_id: &str,
) -> Result<(
    tempfile::TempDir,
    bijux_dna_environment::evidence::tool_contract::PreparedToolContractBuild,
)> {
    let root = repository_root()?;
    let test_root = root.join("artifacts/tests/tool-contract");
    bijux_dna_infra::ensure_dir(&test_root)?;
    let output = bijux_dna_infra::temp_dir_in(&test_root, "build-inputs-")?;
    let prepared = prepare_apptainer_tool_contract(&ToolContractBuildRequest {
        repository_root: root,
        tool_id: tool_id.to_string(),
        repository_commit: "1234567890abcdef1234567890abcdef12345678".to_string(),
        architecture: "linux/amd64".to_string(),
        output_root: output.path().to_path_buf(),
    })?;
    Ok((output, prepared))
}

#[test]
fn retained_contract_is_catalog_derived_and_installed_once() -> Result<()> {
    let (_output, prepared) = prepare("samtools")?;
    validate_tool_contract(&prepared.contract)?;
    assert_eq!(prepared.contract.tool_id, "samtools");
    assert_eq!(prepared.contract.version, "1.20");
    assert_eq!(prepared.contract.support_disposition, "retained");
    assert_eq!(prepared.contract.recipe_path, "containers/apptainer/samtools.def");
    assert_eq!(prepared.contract.build_evidence_id.len(), 64);
    assert!(prepared.contract.source_sha256.is_some());
    assert!(prepared.contract.pinned_commit.is_none());

    let rendered = fs::read_to_string(&prepared.rendered_definition_path)?;
    assert_eq!(rendered.matches("/usr/share/bijux/tool-contract.json").count(), 1);
    for (key, value) in tool_contract_labels(&prepared.contract) {
        assert!(
            rendered.contains(&format!("    {key} {value}\n")),
            "rendered definition omits {key}"
        );
        assert!(!rendered.contains(&format!("{key} \"{value}\"")));
    }
    Ok(())
}

#[test]
fn every_retained_apptainer_recipe_renders_one_contract() -> Result<()> {
    let root = repository_root()?;
    let retained = retained_apptainer_tool_ids(&root, "linux/amd64")?;
    assert_eq!(retained.len(), 85);

    for tool_id in retained {
        let (_output, prepared) = prepare(&tool_id)?;
        let rendered = fs::read_to_string(prepared.rendered_definition_path)?;
        assert_eq!(
            rendered.matches("/usr/share/bijux/tool-contract.json").count(),
            1,
            "{tool_id} must install exactly one contract"
        );
        assert_eq!(prepared.contract.tool_id, tool_id);
    }
    Ok(())
}

#[test]
fn retained_roster_rejects_non_catalog_architecture() -> Result<()> {
    let Err(error) = retained_apptainer_tool_ids(&repository_root()?, "linux/arm64") else {
        panic!("non-catalog architecture must fail");
    };
    assert!(error.to_string().contains("differs from catalog architecture"));
    Ok(())
}

#[test]
fn canonical_catalog_loader_preserves_the_complete_contract_and_rejects_unknown_fields(
) -> Result<()> {
    let root = repository_root()?;
    let catalog = load_container_catalog(&root)?;
    assert_eq!(catalog.tools.len(), 89);
    assert_eq!(
        catalog.tools.iter().filter(|tool| tool.support_disposition == "retained").count(),
        85
    );
    let samtools = catalog
        .tools
        .iter()
        .find(|tool| tool.id == "samtools")
        .context("catalog omits samtools")?;
    assert_eq!(samtools.version, "1.20");
    assert_eq!(samtools.domains, ["bam"]);
    assert_eq!(samtools.apptainer_definition, "containers/apptainer/samtools.def");
    assert!(!samtools.version_command.is_empty());
    assert!(!samtools.help_command.is_empty());
    assert!(!samtools.expected_version_regex.is_empty());

    let malformed_root = bijux_dna_testkit::tempdir_for("container-catalog-unknown-field");
    let contract_root = malformed_root.path().join("containers/contracts");
    fs::create_dir_all(&contract_root)?;
    let mut source = fs::read_to_string(root.join("containers/contracts/catalog.toml"))?;
    source.push_str("\nunowned_contract_field = true\n");
    fs::write(contract_root.join("catalog.toml"), source)?;
    assert!(load_container_catalog(malformed_root.path()).is_err());
    Ok(())
}

#[test]
fn catalog_validation_owns_exact_roster_and_recipe_closure() -> Result<()> {
    let root = repository_root()?;
    let validation = validate_container_catalog(&root)?;
    assert!(validation.is_valid(), "{:?}", validation.errors);
    assert_eq!(validation.tool_count, 89);
    assert_eq!(validation.retained_count, 85);
    assert_eq!(validation.unsupported_count, 4);

    let malformed_root = bijux_dna_testkit::tempdir_for("container-catalog-roster-drift");
    let contract_root = malformed_root.path().join("containers/contracts");
    fs::create_dir_all(contract_root.join("sites"))?;
    fs::create_dir_all(malformed_root.path().join("containers/apptainer"))?;
    fs::create_dir_all(malformed_root.path().join("containers/docker"))?;
    let source = fs::read_to_string(root.join("containers/contracts/catalog.toml"))?.replacen(
        "support_disposition = \"retained\"",
        "support_disposition = \"unsupported\"",
        1,
    );
    fs::write(contract_root.join("catalog.toml"), source)?;
    let invalid = validate_container_catalog(malformed_root.path())?;
    assert!(!invalid.is_valid());
    assert!(invalid.errors.iter().any(|error| error.contains("catalog roster must contain")));
    Ok(())
}

#[test]
fn contract_admission_rejects_identity_and_label_tampering() -> Result<()> {
    let (_output, prepared) = prepare("samtools")?;
    let mut malformed = serde_json::to_value(&prepared.contract)?;
    malformed
        .as_object_mut()
        .context("serialized contract is not an object")?
        .insert("unexpected".to_string(), serde_json::json!(true));
    assert!(serde_json::from_value::<
        bijux_dna_environment::evidence::tool_contract::ContainerToolContract,
    >(malformed)
    .is_err());

    let mut stale = prepared.contract.clone();
    stale.catalog_sha256 = "0".repeat(64);
    assert!(
        validate_tool_contract(&stale).is_err_and(|error| error.to_string().contains("catalog"))
    );

    let mut unsafe_label = prepared.contract.clone();
    unsafe_label.version_command.push('\n');
    assert!(validate_tool_contract(&unsafe_label)
        .is_err_and(|error| error.to_string().contains("not safe for an Apptainer label")));

    let mut ambiguous = prepared.contract.clone();
    ambiguous.pinned_commit = Some("1234567890abcdef1234567890abcdef12345678".to_string());
    assert!(validate_tool_contract(&ambiguous)
        .is_err_and(|error| error.to_string().contains("only one source identity")));

    let mut labels = tool_contract_labels(&prepared.contract);
    labels.insert("io.bijux.tool-contract.version".to_string(), "stale".to_string());
    assert!(validate_tool_contract_labels(&prepared.contract, &labels)
        .is_err_and(|error| error.to_string().contains("mismatches embedded contract")));

    labels = tool_contract_labels(&prepared.contract);
    labels.insert("io.bijux.tool-contract.unowned".to_string(), "value".to_string());
    assert!(validate_tool_contract_labels(&prepared.contract, &labels)
        .is_err_and(|error| error.to_string().contains("unexpected tool-contract label")));
    Ok(())
}

#[test]
fn committed_recipe_identity_owns_definition_and_files_context() -> Result<()> {
    let root = repository_root()?;
    let head = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", "HEAD"])
        .output()?;
    let commit = String::from_utf8(head.stdout)?.trim().to_string();
    let identity =
        apptainer_recipe_identity_at_commit(&root, &commit, "containers/apptainer/ibdhap.def")?;
    assert_eq!(identity.repository_commit, commit);
    assert_eq!(identity.definition_sha256.len(), 64);
    assert_eq!(identity.build_context.len(), 2);
    assert_eq!(identity.build_context[1].path, "containers/tools/ibdhap/ibdhap");

    let evidence = bijux_dna_environment::evidence::ContainerBuildEvidence {
        status: bijux_dna_environment::evidence::BuildEvidenceStatus::Passed,
        definition_sha256: identity.definition_sha256.clone(),
        repository_commit: identity.repository_commit.clone(),
        build_context: Some(identity.build_context.clone()),
    };
    assert!(container_build_evidence_matches_recipe(&evidence, &identity));
    let mut stale = evidence;
    stale.build_context.as_mut().context("build context fixture")?[0].git_object = "f".repeat(40);
    assert!(!container_build_evidence_matches_recipe(&stale, &identity));
    Ok(())
}

#[cfg(unix)]
#[test]
fn runtime_verification_binds_contract_labels_and_image_digest() -> Result<()> {
    use std::os::unix::fs::PermissionsExt as _;

    let (output, prepared) = prepare("samtools")?;
    let image = output.path().join("samtools.sif");
    fs::write(&image, b"immutable-sif-fixture")?;
    let image_sha256 = hex_digest(&Sha256::digest(fs::read(&image)?));
    let labels_path = output.path().join("labels.json");
    let labels = tool_contract_labels(&prepared.contract).into_iter().collect::<BTreeMap<_, _>>();
    bijux_dna_infra::atomic_write_json(
        &labels_path,
        &serde_json::json!({"data": {"attributes": {"labels": labels}}}),
    )?;
    let fake_apptainer = output.path().join("apptainer");
    let script = format!(
        "#!/bin/sh\ncase \"$1\" in\n  exec) cat '{}' ;;\n  inspect) cat '{}' ;;\n  *) exit 64 ;;\nesac\n",
        prepared.contract_path.display(),
        labels_path.display()
    );
    fs::write(&fake_apptainer, script)?;
    fs::set_permissions(&fake_apptainer, fs::Permissions::from_mode(0o755))?;

    let verified =
        verify_apptainer_tool_contract(&fake_apptainer, &image, &image_sha256, &prepared.contract)?;
    assert_eq!(verified.contract, prepared.contract);
    assert_eq!(verified.image_sha256, image_sha256);

    assert!(verify_apptainer_tool_contract(
        &fake_apptainer,
        &image,
        &"f".repeat(64),
        &prepared.contract,
    )
    .is_err_and(|error| error.to_string().contains("image digest mismatch")));

    let mut tampered = prepared.contract.clone();
    tampered.version = "9.9.9".to_string();
    bijux_dna_infra::atomic_write_json(&prepared.contract_path, &tampered)?;
    assert!(verify_apptainer_tool_contract(
        &fake_apptainer,
        &image,
        &image_sha256,
        &prepared.contract,
    )
    .is_err_and(|error| error.to_string().contains("differs from expected")));
    Ok(())
}

fn hex_digest(bytes: &[u8]) -> String {
    bytes.iter().fold(String::with_capacity(bytes.len() * 2), |mut output, byte| {
        let _ = write!(output, "{byte:02x}");
        output
    })
}
