use std::path::Path;

use anyhow::{anyhow, Result};
use clap::Parser;

use crate::commands::cli;

fn global_option_value_arity(flag: &str) -> Option<usize> {
    let flag = flag.split_once('=').map_or(flag, |(name, _)| name);
    match flag {
        "--log-level" | "--profile" | "--platform" | "--telemetry-jsonl" => Some(1),
        _ => None,
    }
}

fn is_global_switch(flag: &str) -> bool {
    matches!(
        flag,
        "-v" | "--verbose"
            | "-q"
            | "--quiet"
            | "--print-effective-config"
            | "--dump-effective-config"
            | "--json"
    )
}

fn normalize_cli_argv(argv: &[String]) -> Vec<String> {
    let raw = match argv.first() {
        Some(first) => {
            let executable = Path::new(first)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or(first.as_str());
            match executable {
                "bijux" | "bijux-dna" => &argv[1..],
                _ => argv,
            }
        }
        None => argv,
    };

    let mut normalized = vec!["bijux-dna".to_string()];
    let mut index = 0usize;
    while index < raw.len() {
        let token = raw[index].as_str();
        if let Some(value_arity) = global_option_value_arity(token) {
            normalized.push(raw[index].clone());
            if token.contains('=') {
                index += 1;
            } else {
                for offset in 1..=value_arity {
                    if let Some(value) = raw.get(index + offset) {
                        normalized.push(value.clone());
                    }
                }
                index = (index + 1 + value_arity).min(raw.len());
            }
            continue;
        }
        if is_global_switch(token) {
            normalized.push(raw[index].clone());
            index += 1;
            continue;
        }
        break;
    }

    if raw.get(index).is_some_and(|token| token == "dna") {
        index += 1;
    }

    normalized.extend(raw[index..].iter().cloned());
    normalized
}

/// # Errors
/// Returns an error if the provided argv cannot be parsed into the direct DNA CLI surface.
pub fn parse_cli_from_argv(argv: &[String]) -> Result<cli::Cli> {
    let normalized = normalize_cli_argv(argv);
    cli::Cli::try_parse_from(normalized).map_err(|err| anyhow!("{err}"))
}

#[must_use]
pub fn parse_process_cli(argv: &[String]) -> cli::Cli {
    let normalized = normalize_cli_argv(argv);
    cli::Cli::parse_from(normalized)
}

#[cfg(test)]
mod tests {
    use super::{normalize_cli_argv, parse_cli_from_argv};
    use crate::commands::cli::{BenchCasesCommand, BenchCommand, BenchCycleCommand, DnaCommand};

