use super::*;
use anyhow::Result;
use bijux_dna_core::ids::ToolId;

#[test]
fn readiness_preserves_independent_metadata_and_spec_dispositions() -> Result<()> {
    let root = tempfile::tempdir()?;
    let tools = root.path().join("domain/bam/tools");
    std::fs::create_dir_all(&tools)?;
    let path = tools.join("fixture-tool.yaml");
    let tool = ToolId::from_static("fixture-tool");
    let stage = StageId::from_static("bam.validate");
    let mut record = serde_json::json!({
        "tool_id": "fixture-tool", "default_version": "1", "status": "planned",
        "stage_ids": ["bam.validate"], "help_cmd": "fixture-tool --help"
    });
    let observe = |record: &serde_json::Value| -> Result<_> {
        std::fs::write(&path, serde_json::to_vec(record)?)?;
        let definition = crate::selection::load_bam_domain_tool_definition(root.path(), &tool)?;
        Ok(assess_bam_tool_definition(Some(&definition), &stage))
    };
    assert_eq!(
        assess_bam_tool_definition(None, &stage),
        BamToolDefinitionReadiness {
            support: BamToolContractStatus::Missing,
            adapter: BamToolAdapterStatus::DeclaredOnly,
        }
    );
    assert_eq!(
        observe(&record)?,
        BamToolDefinitionReadiness {
            support: BamToolContractStatus::Admitted(BamDomainToolSupportLevel::Planned),
            adapter: BamToolAdapterStatus::Plannable,
        }
    );
    record["container"] = serde_json::json!({"image": "fixture-image:1"});
    record["status"] = serde_json::json!("supported");
    assert_eq!(
        observe(&record)?,
        BamToolDefinitionReadiness {
            support: BamToolContractStatus::Admitted(BamDomainToolSupportLevel::Supported),
            adapter: BamToolAdapterStatus::Runnable,
        }
    );
    record["stage_ids"] = serde_json::json!(["bam.align"]);
    assert_eq!(
        observe(&record)?,
        BamToolDefinitionReadiness {
            support: BamToolContractStatus::Mismatched,
            adapter: BamToolAdapterStatus::DeclaredOnly,
        }
    );
    record["stage_ids"] = serde_json::json!(["bam.validate"]);
    record["status"] = serde_json::json!("unknown");
    assert_eq!(
        observe(&record)?,
        BamToolDefinitionReadiness {
            support: BamToolContractStatus::Missing,
            adapter: BamToolAdapterStatus::Runnable,
        }
    );
    record["status"] = serde_json::json!("supported");
    record["help_cmd"] = serde_json::Value::Null;
    assert_eq!(
        observe(&record)?,
        BamToolDefinitionReadiness {
            support: BamToolContractStatus::Admitted(BamDomainToolSupportLevel::Supported),
            adapter: BamToolAdapterStatus::DeclaredOnly,
        }
    );
    Ok(())
}
