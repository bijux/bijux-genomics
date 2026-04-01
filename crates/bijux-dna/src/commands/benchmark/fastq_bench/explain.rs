use crate::commands::support::prelude::{anyhow, render, Result, StageId};

pub(super) fn explain_fastq_stage(
    registry: &bijux_dna_api::v1::api::run::ToolRegistry,
    stage_id: &str,
) -> Result<()> {
    if stage_id == "fastq.trim_reads" {
        let default_profile = bijux_dna_api::v1::api::plan::select_pipeline(
            bijux_dna_api::v1::api::plan::Domain::Fastq,
            "fastq-to-fastq__default__v1",
        )?;
        let reference_profile = bijux_dna_api::v1::api::plan::select_pipeline(
            bijux_dna_api::v1::api::plan::Domain::Fastq,
            "fastq-to-fastq__reference_adna__v1",
        )?;
        let trim_stage = StageId::from_static("fastq.trim_reads");
        let param_schema = lookup_param_schema_id("fastq.trim_reads")
            .ok_or_else(|| anyhow!("param schema for fastq.trim_reads missing from registry"))?;
        let payload = serde_json::json!({
            "stage_id": "fastq.trim_reads",
            "param_schema": param_schema,
            "param_variant": "FastqTrim (effective defaults payload)",
            "defaults": {
                "fastq-default": default_profile.defaults.params.get(&trim_stage),
                "fastq-reference-adna": reference_profile.defaults.params.get(&trim_stage),
            },
            "invariants": {
                "fastq-default": bijux_dna_api::v1::api::plan::validate_fastq_profile(&default_profile),
                "fastq-reference-adna": bijux_dna_api::v1::api::plan::validate_fastq_profile(&reference_profile),
            },
            "metrics_schema": "bijux.fastq.trim_reads.v1",
        });
        render::json::print_pretty(&payload)?;
        return Ok(());
    }
    let stage_id =
        StageId::try_from(stage_id).map_err(|_| anyhow!("invalid stage id: {stage_id}"))?;
    let stage = registry
        .stages()
        .get(&stage_id)
        .ok_or_else(|| anyhow!("unknown stage {stage_id}"))?;
    println!("stage: {}", stage.stage_id);
    if let Some(description) = stage.description.as_ref() {
        if !description.is_empty() {
            println!("description: {description}");
        }
    }
    println!("inputs:");
    for input in &stage.inputs {
        println!("- {} ({})", input.name, input.data_type);
    }
    println!("outputs:");
    for output in &stage.outputs {
        println!("- {} ({})", output.name, output.data_type);
    }
    Ok(())
}

fn lookup_param_schema_id(stage_id: &str) -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root().ok()?;
    let mut roots = vec![cwd, repo_root];
    roots.sort();
    roots.dedup();
    for root in roots {
        for rel in [
            "ci/params/param_registry.toml",
            "ci/params/param_registry_vcf.toml",
        ] {
            let path = bijux_dna_infra::configs_file(&root, rel);
            if !path.exists() {
                continue;
            }
            let raw = std::fs::read_to_string(&path).ok()?;
            let parsed: toml::Value = raw.parse().ok()?;
            let rows = parsed.get("params").and_then(toml::Value::as_array)?;
            for row in rows {
                let id = row.get("stage_id").and_then(toml::Value::as_str)?;
                if id == stage_id {
                    return row
                        .get("schema_version")
                        .and_then(toml::Value::as_str)
                        .map(str::to_string);
                }
            }
        }
    }
    None
}
