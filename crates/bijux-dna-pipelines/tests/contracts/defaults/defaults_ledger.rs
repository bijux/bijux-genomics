/// Snapshot intent: verifies stable, reviewed output for this contract.
use std::path::PathBuf;

use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_pipelines::registry::{bam_profiles, cross_profiles, fastq_profiles};
use bijux_dna_pipelines::{
    DefaultParams, DefaultProvenanceV1, DefaultsLedgerV1, EmptyParams, PipelineId,
};
use bijux_dna_testkit::snapshot_name;

fn prune_bam_downstream(value: &mut serde_json::Value) {
    let banned = [
        "bam.bias_mitigation",
        "bam.genotyping",
        "bam.haplogroups",
        "bam.kinship",
    ];
    match value {
        serde_json::Value::Array(items) => {
            items.retain(|item| !item.as_str().is_some_and(|entry| banned.contains(&entry)));
            for item in items {
                prune_bam_downstream(item);
            }
        }
        serde_json::Value::Object(map) => {
            for key in banned {
                map.remove(key);
            }
            for value in map.values_mut() {
                prune_bam_downstream(value);
            }
        }
        _ => {}
    }
}

#[test]
fn defaults_ledger_snapshots_are_stable() {
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    let _guard = settings.bind_to_scope();

    let mut profiles = Vec::new();
    profiles.extend(fastq_profiles());
    profiles.extend(bam_profiles());
    profiles.extend(cross_profiles());

    for profile in profiles {
        let base = format!("defaults__{}", profile.id.as_str().replace([':', '.'], "_"));
        let name = snapshot_name("contracts", &base);
        let ledger = profile.defaults_ledger();
        let mut json = serde_json::to_value(&ledger).expect("serialize ledger");
        prune_bam_downstream(&mut json);
        insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
    }
}

fn complete_provenance() -> DefaultProvenanceV1 {
    DefaultProvenanceV1 {
        rationale: "test default".to_string(),
        assumptions: vec!["test assumption".to_string()],
        comparability_implications: vec!["test comparability".to_string()],
        citations: vec!["test citation".to_string()],
    }
}

#[test]
fn strict_defaults_ledger_rejects_missing_provenance() {
    let stage = StageId::from_static("core.prepare_reference");
    let mut ledger = DefaultsLedgerV1 {
        pipeline_id: PipelineId::from_static("fastq-to-bam__default__v1"),
        tools: [(stage.clone(), ToolId::from_static("samtools"))].into(),
        params: [(stage, DefaultParams::Empty(EmptyParams::default()))].into(),
        thresholds: Default::default(),
        tool_provenance: Default::default(),
        param_provenance: Default::default(),
        assumptions: Default::default(),
        citations: Default::default(),
    };

    let err = ledger.validate_strict().expect_err("missing provenance must fail");
    assert!(err.to_string().contains("tool_provenance is missing stage"));

    let stage = StageId::from_static("core.prepare_reference");
    ledger.tool_provenance.insert(stage, complete_provenance());
    let err = ledger.validate_strict().expect_err("missing param provenance must fail");
    assert!(err.to_string().contains("param_provenance is missing stage"));
}

#[test]
fn strict_defaults_ledger_rejects_orphan_provenance() {
    let stage = StageId::from_static("core.prepare_reference");
    let orphan = StageId::from_static("core.orphan");
    let ledger = DefaultsLedgerV1 {
        pipeline_id: PipelineId::from_static("fastq-to-bam__default__v1"),
        tools: [(stage.clone(), ToolId::from_static("samtools"))].into(),
        params: [(stage.clone(), DefaultParams::Empty(EmptyParams::default()))].into(),
        thresholds: Default::default(),
        tool_provenance: [(stage.clone(), complete_provenance()), (orphan, complete_provenance())]
            .into(),
        param_provenance: [(stage, complete_provenance())].into(),
        assumptions: Default::default(),
        citations: Default::default(),
    };

    let err = ledger.validate_strict().expect_err("orphan provenance must fail");
    assert!(err.to_string().contains("tool_provenance has orphan stage"));
}

#[test]
fn generated_defaults_ledger_uses_domain_specific_citations() {
    let ledger = bijux_dna_pipelines::cross::fastq_to_bam_default_profile().defaults_ledger();

    let fastq_citations =
        &ledger.param_provenance[&StageId::from_static("fastq.trim_reads")].citations;
    assert_eq!(fastq_citations, &["docs/20-science/fastq/GOLD_PIPELINE_SPEC.md"]);

    let bam_citations = &ledger.param_provenance[&StageId::from_static("bam.align")].citations;
    assert_eq!(bam_citations, &["docs/20-science/bam/GOLD_PIPELINE_SPEC.md"]);

    let core_citations =
        &ledger.param_provenance[&StageId::from_static("core.prepare_reference")].citations;
    assert_eq!(core_citations, &["docs/20-science/cross/GOLD_PIPELINE_SPEC.md"]);
}

#[test]
fn strict_defaults_ledger_rejects_invalid_stage_and_tool_ids() {
    let stage = StageId::new("fastq..trim_reads".to_string());
    let mut ledger = DefaultsLedgerV1 {
        pipeline_id: PipelineId::from_static("fastq-to-fastq__default__v1"),
        tools: [(stage.clone(), ToolId::from_static("fastp"))].into(),
        params: [(stage.clone(), DefaultParams::Empty(EmptyParams::default()))].into(),
        thresholds: Default::default(),
        tool_provenance: [(stage.clone(), complete_provenance())].into(),
        param_provenance: [(stage, complete_provenance())].into(),
        assumptions: Default::default(),
        citations: Default::default(),
    };
    let err = ledger.validate_strict().expect_err("invalid stage id must fail");
    assert!(err.to_string().contains("invalid tool stage id"), "unexpected error: {err}");

    let stage = StageId::from_static("fastq.trim_reads");
    ledger.tools = [(stage.clone(), ToolId::new("_bad".to_string()))].into();
    ledger.params = [(stage.clone(), DefaultParams::Empty(EmptyParams::default()))].into();
    ledger.tool_provenance = [(stage.clone(), complete_provenance())].into();
    ledger.param_provenance = [(stage, complete_provenance())].into();
    let err = ledger.validate_strict().expect_err("invalid tool id must fail");
    assert!(err.to_string().contains("invalid tool id"), "unexpected error: {err}");
}
