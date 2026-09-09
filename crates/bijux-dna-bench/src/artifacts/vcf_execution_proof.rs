//! Owner: benchmark artifact interpretation.
//! Versioned VCF execution-report interpretation, independent of filesystem acquisition.

use anyhow::{ensure, Result};
use serde_json::Value;

pub(crate) fn report_contract(stage: &str) -> Option<(&'static str, &'static str, &'static str)> {
    Some(match stage {
        "vcf.admixture" => {
            ("admixture_json", "bijux.bench.local_vcf_admixture_smoke.v1", "tool_id")
        }
        "vcf.phasing" => ("phasing_manifest", "bijux.vcf.phasing.manifest.v1", "backend"),
        "vcf.ibd" => ("ibd_json", "bijux.bench.local_vcf_ibd_smoke.v1", "tool_id"),
        "vcf.pca" => ("pca_json", "bijux.bench.local_vcf_pca_smoke.v1", "tool_id"),
        "vcf.population_structure" => {
            ("source_population_structure_json", "bijux.vcf.population_structure.v1", "toolchain")
        }
        "vcf.demography" => {
            ("demography_json", "bijux.bench.local_vcf_demography_smoke.v1", "tool_id")
        }
        _ => return None,
    })
}

pub(crate) fn missing_adapter_blocker(stage: &str, tool: &str) -> String {
    format!("unverified_execution_report: VCF stage `{stage}` / `{tool}` has no admitted execution-proof adapter")
}

pub(crate) fn execution_blocker(stage: &str, tool: &str, report: &Value) -> Result<Option<String>> {
    let Some((_, schema, producer_field)) = report_contract(stage) else {
        return Ok(Some(missing_adapter_blocker(stage, tool)));
    };
    ensure!(
        !tool.is_empty() && tool.trim() == tool,
        "VCF execution proof requires a canonical tool identity"
    );
    ensure!(
        report.get("schema_version").and_then(Value::as_str) == Some(schema),
        "VCF execution report has an unknown or missing schema for `{stage}`"
    );
    ensure!(
        report.get(producer_field).and_then(Value::as_str) == Some(tool),
        "VCF execution report producer does not match selected tool `{tool}`"
    );
    // The population domain report has no stage envelope; its versioned schema owns that identity.
    if stage != "vcf.population_structure" {
        ensure!(
            report.get("stage_id").and_then(Value::as_str) == Some(stage),
            "VCF execution report has a missing or mismatched stage"
        );
    }
    if stage == "vcf.population_structure" {
        return Ok(population_structure_execution_blocker(tool, report));
    }
    if stage == "vcf.phasing" {
        if report.get("backend_execution_attempted").and_then(Value::as_bool) != Some(true) {
            return Ok(Some(format!("selected VCF tool `{tool}` was not executed; phasing smoke reported `backend_execution_attempted=false`")));
        }
    } else if stage == "vcf.ibd" {
        let execution_mode =
            report.get("execution_mode").and_then(Value::as_str).unwrap_or("missing");
        let field = format!("{tool}_tool_ok");
        let selected_tool_ok = report.get(&field).and_then(Value::as_bool).unwrap_or(false);
        if execution_mode != "tool_backed" || !selected_tool_ok {
            return Ok(Some(format!("selected VCF tool `{tool}` was not executed; smoke reported execution_mode `{execution_mode}` and `{field}={selected_tool_ok}`")));
        }
    } else if report.get("tool_ok").and_then(Value::as_bool) != Some(true) {
        return Ok(Some(format!(
            "selected VCF tool `{tool}` was not executed; smoke reported `tool_ok=false`"
        )));
    }
    Ok(None)
}

