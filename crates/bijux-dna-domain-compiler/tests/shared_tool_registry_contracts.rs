#![allow(clippy::expect_used)]

#[path = "support/mod.rs"]
mod support;

fn tools_array(path: &std::path::Path) -> anyhow::Result<Vec<toml::Value>> {
    let raw = std::fs::read_to_string(path)?;
    let value = raw.parse::<toml::Value>()?;
    Ok(value.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default())
}

fn stages_array(path: &std::path::Path) -> anyhow::Result<Vec<toml::Value>> {
    let raw = std::fs::read_to_string(path)?;
    let value = raw.parse::<toml::Value>()?;
    Ok(value.get("stages").and_then(toml::Value::as_array).cloned().unwrap_or_default())
}

#[test]
fn shared_tools_publish_multidomain_stage_bindings_in_checked_in_registries() -> anyhow::Result<()>
{
    let root = support::repo_root();
    let tool_registry = tools_array(&root.join("configs/ci/registry/tool_registry.toml"))?;
    let bowtie2 = tool_registry
        .iter()
        .find(|row| row.get("id").and_then(toml::Value::as_str) == Some("bowtie2"))
        .expect("bowtie2 tool row");
    let bowtie2_domains =
        bowtie2.get("domains").and_then(toml::Value::as_array).expect("bowtie2 domains");
    assert_eq!(
        bowtie2_domains,
        &vec![toml::Value::String("bam".to_string()), toml::Value::String("fastq".to_string())]
    );
    let bowtie2_stage_ids =
        bowtie2.get("stage_ids").and_then(toml::Value::as_array).expect("bowtie2 stage_ids");
    assert!(
        bowtie2_stage_ids.contains(&toml::Value::String("bam.align".to_string())),
        "bowtie2 registry row must retain BAM alignment admission"
    );
    assert!(
        bowtie2_stage_ids.contains(&toml::Value::String("fastq.deplete_host".to_string())),
        "bowtie2 registry row must retain FASTQ host-depletion admission"
    );
    assert!(
        bowtie2_stage_ids
            .contains(&toml::Value::String("fastq.deplete_reference_contaminants".to_string(),)),
        "bowtie2 registry row must retain FASTQ contaminant-depletion admission"
    );

    let multiqc = tool_registry
        .iter()
        .find(|row| row.get("id").and_then(toml::Value::as_str) == Some("multiqc"))
        .expect("multiqc tool row");
    let multiqc_stage_ids =
        multiqc.get("stage_ids").and_then(toml::Value::as_array).expect("multiqc stage_ids");
    assert!(
        multiqc_stage_ids.contains(&toml::Value::String("bam.qc_pre".to_string())),
        "multiqc registry row must retain BAM qc_pre reporting coverage after shared-tool merging"
    );
    assert!(
        multiqc_stage_ids.contains(&toml::Value::String("fastq.report_qc".to_string())),
        "multiqc registry row must retain FASTQ report-qc coverage after shared-tool merging"
    );

    let stages = stages_array(&root.join("configs/ci/stages/stages.toml"))?;
    let bam_align = stages
        .iter()
        .find(|row| row.get("id").and_then(toml::Value::as_str) == Some("bam.align"))
        .expect("bam.align stage row");
    assert_eq!(
        bam_align.get("tools").and_then(toml::Value::as_array),
        Some(&vec![
            toml::Value::String("bowtie2".to_string()),
            toml::Value::String("bwa".to_string()),
        ]),
        "bam.align stage registry must publish both admitted aligners"
    );

    Ok(())
}
