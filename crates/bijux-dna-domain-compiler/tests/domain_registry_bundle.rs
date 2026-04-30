use bijux_dna_domain_compiler::{
    build_domain_registry_bundle, domain_artifact_contract_snapshots, domain_defaults_snapshot,
    domain_metric_catalogs, query_domain_registry_bundle, DomainRegistryQuery,
    DomainRegistryQueryKind, DEFAULT_DOMAIN_DIR,
};

#[path = "support/mod.rs"]
mod support;

#[test]
fn bundle_compiles_all_domains_into_a_release_surface() -> anyhow::Result<()> {
    let root = support::repo_root();
    let bundle = build_domain_registry_bundle(&root.join(DEFAULT_DOMAIN_DIR), "test-source")?;

    assert_eq!(bundle.schema_version, "bijux.domain.registry.release_bundle.v1");
    assert_eq!(
        bundle
            .domains
            .iter()
            .map(|domain| domain.domain_id.as_str())
            .collect::<Vec<_>>(),
        vec!["bam", "fastq", "vcf"]
    );

    let fastq = bundle.domains.iter().find(|domain| domain.domain_id == "fastq").expect("fastq domain");
    assert!(
        fastq.stages.iter().any(|stage| stage.stage_id == "fastq.trim_reads"),
        "fastq bundle must preserve governed stage ids"
    );
    assert!(
        fastq.tools.iter().any(|tool| tool.tool_id == "fastp"),
        "fastq bundle must preserve tool ids"
    );
    assert!(
        fastq.metrics.iter().any(|metric| metric.metric_id == "read_count"),
        "fastq bundle must preserve stable metric ids"
    );
    assert!(
        fastq.artifacts.iter().any(|artifact| artifact.artifact_id == "trimmed_reads_r1"),
        "fastq bundle must preserve artifact roles"
    );
    assert!(
        fastq
            .stages
            .iter()
            .find(|stage| stage.stage_id == "fastq.trim_reads")
            .is_some_and(|stage| stage.parameters.iter().any(|parameter| parameter.name == "min_length")),
        "fastq bundle must preserve stage default parameters"
    );

    let vcf = bundle.domains.iter().find(|domain| domain.domain_id == "vcf").expect("vcf domain");
    assert_eq!(vcf.schemas.stage_schema_version, "bijux.stage.v1");
    assert!(
        vcf.defaults.iter().any(|default| default.stage_id == "vcf.call"),
        "vcf bundle must preserve domain defaults"
    );

    Ok(())
}

#[test]
fn bundle_queries_and_derived_snapshots_cover_registry_surfaces() -> anyhow::Result<()> {
    let root = support::repo_root();
    let bundle = build_domain_registry_bundle(&root.join(DEFAULT_DOMAIN_DIR), "test-source")?;

    let stages = query_domain_registry_bundle(
        &bundle,
        &DomainRegistryQuery {
            kind: DomainRegistryQueryKind::Stages,
            domain_id: Some("vcf".to_string()),
            stage_id: Some("vcf.call".to_string()),
            tool_id: None,
        },
    );
    let stage_rows = stages.as_array().expect("stage query returns array");
    assert_eq!(stage_rows.len(), 1);

    let defaults = domain_defaults_snapshot(&bundle);
    assert!(
        defaults.iter().any(|domain| {
            domain.domain_id == "bam"
                && domain.defaults.iter().any(|default| default.stage_id == "bam.align")
        }),
        "defaults snapshot must expose BAM default contracts"
    );

    let artifacts = domain_artifact_contract_snapshots(&bundle);
    assert!(
        artifacts.iter().any(|domain| {
            domain.domain_id == "vcf"
                && domain
                    .stage_outputs
                    .get("vcf.call")
                    .is_some_and(|outputs| outputs.iter().any(|output| output == "called_vcf"))
        }),
        "artifact snapshot must link VCF stage outputs to artifact roles"
    );

    let metrics = domain_metric_catalogs(&bundle);
    assert!(
        metrics.iter().any(|domain| {
            domain.domain_id == "vcf"
                && domain.metrics.iter().any(|metric| metric.metric_id == "called_variants")
        }),
        "metric catalogs must preserve VCF metric identifiers"
    );

    Ok(())
}
