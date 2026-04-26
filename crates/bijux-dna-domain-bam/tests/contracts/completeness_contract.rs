use std::path::PathBuf;

use bijux_dna_domain_bam::defaults::default_params_json;
use bijux_dna_domain_bam::metrics::{
    parse_mapdamage2_misincorporation, parse_mosdepth_summary, parse_pydamage_json,
    parse_samtools_flagstat, parse_samtools_idxstats, parse_samtools_stats,
};
use bijux_dna_domain_bam::pipeline_contract::{forbidden_transitions, optional_branches};
use bijux_dna_domain_bam::{
    required_audit_artifacts, stage_contract_json, stage_spec, BamStage,
};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("bam")
        .join("default")
        .join(name)
}

#[test]
fn pipeline_branch_metadata_references_known_stages() {
    for (branch_id, stages) in optional_branches() {
        assert!(!branch_id.trim().is_empty(), "empty optional branch id");
        assert!(!stages.is_empty(), "optional branch {branch_id} has no stages");
        for stage_id in *stages {
            BamStage::try_from(*stage_id)
                .unwrap_or_else(|err| panic!("optional branch {branch_id} has bad stage: {err}"));
        }
    }

    for (from, to) in forbidden_transitions() {
        BamStage::try_from(*from)
            .unwrap_or_else(|err| panic!("bad forbidden transition source {from}: {err}"));
        BamStage::try_from(*to)
            .unwrap_or_else(|err| panic!("bad forbidden transition target {to}: {err}"));
    }
}

#[test]
fn every_stage_contract_names_responsible_tools() {
    for stage in BamStage::all() {
        let contract = stage_contract_json(stage.as_str())
            .unwrap_or_else(|| panic!("missing contract json for {}", stage.as_str()));
        let tool_ids = contract
            .get("tool_ids")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("contract for {} missing tool_ids", stage.as_str()));
        assert!(!tool_ids.is_empty(), "{} has no responsible tools", stage.as_str());
        for tool_id in tool_ids {
            let tool_id = tool_id
                .as_str()
                .unwrap_or_else(|| panic!("{} has non-string tool id", stage.as_str()));
            assert!(!tool_id.trim().is_empty(), "{} has empty tool id", stage.as_str());
        }
    }
}

#[test]
fn bam_stages_meet_completeness_contract() {
    for stage in BamStage::all() {
        let spec = stage_spec(*stage);
        let audit = required_audit_artifacts(*stage);
        assert!(!audit.is_empty(), "stage {} missing audit artifacts", stage.as_str());
        assert!(
            !spec.artifact_policy.required_outputs.is_empty(),
            "stage {} missing required outputs",
            stage.as_str()
        );
        let params_value = default_params_json(*stage);
        stage
            .parse_effective_params(&params_value)
            .unwrap_or_else(|_| panic!("default params invalid for {}", stage.as_str()));
    }
}

#[test]
fn bam_truth_stage_parsers_have_fixtures() -> anyhow::Result<()> {
    let flagstat = fixture_path("flagstat.txt");
    let idxstats = fixture_path("idxstats.txt");
    let stats = fixture_path("samtools_stats.txt");
    let mosdepth = fixture_path("mosdepth.summary.txt");
    let pydamage = fixture_path("pydamage.json");
    let mapdamage2 = fixture_path("mapdamage2.txt");

    assert!(flagstat.exists());
    assert!(idxstats.exists());
    assert!(stats.exists());
    assert!(mosdepth.exists());
    assert!(pydamage.exists());
    assert!(mapdamage2.exists());

    parse_samtools_flagstat(&flagstat)?;
    parse_samtools_idxstats(&idxstats)?;
    parse_samtools_stats(&stats)?;
    parse_mosdepth_summary(&mosdepth)?;
    parse_pydamage_json(&pydamage)?;
    parse_mapdamage2_misincorporation(&mapdamage2)?;

    Ok(())
}
