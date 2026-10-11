use super::*;

#[test]
fn report_identity_admission_rejects_duplicates_and_missing_ids_before_set_collapse() -> Result<()>
{
    let canonical = unique_result_ids("expected", ["bam:first", "fastq:second"])?;
    let reordered = unique_result_ids("commands", ["fastq:second", "bam:first"])?;
    ensure_matching_set("commands", &canonical, &reordered)?;
    for observed in [
        vec!["bam:first", "bam:first", "fastq:second"],
        vec!["bam:first", ""],
        vec!["bam:first", "  "],
        vec!["bam:first", " padded"],
    ] {
        assert!(unique_result_ids("commands", observed).is_err());
    }
    let substituted = unique_result_ids("commands", ["bam:first", "vcf:extra"])?;
    let error = ensure_matching_set("commands", &canonical, &substituted)
        .err()
        .ok_or_else(|| anyhow::anyhow!("missing and extra coverage was accepted"))?
        .to_string();
    assert!(error.contains("commands"));
    assert!(error.contains("missing=[\"fastq:second\"]"));
    assert!(error.contains("extra=[\"vcf:extra\"]"));
    Ok(())
}

#[test]
fn binding_alignment_retains_domain_stage_tool_corpus_and_asset_identity() -> Result<()> {
    let row = crate::AllDomainExpectedBenchmarkResultRow {
        result_id: "result".into(),
        domain: "bam".into(),
        stage_id: "bam.validate".into(),
        tool_id: "samtools".into(),
        corpus_id: "truth".into(),
        asset_profile_id: "diploid".into(),
        expected_outputs: vec![],
        expected_metrics: vec![],
        report_section: "validation".into(),
    };
    let table = crate::AllDomainStageToolTableRow {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
        adapter_id: "adapter".into(),
        parser_id: "parser".into(),
        benchmark_status: "benchmark_ready".into(),
    };
    let expected = BTreeSet::from([expected_binding_key(&row)]);
    ensure_matching_binding_set(
        "stage tool table",
        &expected,
        &BTreeSet::from([stage_tool_binding_key(&table)]),
    )?;
    for field in ["domain", "stage", "tool", "corpus", "asset"] {
        let mut changed = table.clone();
        match field {
            "domain" => changed.domain = "other".into(),
            "stage" => changed.stage_id = "other".into(),
            "tool" => changed.tool_id = "other".into(),
            "corpus" => changed.corpus_id = "other".into(),
            _ => changed.asset_profile_id = "other".into(),
        }
        assert!(ensure_matching_binding_set(
            "stage tool table",
            &expected,
            &BTreeSet::from([stage_tool_binding_key(&changed)])
        )
        .is_err());
    }
    Ok(())
}
