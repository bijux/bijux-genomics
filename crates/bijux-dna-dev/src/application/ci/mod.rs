//! Admission of declared test inventories and observed CI execution evidence.

mod compiled_files;
mod coverage;
mod discovery;
mod events;
mod executables;
mod execution_inputs;
mod inventory;
mod partition;

pub(crate) use compiled_files::admit_compiled_executable_paths;
pub(crate) use coverage::CoverageAttestation;
pub(crate) use discovery::BinaryDiscoveryPlan;
pub(crate) use executables::CompiledWorkspaceExecutables;
pub(crate) use execution_inputs::NextestReuseInputs;
pub(crate) use inventory::ExpectedTestInventory;
pub(crate) use partition::ShardDiscoveryPlan;

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serde_json::{json, Value};

    use super::{CoverageAttestation, ExpectedTestInventory};

    fn partition_inventory() -> Result<ExpectedTestInventory> {
        let mut value = inventory_value();
        value["test-count"] = json!(3);
        value["rust-suites"]["owner::contracts"]["testcases"] = json!({
            "ordinary": {"ignored":false, "filter-match":{"status":"matches"}},
            "ignored": {"ignored":true, "filter-match":{"status":"matches"}},
            "failure": {"ignored":false, "filter-match":{"status":"matches"}}
        });
        value["rust-suites"]["owner::empty"] = json!({
            "package-name":"owner", "binary-id":"owner::empty", "binary-name":"empty",
            "kind":"test", "status":"listed", "testcases":{}
        });
        ExpectedTestInventory::parse(&serde_json::to_vec(&value)?)
    }

    #[test]
    fn discovery_plans_preserve_exact_hash_membership_and_exclude_empty_binaries() -> Result<()> {
        let inventory = partition_inventory()?;
        let mut union = std::collections::BTreeSet::new();
        for (index, names) in [vec!["ordinary"], vec![], vec![], vec!["ignored"], vec!["failure"]]
            .into_iter()
            .enumerate()
        {
            let plan = super::ShardDiscoveryPlan::from_inventory(
                &inventory,
                &format!("hash:{}/5", index + 1),
                "0.9.100",
            )?;
            assert_eq!(plan.expected_case_count, names.len());
            assert_eq!(
                plan.expected.iter().map(|case| case.test_name.as_str()).collect::<Vec<_>>(),
                names
            );
            assert_eq!(
                plan.filterset,
                if names.is_empty() { "none()" } else { "binary_id(=owner::contracts)" }
            );
            for case in plan.expected {
                assert!(union.insert(case), "hash partitions must not overlap");
            }
        }
        assert_eq!(union.len(), inventory.case_count());
        Ok(())
    }

    #[test]
    fn discovery_plans_reject_invalid_partitions_and_unqualified_executors() -> Result<()> {
        let inventory = partition_inventory()?;
        for partition in
            ["", "count:1/5", "hash:0/5", "hash:6/5", "hash:1/0", "hash:1/5/2", "hash:a/5"]
        {
            assert!(super::ShardDiscoveryPlan::from_inventory(&inventory, partition, "0.9.100")
                .is_err());
        }
        for version in ["", "0.9.99", "0.9.101", "1.0.0"] {
            assert!(
                super::ShardDiscoveryPlan::from_inventory(&inventory, "hash:1/5", version).is_err()
            );
        }
        Ok(())
    }

    #[test]
    fn partial_recovery_requires_all_and_only_its_planned_cases() -> Result<()> {
        let inventory = partition_inventory()?;
        let plan = super::ShardDiscoveryPlan::from_inventory(&inventory, "hash:1/5", "0.9.100")?;
        let ordinary = events("ordinary", true);
        let ignored = events("ignored", true);
        let failure = events("failure", true);
        let valid = CoverageAttestation::reconcile_planned(
            &inventory,
            [("selected", ordinary.as_bytes(), &plan)],
        );
        assert!(valid.selected_shards_valid);
        assert!(!valid.complete);
        let foreign = CoverageAttestation::reconcile_planned(
            &inventory,
            [("selected", ignored.as_bytes(), &plan)],
        );
        assert!(!foreign.selected_shards_valid);
        let all = super::ShardDiscoveryPlan::from_inventory(&inventory, "hash:1/1", "0.9.100")?;
        let incomplete = CoverageAttestation::reconcile_planned(
            &inventory,
            [("selected", ordinary.as_bytes(), &all)],
        );
        assert!(!incomplete.selected_shards_valid);
        let complete = [ordinary, ignored, failure].join("\n");
        assert!(
            CoverageAttestation::reconcile_planned(
                &inventory,
                [("selected", complete.as_bytes(), &all)],
            )
            .complete
        );
        Ok(())
    }

    fn inventory_value() -> Value {
        json!({"test-count": 2, "rust-suites": {
            "owner::contracts": {"package-name":"owner", "binary-id":"owner::contracts", "binary-name":"contracts", "kind":"test", "status":"listed", "testcases": {
                "first": {"ignored":false, "filter-match":{"status":"matches"}},
                "second": {"ignored":true, "filter-match":{"status":"matches"}}
            }}
        }})
    }

    fn inventory() -> Result<ExpectedTestInventory> {
        ExpectedTestInventory::parse(&serde_json::to_vec(&inventory_value())?)
    }

    fn events(name: &str, passed: bool) -> String {
        let identity = json!({"crate":"owner", "test_binary":"contracts", "kind":"test"});
        let verdict = if passed { "ok" } else { "failed" };
        let name = format!("owner::contracts${name}");
        [
            json!({"type":"suite", "event":"started", "test_count":1, "nextest":identity}),
            json!({"type":"test", "event":"started", "name":name}),
            json!({"type":"test", "event":verdict, "name":name}),
            json!({"type":"suite", "event":verdict, "passed":usize::from(passed), "failed":usize::from(!passed), "ignored":0, "measured":0, "nextest":identity}),
        ].into_iter().map(|value| value.to_string()).collect::<Vec<_>>().join("\n")
    }

    #[test]
    fn all_declared_cases_including_ignored_cases_must_execute_once() -> Result<()> {
        let inventory = inventory()?;
        let first = events("first", true);
        let second = events("second", true);
        let report = CoverageAttestation::reconcile(
            &inventory,
            [("one", first.as_bytes()), ("two", second.as_bytes())],
        );
        assert!(report.complete);
        assert!(report.selected_shards_valid);
        assert_eq!(serde_json::to_value(report)?["observed_count"], 2);
        Ok(())
    }

    #[test]
    fn selected_shard_success_does_not_claim_full_coverage() -> Result<()> {
        let inventory = inventory()?;
        let first = events("first", true);
        let report = CoverageAttestation::reconcile(&inventory, [("one", first.as_bytes())]);
        assert!(report.selected_shards_valid);
        assert!(!report.complete);
        assert_eq!(serde_json::to_value(report)?["missing"][0]["test_name"], "second");
        Ok(())
    }

    #[test]
    fn duplicate_shards_or_cross_shard_case_overlap_fail_admission() -> Result<()> {
        let inventory = inventory()?;
        let first = events("first", true);
        let second = events("second", true);
        for streams in [
            vec![("one", first.as_bytes()), ("one", second.as_bytes())],
            vec![
                ("one", first.as_bytes()),
                ("two", first.as_bytes()),
                ("three", second.as_bytes()),
            ],
        ] {
            let report = CoverageAttestation::reconcile(&inventory, streams);
            assert!(!report.selected_shards_valid);
            assert!(!report.complete);
        }
        Ok(())
    }

    #[test]
    fn failed_case_is_retained_as_failure_even_with_complete_membership() -> Result<()> {
        let inventory = inventory()?;
        let first = events("first", false);
        let second = events("second", true);
        let report = CoverageAttestation::reconcile(
            &inventory,
            [("one", first.as_bytes()), ("two", second.as_bytes())],
        );
        assert!(!report.complete);
        assert!(!report.selected_shards_valid);
        let value = serde_json::to_value(report)?;
        assert_eq!(value["observed_count"], 2);
        assert_eq!(value["failed"][0]["test_name"], "first");
        Ok(())
    }

    #[test]
    fn malformed_empty_foreign_truncated_or_inconsistent_events_cannot_pass() -> Result<()> {
        let inventory = inventory()?;
        let valid = events("first", true);
        let lines = valid.lines().collect::<Vec<_>>();
        let bad_streams = [
            String::new(),
            "not json".to_string(),
            events("foreign", true),
            lines[..3].join("\n"),
            lines[1..].join("\n"),
            [lines[0], lines[2], lines[3]].join("\n"),
            [lines[0], lines[1], lines[1], lines[2], lines[3]].join("\n"),
            [lines[0], lines[1], lines[2], lines[2], lines[3]].join("\n"),
            [lines[0], lines[1], lines[2], lines[3], lines[2]].join("\n"),
            valid.replace("\"test_count\":1", "\"test_count\":999"),
            valid.replace("\"passed\":1", "\"passed\":0"),
            valid.replace("\"event\":\"ok\"", "\"event\":\"ignored\""),
            valid.replace("\"kind\":\"test\"", "\"kind\":\"lib\""),
        ];
        for stream in bad_streams {
            let report = CoverageAttestation::reconcile(&inventory, [("one", stream.as_bytes())]);
            assert!(!report.selected_shards_valid, "unexpectedly admitted {stream}");
            assert!(!report.complete);
        }
        assert!(!CoverageAttestation::reconcile(&inventory, []).selected_shards_valid);
        Ok(())
    }

    #[test]
    fn expected_inventory_rejects_count_filter_discovery_and_identity_gaps() -> Result<()> {
        let valid = inventory_value();
        let mut count = valid.clone();
        count["test-count"] = json!(1);
        let mut filtered = valid.clone();
        filtered["rust-suites"]["owner::contracts"]["testcases"]["first"]["filter-match"]
            ["status"] = json!("mismatch");
        let mut not_discovered = valid.clone();
        not_discovered["rust-suites"]["owner::contracts"]["status"] = json!("skipped");
        let mut wrong_id = valid.clone();
        wrong_id["rust-suites"]["owner::contracts"]["binary-id"] = json!("other");
        let mut alias = valid.clone();
        let mut duplicate = valid["rust-suites"]["owner::contracts"].clone();
        duplicate["binary-id"] = json!("other");
        duplicate["kind"] = json!("lib");
        alias["rust-suites"]["other"] = duplicate;
        for value in [
            json!({"test-count":0,"rust-suites":{}}),
            count,
            filtered,
            not_discovered,
            wrong_id,
            alias,
        ] {
            assert!(ExpectedTestInventory::parse(&serde_json::to_vec(&value)?).is_err());
        }
        Ok(())
    }

    #[test]
    fn pinned_nextest_library_and_integration_events_map_to_declared_binary_ids() -> Result<()> {
        for (package, binary, kind, id, name, stream) in [
            ("bijux-dna-bench", "bijux_dna_bench", "lib", "bijux-dna-bench", "workflow::summary::row_metrics::tests::metric_outliers_map_to_metric_bearing_replicates", include_str!("../../../tests/fixtures/ci/nextest/library.jsonl")),
            ("bijux-dna-bench-model", "contracts", "test", "bijux-dna-bench-model::contracts", "mod_contracts_active_scope_rs::active_scope_report_validates_exact_summary_closure", include_str!("../../../tests/fixtures/ci/nextest/integration.jsonl")),
        ] {
            let value = json!({"test-count":1,"rust-suites": {id: {"package-name":package,"binary-id":id,"binary-name":binary,"kind":kind,"status":"listed","testcases":{name:{"ignored":false,"filter-match":{"status":"matches"}}}}}});
            let inventory = ExpectedTestInventory::parse(&serde_json::to_vec(&value)?)?;
            let report = CoverageAttestation::reconcile(&inventory, [("observed", stream.as_bytes())]);
            assert!(report.complete);
            assert_eq!(serde_json::to_value(report)?["observed_count"], 1);
        }
        Ok(())
    }

    #[test]
    fn executed_ignored_cases_are_counted_from_events_despite_legacy_suite_bookkeeping(
    ) -> Result<()> {
        let value = json!({"test-count":2,"rust-suites":{"bijux-dna-bench-model::contracts": {
            "package-name":"bijux-dna-bench-model", "binary-id":"bijux-dna-bench-model::contracts", "binary-name":"contracts", "kind":"test", "status":"listed", "testcases": {
                "ordinary":{"ignored":false,"filter-match":{"status":"matches"}},
                "ignored":{"ignored":true,"filter-match":{"status":"matches"}}
            }
        }}});
        let inventory = ExpectedTestInventory::parse(&serde_json::to_vec(&value)?)?;
        let stream = include_str!("../../../tests/fixtures/ci/nextest/ignored.jsonl");
        let report = CoverageAttestation::reconcile(&inventory, [("observed", stream.as_bytes())]);
        assert!(report.complete);
        assert_eq!(serde_json::to_value(report)?["observed_count"], 2);
        let first_block = stream.lines().take(4).collect::<Vec<_>>().join("\n");
        let incomplete =
            CoverageAttestation::reconcile(&inventory, [("observed", first_block.as_bytes())]);
        assert!(
            !incomplete.complete,
            "ignored summary counters must not replace missing case events"
        );
        Ok(())
    }

    #[test]
    fn binary_names_can_overlap_when_their_case_identities_are_distinct() -> Result<()> {
        let mut value = inventory_value();
        value["rust-suites"]["owner::bin/contracts"] = json!({
            "package-name":"owner", "binary-id":"owner::bin/contracts", "binary-name":"contracts",
            "kind":"bin", "status":"listed", "testcases":{}
        });
        let empty_binary_inventory = ExpectedTestInventory::parse(&serde_json::to_vec(&value)?)?;
        assert_eq!(empty_binary_inventory.case_count(), 2);
        value["test-count"] = json!(3);
        value["rust-suites"]["owner::bin/contracts"]["testcases"]["third"] =
            json!({"ignored":false,"filter-match":{"status":"matches"}});
        let inventory = ExpectedTestInventory::parse(&serde_json::to_vec(&value)?)?;
        let first = events("first", true);
        let second = events("second", true);
        let third = events("third", true).replace("\"kind\":\"test\"", "\"kind\":\"bin\"");
        let report = CoverageAttestation::reconcile(
            &inventory,
            [("one", first.as_bytes()), ("two", second.as_bytes()), ("three", third.as_bytes())],
        );
        assert!(report.complete);
        Ok(())
    }

    fn executable_inventory() -> Value {
        json!({"rust-build-meta":{"non-test-binaries":{
            "cli-package":[{"name":"bijux-dna","kind":"bin-exe","path":"debug/bijux-dna"}],
            "developer-package":[{"name":"bijux-dna-dev","kind":"bin-exe","path":"debug/bijux-dna-dev"}]
        }}})
    }

    #[test]
    fn archived_workspace_executables_are_explicitly_remapped_for_cross_package_tests() -> Result<()>
    {
        let bindings = super::CompiledWorkspaceExecutables::parse(&serde_json::to_vec(
            &executable_inventory(),
        )?)?;
        let root = std::env::current_dir()?.join("archive with spaces");
        let env = bindings.remapped_environment(&root.join("target"))?;
        assert_eq!(
            env,
            vec![
                (
                    "NEXTEST_BIN_EXE_bijux-dna".to_owned(),
                    root.join("target/debug/bijux-dna").display().to_string()
                ),
                (
                    "NEXTEST_BIN_EXE_bijux-dna-dev".to_owned(),
                    root.join("target/debug/bijux-dna-dev").display().to_string()
                ),
            ]
        );
        assert!(bindings.remapped_environment(std::path::Path::new("relative")).is_err());
        Ok(())
    }

    #[test]
    fn archived_workspace_executables_reject_absent_ambiguous_and_wrong_kind_bindings() -> Result<()>
    {
        let mut missing = executable_inventory();
        missing["rust-build-meta"]["non-test-binaries"]["developer-package"] = json!([]);
        assert!(super::CompiledWorkspaceExecutables::parse(&serde_json::to_vec(&missing)?).is_err());
        let mut duplicate = executable_inventory();
        duplicate["rust-build-meta"]["non-test-binaries"]["foreign-package"] =
            json!([{"name":"bijux-dna","kind":"bin-exe","path":"debug/bijux-dna"}]);
        assert!(
            super::CompiledWorkspaceExecutables::parse(&serde_json::to_vec(&duplicate)?).is_err()
        );
        let mut wrong_kind = executable_inventory();
        wrong_kind["rust-build-meta"]["non-test-binaries"]["cli-package"][0]["kind"] =
            json!("dylib");
        assert!(
            super::CompiledWorkspaceExecutables::parse(&serde_json::to_vec(&wrong_kind)?).is_err()
        );
        Ok(())
    }

    #[test]
    fn archived_executable_paths_cannot_escape_or_change_binary_identity() -> Result<()> {
        for path in [
            "",
            "/debug/bijux-dna",
            "../bijux-dna",
            "debug/../bijux-dna",
            "debug/./bijux-dna",
            "debug//bijux-dna",
            "debug\\bijux-dna",
            "debug/other",
            "bad\0/debug/bijux-dna",
        ] {
            let mut value = executable_inventory();
            value["rust-build-meta"]["non-test-binaries"]["cli-package"][0]["path"] = json!(path);
            assert!(
                super::CompiledWorkspaceExecutables::parse(&serde_json::to_vec(&value)?).is_err(),
                "{path:?}"
            );
        }
        let mut windows = executable_inventory();
        windows["rust-build-meta"]["non-test-binaries"]["cli-package"][0]["path"] =
            json!("debug/bijux-dna.exe");
        assert!(super::CompiledWorkspaceExecutables::parse(&serde_json::to_vec(&windows)?).is_ok());
        Ok(())
    }
    #[test]
    fn compiled_workspace_executables_accept_an_explicit_local_target_root() -> Result<()> {
        let bindings = super::CompiledWorkspaceExecutables::parse(&serde_json::to_vec(
            &executable_inventory(),
        )?)?;
        let root = std::env::current_dir()?.join("artifacts/build/cargo");
        let env = bindings.remapped_environment(&root)?;
        for (name, path) in env {
            let binary = name
                .strip_prefix("NEXTEST_BIN_EXE_")
                .ok_or_else(|| anyhow::anyhow!("unexpected executable variable"))?;
            assert_eq!(std::path::PathBuf::from(path), root.join("debug").join(binary));
        }
        Ok(())
    }
}
