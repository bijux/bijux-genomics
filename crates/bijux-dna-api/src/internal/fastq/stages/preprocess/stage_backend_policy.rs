fn canonical_sample_identity(sample_id: &str) -> String {
    let mut out = String::with_capacity(sample_id.len());
    for ch in sample_id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    out.trim_matches('_').to_string()
}

fn parse_low_complexity_filtered_count(stdout: &str, stderr: &str) -> Option<u64> {
    let haystack = format!("{stdout}\n{stderr}");
    for line in haystack.lines() {
        if line.to_ascii_lowercase().contains("filtered") {
            let digits: String = line.chars().filter(char::is_ascii_digit).collect();
            if let Ok(parsed) = digits.parse::<u64>() {
                return Some(parsed);
            }
        }
    }
    None
}

fn parse_first_u64_after_key(text: &str, key: &str) -> Option<u64> {
    for line in text.lines() {
        if !line.to_ascii_lowercase().contains(&key.to_ascii_lowercase()) {
            continue;
        }
        let digits: String = line.chars().filter(char::is_ascii_digit).collect();
        if let Ok(parsed) = digits.parse::<u64>() {
            return Some(parsed);
        }
    }
    None
}

fn parse_validate_reads_metrics(
    out_dir: &std::path::Path,
    execution: &StageResultV1,
) -> serde_json::Value {
    let report_path = out_dir.join("validation.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_stages_fastq::observer::parse_validation_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.validate_reads",
                "validator": report.tool_id,
                "validation_mode": report.validation_mode,
                "pair_sync_policy": report.pair_sync_policy,
                "validated_inputs": report.validated_inputs,
                "validated_reads_r1": report.validated_reads_r1,
                "validated_reads_r2": report.validated_reads_r2,
                "validated_pairs": report.validated_pairs,
                "status_r1": report.status_r1,
                "status_r2": report.status_r2,
                "pair_sync_checked": report.pair_sync_checked,
                "pair_sync_pass": report.pair_sync_pass,
                "pair_count_match": report.pair_count_match,
                "failure_class": report.failure_class,
                "strict_pass": report.strict_pass,
                "exit_code": report.exit_code,
                "report_json": report_path,
            });
        }
    }

    let merged = format!("{}\n{}", execution.stdout, execution.stderr);
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.validate_reads",
        "validator": "tool_stdout_stderr_parser",
        "validated_inputs": parse_first_u64_after_key(&merged, "read")
            .or_else(|| parse_first_u64_after_key(&merged, "sequences")),
        "failure_class": serde_json::Value::Null,
        "strict_pass": execution.exit_code == 0,
        "exit_code": execution.exit_code,
    })
}

pub(crate) fn parse_trim_terminal_damage_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("trim_terminal_damage_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_stages_fastq::observer::parse_terminal_damage_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.trim_terminal_damage",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "damage_mode": report.damage_mode,
                "execution_policy": report.execution_policy,
                "trim_5p_bases": report.trim_5p_bases,
                "trim_3p_bases": report.trim_3p_bases,
                "requested_trim_5p_bases": report.requested_trim_5p_bases,
                "requested_trim_3p_bases": report.requested_trim_3p_bases,
                "udg_classification": report.udg_classification,
                "ct_ga_asymmetry_pre": report.ct_ga_asymmetry_pre,
                "ct_ga_asymmetry_post": report.ct_ga_asymmetry_post,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.trim_terminal_damage",
        "tool": "report_missing",
        "udg_classification": serde_json::Value::Null,
        "ct_ga_asymmetry_pre": serde_json::Value::Null,
        "ct_ga_asymmetry_post": serde_json::Value::Null,
        "report_json": report_path,
    })
}

