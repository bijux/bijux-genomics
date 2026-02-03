use std::fs;
use std::path::PathBuf;

use bijux_pipelines::registry::PipelineRegistry;
use tempfile::TempDir;

fn write_profile(temp: &TempDir) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let configs_dir = temp.path().join("configs").join("profiles");
    fs::create_dir_all(&configs_dir)?;
    let profile_path = configs_dir.join("local.yaml");
    fs::write(
        &profile_path,
        "container_runtime: \"docker\"\n\
default_threads: 4\n\
default_mem_gb: 8\n\
default_time_minutes: 60\n\
run_base_dir: \"./runs\"\n",
    )?;
    Ok(profile_path)
}

fn write_manifests(temp: &TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let domain_dir = temp.path().join("domain").join("fastq");
    let stages_dir = domain_dir.join("stages");
    let tools_dir = domain_dir.join("tools");
    fs::create_dir_all(&stages_dir)?;
    fs::create_dir_all(&tools_dir)?;
    fs::write(
        stages_dir.join("trim.yaml"),
        r#"schema_version: "bijux.stage.v1"
stage_id: "fastq.trim"
domain: "fastq"
inputs:
  - name: "reads"
    data_type: "fastq"
    cardinality: "Many"
outputs:
  - name: "trimmed_reads"
    data_type: "fastq"
    cardinality: "Many"
parameters:
  - name: "min_length"
    param_type: "int"
    default: "30"
metrics:
  - name: "reads_in"
    meaning: "Number of input reads"
  - name: "reads_out"
    meaning: "Number of output reads"
description: "FASTQ stage for trimming and preprocessing reads."
"#,
    )?;
    fs::write(
        tools_dir.join("fastp.yaml"),
        r#"schema_version: "bijux.tool.v1"
tool_id: "fastp"
stage_id: "fastq.trim"
role: "transform"
authoritative: false
strict_capable: false
status: "stable"
capabilities:
  - "SE"
  - "PE"
  - "adapters"
container:
  image: "bijuxdna/fastp"
  digest: "sha256:603656aa361eee1cbd1370db9412e588da91708da5542173e5ae74aab71cbc10"
command_template:
  - "fastp"
  - "--in1"
  - "{{reads}}"
  - "--out1"
  - "{{trimmed_reads}}"
outputs:
  - name: "trimmed_reads"
    data_type: "fastq"
    cardinality: "Many"
metrics_parser: "fastp_v0"
constraints:
  runtime: "local"
  mem_gb: 8
  tmp_gb: 4
  threads: 4
execution_contract:
  required_inputs:
    - "reads_r1"
  expected_outputs:
    - "*"
  forbidden_outputs: []
  forbid_unexpected_outputs: false
"#,
    )?;
    Ok(())
}

#[test]
fn fastq_trim_creates_run_artifacts() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _profile_path = write_profile(&temp)?;
    write_manifests(&temp)?;

    let mut cmd = assert_cmd::cargo_bin_cmd!("bijux");
    cmd.current_dir(temp.path())
        .args(["--profile", "local", "fastq", "trim"]);

    cmd.assert().success();

    let runs_dir = temp.path().join("runs");
    let mut entries = fs::read_dir(&runs_dir)?;
    let run_entry = entries.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "run dir entry missing")
    })??;
    let run_id_dir = run_entry.path();

    let stage_dir = run_id_dir.join("fastq.trim").join("fastp");
    let report_path = stage_dir.join("report.json");
    let log_path = stage_dir.join("logs").join("bijux.log");
    assert!(stage_dir.is_dir(), "stage dir missing");
    assert!(report_path.is_file(), "report.json missing");
    assert!(log_path.is_file(), "bijux.log missing");
    Ok(())
}

#[test]
fn pipelines_list_matches_registry() -> Result<(), Box<dyn std::error::Error>> {
    let registry = PipelineRegistry::v1();
    let mut expected: Vec<String> = registry
        .list(false)
        .into_iter()
        .map(|profile| {
            format!(
                "{}\t{}\t{}",
                profile.id.as_str(),
                profile.stability.as_str(),
                profile.description
            )
        })
        .collect();
    expected.sort();

    let mut cmd = assert_cmd::cargo_bin_cmd!("bijux");
    let output = cmd.args(["pipelines", "list"]).output()?;
    assert!(output.status.success(), "pipelines list failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut actual: Vec<String> = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();
    actual.sort();
    assert_eq!(actual, expected);
    Ok(())
}
