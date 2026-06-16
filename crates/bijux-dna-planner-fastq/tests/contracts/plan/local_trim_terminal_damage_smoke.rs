use anyhow::Result;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

#[test]
fn local_trim_terminal_damage_smoke_plans_use_governed_corpus_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans =
        bijux_dna_planner_fastq::stage_api::local_trim_terminal_damage_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 2, "governed terminal-damage smoke should keep SE and PE cases");

    let se_case = plans
        .iter()
        .find(|case| case.sample_id == "adna-like-se")
        .unwrap_or_else(|| panic!("single-end terminal-damage smoke case missing"));
    assert_eq!(se_case.plan.stage_id.as_str(), "fastq.trim_terminal_damage");
    assert_eq!(se_case.plan.tool_id.as_str(), "cutadapt");
    assert_eq!(
        se_case.r1,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/adna_like_se_compact_R1.fastq.gz"
        )
    );
    assert_eq!(se_case.r2, None);
    assert_eq!(
        se_case.plan.out_dir,
        PathBuf::from("runs/bench/local-smoke/fastq.trim_terminal_damage/adna-like-se/cutadapt")
    );
    assert_eq!(se_case.plan.resources.threads, 1);
    assert_eq!(
        se_case.plan.params["report_json"],
        serde_json::json!(
            "runs/bench/local-smoke/fastq.trim_terminal_damage/adna-like-se/cutadapt/trim_terminal_damage_report.json"
        )
    );
    assert_eq!(se_case.plan.effective_params["damage_mode"], serde_json::json!("ancient"));
    assert_eq!(
        se_case.plan.effective_params["execution_policy"],
        serde_json::json!("explicit_terminal_trim")
    );
    assert_eq!(se_case.plan.effective_params["trim_5p_bases"], serde_json::json!(2));
    assert_eq!(se_case.plan.effective_params["trim_3p_bases"], serde_json::json!(1));

    let pe_case = plans
        .iter()
        .find(|case| case.sample_id == "adna-like-pe")
        .unwrap_or_else(|| panic!("paired-end terminal-damage smoke case missing"));
    assert_eq!(
        pe_case.r1,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/adna_like_pe_trim_signals_R1.fastq.gz"
        )
    );
    assert_eq!(
        pe_case.r2,
        Some(PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/adna_like_pe_trim_signals_R2.fastq.gz"
        ))
    );
    assert_eq!(
        pe_case.plan.out_dir,
        PathBuf::from("runs/bench/local-smoke/fastq.trim_terminal_damage/adna-like-pe/cutadapt")
    );
    assert_eq!(pe_case.plan.effective_params["paired_mode"], serde_json::json!("paired_end"));

    Ok(())
}

#[test]
fn local_trim_terminal_damage_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_fastq::LocalTrimTerminalDamageSmokeCasePlan>,
    > = bijux_dna_planner_fastq::stage_api::local_trim_terminal_damage_smoke_plans;
}
