use super::{anyhow, Context, ExecutionStep, Result, StageResultV1};

pub(crate) fn enforce_stage_applicability(
    planned: &ExecutionStep,
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqPreprocessArgs,
    contaminant_bank: Option<&serde_json::Value>,
) -> Result<()> {
    let stage = planned.step_id.as_str();
    if stage == "fastq.merge_pairs" && args.r2.is_none() {
        return Err(anyhow!("stage fastq.merge_pairs requires paired-end input (missing R2)"));
    }
    if matches!(
        stage,
        "fastq.normalize_primers"
            | "fastq.remove_chimeras"
            | "fastq.infer_asvs"
            | "fastq.cluster_otus"
            | "fastq.normalize_abundance"
    ) && !matches!(
        args.mode,
        bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::EdnaAmplicon
            | bijux_dna_planner_fastq::stage_api::args::FastqPlannerMode::PollenAmplicon
    ) {
        return Err(anyhow!("stage {stage} is only applicable in eDNA/pollen amplicon modes"));
    }
    if stage == "fastq.deplete_reference_contaminants" {
        let template = planned.command.template.join(" ");
        let contaminant_root = declared_contaminant_asset_root()?;
        if !template.contains(&contaminant_root.display().to_string()) {
            return Err(anyhow!(
                "fastq.deplete_reference_contaminants requires contaminant assets under {}",
                contaminant_root.display()
            ));
        }
        if contaminant_bank.is_none() {
            return Err(anyhow!(
                "fastq.deplete_reference_contaminants requires contaminant bank context"
            ));
        }
    }
    Ok(())
}

fn declared_contaminant_asset_root() -> Result<std::path::PathBuf> {
    if let Some(root) = std::env::var_os("BIJUX_CONTAMINANT_ROOT")
        .filter(|value| !value.is_empty())
        .map(std::path::PathBuf::from)
    {
        return Ok(root);
    }
    if let Some(root) = std::env::var_os("BIJUX_REFERENCE_ROOT")
        .filter(|value| !value.is_empty())
        .map(std::path::PathBuf::from)
    {
        return Ok(root.join("contaminants"));
    }
    Err(anyhow!(
        "BIJUX_CONTAMINANT_ROOT or BIJUX_REFERENCE_ROOT must be declared for contaminant governance"
    ))
}

pub(crate) fn write_stage_governance_artifacts(
    stage_root: &std::path::Path,
    planned: &ExecutionStep,
    contaminant_bank: Option<&serde_json::Value>,
) -> Result<()> {
    let stage = planned.step_id.as_str();
    if !matches!(
        stage,
        "fastq.screen_taxonomy"
            | "fastq.deplete_rrna"
            | "fastq.deplete_host"
            | "fastq.deplete_reference_contaminants"
    ) {
        return Ok(());
    }
    let template = planned.command.template.join(" ");
    let lower = template.to_ascii_lowercase();
    let db_flags_present =
        [" --db ", "--database", "--index", "kraken_db", "db_path", "--ref", "--reference"]
            .iter()
            .any(|needle| lower.contains(needle));
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.governance.v1",
        "stage_id": stage,
        "db_flags_present": db_flags_present,
        "command_template": planned.command.template,
        "contaminant_bank": if stage == "fastq.deplete_reference_contaminants" { contaminant_bank.cloned() } else { None::<serde_json::Value> },
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.governance.json"), &payload)
        .context("write stage.governance.json")
}

pub(crate) fn write_fastq_output_contract(
    stage_root: &std::path::Path,
    planned: &ExecutionStep,
    execution: &StageResultV1,
) -> Result<()> {
    let declared_outputs = planned
        .io
        .outputs
        .iter()
        .map(|artifact| {
            serde_json::json!({
                "name": artifact.name,
                "role": artifact.role.as_str(),
                "path": artifact.path,
            })
        })
        .collect::<Vec<_>>();
    let emitted_outputs = execution
        .outputs
        .iter()
        .map(|path| serde_json::json!({ "path": path }))
        .collect::<Vec<_>>();
    let expected_ecological_outputs = match planned.stage_id.as_str() {
        "fastq.trim_terminal_damage" => {
            vec!["trimmed_reads_r1", "trimmed_reads_r2", "report_json"]
        }
        "fastq.normalize_primers" => vec!["primer_orientation_report"],
        "fastq.remove_chimeras" => vec!["report_json", "chimera_metrics_json"],
        "fastq.infer_asvs" => vec!["asv_table_tsv", "asv_sequences_fasta", "report_json"],
        "fastq.cluster_otus" => vec!["otu_table", "otu_representatives"],
        "fastq.normalize_abundance" => vec!["normalized_abundance_tsv"],
        _ => Vec::new(),
    };
    let ecological_checksums = planned
        .io
        .outputs
        .iter()
        .filter(|artifact| {
            expected_ecological_outputs.iter().any(|name| *name == artifact.name.as_str())
        })
        .map(|artifact| {
            let sha256 = if artifact.path.exists() {
                bijux_dna_infra::hash_file_sha256(&artifact.path).ok()
            } else {
                None
            };
            serde_json::json!({
                "name": artifact.name,
                "path": artifact.path,
                "sha256": sha256
            })
        })
        .collect::<Vec<_>>();
    let contract = serde_json::json!({
        "schema_version": "bijux.fastq.output_contract.v1",
        "stage_id": planned.stage_id,
        "step_id": planned.step_id,
        "declared_outputs": declared_outputs,
        "emitted_outputs": emitted_outputs,
        "expected_ecological_outputs": expected_ecological_outputs,
        "ecological_output_checksums": ecological_checksums,
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.output.contract.json"), &contract)
        .context("write stage output contract")
}

pub(crate) fn write_taxonomy_db_drift_report(
    run_root: &std::path::Path,
    contaminant_bank: Option<&serde_json::Value>,
) -> Result<()> {
    let report_path = run_root.join("taxonomy_db_drift.json");
    let current = contaminant_bank.cloned().unwrap_or_else(|| serde_json::json!({}));
    let lock_path = run_root.join("taxonomy_db.lock.json");
    let previous = if lock_path.exists() {
        let raw = std::fs::read_to_string(&lock_path)
            .with_context(|| format!("read {}", lock_path.display()))?;
        serde_json::from_str::<serde_json::Value>(&raw)
            .with_context(|| format!("parse {}", lock_path.display()))?
    } else {
        serde_json::json!({})
    };
    let current_hash =
        bijux_dna_core::prelude::params_hash(&current).context("hash taxonomy drift current")?;
    let previous_hash = previous
        .get("current_hash")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let drift_detected = lock_path.exists()
        && previous_hash.as_deref().is_some_and(|previous_hash| current_hash != previous_hash);
    let report = serde_json::json!({
        "schema_version": "bijux.taxonomy_db_drift.v1",
        "drift_detected": drift_detected,
        "current_hash": current_hash,
        "previous_hash": previous_hash,
        "current": current,
    });
    bijux_dna_infra::atomic_write_json(&report_path, &report).context("write taxonomy_db_drift")?;
    bijux_dna_infra::atomic_write_json(&lock_path, &report).context("write taxonomy_db lock")?;
    Ok(())
}
