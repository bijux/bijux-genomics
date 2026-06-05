#![cfg(feature = "bam_downstream")]

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

fn stage_api_temp_repo() -> Result<tempfile::TempDir> {
    let temp = tempfile::tempdir()?;
    let repo_root = repo_root();
    let tool_dir = temp.path().join("domain/bam/tools");
    fs::create_dir_all(&tool_dir)?;
    fs::copy(repo_root.join("domain/bam/tools/king.yaml"), tool_dir.join("king.yaml"))?;
    let runtime_dir = temp.path().join("configs/runtime/profiles");
    fs::create_dir_all(&runtime_dir)?;
    fs::copy(
        repo_root.join("configs/runtime/profiles/local.toml"),
        runtime_dir.join("local.toml"),
    )?;
    Ok(temp)
}

fn write_local_kinship_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-kinship.toml"), body)?;
    Ok(())
}

#[test]
fn local_kinship_smoke_plans_use_governed_pair_expectations() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_kinship_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 2, "governed local-smoke config must keep two BAM kinship cases");

    let insufficient = plans
        .iter()
        .find(|case| case.sample_id == "human_like_kinship_low_overlap_pair")
        .unwrap_or_else(|| panic!("governed BAM kinship insufficient-overlap case missing"));
    assert_eq!(insufficient.plan.stage_id.as_str(), "bam.kinship");
    assert_eq!(insufficient.plan.tool_id.as_str(), "king");
    assert_eq!(insufficient.plan.resources.threads, 2);
    assert_eq!(
        insufficient.bam,
        PathBuf::from(
            "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_kinship_low_overlap_pair.sam"
        )
    );
    assert_eq!(insufficient.reference_panel, "human_like_relatedness_panel");
    assert_eq!(insufficient.reference_build, "grch38");
    assert_eq!(insufficient.population_scope, "human_diploid_panel");
    assert_eq!(insufficient.min_overlap_snps, 5);
    assert!(insufficient.requires_cohort_context);
    assert_eq!(insufficient.expected_status, "insufficient");
    assert_eq!(insufficient.expected_observed_max_overlap_snps, 4);
    assert_eq!(
        insufficient.expected_insufficiency_reason.as_deref(),
        Some("insufficient_overlap_snps")
    );
    assert!(insufficient.expected_pairwise_results.is_empty());
    assert_eq!(
        insufficient.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.kinship/human_like_kinship_low_overlap_pair/king")
    );
    assert_eq!(
        insufficient.plan.params["reference_panel"],
        serde_json::json!("human_like_relatedness_panel")
    );
    assert_eq!(insufficient.plan.params["min_overlap_snps"], serde_json::json!(5));
    assert_eq!(insufficient.plan.params["requires_cohort_context"], serde_json::json!(true));

    let valid = plans
        .iter()
        .find(|case| case.sample_id == "human_like_kinship_related_pair")
        .unwrap_or_else(|| panic!("governed BAM kinship valid pair case missing"));
    assert_eq!(valid.plan.stage_id.as_str(), "bam.kinship");
    assert_eq!(valid.plan.tool_id.as_str(), "king");
    assert_eq!(
        valid.bam,
        PathBuf::from(
            "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_kinship_related_pair.sam"
        )
    );
    assert_eq!(valid.reference_panel, "human_like_relatedness_panel");
    assert_eq!(valid.reference_build, "grch38");
    assert_eq!(valid.population_scope, "human_diploid_panel");
    assert_eq!(valid.min_overlap_snps, 6);
    assert!(valid.requires_cohort_context);
    assert_eq!(valid.expected_status, "ok");
    assert_eq!(valid.expected_observed_max_overlap_snps, 6);
    assert_eq!(valid.expected_insufficiency_reason, None);
    assert_eq!(valid.expected_pairwise_results.len(), 1);
    let pair = &valid.expected_pairwise_results[0];
    assert_eq!(pair.sample_a, "sample_a");
    assert_eq!(pair.sample_b, "sample_b");
    assert_eq!(pair.overlap_snps, 6);
    assert_eq!(pair.matching_sites, 5);
    assert_eq!(pair.mismatch_sites, 1);
    assert!((pair.concordance - 0.833333).abs() <= 1e-9);
    assert!((pair.kinship_coefficient - 0.416667).abs() <= 1e-9);
    assert_eq!(pair.relationship_label, "first_degree");

    let output_names = valid
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(output_names, vec!["kinship_report", "summary", "stage_metrics"]);

    let summary_output = valid
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("kinship summary output missing from BAM kinship plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.kinship/human_like_kinship_related_pair/king/kinship.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_kinship_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalKinshipSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_kinship_smoke_plans;
}

#[test]
fn local_kinship_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_kinship_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_kinship.v1"
tool_id = "king"
threads = 2
output_dir = "target/local-smoke/bam.kinship"