pub(crate) fn parse_trim_reads_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let report_path = out_dir.join("trim_report.json");
    if let Ok(raw) = std::fs::read_to_string(&report_path) {
        if let Ok(report) = bijux_dna_stages_fastq::observer::parse_trim_reads_report(&raw) {
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.trim_reads",
                "tool": report.tool_id,
                "paired_mode": report.paired_mode,
                "min_length": report.min_length,
                "quality_cutoff": report.quality_cutoff,
                "adapter_policy": report.adapter_policy,
                "polyx_policy": report.polyx_policy,
                "n_policy": report.n_policy,
                "contaminant_policy": report.contaminant_policy,
                "adapter_bank_id": report.adapter_bank_id,
                "adapter_bank_hash": report.adapter_bank_hash,
                "adapter_preset": report.adapter_preset,
                "polyx_bank_id": report.polyx_bank_id,
                "polyx_bank_hash": report.polyx_bank_hash,
                "polyx_preset": report.polyx_preset,
                "contaminant_bank_id": report.contaminant_bank_id,
                "contaminant_bank_hash": report.contaminant_bank_hash,
                "contaminant_preset": report.contaminant_preset,
                "reads_in": report.reads_in,
                "reads_out": report.reads_out,
                "bases_in": report.bases_in,
                "bases_out": report.bases_out,
                "pairs_in": report.pairs_in,
                "pairs_out": report.pairs_out,
                "mean_q_before": report.mean_q_before,
                "mean_q_after": report.mean_q_after,
                "runtime_s": report.runtime_s,
                "memory_mb": report.memory_mb,
                "raw_backend_report_format": report.raw_backend_report_format,
                "report_json": report_path,
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.trim_reads",
        "tool": "report_missing",
        "report_json": report_path,
    })
}

fn parse_detect_adapters_metrics(out_dir: &std::path::Path) -> serde_json::Value {
    let fastp_json = out_dir.join("fastp.json");
    if let Ok(raw) = std::fs::read_to_string(&fastp_json) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw) {
            let adapter_cut = parsed
                .pointer("/adapter_cutting/adapter_trimmed_reads")
                .and_then(serde_json::Value::as_u64);
            let total = parsed
                .pointer("/summary/before_filtering/total_reads")
                .and_then(serde_json::Value::as_u64);
            let fraction = match (adapter_cut, total) {
                (Some(cut), Some(t)) if t > 0 => {
                    let cut_f = cut.to_string().parse::<f64>().ok();
                    let total_f = t.to_string().parse::<f64>().ok();
                    match (cut_f, total_f) {
                        (Some(c), Some(total_reads)) => Some(c / total_reads),
                        _ => None,
                    }
                }
                _ => None,
            };
            return serde_json::json!({
                "schema_version": "bijux.fastq_stage_metrics.v1",
                "stage": "fastq.detect_adapters",
                "adapter_inference": {
                    "source": "fastp",
                    "adapter_trimmed_reads": adapter_cut,
                    "reads_total": total,
                    "adapter_trimmed_fraction": fraction,
                }
            });
        }
    }
    serde_json::json!({
        "schema_version": "bijux.fastq_stage_metrics.v1",
        "stage": "fastq.detect_adapters",
        "adapter_inference": {
            "detected": out_dir.join("fastqc").exists(),
            "source": "stage_outputs",
            "output_dir": out_dir.join("fastqc"),
        },
    })
}

fn stage_network_policy(stage_id: &str) -> NetworkPolicy {
    match stage_id {
        "fastq.validate_reads"
        | "fastq.detect_adapters"
        | "fastq.trim_terminal_damage"
        | "fastq.trim_reads"
        | "fastq.merge_pairs"
        | "fastq.remove_duplicates"
        | "fastq.correct_errors"
        | "fastq.filter_reads"
        | "fastq.filter_low_complexity"
        | "fastq.trim_polyg_tails"
        | "fastq.screen_taxonomy" => NetworkPolicy::Forbid,
        _ => NetworkPolicy::Allow,
    }
}

fn fastq_backend_allowlist(stage_id: &str) -> Option<Vec<String>> {
    if !stage_id.starts_with("fastq.") {
        return None;
    }
    let tools = bijux_dna_planner_fastq::stage_api::allowed_tools_for_stage(
        &bijux_dna_core::ids::StageId::new(stage_id.to_string()),
    );
    Some(
        tools
            .into_iter()
            .map(|tool| tool.to_string())
            .collect::<Vec<_>>(),
    )
}

