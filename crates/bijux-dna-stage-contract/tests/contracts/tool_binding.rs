use std::collections::BTreeMap;

use anyhow::Result;
use bijux_dna_core::contract::ToolConstraints;
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_stage_contract::{
    render_tool_command, validate_tool_binding, BindingInputV1, BindingOutputV1,
    BindingParameterV1, OutputDiscoveryV1, ToolArtifactSourceV1, ToolBindingRegistry,
    ToolBindingRenderContext, ToolBindingV1, TOOL_BINDING_SCHEMA_V1,
};

#[test]
fn binding_renders_declared_projections_without_tool_specific_code() -> Result<()> {
    let binding = binding(
        "fastq.trim_reads",
        ToolArtifactSourceV1::CatalogImage { catalog_tool_id: ToolId::new("fastp") },
    );
    validate_tool_binding(&binding)?;
    let context = ToolBindingRenderContext {
        executable: "/images/fastp.sif".to_string(),
        inputs: BTreeMap::from([("reads".to_string(), "reads.fastq.gz".to_string())]),
        parameters: BTreeMap::from([("threads".to_string(), "8".to_string())]),
        outputs: BTreeMap::from([("trimmed".to_string(), "trimmed.fastq.gz".to_string())]),
    };

    assert_eq!(
        render_tool_command(&binding, &context)?,
        ["/images/fastp.sif", "--in=reads.fastq.gz", "--out", "trimmed.fastq.gz", "--thread", "8",]
    );
    Ok(())
}

#[test]
fn registry_allows_one_tool_to_bind_multiple_stages() -> Result<()> {
    let mut registry = ToolBindingRegistry::default();
    registry.insert(binding("fastq.trim_reads", catalog_source()))?;
    registry.insert(binding("fastq.quality_filter", catalog_source()))?;

    assert_eq!(registry.bindings_for_tool(&ToolId::new("fastp")).len(), 2);
    assert_eq!(registry.bindings().len(), 2);
    Ok(())
}

#[test]
fn external_artifacts_use_the_same_binding_interface() -> Result<()> {
    let mut external = binding(
        "bam.ancient_dna_damage",
        ToolArtifactSourceV1::ExternalArtifact {
            artifact_kind: "operator_supplied_sif".to_string(),
            version_probe: vec!["${executable}".to_string(), "--version".to_string()],
            checksum_required: true,
        },
    );
    external.binding_id = "bam.ancient_dna_damage__angsd".to_string();
    external.tool_id = ToolId::new("angsd");
    external.inputs = vec![BindingInputV1 {
        name: "alignment".to_string(),
        artifact_contract_id: "bijux.artifact.bam.alignment.v1".to_string(),
        required: true,
    }];
    external.outputs = vec![BindingOutputV1 {
        name: "damage_report".to_string(),
        artifact_contract_id: "bijux.artifact.report.ancient_dna_damage.v1".to_string(),
        discovery: OutputDiscoveryV1::ExactPath,
    }];
    external.command_template = vec![
        "${executable}".to_string(),
        "-i".to_string(),
        "${input:alignment}".to_string(),
        "-out".to_string(),
        "${output:damage_report}".to_string(),
        "-P".to_string(),
        "${parameter:threads}".to_string(),
    ];
    external.normalizer_id = "bam.ancient_dna_damage.v1".to_string();
    external.downstream_probe_id = "report.ancient_dna_damage-readable.v1".to_string();

    validate_tool_binding(&external)?;
    let context = ToolBindingRenderContext {
        executable: "/operator/artifacts/angsd.sif".to_string(),
        inputs: BTreeMap::from([("alignment".to_string(), "sample.bam".to_string())]),
        parameters: BTreeMap::from([("threads".to_string(), "8".to_string())]),
        outputs: BTreeMap::from([("damage_report".to_string(), "damage.tsv".to_string())]),
    };
    assert_eq!(render_tool_command(&external, &context)?[0], "/operator/artifacts/angsd.sif");

    let mut registry = ToolBindingRegistry::default();
    registry.insert(external)?;
    assert!(registry
        .binding(&StageId::new("bam.ancient_dna_damage"), &ToolId::new("angsd"))
        .is_some());
    Ok(())
}

#[test]
fn registry_rejects_duplicate_stage_tool_pairs() -> Result<()> {
    let mut registry = ToolBindingRegistry::default();
    registry.insert(binding("fastq.trim_reads", catalog_source()))?;

    let Err(error) = registry.insert(binding("fastq.trim_reads", catalog_source())) else {
        anyhow::bail!("duplicate stage/tool binding was accepted");
    };
    assert!(error.to_string().contains("duplicate tool binding"));
    Ok(())
}

#[test]
fn undeclared_command_projection_fails_conformance() -> Result<()> {
    let mut invalid = binding("fastq.trim_reads", catalog_source());
    invalid.command_template.push("${input:reference}".to_string());

    let Err(error) = validate_tool_binding(&invalid) else {
        anyhow::bail!("undeclared projection was accepted");
    };
    assert!(error.to_string().contains("undeclared input `reference`"));
    Ok(())
}

fn catalog_source() -> ToolArtifactSourceV1 {
    ToolArtifactSourceV1::CatalogImage { catalog_tool_id: ToolId::new("fastp") }
}

fn binding(stage_id: &str, artifact_source: ToolArtifactSourceV1) -> ToolBindingV1 {
    ToolBindingV1 {
        schema_version: TOOL_BINDING_SCHEMA_V1.to_string(),
        binding_id: format!("{stage_id}__fastp"),
        stage_id: StageId::new(stage_id),
        tool_id: ToolId::new("fastp"),
        artifact_source,
        capabilities: vec!["paired_or_single_reads".to_string()],
        inputs: vec![BindingInputV1 {
            name: "reads".to_string(),
            artifact_contract_id: "bijux.artifact.fastq.reads.v1".to_string(),
            required: true,
        }],
        parameters: vec![BindingParameterV1 { name: "threads".to_string(), required: true }],
        outputs: vec![BindingOutputV1 {
            name: "trimmed".to_string(),
            artifact_contract_id: "bijux.artifact.fastq.reads.v1".to_string(),
            discovery: OutputDiscoveryV1::ExactPath,
        }],
        command_template: vec![
            "${executable}".to_string(),
            "--in=${input:reads}".to_string(),
            "--out".to_string(),
            "${output:trimmed}".to_string(),
            "--thread".to_string(),
            "${parameter:threads}".to_string(),
        ],
        resources: ToolConstraints {
            runtime: "apptainer".to_string(),
            mem_gb: 4,
            tmp_gb: 2,
            threads: 8,
        },
        normalizer_id: "fastq.trim_reads.v1".to_string(),
        downstream_probe_id: "fastq.reads-readable.v1".to_string(),
        limitations: vec!["interleaved_reads_not_supported".to_string()],
    }
}
