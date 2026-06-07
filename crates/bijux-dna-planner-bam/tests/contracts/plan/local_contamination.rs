use anyhow::Result;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

fn stage_api_temp_repo() -> Result<tempfile::TempDir> {
    let temp = tempfile::tempdir()?;
    let repo_root = repo_root();
    let tool_dir = temp.path().join("domain/bam/tools");
    fs::create_dir_all(&tool_dir)?;
    fs::copy(
        repo_root.join("domain/bam/tools/verifybamid2.yaml"),
        tool_dir.join("verifybamid2.yaml"),
    )?;
    let runtime_dir = temp.path().join("configs/runtime/profiles");
    fs::create_dir_all(&runtime_dir)?;
    fs::copy(
        repo_root.join("configs/runtime/profiles/local.toml"),
        runtime_dir.join("local.toml"),
    )?;
    Ok(temp)
}

fn write_local_contamination_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("benchmarks/configs/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-contamination.toml"), body)?;
    Ok(())
}

#[test]
fn local_contamination_plan_uses_governed_bam_reference_and_panel_inputs() -> Result<()> {
    let repo_root = repo_root();
    let plan = bijux_dna_planner_bam::stage_api::local_contamination_plan(&repo_root)?;

    assert_eq!(plan.stage_id.as_str(), "bam.contamination");
    assert_eq!(plan.tool_id.as_str(), "verifybamid2");
    assert_eq!(plan.resources.threads, 2);
    assert_eq!(plan.resources.mem_gb, 8);
    assert_eq!(plan.out_dir, PathBuf::from("target/local-ready/bam.contamination"));

    let bam = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "bam")
        .unwrap_or_else(|| panic!("bam input missing from local-ready plan"));
    assert_eq!(
        bam.path,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_contamination_panel_screen.sam"
        )
    );

    let bai = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "bam_bai")
        .unwrap_or_else(|| panic!("bam_bai input missing from local-ready plan"));
    assert_eq!(
        bai.path,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_contamination_panel_screen.sam.bai"
        )
    );

    let reference = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "reference")
        .unwrap_or_else(|| panic!("reference input missing from local-ready plan"));
    assert_eq!(
        reference.path,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta"
        )
    );

    let reference_panel = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "reference_panel")
        .unwrap_or_else(|| panic!("reference_panel input missing from local-ready plan"));
    assert_eq!(
        reference_panel.path,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat"
        )
    );

    let contamination_report = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "contamination_report")
        .unwrap_or_else(|| panic!("contamination_report output missing from local-ready plan"));
    assert_eq!(
        contamination_report.path,
        PathBuf::from("target/local-ready/bam.contamination/contamination.json")
    );
    let contamination_summary = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("summary output missing from local-ready plan"));
    assert_eq!(
        contamination_summary.path,
        PathBuf::from("target/local-ready/bam.contamination/contamination.summary.json")
    );
    let stage_metrics = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "stage_metrics")
        .unwrap_or_else(|| panic!("stage_metrics output missing from local-ready plan"));
    assert_eq!(
        stage_metrics.path,
        PathBuf::from("target/local-ready/bam.contamination/stage.metrics.json")
    );
    assert_eq!(plan.params["scope"], serde_json::json!("nuclear"));
    assert_eq!(plan.params["prior"], serde_json::json!(0.02));
    assert_eq!(plan.params["sex_specific"], serde_json::json!(false));
    assert_eq!(plan.params["chromosome_system"], serde_json::json!("xy"));
    assert_eq!(plan.params["minimum_mean_coverage"], serde_json::json!(0.5));
    assert_eq!(plan.params["emit_confidence_caveats"], serde_json::json!(true));
    assert_eq!(
        plan.params["assumptions"],
        serde_json::json!(
            "governed aDNA BAM corpus contamination panel with shared non-UDG reference for local contamination planning"
        )
    );
    assert_eq!(
        plan.params["reference_panels"],
        serde_json::json!([
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat"
        ])
    );
    assert_eq!(plan.params["sample_id"], serde_json::json!("adna_contamination_panel_screen"));
    assert_eq!(plan.params["tool"], serde_json::json!("verifybamid2"));
    assert_eq!(
        plan.params["required_reference_digest"],
        serde_json::json!("ade9a8686ba2679e772a4ce5cf74fb805460070be21bfdbed523bc6d8e566b1c")
    );
    assert_eq!(plan.params["tool_scope"], serde_json::json!("nuclear"));
    assert_eq!(plan.effective_params["chromosome_system"], serde_json::json!("xy"));
    assert_eq!(plan.effective_params["minimum_mean_coverage"], serde_json::json!(0.5));
    assert_eq!(
        plan.effective_params["assumptions"],
        serde_json::json!(
            "governed aDNA BAM corpus contamination panel with shared non-UDG reference for local contamination planning"
        )
    );
    assert_eq!(
        plan.effective_params["required_reference_digest"],
        serde_json::json!("ade9a8686ba2679e772a4ce5cf74fb805460070be21bfdbed523bc6d8e566b1c")
    );
    assert_eq!(plan.effective_params["emit_confidence_caveats"], serde_json::json!(true));

    let command =
        plan.command.template.iter().last().unwrap_or_else(|| {
            panic!("bam.contamination command template must contain a shell body")
        });
    assert!(
        command.contains(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_contamination_panel_screen.sam.bai"
        ) && command.contains(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta"
        ) && command.contains(
            "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat"
        )
            && command.contains("target/local-ready/bam.contamination/contamination")
            && command.contains("target/local-ready/bam.contamination/contamination.summary.json"),
        "local-ready contamination command must carry the governed BAI, reference, panel, report prefix, and summary output"
    );

    Ok(())
}

