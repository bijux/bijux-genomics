use super::{OpsCommandOutcome, Result, Workspace};

pub(in super::super) fn tooling_benchmark_protocol_portfolio(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mode = match args {
        [] => bijux_dna_bench::BenchmarkProtocolPortfolioWriteMode::Write,
        [flag] if flag == "--check" => {
            bijux_dna_bench::BenchmarkProtocolPortfolioWriteMode::Check
        }
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run benchmark-protocol-portfolio -- [--check]\n",
            ))
        }
    };
    let portfolio =
        bijux_dna_bench::publish_checked_in_benchmark_protocol_portfolio(&workspace.root, mode)?;
    Ok(OpsCommandOutcome::success(format!(
        "benchmark protocols: {} campaigns, {} interfaces, {} reviewed, {} derived\n",
        portfolio.campaigns.campaigns.len(),
        portfolio.interfaces.interfaces.len(),
        portfolio.reviewed_stage_count,
        portfolio.derived_stage_count,
    )))
}
