use std::collections::BTreeMap;

use bijux_dna_domain_bam::alignment::AlignmentBoundary;

#[test]
fn alignment_boundary_round_trips_all_governed_fields() -> anyhow::Result<()> {
    let boundary = AlignmentBoundary {
        bam_path: "results/sample/align.bam".to_string(),
        bai_path: Some("results/sample/align.bam.bai".to_string()),
        reference: Some("references/grch38.fa".to_string()),
        rg_policy: Some("requires_read_groups".to_string()),
        aligner_meta: Some(BTreeMap::from([
            ("build_id".to_string(), "grch38-p14".to_string()),
            ("species_id".to_string(), "homo-sapiens".to_string()),
        ])),
    };

    let encoded = serde_json::to_value(&boundary)?;
    let decoded: AlignmentBoundary = serde_json::from_value(encoded)?;

    assert_eq!(decoded, boundary);
    Ok(())
}

#[test]
fn alignment_boundary_defaults_optional_fields_and_rejects_unknown_fields() -> anyhow::Result<()> {
    let minimal: AlignmentBoundary = serde_json::from_value(serde_json::json!({
        "bam_path": "results/sample/align.bam"
    }))?;

    assert_eq!(minimal.bai_path, None);
    assert_eq!(minimal.reference, None);
    assert_eq!(minimal.rg_policy, None);
    assert_eq!(minimal.aligner_meta, None);

    let error = serde_json::from_value::<AlignmentBoundary>(serde_json::json!({
        "bam_path": "results/sample/align.bam",
        "undeclared": true
    }))
    .err()
    .ok_or_else(|| anyhow::anyhow!("unknown alignment boundary fields must be rejected"))?;
    assert!(error.to_string().contains("unknown field `undeclared`"));
    Ok(())
}
