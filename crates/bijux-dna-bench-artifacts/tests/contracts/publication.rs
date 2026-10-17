use bijux_dna_bench_artifacts::{
    execution_parent, PublicationPlan, PublicationSchema, PublishedArtifactPaths, ReportOwner,
};
use std::path::{Path, PathBuf};

const SCHEMA: PublicationSchema = PublicationSchema {
    inventory_name: "owned-artifacts.json",
    schema_version: "example.owned_artifacts.v1",
};
fn owner() -> ReportOwner<'static> {
    ReportOwner {
        report_name: "metrics.json",
        schema: "example.metrics.v1",
        stage: "vcf.qc",
        tool: "plink2",
    }
}
fn execution(destination: &Path, name: &str) -> anyhow::Result<PathBuf> {
    let run = execution_parent(destination)?.join(name);
    std::fs::create_dir_all(&run)?;
    std::fs::write(
        run.join("metrics.json"),
        br#"{"schema_version":"example.metrics.v1","stage_id":"vcf.qc","tool_id":"plink2"}"#,
    )?;
    std::fs::write(run.join("native.txt"), name)?;
    Ok(run)
}

#[test]
fn publication_schema_refuses_escaping_reserved_and_empty_identity_before_replacement(
) -> anyhow::Result<()> {
    let repository = tempfile::tempdir()?;
    let destination = repository.path().join("publication");
    let run = execution(&destination, "run-one")?;
    for schema in [
        PublicationSchema { inventory_name: "../escape.json", ..SCHEMA },
        PublicationSchema { inventory_name: "metrics.json", ..SCHEMA },
        PublicationSchema { inventory_name: "stage-result.json", ..SCHEMA },
        PublicationSchema { inventory_name: "executions", ..SCHEMA },
        PublicationSchema { schema_version: "", ..SCHEMA },
    ] {
        assert!(PublicationPlan::admit(repository.path(), &run, &destination, &owner(), &schema)
            .is_err());
        assert!(!destination.join("metrics.json").exists());
    }
    assert!(!repository.path().join("escape.json").exists());
    Ok(())
}

#[test]
fn publication_preserves_native_bytes_and_unknown_siblings_and_rejects_foreign_inventory(
) -> anyhow::Result<()> {
    let repository = tempfile::tempdir()?;
    let destination = repository.path().join("publication");
    let first = execution(&destination, "run-one")?;
    PublicationPlan::admit(repository.path(), &first, &destination, &owner(), &SCHEMA)?
        .publish()?;
    std::fs::write(destination.join("evidence.txt"), b"unknown evidence")?;
    let second = execution(&destination, "run-two")?;
    PublicationPlan::admit(repository.path(), &second, &destination, &owner(), &SCHEMA)?
        .publish()?;
    assert_eq!(std::fs::read(first.join("native.txt"))?, b"run-one");
    assert_eq!(std::fs::read(destination.join("native.txt"))?, b"run-two");
    assert_eq!(std::fs::read(destination.join("evidence.txt"))?, b"unknown evidence");
    let inventory = destination.join(SCHEMA.inventory_name);
    let mut value: serde_json::Value = serde_json::from_slice(&std::fs::read(&inventory)?)?;
    value["schema_version"] = "foreign.schema.v1".into();
    std::fs::write(&inventory, serde_json::to_vec(&value)?)?;
    let third = execution(&destination, "run-three")?;
    assert!(
        PublicationPlan::admit(repository.path(), &third, &destination, &owner(), &SCHEMA).is_err()
    );
    assert_eq!(std::fs::read(destination.join("native.txt"))?, b"run-two");
    Ok(())
}

#[test]
fn public_artifact_projection_rejects_escaping_paths() -> anyhow::Result<()> {
    let repository = tempfile::tempdir()?;
    let destination = repository.path().join("publication");
    let run = execution(&destination, "run-one")?;
    let paths = PublishedArtifactPaths {
        repository: repository.path(),
        execution: &run,
        destination: &destination,
    };
    assert_eq!(paths.path(&run.join("native.txt"))?, "publication/native.txt");
    assert!(paths.path(&run.join("../foreign.txt")).is_err());
    assert!(paths.path(&destination.join("outside.txt")).is_err());
    Ok(())
}

#[test]
fn malformed_current_report_is_refused_before_stable_replacement() -> anyhow::Result<()> {
    let repository = tempfile::tempdir()?;
    let destination = repository.path().join("publication");
    let first = execution(&destination, "existing")?;
    PublicationPlan::admit(repository.path(), &first, &destination, &owner(), &SCHEMA)?
        .publish()?;
    let report = std::fs::read(destination.join("metrics.json"))?;
    let native = std::fs::read(destination.join("native.txt"))?;
    let inventory = std::fs::read(destination.join(SCHEMA.inventory_name))?;
    let malformed = execution(&destination, "malformed-current-report")?;
    std::fs::write(malformed.join("metrics.json"), b"{malformed native report")?;
    assert!(
        PublicationPlan::admit(repository.path(), &malformed, &destination, &owner(), &SCHEMA)
            .is_err(),
        "current malformed report must not become a publication plan"
    );
    assert_eq!(std::fs::read(destination.join("metrics.json"))?, report);
    assert_eq!(std::fs::read(destination.join("native.txt"))?, native);
    assert_eq!(std::fs::read(destination.join(SCHEMA.inventory_name))?, inventory);
    Ok(())
}