fn enforce_fastq_backend_allowlist(stage_id: &str, tool_id: &str) -> Result<()> {
    let Some(allowed) = fastq_backend_allowlist(stage_id) else {
        return Ok(());
    };
    if allowed.iter().any(|allowed_tool| allowed_tool == tool_id) {
        return Ok(());
    }
    Err(anyhow!(
        "unsupported backend for {stage_id}: `{tool_id}` not in allowlist {}",
        allowed.join(",")
    ))
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};

    use anyhow::Result;
    use bijux_dna_runner::step_runner::StageResultV1;

    use super::{
        fastq_backend_allowlist, parse_trim_terminal_damage_metrics, parse_validate_reads_metrics,
        required_metrics_keys,
        workspace_root_path,
    };

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        key: &'static str,
        value: Option<String>,
    }

    impl EnvGuard {
        fn capture(key: &'static str) -> Self {
            Self {
                key,
                value: std::env::var(key).ok(),
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = self.value.take() {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn fastq_backend_allowlist_matches_planner_registry_selection() -> Result<()> {
        let _lock = env_lock().lock().expect("lock env mutation tests");
        let _include_guard = EnvGuard::capture("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
        let _api_guard = EnvGuard::capture("BIJUX_EXPERIMENTAL_TOOLS");
        std::env::remove_var("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
        std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS");

        let stages_dir = workspace_root_path().join("domain/fastq/stages");
        for entry in std::fs::read_dir(&stages_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let raw = std::fs::read_to_string(&path)?;
            let Some(stage_id) = raw
                .lines()
                .find_map(|line| line.strip_prefix("stage_id: "))
                .map(|value| value.trim().trim_matches('"').to_string())
            else {
                continue;
            };
            let expected = bijux_dna_planner_fastq::stage_api::allowed_tools_for_stage(
                &bijux_dna_core::ids::StageId::new(stage_id.clone()),
            )
            .into_iter()
            .map(|tool| tool.to_string())
            .collect::<Vec<_>>();
            let actual = fastq_backend_allowlist(&stage_id)
                .unwrap_or_default();
            assert_eq!(
                actual, expected,
                "fastq API backend allowlist drifted from planner registry selection for {stage_id}"
            );
        }
        Ok(())
    }

    #[test]
    fn fastq_backend_allowlist_loads_experimental_registry_with_env_toggle() {
        let _lock = env_lock().lock().expect("lock env mutation tests");
        let _include_guard = EnvGuard::capture("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
        let _api_guard = EnvGuard::capture("BIJUX_EXPERIMENTAL_TOOLS");
        std::env::remove_var("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
        std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS");

        let governed = fastq_backend_allowlist("fastq.trim_reads").unwrap_or_default();
        assert!(
            !governed.iter().any(|tool| tool == "prinseq"),
            "experimental trim backend must stay out of governed API allowlists"
        );

        std::env::set_var("BIJUX_EXPERIMENTAL_TOOLS", "1");
        let experimental = fastq_backend_allowlist("fastq.trim_reads").unwrap_or_default();
        assert!(
            experimental.iter().any(|tool| tool == "prinseq"),
            "API allowlist must include experimental trim backends when the registry toggle is enabled"
        );
    }

    #[test]
    fn report_qc_uses_stage_specific_metrics_policy() {
        assert_eq!(
            required_metrics_keys("fastq.report_qc"),
            &["schema_version", "stage", "report_html", "report_data_dir"]
        );
    }

    #[test]
    fn validate_reads_uses_governed_report_metrics_policy() {
        assert_eq!(
            required_metrics_keys("fastq.validate_reads"),
            &[
                "schema_version",
                "stage",
                "validator",
                "failure_class",
                "strict_pass",
                "exit_code",
            ]
        );
    }

    #[test]
    fn parse_validate_reads_metrics_prefers_governed_validation_report() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let out_dir = temp.path();
        std::fs::write(
            out_dir.join("validation.json"),
            serde_json::json!({
                "schema_version": "bijux.fastq.validate.report.v1",
                "stage": "fastq.validate_reads",
                "stage_id": "fastq.validate_reads",
                "tool_id": "seqtk",
                "validation_mode": "strict",
                "pair_sync_policy": "require_header_sync",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "validation_log_r1": "validation_r1.log",
                "validation_log_r2": "validation_r2.log",
                "validated_inputs": 2,
                "validated_reads_r1": 42,
                "validated_reads_r2": 41,
                "validated_pairs": 41,
                "status_r1": 0,
                "status_r2": 0,
                "pair_sync_checked": true,
                "pair_sync_pass": false,
                "pair_count_match": false,
                "failure_class": "pair_count_mismatch",
                "strict_pass": false,
                "exit_code": 96
            })
            .to_string(),
        )?;

        let metrics = parse_validate_reads_metrics(
            out_dir,
            &StageResultV1 {
                run_id: "run-1".to_string(),
                runtime_s: 1.0,
                memory_mb: 32.0,
                exit_code: 1,
                outputs: Vec::new(),
                metrics_path: None,
                stdout: "ignored".to_string(),
                stderr: "ignored".to_string(),
                command: "seqtk".to_string(),
            },
        );

        assert_eq!(metrics["validator"], serde_json::json!("seqtk"));
        assert_eq!(
            metrics["failure_class"],
            serde_json::json!("pair_count_mismatch")
        );
        assert_eq!(metrics["validated_reads_r1"], serde_json::json!(42));
        assert_eq!(metrics["validated_reads_r2"], serde_json::json!(41));
        assert_eq!(metrics["pair_sync_pass"], serde_json::json!(false));
        assert_eq!(metrics["exit_code"], serde_json::json!(96));
        assert_eq!(
            metrics["report_json"],
            serde_json::json!(Path::new(out_dir).join("validation.json"))
        );
        Ok(())
    }

    #[test]
    fn trim_terminal_damage_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("trim_terminal_damage_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.trim_terminal_damage.report.v2",
                "stage": "fastq.trim_terminal_damage",
                "stage_id": "fastq.trim_terminal_damage",
                "tool_id": "cutadapt",
                "paired_mode": "single_end",
                "damage_mode": "ancient",
                "execution_policy": "explicit_terminal_trim",
                "trim_5p_bases": 2,
                "trim_3p_bases": 2,
                "requested_trim_5p_bases": 2,
                "requested_trim_3p_bases": 2,
                "udg_classification": "non_udg",
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "trimmed.fastq.gz",
                "output_r2": null,
                "reads_in": null,
                "reads_out": null,
                "bases_in": null,
                "bases_out": null,
                "mean_q_before": null,
                "mean_q_after": null,
                "ct_ga_asymmetry_pre": 0.42,
                "ct_ga_asymmetry_post": 0.11,
                "ct_ga_asymmetry_pre_r1": null,
                "ct_ga_asymmetry_post_r1": null,
                "ct_ga_asymmetry_pre_r2": null,
                "ct_ga_asymmetry_post_r2": null,
                "terminal_base_composition_pre_r1": null,
                "terminal_base_composition_post_r1": null,
                "terminal_base_composition_pre_r2": null,
                "terminal_base_composition_post_r2": null,
                "raw_backend_report": "cutadapt.raw.json",
                "raw_backend_report_format": "cutadapt_json",
                "runtime_s": null,
                "memory_mb": null
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_trim_terminal_damage_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("cutadapt"));
        assert_eq!(metrics["execution_policy"], serde_json::json!("explicit_terminal_trim"));
        assert_eq!(metrics["ct_ga_asymmetry_post"], serde_json::json!(0.11));
    }

    #[test]
    fn trim_reads_standardized_metrics_prefer_governed_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("trim_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.trim_reads.report.v2",
                "stage": "fastq.trim_reads",
                "stage_id": "fastq.trim_reads",
                "tool_id": "fastp",
                "paired_mode": "single_end",
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "trimmed.fastq.gz",
                "output_r2": null,
                "min_length": 30,
                "quality_cutoff": 20,
                "adapter_policy": "bank",
                "polyx_policy": "trim",
                "n_policy": "drop",
                "contaminant_policy": "none",
                "adapter_bank_id": "illumina",
                "adapter_bank_hash": "sha256:adapter",
                "adapter_preset": "default",
                "polyx_bank_id": "polyx",
                "polyx_bank_hash": "sha256:polyx",
                "polyx_preset": "illumina_twocolor",
                "contaminant_bank_id": null,
                "contaminant_bank_hash": null,
                "contaminant_preset": null,
                "reads_in": 100,
                "reads_out": 92,
                "bases_in": 1000,
                "bases_out": 850,
                "pairs_in": null,
                "pairs_out": null,
                "mean_q_before": 28.0,
                "mean_q_after": 30.0,
                "runtime_s": 5.0,
                "memory_mb": 64.0,
                "raw_backend_report": "trim.fastp.json",
                "raw_backend_report_format": "fastp_json"
            })
            .to_string(),
        )
        .expect("write report");

        let metrics = parse_trim_reads_metrics(temp.path());
        assert_eq!(metrics["tool"], serde_json::json!("fastp"));
        assert_eq!(metrics["adapter_policy"], serde_json::json!("bank"));
        assert_eq!(metrics["reads_in"], serde_json::json!(100));
        assert_eq!(metrics["reads_out"], serde_json::json!(92));
        assert_eq!(
            metrics["raw_backend_report_format"],
            serde_json::json!("fastp_json")
        );
    }
}

fn workspace_root_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map_or_else(|| PathBuf::from("."), std::path::Path::to_path_buf)
}

fn required_fastq_tools() -> Result<std::collections::BTreeSet<String>> {
    let raw = std::fs::read_to_string(
        workspace_root_path().join("configs/ci/tools/required_tools.toml"),
    )?;
    let parsed: toml::Value = toml::from_str(&raw)?;
    let mut set = std::collections::BTreeSet::new();
    let items = parsed
        .get("required_tools")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow!("missing required_tools in required_tools.toml"))?;
    for item in items {
        if let Some(id) = item.as_str() {
            set.insert(id.to_string());
        }
    }
    Ok(set)
}

