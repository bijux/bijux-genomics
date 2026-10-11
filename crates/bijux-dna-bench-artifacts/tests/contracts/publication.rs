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