[[cases]]
sample_id = " "
bam = "{bam}"
reference_panel = "human_like_relatedness_panel"
reference_build = "grch38"
population_scope = "human_diploid_panel"
min_overlap_snps = 5
requires_cohort_context = true
expected_status = "insufficient"
expected_observed_max_overlap_snps = 4
expected_insufficiency_reason = "insufficient_overlap_snps"
"#,
            bam = repo_root
                .join(
                    "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_kinship_low_overlap_pair.sam"
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_kinship_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before kinship smoke planning");
    assert_eq!(error.to_string(), "local-smoke bam.kinship sample_id must not be empty");
    Ok(())
}

#[test]
fn local_kinship_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_kinship_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_kinship.v1"
tool_id = "king"
threads = 2
output_dir = "target/local-smoke/bam.kinship"

[[cases]]
sample_id = "duplicate-kinship-case"
bam = "{insufficient_bam}"
reference_panel = "human_like_relatedness_panel"
reference_build = "grch38"
population_scope = "human_diploid_panel"
min_overlap_snps = 5
requires_cohort_context = true
expected_status = "insufficient"
expected_observed_max_overlap_snps = 4
expected_insufficiency_reason = "insufficient_overlap_snps"

[[cases]]
sample_id = "duplicate-kinship-case"
bam = "{valid_bam}"
reference_panel = "human_like_relatedness_panel"
reference_build = "grch38"
population_scope = "human_diploid_panel"
min_overlap_snps = 6
requires_cohort_context = true
expected_status = "ok"
expected_observed_max_overlap_snps = 6

[[cases.expected_pairwise_results]]
sample_a = "sample_a"
sample_b = "sample_b"
overlap_snps = 6
matching_sites = 5
mismatch_sites = 1
concordance = 0.833333
kinship_coefficient = 0.416667
relationship_label = "first_degree"
"#,
            insufficient_bam =
                repo_root
                    .join(
                        "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_kinship_low_overlap_pair.sam"
                    )
                    .display(),
            valid_bam = repo_root
                .join(
                    "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_kinship_related_pair.sam"
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_kinship_smoke_plans(temp.path())
        .expect_err("duplicate sample_ids must be rejected before kinship smoke planning");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.kinship sample_id `duplicate-kinship-case` must be unique"
    );
    Ok(())
}

#[test]
fn local_kinship_smoke_plans_reject_pairwise_results_for_insufficient_cases() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_kinship_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_kinship.v1"
tool_id = "king"
threads = 2
output_dir = "target/local-smoke/bam.kinship"

[[cases]]
sample_id = "insufficient-case-with-pairs"
bam = "{bam}"
reference_panel = "human_like_relatedness_panel"
reference_build = "grch38"
population_scope = "human_diploid_panel"
min_overlap_snps = 5
requires_cohort_context = true
expected_status = "insufficient"
expected_observed_max_overlap_snps = 4
expected_insufficiency_reason = "insufficient_overlap_snps"

[[cases.expected_pairwise_results]]
sample_a = "sample_a"
sample_b = "sample_b"
overlap_snps = 4
matching_sites = 3
mismatch_sites = 1
concordance = 0.75
kinship_coefficient = 0.125
relationship_label = "unrelated"
"#,
            bam = repo_root
                .join(
                    "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_kinship_low_overlap_pair.sam"
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_kinship_smoke_plans(temp.path())
        .expect_err("insufficient kinship cases must not declare pairwise results");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.kinship case `insufficient-case-with-pairs` must not declare pairwise results when expected_status is insufficient"
    );
    Ok(())
}

#[test]
fn local_kinship_smoke_plans_reject_duplicate_pairwise_sample_combinations() -> Result<()> {
    let temp = stage_api_temp_repo()?;
    let repo_root = repo_root();
    write_local_kinship_config(
        temp.path(),
        &format!(
            r#"
schema_version = "bijux.bench.bam.local_kinship.v1"
tool_id = "king"
threads = 2
output_dir = "target/local-smoke/bam.kinship"

[[cases]]
sample_id = "duplicate-pairwise-combination"
bam = "{bam}"
reference_panel = "human_like_relatedness_panel"
reference_build = "grch38"
population_scope = "human_diploid_panel"
min_overlap_snps = 6
requires_cohort_context = true
expected_status = "ok"
expected_observed_max_overlap_snps = 6

[[cases.expected_pairwise_results]]
sample_a = "sample_a"
sample_b = "sample_b"
overlap_snps = 6
matching_sites = 5
mismatch_sites = 1
concordance = 0.833333
kinship_coefficient = 0.416667
relationship_label = "first_degree"

[[cases.expected_pairwise_results]]
sample_a = "sample_b"
sample_b = "sample_a"
overlap_snps = 6
matching_sites = 5
mismatch_sites = 1
concordance = 0.833333
kinship_coefficient = 0.416667
relationship_label = "first_degree"
"#,
            bam = repo_root
                .join(
                    "tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_kinship_related_pair.sam"
                )
                .display(),
        ),
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_kinship_smoke_plans(temp.path())
        .expect_err("duplicate pairwise sample combinations must be rejected");
    assert_eq!(
        error.to_string(),
        "local-smoke bam.kinship case `duplicate-pairwise-combination` declared a duplicate pairwise sample combination"
    );
    Ok(())
}