#[test]
fn invalid_current_report_identity_preserves_fresh_and_existing_publication() -> anyhow::Result<()>
{
    let repository = tempfile::tempdir()?;
    let malformed: [&[u8]; 13] = [
        b"{malformed report",
        b"null",
        b"[]",
        br#"["example.metrics.v1","vcf.qc","plink2"]"#,
        br#"{"schema_version":"example.metrics.v1","stage_id":"vcf.qc"}"#,
        br#"{"schema_version":1,"stage_id":"vcf.qc","tool_id":"plink2"}"#,
        br#"{"schema_version":"example.metrics.v1","stage_id":null,"tool_id":"plink2"}"#,
        br#"{"schema_version":"foreign.metrics.v1","stage_id":"vcf.qc","tool_id":"plink2"}"#,
        br#"{"schema_version":"example.metrics.v1","stage_id":"foreign.stage","tool_id":"plink2"}"#,
        br#"{"schema_version":"example.metrics.v1","stage_id":"vcf.qc","tool_id":"foreign.tool"}"#,
        br#"{"schema_version":"foreign","schema_version":"example.metrics.v1","stage_id":"vcf.qc","tool_id":"plink2"}"#,
        br#"{"schema_version":"example.metrics.v1","stage_id":"foreign","stage_id":"vcf.qc","tool_id":"plink2"}"#,
        br#"{"schema_version":"example.metrics.v1","stage_id":"vcf.qc","tool_id":"foreign","tool_id":"plink2"}"#,
    ];
    for retained in [false, true] {
        let destination =
            repository.path().join(if retained { "published" } else { "unpublished" });
        let checkpoints = if retained {
            let original = execution(&destination, "retained")?;
            PublicationPlan::admit(repository.path(), &original, &destination, &owner(), &SCHEMA)?
                .publish()?;
            ["metrics.json", "native.txt", SCHEMA.inventory_name]
                .into_iter()
                .map(|name| Ok((name, std::fs::read(destination.join(name))?)))
                .collect::<anyhow::Result<Vec<_>>>()?
        } else {
            Vec::new()
        };
        let run = execution(&destination, "invalid-report")?;
        for payload in malformed {
            std::fs::write(run.join("metrics.json"), payload)?;
            assert!(PublicationPlan::admit(
                repository.path(),
                &run,
                &destination,
                &owner(),
                &SCHEMA
            )
            .is_err());
            assert_eq!(std::fs::read(run.join("metrics.json"))?, payload);
            if retained {
                for (name, bytes) in &checkpoints {
                    assert_eq!(std::fs::read(destination.join(name))?, *bytes);
                }
            } else {
                assert!(!destination.join("metrics.json").exists());
                assert!(!destination.join("native.txt").exists());
                assert!(!destination.join(SCHEMA.inventory_name).exists());
            }
        }
    }
    Ok(())
}

#[test]
fn report_identity_allows_scientific_fields_and_enforces_existing_report_byte_bound(
) -> anyhow::Result<()> {
    let repository = tempfile::tempdir()?;
    let destination = repository.path().join("publication");
    let run = execution(&destination, "bounded-report")?;
    let prefix = br#"{"schema_version":"example.metrics.v1","stage_id":"vcf.qc","tool_id":"plink2","scientific_payload":""#;
    let mut payload = prefix.to_vec();
    payload.resize(8 * 1024 * 1024 - 2, b'x');
    payload.extend_from_slice(b"\"}");
    std::fs::write(run.join("metrics.json"), &payload)?;
    PublicationPlan::admit(repository.path(), &run, &destination, &owner(), &SCHEMA)?.publish()?;
    assert_eq!(std::fs::read(destination.join("metrics.json"))?, payload);
    assert_eq!(std::fs::read(run.join("metrics.json"))?, payload);
    payload.push(b' ');
    let oversized = execution(&destination, "oversized-report")?;
    std::fs::write(oversized.join("metrics.json"), &payload)?;
    assert!(PublicationPlan::admit(repository.path(), &oversized, &destination, &owner(), &SCHEMA)
        .is_err());
    assert_eq!(std::fs::metadata(destination.join("metrics.json"))?.len(), 8 * 1024 * 1024);
    Ok(())
}

#[test]
fn empty_declared_report_owner_cannot_admit_a_fresh_publication() -> anyhow::Result<()> {
    let repository = tempfile::tempdir()?;
    let destination = repository.path().join("publication");
    let run = execution(&destination, "owned-report")?;
    for declared in [
        ReportOwner { schema: "", ..owner() },
        ReportOwner { stage: " ", ..owner() },
        ReportOwner { tool: "", ..owner() },
    ] {
        assert!(PublicationPlan::admit(repository.path(), &run, &destination, &declared, &SCHEMA)
            .is_err());
        assert!(!destination.join("metrics.json").exists());
    }
    Ok(())
}
