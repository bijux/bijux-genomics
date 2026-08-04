use anyhow::Result;

#[test]
fn checked_in_benchmark_contracts_load_from_rust_authority() -> Result<()> {
    let (registry, portfolio, interfaces) =
        bijux_dna_bench::load_checked_in_benchmark_planning_contracts()?;
    assert_eq!(registry.schema_version, bijux_dna_bench::BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1,);
    assert_eq!(portfolio.schema_version, bijux_dna_bench::BENCHMARK_PORTFOLIO_SCHEMA_V1,);
    assert_eq!(portfolio.stage_runs.len(), 71);
    assert!(
        registry.campaigns.len() >= 6,
        "the checked-in stage campaign registry must include the audited seed campaign set"
    );
    assert_eq!(interfaces.schema_version, bijux_dna_bench::BENCHMARK_STAGE_INTERFACES_SCHEMA_V1);
    assert_eq!(interfaces.interfaces.len(), 2);
    Ok(())
}

#[test]
fn checked_in_benchmark_contracts_keep_run_2042_reserved_and_assigned() -> Result<()> {
    let (registry, portfolio, interfaces) =
        bijux_dna_bench::load_checked_in_benchmark_planning_contracts()?;
    let run_2042 =
        registry.campaigns.iter().find(|campaign| campaign.run_id == "run-2042").ok_or_else(
            || anyhow::anyhow!("run-2042 is absent from the checked-in benchmark registry"),
        )?;
    assert_eq!(run_2042.stage_id, "fastq.profile_read_lengths");
    assert_eq!(
        portfolio.stage_runs.get("fastq.profile_read_lengths"),
        Some(&"run-2042".to_string())
    );
    assert_eq!(run_2042.profile_id, "fastq-observational-qc");
    let interface = interfaces
        .interfaces
        .iter()
        .find(|interface| interface.stage_id == run_2042.stage_id)
        .ok_or_else(|| anyhow::anyhow!("run-2042 is absent from the stage interface catalog"))?;
    assert_eq!(interface.tool_ids, run_2042.tool_ids);
    Ok(())
}
