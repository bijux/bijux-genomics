use bijux_dna_stages_bam::implemented_stages;

fn main() -> anyhow::Result<()> {
    let mut stage_ids = implemented_stages()
        .into_iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<Vec<_>>();
    stage_ids.sort();

    for required in ["bam.align", "bam.mapping_summary", "bam.coverage"] {
        anyhow::ensure!(
            stage_ids.iter().any(|stage| stage == required),
            "missing governed BAM stage: {required}"
        );
    }

    let payload = serde_json::json!({
        "example": "bam_release_surface",
        "implemented_stage_count": stage_ids.len(),
        "implemented_stages": stage_ids,
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}
