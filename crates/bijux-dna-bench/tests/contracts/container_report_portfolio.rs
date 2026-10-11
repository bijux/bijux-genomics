use std::fs;

use anyhow::Result;
use bijux_dna_bench::{
    container_report_portfolio_index_schema, derive_container_report_portfolio_index,
    render_container_report_portfolio_index_json, render_container_report_portfolio_index_markdown,
    validate_container_report_portfolio_index, ContainerReportFreshness,
};

#[test]
fn portfolio_promotes_only_the_current_permanent_run_index() -> Result<()> {
    let index = derive_container_report_portfolio_index(&super::support::repo_root()?)?;
    assert_eq!(index.summary.current_count, 1);
    assert!(index.summary.historical_unvalidated_count > 0);
    assert_eq!(index.reports[0].report_id, "permanent-run-index");
    assert_eq!(index.reports[0].freshness, ContainerReportFreshness::Current);
    assert!(index.reports.iter().skip(1).all(|report| {
        report.freshness == ContainerReportFreshness::HistoricalUnvalidated
            && !report.limitations.is_empty()
    }));

    let mut promoted = index;
    promoted.reports[1].freshness = ContainerReportFreshness::Current;
    promoted.summary.current_count += 1;
    promoted.summary.historical_unvalidated_count -= 1;
    assert!(validate_container_report_portfolio_index(&promoted).is_err());
    Ok(())
}

#[test]
fn checked_publication_and_schema_match_their_rust_owners() -> Result<()> {
    let repository_root = super::support::repo_root()?;
    let index = derive_container_report_portfolio_index(&repository_root)?;
    assert_eq!(
        fs::read_to_string(repository_root.join("containers/reports/REPORT_INDEX.json"))?,
        render_container_report_portfolio_index_json(&index)?
    );
    assert_eq!(
        fs::read_to_string(repository_root.join("containers/reports/REPORT_INDEX.md"))?,
        render_container_report_portfolio_index_markdown(&index)
    );
    assert_eq!(
        fs::read_to_string(
            repository_root.join("containers/contracts/schemas/report-portfolio-index.v1.json")
        )?,
        serde_json::to_string_pretty(&container_report_portfolio_index_schema())? + "\n"
    );
    Ok(())
}
