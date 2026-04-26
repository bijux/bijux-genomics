use std::path::PathBuf;

use bijux_dna_domain_bam::defaults::default_params_json;
use bijux_dna_domain_bam::metrics::{
    parse_mapdamage2_misincorporation, parse_mosdepth_summary, parse_pydamage_json,
    parse_samtools_flagstat, parse_samtools_idxstats, parse_samtools_stats,
};
use bijux_dna_domain_bam::pipeline_contract::{
    forbidden_transitions, optional_branches, stage_criticality, StageCriticality,
};
use bijux_dna_domain_bam::{
    required_audit_artifacts, stage_contract_json, stage_spec, stage_spec_opt, BamStage,
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
fn stage_criticality_uses_all_declared_classes() {
    let mut essential = 0;
    let mut optional = 0;
    let mut experimental = 0;
    for stage in BamStage::all() {
        match stage_criticality(stage.as_str())
            .unwrap_or_else(|| panic!("missing criticality for {}", stage.as_str()))
        {
            StageCriticality::Essential => essential += 1,
            StageCriticality::Optional => optional += 1,
            StageCriticality::Experimental => experimental += 1,
        }
    }
    assert!(essential > 0, "no essential BAM stages declared");
    assert!(optional > 0, "no optional BAM stages declared");
    assert!(experimental > 0, "no experimental BAM stages declared");
}

#[test]
fn every_stage_contract_names_responsible_tools() {
    for stage in BamStage::all() {
        let contract = stage_contract_json(stage.as_str())
            .unwrap_or_else(|| panic!("missing contract json for {}", stage.as_str()));
        let io = contract
            .get("io")
            .and_then(serde_json::Value::as_object)
            .unwrap_or_else(|| panic!("contract for {} missing io object", stage.as_str()));
        for key in ["input_kind", "output_kind"] {
            let kind = io
                .get(key)
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("contract for {} missing io.{key}", stage.as_str()));
            assert!(
                kind.chars().all(|ch| ch.is_ascii_lowercase() || ch == '_'),
                "{} io.{key} is not a stable snake-case contract value: {kind}",
                stage.as_str()
            );
        }
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
        let spec = stage_spec_opt(*stage)
            .unwrap_or_else(|| panic!("stage {} missing spec", stage.as_str()));
        assert_eq!(stage_spec(*stage).stage, spec.stage);
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
