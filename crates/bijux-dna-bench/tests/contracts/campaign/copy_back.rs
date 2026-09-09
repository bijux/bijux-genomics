//! Campaign copy-back publication and persisted contract regressions.
use super::*;
use bijux_dna_bench_model::{quote_slurm_argument, CopyBackManifestReport};
use std::collections::BTreeMap;

#[test]
fn copy_back_preserves_paths_replays_closed_records_and_reports_publication_errors(
) -> anyhow::Result<()> {
    let root = bijux_dna_infra::temp_dir("campaign-copy-back-")?;
    let context = CampaignContext {
        workspace_root: root.path().to_path_buf(),
        campaign_root: root.path().to_path_buf(),
        process_environment: BTreeMap::new(),
        executable_search_path: None,
        timestamp: "1704067200".to_string(),
    };
    let config_path = root.path().join("campaign.toml");
    let config = r#"
schema_version = "bijux.hpc.campaign.v1"

[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[slurm]
site_profile = "generic"
default_resource_template = "standard"

[resources]
default = "standard"

[resources.templates.standard]
cpus = 8
mem_gb = 32
walltime = "02:00:00"
scratch_gb = 64

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
resource_template = "standard"
"#;

    bijux_dna_infra::write_bytes(&config_path, config)?;
    let manifest = write_campaign_copy_back_manifest(&context, &config_path, None, None, None)?;
    let expected_path = root.path().join("copy-back-manifest.json");
    assert_eq!(manifest.manifest_path, expected_path.display().to_string());
    assert_eq!(manifest.entries.len(), 1);
    let entry = &manifest.entries[0];
    assert_eq!(entry.stage, "fastq.validate_reads");
    assert_eq!(entry.tool, "seqkit_v2");
    assert_eq!(entry.sample, "sample-1");
    assert_eq!(entry.script_path, format!("{}.sbatch.sh", entry.log_path));
    assert_eq!(entry.results_sidecar_path, format!("{}.sidecar.json", entry.results_path));
    assert_eq!(entry.code_sidecar_path, format!("{}.sidecar.json", entry.code_path));
    let bytes = std::fs::read(&expected_path)?;
    assert_eq!(bytes, serde_json::to_vec_pretty(&manifest)?);
    assert_eq!(serde_json::from_slice::<CopyBackManifestReport>(&bytes)?, manifest);
    assert_eq!(
        write_campaign_copy_back_manifest(&context, &config_path, None, None, None)?,
        manifest
    );
    for (field, value) in [
        ("schema_version", serde_json::json!("unsupported")),
        ("unrecognized", serde_json::json!(true)),
    ] {
        let mut value_record = serde_json::to_value(&manifest)?;
        value_record[field] = value;
        assert!(serde_json::from_value::<CopyBackManifestReport>(value_record).is_err());
    }
    let mut value = serde_json::to_value(&manifest)?;
    value["entries"][0]["extra"] = serde_json::json!(true);
    assert!(serde_json::from_value::<CopyBackManifestReport>(value).is_err());
    let blocker = root.path().join("existing-file");
    bijux_dna_infra::write_bytes(&blocker, b"preserve")?;
    assert!(write_campaign_copy_back_manifest(
        &context,
        &config_path,
        None,
        None,
        Some(&blocker.join("out.json"))
    )
    .is_err());
    assert_eq!(std::fs::read(&blocker)?, b"preserve");
    assert_eq!(std::fs::read(&expected_path)?, bytes);
    Ok(())
}

#[cfg(unix)]
#[test]
fn slurm_shell_arguments_round_trip_literal_bytes() -> anyhow::Result<()> {
    for value in
        ["", "plain/path", "space and 'quote'", "$(touch forbidden)", "a;b", "line\nbreak", "é"]
    {
        let script = format!(
            "set -- {}; test \"$#\" -eq 1 && printf '%s' \"$1\"",
            quote_slurm_argument(value)
        );
        let output = std::process::Command::new("/bin/sh").args(["-c", &script]).output()?;
        assert!(output.status.success());
        assert_eq!(output.stdout, value.as_bytes());
    }
    Ok(())
}