    fn argv(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn direct_runtime_and_host_runtime_routes_normalize_to_the_same_surface() {
        let direct = normalize_cli_argv(&argv(&["bijux-dna", "registry", "list-stages"]));
        let host_route = normalize_cli_argv(&argv(&["bijux", "dna", "registry", "list-stages"]));
        let legacy_direct =
            normalize_cli_argv(&argv(&["bijux-dna", "dna", "registry", "list-stages"]));

        assert_eq!(direct, host_route);
        assert_eq!(direct, legacy_direct);
    }

    #[test]
    fn host_runtime_route_preserves_global_options_before_the_namespace() {
        let host_route = normalize_cli_argv(&argv(&[
            "bijux",
            "--json",
            "--platform",
            "test",
            "dna",
            "env",
            "info",
        ]));

        assert_eq!(host_route, argv(&["bijux-dna", "--json", "--platform", "test", "env", "info"]));
    }

    #[test]
    fn host_runtime_route_preserves_inline_global_option_values_before_the_namespace() {
        let host_route = normalize_cli_argv(&argv(&[
            "bijux",
            "--platform=test",
            "--log-level=debug",
            "dna",
            "env",
            "info",
        ]));

        assert_eq!(
            host_route,
            argv(&["bijux-dna", "--platform=test", "--log-level=debug", "env", "info"])
        );
    }

    #[test]
    fn benchmark_product_namespace_retains_the_short_alias() -> anyhow::Result<()> {
        for namespace in ["benchmark", "bench"] {
            let parsed = parse_cli_from_argv(&argv(&["bijux-dna", namespace, "status"]))?;
            assert!(matches!(parsed.command, DnaCommand::Bench(_)));
        }
        Ok(())
    }

    #[test]
    fn benchmark_reference_materialization_retains_all_governed_authorities() -> anyhow::Result<()>
    {
        let parsed = parse_cli_from_argv(&argv(&[
            "bijux-dna",
            "benchmark",
            "materialize-reference-bundle",
            "--data-root",
            "/study/data",
            "--source-archive",
            "/study/data/evidence/reference.fna.gz",
            "--reference-id",
            "grch38-reference",
            "--species-id",
            "Homo sapiens",
            "--build-id",
            "GRCh38",
            "--source-commit",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "--previous-registry",
            "/study/data/catalog/registries/prior.json",
            "--admitted-at-utc",
            "2026-08-11T00:00:00Z",
        ]))?;
        let DnaCommand::Bench(root) = parsed.command else {
            panic!("expected benchmark command");
        };
        let BenchCommand::MaterializeReferenceBundle(args) = root.command else {
            panic!("expected reference materialization command");
        };
        assert_eq!(args.reference_id, "grch38-reference");
        assert_eq!(args.species_id, "Homo sapiens");
        assert_eq!(args.build_id, "GRCh38");
        assert!(args.previous_registry.is_some());
        Ok(())
    }

    #[test]
    fn benchmark_cycle_plan_uses_the_public_product_namespace() -> anyhow::Result<()> {
        let parsed = parse_cli_from_argv(&argv(&[
            "bijux-dna",
            "benchmark",
            "cycle",
            "plan",
            "--run-id",
            "run-2002",
            "--case-matrix",
            "artifacts/benchmark-cycle/run-2002-cases.json",
            "--campaign-id",
            "campaign-run-2002",
            "--attempt-id",
            "attempt-0001",
            "--observed-at-utc",
            "2026-08-10T00:00:00Z",
            "--output",
            "artifacts/benchmark-cycle/run-2002.json",
        ]))?;
        let DnaCommand::Bench(root) = parsed.command else {
            panic!("expected benchmark command");
        };
        let BenchCommand::Cycle { command } = root.command else {
            panic!("expected benchmark cycle command");
        };
        let BenchCycleCommand::Plan(args) = command else {
            panic!("expected benchmark cycle plan command");
        };
        assert_eq!(
            args.case_matrix.as_deref(),
            Some(std::path::Path::new("artifacts/benchmark-cycle/run-2002-cases.json"))
        );
        Ok(())
    }

    #[test]
    fn benchmark_cycle_retry_retains_prior_and_current_case_authorities() -> anyhow::Result<()> {
        let parsed = parse_cli_from_argv(&argv(&[
            "bijux-dna",
            "benchmark",
            "cycle",
            "retry",
            "--benchmark-root",
            "artifacts/benchmark-cycle/evidence",
            "--case-matrix",
            "artifacts/benchmark-cycle/current-cases.json",
            "--prior-plan",
            "artifacts/benchmark-cycle/prior-plan.json",
            "--prior-case-matrix",
            "artifacts/benchmark-cycle/prior-cases.json",
            "--attempt-id",
            "attempt-002",
            "--observed-at-utc",
            "2026-08-10T00:02:00Z",
            "--output",
            "artifacts/benchmark-cycle/retry-plan.json",
        ]))?;
        let DnaCommand::Bench(root) = parsed.command else {
            panic!("expected benchmark command");
        };
        let BenchCommand::Cycle { command } = root.command else {
            panic!("expected benchmark cycle command");
        };
        let BenchCycleCommand::Retry(args) = command else {
            panic!("expected benchmark cycle retry command");
        };
        assert_eq!(args.attempt_id, "attempt-002");
        assert_eq!(
            args.prior_case_matrix,
            std::path::Path::new("artifacts/benchmark-cycle/prior-cases.json")
        );
        Ok(())
    }

    #[test]
    fn benchmark_cases_plan_retains_data_and_output_authorities() -> anyhow::Result<()> {
        let parsed = parse_cli_from_argv(&argv(&[
            "bijux-dna",
            "benchmark",
            "cases",
            "plan",
            "--study-root",
            "/study",
            "--data-index",
            "/study/data/evidence/benchmark-data-index.json",
            "--run-id",
            "run-2042",
            "--output",
            "/study/data/evidence/benchmark-case-matrices/current/run-2042.json",
        ]))?;
        let DnaCommand::Bench(root) = parsed.command else {
            panic!("expected benchmark command");
        };
        let BenchCommand::Cases { command } = root.command else {
            panic!("expected benchmark cases command");
        };
        let BenchCasesCommand::Plan(args) = command else {
            panic!("expected benchmark cases plan command");
        };
        assert_eq!(args.run_id, "run-2042");
        assert_eq!(
            args.data_index,
            std::path::Path::new("/study/data/evidence/benchmark-data-index.json")
        );
        Ok(())
    }

    #[test]
    fn benchmark_cycle_supersede_retains_ancestor_case_authorities() -> anyhow::Result<()> {
        let parsed = parse_cli_from_argv(&argv(&[
            "bijux-dna",
            "benchmark",
            "cycle",
            "supersede",
            "--benchmark-root",
            "artifacts/benchmark-cycle/evidence",
            "--prior-plan",
            "artifacts/benchmark-cycle/prior-plan.json",
            "--prior-case-matrix",
            "artifacts/benchmark-cycle/prior-cases.json",
            "--superseded-at-utc",
            "2026-08-10T00:02:00Z",
            "--reason",
            "source execution authority was replaced",
        ]))?;
        let DnaCommand::Bench(root) = parsed.command else {
            panic!("expected benchmark command");
        };
        let BenchCommand::Cycle { command } = root.command else {
            panic!("expected benchmark cycle command");
        };
        let BenchCycleCommand::Supersede(args) = command else {
            panic!("expected benchmark cycle supersede command");
        };
        assert_eq!(args.reason, "source execution authority was replaced");
        assert_eq!(
            args.prior_case_matrix,
            std::path::Path::new("artifacts/benchmark-cycle/prior-cases.json")
        );
        Ok(())
    }

    #[test]
    fn benchmark_cycle_status_uses_the_public_product_namespace() -> anyhow::Result<()> {
        let parsed = parse_cli_from_argv(&argv(&[
            "bijux-dna",
            "benchmark",
            "cycle",
            "status",
            "--benchmark-root",
            "artifacts/benchmark-cycle/evidence",
            "--plan",
            "artifacts/benchmark-cycle/run-2002.json",
            "--observed-at-utc",
            "2026-08-10T00:00:00Z",
        ]))?;
        let DnaCommand::Bench(root) = parsed.command else {
            panic!("expected benchmark command");
        };
        let BenchCommand::Cycle { command } = root.command else {
            panic!("expected benchmark cycle command");
        };
        assert!(matches!(command, BenchCycleCommand::Status(_)));
        Ok(())
    }

    #[test]
    fn benchmark_cycle_advance_uses_the_public_product_namespace() -> anyhow::Result<()> {
        let parsed = parse_cli_from_argv(&argv(&[
            "bijux-dna",
            "benchmark",
            "cycle",
            "advance",
            "--study-root",
            "/study",
            "--data-index",
            "/study/data/index.json",
            "--inventory",
            "/study/containers/inventory.json",
            "--benchmark-root",
            "/study/benchmarks/attempts",
            "--plan",
            "/study/benchmarks/plans/run-2002.json",
            "--operator",
            "operator",
            "--advanced-at-utc",
            "2026-08-10T00:00:00Z",
            "--account",
            "project",
            "--partition",
            "compute",
            "--maximum-submissions",
            "8",
        ]))?;
        let DnaCommand::Bench(root) = parsed.command else {
            panic!("expected benchmark command");
        };
        let BenchCommand::Cycle { command } = root.command else {
            panic!("expected benchmark cycle command");
        };
        assert!(matches!(command, BenchCycleCommand::Advance(_)));
        Ok(())
    }

    #[test]
    fn benchmark_cycle_collect_uses_the_public_product_namespace() -> anyhow::Result<()> {
        let parsed = parse_cli_from_argv(&argv(&[
            "bijux-dna",
            "benchmark",
            "cycle",
            "collect",
            "--benchmark-root",
            "artifacts/benchmark-cycle/evidence",
            "--plan",
            "artifacts/benchmark-cycle/run-2002.json",
            "--observed-at-utc",
            "2026-08-10T00:00:00Z",
        ]))?;
        let DnaCommand::Bench(root) = parsed.command else {
            panic!("expected benchmark command");
        };
        let BenchCommand::Cycle { command } = root.command else {
            panic!("expected benchmark cycle command");
        };
        assert!(matches!(command, BenchCycleCommand::Collect(_)));
        Ok(())
    }

    #[test]
    fn benchmark_cycle_admit_uses_the_public_product_namespace() -> anyhow::Result<()> {
        let parsed = parse_cli_from_argv(&argv(&[
            "bijux-dna",
            "benchmark",
            "cycle",
            "admit",
            "--study-root",
            "/study",
            "--benchmark-root",
            "/study/benchmarks/attempts",
            "--plan",
            "artifacts/benchmark-cycle/run-2002.json",
            "--admitted-at-utc",
            "2026-08-10T00:00:00Z",
        ]))?;
        let DnaCommand::Bench(root) = parsed.command else {
            panic!("expected benchmark command");
        };
        let BenchCommand::Cycle { command } = root.command else {
            panic!("expected benchmark cycle command");
        };
        assert!(matches!(command, BenchCycleCommand::Admit(_)));
        Ok(())
    }

    #[test]
    fn benchmark_cycle_book_uses_the_public_product_namespace() -> anyhow::Result<()> {
        let parsed = parse_cli_from_argv(&argv(&[
            "bijux-dna",
            "benchmark",
            "cycle",
            "book",
            "--study-root",
            "/study",
            "--book-id",
            "benchmark-book-2026-08-10",
            "--generated-at-utc",
            "2026-08-10T00:00:00Z",
            "--limitation",
            "Only independently admitted current publications are included.",
            "--output",
            "/study/benchmarks/evidence-books/book.json",
        ]))?;
        let DnaCommand::Bench(root) = parsed.command else {
            panic!("expected benchmark command");
        };
        let BenchCommand::Cycle { command } = root.command else {
            panic!("expected benchmark cycle command");
        };
        assert!(matches!(command, BenchCycleCommand::Book(_)));
        Ok(())
    }

    #[test]
    fn benchmark_cycle_replay_uses_the_public_product_namespace() -> anyhow::Result<()> {
        let parsed = parse_cli_from_argv(&argv(&[
            "bijux-dna",
            "benchmark",
            "cycle",
            "replay",
            "--study-root",
            "/study",
            "--run-id",
            "run-2002",
            "--replay-id",
            "run-2002-replay-20260810",
            "--replayed-at-utc",
            "2026-08-10T00:00:00Z",
            "--run-definition-uri",
            "study://contracts/run-2002.json",
            "--case-definition-uri",
            "study://contracts/run-2002-bedtools.json",
            "--immutable-input-uri",
            "study://data/bam/input.json",
            "--difference",
            "The governed replay image is not currently available.",
            "--limitation",
            "Replay compares semantic products rather than scheduler identities.",
            "--output",
            "/study/benchmarks/replays/run-2002.json",
        ]))?;
        let DnaCommand::Bench(root) = parsed.command else {
            panic!("expected benchmark command");
        };
        let BenchCommand::Cycle { command } = root.command else {
            panic!("expected benchmark cycle command");
        };
        assert!(matches!(command, BenchCycleCommand::Replay(_)));
        Ok(())
    }

    #[test]
    fn benchmark_cycle_publish_uses_the_public_product_namespace() -> anyhow::Result<()> {
        let parsed = parse_cli_from_argv(&argv(&[
            "bijux-dna",
            "benchmark",
            "cycle",
            "publish",
            "--repository-root",
            "/source",
            "--presentation-repository-root",
            "/presentation",
            "--study-root",
            "/study",
            "--benchmark-root",
            "/study/benchmarks/campaign",
            "--plan",
            "/study/benchmarks/plan.json",
            "--output-root",
            "/study/benchmarks/reports",
        ]))?;
        let DnaCommand::Bench(root) = parsed.command else {
            panic!("expected benchmark command");
        };
        let BenchCommand::Cycle { command } = root.command else {
            panic!("expected benchmark cycle command");
        };
        assert!(matches!(command, BenchCycleCommand::Publish(_)));
        Ok(())
    }
}