fn enforce_screen_db_governance(planned: &ExecutionStep) -> Result<()> {
    let stage = planned.step_id.as_str();
    if !matches!(
        stage,
        "fastq.screen_taxonomy" | "fastq.deplete_rrna" | "fastq.deplete_host" | "fastq.deplete_reference_contaminants"
    ) {
        return Ok(());
    }
    let template = planned.command.template.join(" ");
    if template.contains("http://") || template.contains("https://") {
        return Err(anyhow!(
            "{stage} may not fetch databases over network at runtime; use pre-mounted references"
        ));
    }
    if template.contains("download") || template.contains("pull") {
        return Err(anyhow!(
            "{stage} command contains database fetch verbs; require immutable pre-resolved DB paths"
        ));
    }
    Ok(())
}

fn required_metrics_keys(stage_id: &str) -> &'static [&'static str] {
    match stage_id {
        "fastq.validate_reads" => &[
            "schema_version",
            "stage",
            "validator",
            "failure_class",
            "strict_pass",
            "exit_code",
        ],
        "fastq.detect_adapters" => &["schema_version", "stage", "adapter_inference"],
        "fastq.trim_reads" => &[
            "schema_version",
            "stage",
            "tool",
            "adapter_policy",
            "reads_in",
            "reads_out",
        ],
        "fastq.trim_terminal_damage" => &[
            "schema_version",
            "stage",
            "execution_policy",
            "udg_classification",
            "ct_ga_asymmetry_pre",
            "ct_ga_asymmetry_post",
        ],
        "fastq.merge_pairs" => &["schema_version", "stage", "tool", "paired_input", "merged_output"],
        "fastq.remove_duplicates" => &["schema_version", "stage", "tool", "duplicates_removed"],
        "fastq.correct_errors" => &["schema_version", "stage", "tool", "corrected_reads"],
        "fastq.filter_reads" => &["schema_version", "stage", "tool", "filtered_reads"],
        "fastq.filter_low_complexity" => &["schema_version", "stage", "tool", "low_complexity_removed"],
        "fastq.trim_polyg_tails" => &["schema_version", "stage", "tool", "trimmed_reads"],
        "fastq.screen_taxonomy" => &["schema_version", "stage", "tool", "taxonomy_profile"],
        "fastq.deplete_reference_contaminants" => &["schema_version", "stage", "tool", "screening_results"],
        "fastq.deplete_host" => &["schema_version", "stage", "tool", "host_removed_fraction"],
        "fastq.report_qc" => &["schema_version", "stage", "report_html", "report_data_dir"],
        _ => &["schema_version", "stage"],
    }
}

