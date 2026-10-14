#![cfg(test)]
//! Admission fixtures bind real native files; they do not claim execution of invented cases.
use super::*;
use serde_json::json;

fn producer_identity() -> Result<CrateGateProducerIdentity> {
    CrateGateProducerIdentity::new("0123456789abcdef0123456789abcdef01234567", &"a".repeat(64))
}

struct Fixture {
    _scratch: tempfile::TempDir,
    producer: PathBuf,
    consumer: PathBuf,
    target: PathBuf,
    inventory_path: PathBuf,
    inventory: Value,
}

fn fixture() -> Result<Fixture> {
    let scratch = tempfile::tempdir()?;
    let root = scratch.path().canonicalize()?;
    let producer = root.join("producer");
    let consumer = root.join("consumer");
    let target = root.join("artifacts/compiled");
    for source in [&producer, &consumer] {
        for manifest in [
            "Cargo.toml",
            "crates/bijux-dna-api/Cargo.toml",
            "crates/bijux-dna-domain-vcf/Cargo.toml",
        ] {
            let path = source.join(manifest);
            std::fs::create_dir_all(path.parent().context("fixture parent")?)?;
            std::fs::write(path, "[workspace]\n")?;
        }
    }
    std::fs::create_dir_all(&target)?;
    let mut suites = serde_json::Map::new();
    let native = std::env::current_exe()?;
    for proof in [Proof::TypedArtifactHandoff, Proof::VcfMetricRegistry] {
        let executable = target.join(proof.binary());
        std::fs::hard_link(&native, &executable)?;
        suites.insert(proof.suite().to_string(), json!({
            "package-name":proof.package(), "binary-id":proof.suite(), "binary-name":proof.binary(),
            "kind":proof.kind(), "binary-path":executable,
            "cwd":producer.join("crates").join(proof.package()), "status":"listed",
            "package-id":format!("path+file://{}/crates/{}#0.1.0",producer.display(),proof.package()),
            "testcases":{proof.case():{"ignored":false,"filter-match":{"status":"matches"}}}
        }));
    }
    let inventory = json!({"test-count":2,"rust-suites":suites,
        "rust-build-meta":{"target-directory":target},"retained-producer-metadata":{"source":"unchanged"}});
    let inventory_path = root.join("artifacts/inventory.json");
    std::fs::write(&inventory_path, serde_json::to_vec(&inventory)?)?;
    Ok(Fixture { _scratch: scratch, producer, consumer, target, inventory_path, inventory })
}

fn capture(fixture: &Fixture, inventory: &Value) -> Result<CrateGateProofInputs> {
    std::fs::write(&fixture.inventory_path, serde_json::to_vec(inventory)?)?;
    CrateGateProofInputs::capture(
        &fixture.inventory_path,
        &fixture.producer,
        &fixture.target,
        producer_identity()?,
    )
}

#[test]
fn consumer_binding_changes_only_two_cwd_fields_and_keeps_complete_producer_provenance(
) -> Result<()> {
    let fixture = fixture()?;
    let original_bytes = std::fs::read(&fixture.inventory_path)?;
    let inputs = capture(&fixture, &fixture.inventory)?;
    let consumer = inputs.publish_for_consumer(&fixture.consumer)?;
    let (key, path) = consumer.environment()?;
    assert_eq!(key, "BIJUX_CRATE_GATE_PROOF_INPUTS");
    let bindings: Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(bindings.as_object().context("binding object")?.len(), 2);
    let inventory_path =
        bindings["typed_artifact_handoff"]["inventory"].as_str().context("inventory path")?;
    let mut restored: Value = serde_json::from_slice(&std::fs::read(inventory_path)?)?;
    for proof in [Proof::TypedArtifactHandoff, Proof::VcfMetricRegistry] {
        assert_eq!(
            restored["rust-suites"][proof.suite()]["cwd"],
            json!(fixture.consumer.join("crates").join(proof.package()))
        );
        restored["rust-suites"][proof.suite()]["cwd"] =
            json!(fixture.producer.join("crates").join(proof.package()));
    }
    assert_eq!(restored, fixture.inventory);
    assert_eq!(consumer.provenance["producer_source_root"], json!(fixture.producer));
    assert_eq!(
        consumer.provenance["original_producer_inventory"]["sha256"],
        bijux_dna_infra::hash_bytes_sha256(&original_bytes)
    );
    assert_eq!(
        consumer.provenance["producer_source_identity"]["source_commit"],
        "0123456789abcdef0123456789abcdef01234567"
    );
    assert_eq!(
        consumer.provenance["producer_source_identity"]["source_patch_sha256"],
        "a".repeat(64)
    );
    assert_eq!(std::fs::read(&fixture.inventory_path)?, original_bytes);
    for input in &consumer.files {
        assert!(input.readonly);
    }
    inputs.verify_unchanged()?;
    consumer.verify_unchanged()?;
    assert!(
        inputs.publish_for_consumer(&fixture.consumer).is_err(),
        "cannot overwrite prior consumer inputs"
    );
    Ok(())
}