fn population_structure_execution_blocker(
    selected_tool_id: &str,
    report: &serde_json::Value,
) -> Option<String> {
    let pca_tool_id = report.pointer("/pca/tool_id").and_then(serde_json::Value::as_str);
    let pca_tool_ok = report.pointer("/pca/tool_ok").and_then(serde_json::Value::as_bool);
    let expected_admixture_tool_id =
        if selected_tool_id == "eigensoft" { "plink2" } else { selected_tool_id };
    let admixture_tool_id =
        report.pointer("/admixture/tool_id").and_then(serde_json::Value::as_str);
    let admixture_tool_ok =
        report.pointer("/admixture/tool_ok").and_then(serde_json::Value::as_bool);
    let expected_prune_tool_id = if selected_tool_id == "plink" { "plink" } else { "plink2" };
    let prune_tool_id =
        report.pointer("/tool_attempts/prune/tool").and_then(serde_json::Value::as_str);
    let prune_tool_ok =
        report.pointer("/tool_attempts/prune/ok").and_then(serde_json::Value::as_bool);
    let mds_tool_id = report.pointer("/tool_attempts/mds/tool").and_then(serde_json::Value::as_str);
    let mds_tool_ok = report.pointer("/tool_attempts/mds/ok").and_then(serde_json::Value::as_bool);
    let mds_plan_ok =
        selected_tool_id != "plink" || (mds_tool_id == Some("plink") && mds_tool_ok == Some(true));
    if report.get("toolchain").and_then(serde_json::Value::as_str) == Some(selected_tool_id)
        && pca_tool_id == Some(selected_tool_id)
        && pca_tool_ok == Some(true)
        && admixture_tool_id == Some(expected_admixture_tool_id)
        && admixture_tool_ok == Some(true)
        && prune_tool_id == Some(expected_prune_tool_id)
        && prune_tool_ok == Some(true)
        && mds_plan_ok
    {
        return None;
    }
    Some(format!(
        "selected VCF tool `{selected_tool_id}` did not complete its population-structure component plan; PCA `{}` ok={}, admixture `{}` ok={}, prune `{}` ok={}, MDS `{}` ok={}",
        pca_tool_id.unwrap_or("missing"),
        pca_tool_ok.unwrap_or(false),
        admixture_tool_id.unwrap_or("missing"),
        admixture_tool_ok.unwrap_or(false),
        prune_tool_id.unwrap_or("missing"),
        prune_tool_ok.unwrap_or(false),
        mds_tool_id.unwrap_or("not_applicable"),
        mds_tool_ok.unwrap_or(selected_tool_id != "plink"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn population_structure_admission_requires_exact_successful_component_plan() {
        for (tool_id, admixture_tool_id, prune_tool_id) in [
            ("eigensoft", "plink2", "plink2"),
            ("plink", "plink", "plink"),
            ("plink2", "plink2", "plink2"),
        ] {
            let report = serde_json::json!({
                "toolchain": tool_id,
                "tool_attempts": {
                    "prune": {"tool": prune_tool_id, "ok": true},
                    "mds": {
                        "tool": if tool_id == "plink" { Some("plink") } else { None },
                        "ok": if tool_id == "plink" { Some(true) } else { None }
                    }
                },
                "pca": {"tool_id": tool_id, "tool_ok": true},
                "admixture": {"tool_id": admixture_tool_id, "tool_ok": true}
            });
            assert_eq!(population_structure_execution_blocker(tool_id, &report), None);

            let mut failed_report = report;
            failed_report["admixture"]["tool_ok"] = serde_json::Value::Bool(false);
            let blocker = population_structure_execution_blocker(tool_id, &failed_report)
                .expect("failed component must block admission");
            assert!(blocker.contains(&format!("selected VCF tool `{tool_id}`")));
            assert!(blocker.contains("admixture"));
            assert!(blocker.contains("ok=false"));
        }
    }

    #[test]
    fn vcf_execution_proof_rejects_wrong_schema_stage_and_producer() -> Result<()> {
        let stage = "vcf.pca";
        let tool = "plink2";
        let valid = serde_json::json!({"schema_version":"bijux.bench.local_vcf_pca_smoke.v1","stage_id":stage,"tool_id":tool,"tool_ok":true});
        assert!(execution_blocker(stage, tool, &valid)?.is_none());
        for field in ["schema_version", "stage_id", "tool_id"] {
            let mut invalid = valid.clone();
            invalid[field] = serde_json::json!("foreign");
            assert!(execution_blocker(stage, tool, &invalid).is_err());
            invalid.as_object_mut().unwrap().remove(field);
            assert!(execution_blocker(stage, tool, &invalid).is_err());
        }
        let mut failed = valid;
        failed["tool_ok"] = serde_json::json!(false);
        assert!(execution_blocker(stage, tool, &failed)?.is_some());
        failed.as_object_mut().unwrap().remove("tool_ok");
        assert!(execution_blocker(stage, tool, &failed)?.is_some());
        Ok(())
    }

    #[test]
    fn vcf_execution_proof_requires_stage_specific_backend_success() -> Result<()> {
        for stage in ["vcf.admixture", "vcf.pca", "vcf.demography"] {
            let (_, schema, _) =
                report_contract(stage).ok_or_else(|| anyhow::anyhow!("missing report contract"))?;
            let report = serde_json::json!({"schema_version":schema,"stage_id":stage,"tool_id":"plink2","tool_ok":true});
            assert!(execution_blocker(stage, "plink2", &report)?.is_none());
        }
        let mut phasing = serde_json::json!({"schema_version":"bijux.vcf.phasing.manifest.v1","stage_id":"vcf.phasing","backend":"shapeit","backend_execution_attempted":true});
        assert!(execution_blocker("vcf.phasing", "shapeit", &phasing)?.is_none());
        phasing["backend_execution_attempted"] = serde_json::json!(false);
        assert!(execution_blocker("vcf.phasing", "shapeit", &phasing)?.is_some());
        let mut ibd = serde_json::json!({"schema_version":"bijux.bench.local_vcf_ibd_smoke.v1","stage_id":"vcf.ibd","tool_id":"plink","execution_mode":"tool_backed","plink_tool_ok":true});
        assert!(execution_blocker("vcf.ibd", "plink", &ibd)?.is_none());
        ibd["execution_mode"] = serde_json::json!("synthetic");
        assert!(execution_blocker("vcf.ibd", "plink", &ibd)?.is_some());
        ibd["execution_mode"] = serde_json::json!("tool_backed");
        ibd["plink_tool_ok"] = serde_json::json!(false);
        assert!(execution_blocker("vcf.ibd", "plink", &ibd)?.is_some());
        Ok(())
    }

    #[test]
    fn vcf_execution_proof_never_admits_unregistered_report_formats() -> Result<()> {
        for stage in ["vcf.qc", "vcf.postprocess", "vcf.unknown"] {
            let report = serde_json::json!({"tool_ok":true,"exit_code":0});
            let blocker = execution_blocker(stage, "bcftools", &report)?;
            assert!(
                blocker.is_some_and(|message| message.starts_with("unverified_execution_report:"))
            );
        }
        Ok(())
    }
}
