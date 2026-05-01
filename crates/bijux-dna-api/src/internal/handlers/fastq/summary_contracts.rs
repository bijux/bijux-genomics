mod tests {
    use super::StageExecutionSummary;
    use super::{write_run_manifest, write_scientific_provenance};
    use bijux_dna_core::metrics::{AdapterBankProvenanceV1, ToolInvocationV1};
    use bijux_dna_core::prelude::{
        ArtifactId, CommandSpecV1, ContainerImageRefV1, StageVersion, ToolConstraints, ToolId,
    };
    use bijux_dna_planner_fastq::stage_api::STAGE_TRIM_READS;
    use bijux_dna_runner::step_runner::StageResultV1;
    use bijux_dna_stage_contract::{StageIO, StagePlanV1};
    use insta::Settings;
    use std::path::PathBuf;

    fn snapshot_name(group: &str, name: &str) -> String {
        format!("bijux-dna-api__{group}__{name}")
    }

    /// Contract intent: run manifest serialization always includes defaults ledger metadata.
    #[test]
    fn run_manifest_includes_defaults_ledger() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-dna-run-manifest")?;
        let out_dir = temp.path();
        let defaults = serde_json::json!({
            "pipeline_id": "fastq-to-fastq__default__v1",
            "tools": {},
            "params": {},
            "thresholds": {},
            "tool_provenance": {},
            "param_provenance": {},
            "assumptions": [],
            "citations": {},
        });
        bijux_dna_infra::write_bytes(
            out_dir.join("defaults_ledger.json"),
            serde_json::to_vec_pretty(&defaults)?,
        )?;

        let stage_out = out_dir.join("stage");
        bijux_dna_infra::ensure_dir(&stage_out)?;
        let plan = StagePlanV1 {
            stage_id: STAGE_TRIM_READS.clone(),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("fastp"),
            tool_version: "0.0.0".to_string(),
            image: ContainerImageRefV1 {
                image: "tool:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 { template: vec![] },
            resources: ToolConstraints {
                runtime: "1h".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
            io: StageIO {
                inputs: Vec::new(),
                outputs: Vec::new(),
            },
            out_dir: stage_out,
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: std::collections::BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        let result = StageResultV1 {
            run_id: "run-1".to_string(),
            exit_code: 0,
            runtime_s: 1.0,
            memory_mb: 1.0,
            outputs: Vec::new(),
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: String::new(),
        };
        let stage_runs = vec![StageExecutionSummary {
            plan: bijux_dna_stage_contract::execution_step_from_stage_plan(&plan),
            result,
        }];
        write_run_manifest(out_dir, &stage_runs, &[])?;
        let manifest_raw = std::fs::read_to_string(out_dir.join("run_manifest.json"))?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)?;
        assert!(manifest.get("defaults_ledger").is_some());
        assert!(manifest.get("defaults_ledger_sha256").is_some());
        assert_no_absolute_paths(&manifest);
        Ok(())
    }

    /// Snapshot intent: scientific provenance JSON remains schema-stable and path-normalized.
    #[test]
    #[allow(clippy::too_many_lines)]
    fn scientific_provenance_contract_is_written() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-dna-scientific-provenance")?;
        let out_dir = temp.path();
        let defaults = serde_json::json!({
            "pipeline_id": "fastq-to-fastq__default__v1",
            "tools": {},
            "params": {},
            "thresholds": {},
            "tool_provenance": {},
            "param_provenance": {},
            "assumptions": [],
            "citations": {},
        });
        bijux_dna_infra::write_bytes(
            out_dir.join("defaults_ledger.json"),
            serde_json::to_vec_pretty(&defaults)?,
        )?;
        std::env::set_var("BIJUX_PLANNER_VERSION", "planner.v1");

        let stage_out = out_dir.join("stage");
        let artifacts = bijux_dna_runtime::recording::run_artifacts_dir_for_out(&stage_out);
        let invocations = artifacts.join("invocations");
        bijux_dna_infra::ensure_dir(&invocations)?;
        let plan = StagePlanV1 {
            stage_id: STAGE_TRIM_READS.clone(),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("fastp"),
            tool_version: "0.0.0".to_string(),
            image: ContainerImageRefV1 {
                image: "tool:latest".to_string(),
                digest: Some("sha256:img".to_string()),
            },
            command: CommandSpecV1 {
                template: vec!["fastp".to_string()],
            },
            resources: ToolConstraints {
                runtime: "1h".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
            io: StageIO {
                inputs: vec![bijux_dna_stage_contract::ArtifactRef::required(
                    ArtifactId::from_static("input"),
                    PathBuf::from("input.fastq.gz"),
                    bijux_dna_core::contract::ArtifactRole::Reads,
                )],
                outputs: vec![bijux_dna_stage_contract::ArtifactRef::required(
                    ArtifactId::from_static("output"),
                    PathBuf::from("output.fastq.gz"),
                    bijux_dna_core::contract::ArtifactRole::Reads,
                )],
            },
            out_dir: stage_out.clone(),
            params: serde_json::json!({"sample_id":"s1"}),
            effective_params: serde_json::json!({}),
            aux_images: std::collections::BTreeMap::new(),
            operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
            canonical_contract: None,
            provenance: None,
            reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
        };
        let invocation = ToolInvocationV1 {
            schema_version: "bijux.tool_invocation.v1".to_string(),
            contract_version: bijux_dna_core::contract::ContractVersion::v1(),
            stage_id: plan.stage_id.clone(),
            tool_id: plan.tool_id.clone(),
            tool_version: plan.tool_version.clone(),
            resolved_tool_version: Some(plan.tool_version.clone()),
            image_digest: "sha256:img".to_string(),
            runner_kind: "docker".to_string(),
            platform: "test".to_string(),
            parameters_json: serde_json::json!({"min_len": 10}),
            parameters_json_normalized: serde_json::json!({"min_len": 10}),
            effective_params_json: serde_json::json!({}),
            effective_params_json_normalized: serde_json::json!({}),
            params_provenance: serde_json::json!({
                "tool_params": serde_json::json!({"min_len": 10}),
                "defaults": serde_json::json!({}),
                "overrides": serde_json::json!({}),
                "effective_params": serde_json::json!({}),
            }),
            params_provenance_normalized: serde_json::json!({}),
            adapter_bank: Some(AdapterBankProvenanceV1 {
                bank_id: "bank".to_string(),
                bank_version: "v1".to_string(),
                bank_hash: "sha256:bank".to_string(),
                presets_hash: "sha256:presets".to_string(),
                preset: "default".to_string(),
                preset_hash: "sha256:preset".to_string(),
                enabled_categories: Vec::new(),
                disabled_categories: Vec::new(),
                enable_adapters: Vec::new(),
                disable_adapters: Vec::new(),
                enabled_entries: Vec::new(),
            }),
            banks: None,
            bank_assets: None,
            resources: plan.resources.clone(),
            environment: std::collections::BTreeMap::new(),
            input_hashes: vec!["sha256:input".to_string()],
            output_hashes: vec!["sha256:output".to_string()],
            executed_command: Some("fastp".to_string()),
        };
        bijux_dna_infra::atomic_write_json(
            &invocations.join(format!("{}.tool_invocation.json", plan.stage_id.0)),
            &invocation,
        )?;
        let metrics_envelope = serde_json::json!({
            "schema_version": "bijux.metrics_envelope.v2",
            "stage_id": plan.stage_id.0,
            "stage_version": 1,
            "tool_id": plan.tool_id.0,
            "tool_version": plan.tool_version,
            "image_digest": "sha256:img",
            "parameters_fingerprint": "params",
            "input_fingerprint": "sha256:input",
            "parameters_json_normalized": serde_json::json!({"min_len": 10}),
            "input_hashes": ["sha256:input"],
            "metrics": {}
        });
        bijux_dna_infra::atomic_write_json(
            &artifacts.join("metrics_envelope.json"),
            &metrics_envelope,
        )?;

        let summary = StageExecutionSummary {
            plan: bijux_dna_stage_contract::execution_step_from_stage_plan(&plan),
            result: StageResultV1 {
                run_id: "run-1".to_string(),
                exit_code: 0,
                runtime_s: 1.0,
                memory_mb: 1.0,
                outputs: Vec::new(),
                metrics_path: None,
                stdout: String::new(),
                stderr: String::new(),
                command: "fastp".to_string(),
            },
        };
        write_scientific_provenance(out_dir, &[summary])?;
        let raw = std::fs::read_to_string(out_dir.join("scientific_provenance.json"))?;
        let payload: serde_json::Value = serde_json::from_str(&raw)?;
        assert_eq!(
            payload.get("pipeline_id").and_then(|v| v.as_str()),
            Some("fastq-to-fastq__default__v1")
        );
        assert_eq!(
            payload.get("planner_version").and_then(|v| v.as_str()),
            Some("planner.v1")
        );
        let name = snapshot_name("schemas", "scientific_provenance_contract");
        let mut settings = Settings::new();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path(
            crate::support::workspace::resolve_repo_root()?
                .join("crates/bijux-dna-api/tests/snapshots"),
        );
        settings.bind(|| {
            insta::assert_json_snapshot!(
                name,
                bijux_dna_testkit::snapshot_normalize_json(&payload)
            );
        });
        Ok(())
    }

    fn assert_no_absolute_paths(value: &serde_json::Value) {
        match value {
            serde_json::Value::String(s) => {
                assert!(
                    !s.starts_with('/') || s.starts_with("//"),
                    "absolute path found: {s}"
                );
                assert!(!s.contains(":\\"), "windows absolute path found: {s}");
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    assert_no_absolute_paths(item);
                }
            }
            serde_json::Value::Object(map) => {
                for (_, value) in map {
                    assert_no_absolute_paths(value);
                }
            }
            _ => {}
        }
    }
}