#[test]
fn missing_ignored_filtered_duplicate_and_wrong_source_proofs_fail_before_publication() -> Result<()>
{
    let fixture = fixture()?;
    let proof = Proof::TypedArtifactHandoff;
    let mut variants = Vec::new();
    let mut absent = fixture.inventory.clone();
    absent["rust-suites"][proof.suite()]["testcases"] = json!({});
    absent["test-count"] = 1.into();
    variants.push(absent);
    let mut ignored = fixture.inventory.clone();
    ignored["rust-suites"][proof.suite()]["testcases"][proof.case()]["ignored"] = true.into();
    variants.push(ignored);
    let mut filtered = fixture.inventory.clone();
    filtered["rust-suites"][proof.suite()]["testcases"][proof.case()]["filter-match"]["status"] =
        "mismatch".into();
    variants.push(filtered);
    let mut duplicate = fixture.inventory.clone();
    duplicate["rust-suites"]["foreign-suite"] = duplicate["rust-suites"][proof.suite()].clone();
    duplicate["test-count"] = 3.into();
    variants.push(duplicate);
    let mut source = fixture.inventory.clone();
    source["rust-suites"][proof.suite()]["cwd"] =
        json!(fixture.consumer.join("crates").join(proof.package()));
    variants.push(source);
    let mut identity = fixture.inventory.clone();
    identity["rust-suites"][proof.suite()]["binary-id"] = "another-owner".into();
    variants.push(identity);
    let mut kind = fixture.inventory.clone();
    kind["rust-suites"][proof.suite()]["kind"] = "test".into();
    variants.push(kind);
    for variant in variants {
        assert!(capture(&fixture, &variant).is_err());
    }
    assert!(!fixture.consumer.join("artifacts").exists());
    let raw = format!(
        "{{\"test-count\":2,\"rust-suites\":{{\"{}\":{},\"{}\":{}}}}}",
        proof.suite(),
        fixture.inventory["rust-suites"][proof.suite()],
        proof.suite(),
        fixture.inventory["rust-suites"][proof.suite()]
    );
    std::fs::write(&fixture.inventory_path, raw)?;
    assert!(
        CrateGateProofInputs::capture(
            &fixture.inventory_path,
            &fixture.producer,
            &fixture.target,
            producer_identity()?
        )
        .is_err(),
        "duplicate JSON suite key cannot replace original identity"
    );
    let suite = &fixture.inventory["rust-suites"][proof.suite()];
    let entry = format!("\"{}\":{}", proof.case(), suite["testcases"][proof.case()]);
    let duplicated = serde_json::to_string(suite)?.replace(&entry, &format!("{entry},{entry}"));
    let raw = format!(
        "{{\"test-count\":2,\"rust-suites\":{{\"{}\":{},\"{}\":{}}}}}",
        proof.suite(),
        duplicated,
        Proof::VcfMetricRegistry.suite(),
        fixture.inventory["rust-suites"][Proof::VcfMetricRegistry.suite()]
    );
    std::fs::write(&fixture.inventory_path, raw)?;
    assert!(
        CrateGateProofInputs::capture(
            &fixture.inventory_path,
            &fixture.producer,
            &fixture.target,
            producer_identity()?
        )
        .is_err(),
        "duplicate JSON proof case key cannot silently replace its observation"
    );
    Ok(())
}

#[test]
fn escaping_executable_script_and_alias_inputs_are_refused() -> Result<()> {
    let fixture = fixture()?;
    let proof = Proof::TypedArtifactHandoff;
    let mut outside = fixture.inventory.clone();
    outside["rust-suites"][proof.suite()]["binary-path"] = json!(std::env::current_exe()?);
    assert!(capture(&fixture, &outside).is_err());
    let script = fixture.target.join("script-proof");
    std::fs::write(&script, "#!/bin/sh\nexit 0\n")?;
    let mut substituted = fixture.inventory.clone();
    substituted["rust-suites"][proof.suite()]["binary-path"] = json!(script);
    assert!(capture(&fixture, &substituted).is_err());
    #[cfg(unix)]
    {
        let alias = fixture.target.join("alias-proof");
        std::os::unix::fs::symlink(fixture.target.join(proof.binary()), &alias)?;
        let mut substituted = fixture.inventory.clone();
        substituted["rust-suites"][proof.suite()]["binary-path"] = json!(alias);
        assert!(capture(&fixture, &substituted).is_err());
    }
    Ok(())
}

#[test]
fn changed_original_or_derived_inventory_is_detected_without_changing_producer_permissions(
) -> Result<()> {
    let fixture = fixture()?;
    let inputs = capture(&fixture, &fixture.inventory)?;
    let readonly = fixture.inventory_path.metadata()?.permissions().readonly();
    let consumer = inputs.publish_for_consumer(&fixture.consumer)?;
    assert_eq!(fixture.inventory_path.metadata()?.permissions().readonly(), readonly);
    let original = std::fs::read(&fixture.inventory_path)?;
    std::fs::write(&fixture.inventory_path, b"changed producer inventory")?;
    assert!(inputs.verify_unchanged().is_err());
    std::fs::write(&fixture.inventory_path, original)?;
    inputs.verify_unchanged()?;
    let derived = &consumer.files[0].path;
    std::fs::remove_file(derived)?;
    std::fs::write(derived, b"changed consumer inventory")?;
    assert!(consumer.verify_unchanged().is_err());
    Ok(())
}

#[test]
fn producer_revision_authority_rejects_missing_short_or_nonhex_identity() -> Result<()> {
    let valid = producer_identity()?;
    assert_eq!(valid.source_commit.len(), 40);
    assert_eq!(valid.source_patch_sha256.len(), 64);
    for commit in ["", "abc", "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"] {
        assert!(CrateGateProducerIdentity::new(commit, &valid.source_patch_sha256).is_err());
    }
    for patch in ["", "abc", "gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg"] {
        assert!(CrateGateProducerIdentity::new(&valid.source_commit, patch).is_err());
    }
    Ok(())
}
