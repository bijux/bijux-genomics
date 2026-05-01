use bijux_dna_stages_vcf::implemented_stages;

fn main() -> anyhow::Result<()> {
    let mut stage_ids = implemented_stages()
        .into_iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<Vec<_>>();
    stage_ids.sort();

    for required in ["vcf.call", "vcf.filter", "vcf.stats"] {
        anyhow::ensure!(
            stage_ids.iter().any(|stage| stage == required),
            "missing governed VCF stage: {required}"
        );
    }

    let payload = serde_json::json!({
        "example": "vcf_release_surface",
        "implemented_stage_count": stage_ids.len(),
        "implemented_stages": stage_ids,
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}
