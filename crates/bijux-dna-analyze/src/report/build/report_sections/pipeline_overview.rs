use bijux_dna_runtime::FactsRowV1;

#[derive(Debug, Clone, Copy)]
enum ToolTier {
    Gold,
    Silver,
    Experimental,
}

fn tool_tier_for(stage_id: &str, tool_id: &str) -> (ToolTier, &'static str) {
    match (stage_id, tool_id) {
        ("fastq.trim_reads" | "fastq.filter_reads", "fastp") => (ToolTier::Gold, "curated_default"),
        ("fastq.profile_reads", "seqkit_stats") => (ToolTier::Silver, "diagnostic_stats"),
        _ => (ToolTier::Experimental, "unknown_tool"),
    }
}

pub(crate) fn pipeline_overview_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let stages: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            let (tier, rationale) = tool_tier_for(&row.stage_id, &row.tool_id);
            serde_json::json!({
                "stage_id": row.stage_id,
                "tool_id": row.tool_id,
                "tool_version": row.tool_version,
                "tool_tier": format!("{tier:?}").to_lowercase(),
                "tier_rationale": rationale,
                "scientific_preset": row.reports.get("scientific_preset").cloned().unwrap_or(serde_json::Value::Null),
                "params_hash": row.params_hash,
                "image_digest": row.image_digest,
                "input_hash": row.input_hash,
                "output_hashes": row.output_hashes,
            })
        })
        .collect();
    serde_json::json!({
        "stages": stages,
    })
}
