type IoDeltas = (
    Option<u64>,
    Option<u64>,
    Option<u64>,
    Option<u64>,
    Option<u64>,
    Option<u64>,
);

fn extract_io_deltas(metrics: &serde_json::Value) -> IoDeltas {
    let reads_in = metrics.get("reads_in").and_then(serde_json::Value::as_u64);
    let reads_out = metrics.get("reads_out").and_then(serde_json::Value::as_u64);
    let bases_in = metrics.get("bases_in").and_then(serde_json::Value::as_u64);
    let bases_out = metrics.get("bases_out").and_then(serde_json::Value::as_u64);
    let pairs_in = metrics.get("pairs_in").and_then(serde_json::Value::as_u64);
    let pairs_out = metrics.get("pairs_out").and_then(serde_json::Value::as_u64);
    (
        reads_in, reads_out, bases_in, bases_out, pairs_in, pairs_out,
    )
}

fn write_effective_fasta(
    run_artifacts_dir: &Path,
    name: &str,
    entries: &[BankEntryRecord],
    extra_fasta: &[String],
) -> Result<Option<(PathBuf, String)>> {
    if entries.is_empty() && extra_fasta.is_empty() {
        return Ok(None);
    }
    let banks_dir = run_artifacts_dir.join("banks");
    bijux_infra::ensure_dir(&banks_dir).context("create banks dir")?;
    let path = banks_dir.join(format!("effective_{name}.fasta"));
    let mut payload = String::new();
    for entry in entries {
        payload.push('>');
        payload.push_str(&entry.id);
        payload.push('\n');
        payload.push_str(&entry.sequence);
        payload.push('\n');
    }
    for fasta in extra_fasta {
        payload.push_str(fasta);
        if !fasta.ends_with('\n') {
            payload.push('\n');
        }
    }
    bijux_infra::atomic_write_bytes(&path, payload.as_bytes())
        .context("write effective bank fasta")?;
    let hash = hash_file_sha256(&path)?;
    Ok(Some((path, hash)))
}

fn bank_asset_name(bank_name: &str) -> &str {
    match bank_name {
        "adapter" => "adapters",
        "contaminant" => "contaminants",
        other => other,
    }
}

fn write_effective_bank_json(
    run_artifacts_dir: &Path,
    name: &str,
    bank_value: &serde_json::Value,
    entries: &[BankEntryRecord],
    references: &[BankReferenceRecord],
) -> Result<Option<(PathBuf, String)>> {
    if entries.is_empty() && references.is_empty() {
        return Ok(None);
    }
    let banks_dir = run_artifacts_dir.join("banks");
    bijux_infra::ensure_dir(&banks_dir).context("create banks dir")?;
    let path = banks_dir.join(format!("effective_{name}.json"));
    let payload = serde_json::json!({
        "bank_id": bank_value.get("bank_id"),
        "bank_hash": bank_value.get("bank_hash"),
        "preset": bank_value.get("preset"),
        "preset_hash": bank_value.get("preset_hash"),
        "enabled_entries": entries.iter().map(|entry| {
            serde_json::json!({
                "id": entry.id,
                "sequence": entry.sequence,
                "rationale": entry.rationale,
                "source": entry.source,
            })
        }).collect::<Vec<_>>(),
        "references": references.iter().map(|reference| {
            serde_json::json!({
                "id": reference.id,
                "file": reference.file,
                "sha256": reference.sha256,
                "rationale": reference.rationale,
                "source": reference.source,
            })
        }).collect::<Vec<_>>(),
    });
    let json =
        bijux_infra::formats::to_json_pretty(&payload).context("serialize effective bank json")?;
    bijux_infra::atomic_write_bytes(&path, json.as_bytes())
        .context("write effective bank json")?;
    let hash = hash_file_sha256(&path)?;
    Ok(Some((path, hash)))
}

fn write_effective_fasta_list(
    run_artifacts_dir: &Path,
    name: &str,
    references: &[BankReferenceRecord],
) -> Result<Option<(PathBuf, String)>> {
    if references.is_empty() {
        return Ok(None);
    }
    let banks_dir = run_artifacts_dir.join("banks");
    bijux_infra::ensure_dir(&banks_dir).context("create banks dir")?;
    let path = banks_dir.join(format!("effective_{name}.fasta.list"));
    let payload = references
        .iter()
        .map(|reference| reference.file.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    bijux_infra::atomic_write_bytes(&path, payload.as_bytes())
        .context("write effective bank fasta list")?;
    let hash = hash_file_sha256(&path)?;
    Ok(Some((path, hash)))
}

fn materialize_bank_assets(
    run_artifacts_dir: &Path,
    banks_value: Option<&serde_json::Value>,
) -> Result<Option<serde_json::Value>> {
    let Some(banks_value) = banks_value.and_then(|value| value.as_object()) else {
        return Ok(None);
    };
    let mut assets = serde_json::Map::new();
    for (bank_name, bank_value) in banks_value {
        let asset_name = bank_asset_name(bank_name);
        let entries = bank_entries_from_value(bank_value);
        let references = bank_references_from_value(bank_value);
        let extra_fasta: Vec<String> = references
            .iter()
            .filter_map(|reference| reference.fasta.clone())
            .collect();
        let fasta = write_effective_fasta(run_artifacts_dir, asset_name, &entries, &extra_fasta)?;
        let json = write_effective_bank_json(
            run_artifacts_dir,
            asset_name,
            bank_value,
            &entries,
            &references,
        )?;
        let fasta_list = if bank_name.as_str() == "contaminant" {
            write_effective_fasta_list(run_artifacts_dir, asset_name, &references)?
        } else {
            None
        };
        let record = serde_json::json!({
            "json": json.as_ref().map(|(path, hash)| serde_json::json!({
                "path": path.display().to_string(),
                "sha256": hash,
            })),
            "fasta": fasta.as_ref().map(|(path, hash)| serde_json::json!({
                "path": path.display().to_string(),
                "sha256": hash,
            })),
            "fasta_list": fasta_list.as_ref().map(|(path, hash)| serde_json::json!({
                "path": path.display().to_string(),
                "sha256": hash,
            })),
        });
        assets.insert(bank_name.clone(), record);
    }
    Ok(Some(serde_json::Value::Object(assets)))
}