fn enforce_metrics_schema(stage_root: &std::path::Path, stage_id: &str) -> Result<()> {
    let metrics_path = stage_root.join("metrics.json");
    let raw = std::fs::read_to_string(&metrics_path)
        .with_context(|| format!("reading metrics {}", metrics_path.display()))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("parsing metrics {}", metrics_path.display()))?;
    let required = required_metrics_keys(stage_id);
    for key in required {
        if parsed.get(*key).is_none() {
            return Err(anyhow!(
                "metrics schema violation for {stage_id}: missing key `{key}` in {}",
                metrics_path.display()
            ));
        }
    }
    Ok(())
}

fn count_fastq_reads_if_plain(path: &std::path::Path) -> Option<u64> {
    let ext = path.extension().and_then(|x| x.to_str()).unwrap_or_default();
    if ext == "gz" {
        return None;
    }
    let file = std::fs::File::open(path).ok()?;
    let lines = std::io::BufReader::new(file).lines().count() as u64;
    Some(lines / 4)
}

fn write_retention_report(stage_root: &std::path::Path, planned: &ExecutionStep) -> Result<()> {
    let out_dir = stage_root.join("out");
    let mut rows = vec!["artifact\treads_estimate".to_string()];
    if let Ok(entries) = std::fs::read_dir(&out_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let reads = count_fastq_reads_if_plain(&path)
                .map_or_else(|| "na".to_string(), |x| x.to_string());
            rows.push(format!("{name}\t{reads}"));
        }
    }
    let payload = rows.join("\n") + "\n";
    std::fs::write(stage_root.join("retention_report.tsv"), payload)?;
    std::fs::write(
        stage_root.join("retention_report.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "bijux.fastq.retention_report.v1",
            "stage_id": planned.step_id.0,
            "out_dir": out_dir,
            "artifacts": rows.len().saturating_sub(1),
        }))?,
    )?;
    Ok(())
}

fn classify_failure_hint(stage_id: &str, stdout: &str, stderr: &str) -> String {
    let merged = format!("{stdout}\n{stderr}").to_ascii_lowercase();
    if merged.contains("out of memory") || merged.contains("killed") {
        return "resource_exhausted_memory".to_string();
    }
    if merged.contains("no space left") {
        return "resource_exhausted_disk".to_string();
    }
    if merged.contains("permission denied") {
        return "filesystem_permissions".to_string();
    }
    if merged.contains("not found") || merged.contains("no such file") {
        return "missing_input_or_tool".to_string();
    }
    format!("{stage_id}_execution_failure")
}

fn write_retry_policy(root: &std::path::Path) -> Result<()> {
    let payload = serde_json::json!({
        "schema_version": "bijux.retry_policy.v1",
        "max_retries": 0,
        "note": "fastq preprocessing stages are deterministic and should not auto-retry by default"
    });
    std::fs::write(root.join("retry_policy.json"), serde_json::to_vec_pretty(&payload)?)?;
    Ok(())
}