#[test]
fn local_contamination_plan_stage_api_surface_stays_callable() {
    let _: fn(&std::path::Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_bam::stage_api::local_contamination_plan;
}

#[test]
fn local_contamination_plan_rejects_empty_sample_ids() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_contamination_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_contamination.v1"
bam = "{bam}"
bai = "{bai}"
reference_fasta = "{reference}"
reference_panels = ["{panel}"]
tool_id = "verifybamid2"
sample_id = " "
scope = "nuclear"
prior = 0.02
sex_specific = false
assumptions = "governed BAM corpus contamination panel with shared corpus reference for local contamination planning"
chromosome_system = "xy"
minimum_mean_coverage = 0.5
emit_confidence_caveats = true
threads = 2
output_dir = "target/local-ready/bam.contamination"
"#,
            bam = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_contamination_panel_screen.sam")
                .display(),
            bai = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_contamination_panel_screen.sam.bai")
                .display(),
            reference = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta")
                .display(),
            panel = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_contamination_panel.dat")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_contamination_plan(temp.path())
        .expect_err("empty sample_id must be rejected before contamination plan construction");
    assert_eq!(error.to_string(), "local-ready bam.contamination sample_id must not be empty");
    Ok(())
}

#[test]
fn local_contamination_plan_requires_governed_assumptions() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_contamination_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_contamination.v1"
bam = "{bam}"
bai = "{bai}"
reference_fasta = "{reference}"
reference_panels = ["{panel}"]
tool_id = "verifybamid2"
sample_id = "missing-assumptions"
scope = "nuclear"
prior = 0.02
sex_specific = false
assumptions = " "
chromosome_system = "xy"
minimum_mean_coverage = 0.5
emit_confidence_caveats = true
threads = 2
output_dir = "target/local-ready/bam.contamination"
"#,
            bam = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_contamination_panel_screen.sam")
                .display(),
            bai = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_contamination_panel_screen.sam.bai")
                .display(),
            reference = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta")
                .display(),
            panel = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_contamination_panel.dat")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_contamination_plan(temp.path())
        .expect_err("blank assumptions must be rejected for governed contamination planning");
    assert_eq!(
        error.to_string(),
        "local-ready bam.contamination requires a non-empty governed assumptions string"
    );
    Ok(())
}

#[test]
fn local_contamination_plan_requires_positive_minimum_mean_coverage() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_contamination_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_contamination.v1"
bam = "{bam}"
bai = "{bai}"
reference_fasta = "{reference}"
reference_panels = ["{panel}"]
tool_id = "verifybamid2"
sample_id = "bad-coverage-threshold"
scope = "nuclear"
prior = 0.02
sex_specific = false
assumptions = "governed BAM corpus contamination panel with shared corpus reference for local contamination planning"
chromosome_system = "xy"
minimum_mean_coverage = 0.0
emit_confidence_caveats = true
threads = 2
output_dir = "target/local-ready/bam.contamination"
"#,
            bam = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_contamination_panel_screen.sam")
                .display(),
            bai = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_contamination_panel_screen.sam.bai")
                .display(),
            reference = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta")
                .display(),
            panel = repo_root
                .join("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_contamination_panel.dat")
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_contamination_plan(temp.path())
        .expect_err("non-positive minimum_mean_coverage must be rejected");
    assert_eq!(
        error.to_string(),
        "local-ready bam.contamination minimum_mean_coverage must be finite and greater than zero"
    );
    Ok(())
}
