use std::path::Path;

use anyhow::Result;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use serde_json::Value;

fn dummy_tool(tool: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool),
        tool_version: "1.0.0".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/test:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: Vec::new(),
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

fn assert_keys(value: &Value, keys: &[&str]) -> Result<()> {
    let obj = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("effective_params is not an object"))?;
    for key in keys {
        assert!(obj.contains_key(*key), "missing key: {key}");
    }
    Ok(())
}

#[test]
fn validate_params_complete() -> Result<()> {
    let plan = bijux_dna_planner_bam::tool_adapters::bam::validate::plan(
        &dummy_tool("samtools"),
        Path::new("reads.bam"),
        None,
        None,
        Path::new("out"),
    )?;
    assert_keys(&plan.effective_params, &["strict"])?;
    Ok(())
}

#[test]
fn qc_pre_params_complete() -> Result<()> {
    let plan = bijux_dna_planner_bam::tool_adapters::bam::qc_pre::plan(
        &dummy_tool("samtools"),
        Path::new("reads.bam"),
        Path::new("out"),
    )?;
    assert_keys(&plan.effective_params, &["regions"])?;
    Ok(())
}

#[test]
fn filter_params_complete() -> Result<()> {
    let params = bijux_dna_domain_bam::params::FilterEffectiveParams {
        mapq_threshold: 30,
        include_flags: Vec::new(),
        exclude_flags: Vec::new(),
        min_length: 30,
        remove_duplicates: false,
        base_quality_threshold: 20,
    };
    let plan = bijux_dna_planner_bam::tool_adapters::bam::filter::plan(
        &dummy_tool("samtools"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_keys(
        &plan.effective_params,
        &[
            "mapq_threshold",
            "include_flags",
            "exclude_flags",
            "min_length",
            "remove_duplicates",
            "base_quality_threshold",
        ],
    )?;
    Ok(())
}

#[test]
fn markdup_params_complete() -> Result<()> {
    let params = bijux_dna_domain_bam::params::MarkDupEffectiveParams {
        optical_duplicates: bijux_dna_domain_bam::params::OpticalDuplicatePolicy::MarkOnly,
        umi_policy: bijux_dna_domain_bam::params::UmiPolicy::Ignore,
        duplicate_action: bijux_dna_domain_bam::params::DuplicateAction::Mark,
    };
    let plan = bijux_dna_planner_bam::tool_adapters::bam::markdup::plan(
        &dummy_tool("gatk"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_keys(
        &plan.effective_params,
        &["optical_duplicates", "umi_policy", "duplicate_action"],
    )?;
    Ok(())
}

#[test]
fn complexity_params_complete() -> Result<()> {
    let params = bijux_dna_domain_bam::params::ComplexityEffectiveParams {
        min_reads: 100_000,
        projection_points: vec![1_000_000, 2_000_000],
    };
    let plan = bijux_dna_planner_bam::tool_adapters::bam::complexity::plan(
        &dummy_tool("preseq"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_keys(&plan.effective_params, &["min_reads", "projection_points"])?;
    Ok(())
}

#[test]
fn coverage_params_complete() -> Result<()> {
    let params = bijux_dna_domain_bam::params::CoverageEffectiveParams {
        regions: None,
        depth_thresholds: vec![1, 3, 5],
    };
    let plan = bijux_dna_planner_bam::tool_adapters::bam::coverage::plan(
        &dummy_tool("mosdepth"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_keys(&plan.effective_params, &["regions", "depth_thresholds"])?;
    Ok(())
}

#[test]
fn damage_params_complete() -> Result<()> {
    let params = bijux_dna_domain_bam::params::DamageEffectiveParams {
        udg_model: bijux_dna_domain_bam::params::UdgModel::NonUdg,
        pmd_threshold_5p: 0.3,
        pmd_threshold_3p: 0.3,
        trim_5p: 2,
        trim_3p: 2,
    };
    let plan = bijux_dna_planner_bam::tool_adapters::bam::damage::plan(
        &dummy_tool("pydamage"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_keys(
        &plan.effective_params,
        &[
            "udg_model",
            "pmd_threshold_5p",
            "pmd_threshold_3p",
            "trim_5p",
            "trim_3p",
        ],
    )?;
    Ok(())
}

#[test]
fn authenticity_params_complete() -> Result<()> {
    let params = bijux_dna_domain_bam::params::AuthenticityEffectiveParams {
        mode: "aggregate".to_string(),
    };
    let plan = bijux_dna_planner_bam::tool_adapters::bam::authenticity::plan(
        &dummy_tool("auth"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_keys(&plan.effective_params, &["mode"])?;
    Ok(())
}

#[test]
fn contamination_params_complete() -> Result<()> {
    let params = bijux_dna_domain_bam::params::ContaminationEffectiveParams {
        reference_panels: vec!["panel.vcf".to_string()],
        scope: bijux_dna_domain_bam::params::ContaminationScope::Both,
        prior: None,
        sex_specific: false,
        assumptions: None,
    };
    let plan = bijux_dna_planner_bam::tool_adapters::bam::contamination::plan(
        &dummy_tool("authentic"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_keys(
        &plan.effective_params,
        &[
            "reference_panels",
            "scope",
            "prior",
            "sex_specific",
            "assumptions",
        ],
    )?;
    Ok(())
}

#[test]
fn sex_params_complete() -> Result<()> {
    let params = bijux_dna_domain_bam::params::SexEffectiveParams {
        expected_sex: None,
        method: "rxy".to_string(),
    };
    let plan = bijux_dna_planner_bam::tool_adapters::bam::sex::plan(
        &dummy_tool("rxy"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_keys(&plan.effective_params, &["expected_sex", "method"])?;
    Ok(())
}

#[test]
#[cfg(feature = "bam_downstream")]
fn bias_mitigation_params_complete() -> Result<()> {
    let params = bijux_dna_domain_bam::params::BiasMitigationEffectiveParams {
        gc_bias_correction: true,
        map_bias_correction: false,
    };
    let plan = bijux_dna_planner_bam::tool_adapters::bam::bias_mitigation::plan(
        &dummy_tool("angsd"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_keys(
        &plan.effective_params,
        &["gc_bias_correction", "map_bias_correction"],
    )?;
    Ok(())
}

#[test]
fn recalibration_params_complete() -> Result<()> {
    let params = bijux_dna_domain_bam::params::BqsrEffectiveParams {
        known_sites: vec!["known.vcf".to_string()],
        mode: bijux_dna_domain_bam::params::BqsrMode::Skip,
        skip_criteria: bijux_dna_domain_bam::params::RecalibrationSkipCriteria {
            min_mean_coverage: 1.0,
            min_breadth_1x: 0.1,
        },
    };
    let plan = bijux_dna_planner_bam::tool_adapters::bam::recalibration::plan(
        &dummy_tool("gatk"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_keys(
        &plan.effective_params,
        &["known_sites", "mode", "skip_criteria"],
    )?;
    Ok(())
}

#[test]
#[cfg(feature = "bam_downstream")]
fn haplogroups_params_complete() -> Result<()> {
    let params = bijux_dna_domain_bam::params::HaplogroupEffectiveParams {
        reference_panel: "mito_default".to_string(),
        min_coverage: Some(1.0),
    };
    let plan = bijux_dna_planner_bam::tool_adapters::bam::haplogroups::plan(
        &dummy_tool("haplogrep"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_keys(&plan.effective_params, &["reference_panel", "min_coverage"])?;
    Ok(())
}

#[test]
#[cfg(feature = "bam_downstream")]
fn genotyping_params_complete() -> Result<()> {
    let params = bijux_dna_domain_bam::params::GenotypingEffectiveParams {
        caller: "angsd".to_string(),
        min_posterior: Some(0.9),
        min_call_rate: Some(0.5),
    };
    let plan = bijux_dna_planner_bam::tool_adapters::bam::genotyping::plan(
        &dummy_tool("angsd"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_keys(
        &plan.effective_params,
        &["caller", "min_posterior", "min_call_rate"],
    )?;
    Ok(())
}

#[test]
#[cfg(feature = "bam_downstream")]
fn kinship_params_complete() -> Result<()> {
    let params = bijux_dna_domain_bam::params::KinshipEffectiveParams {
        reference_panel: "king_default".to_string(),
        min_overlap_snps: 1000,
    };
    let plan = bijux_dna_planner_bam::tool_adapters::bam::kinship::plan(
        &dummy_tool("king"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_keys(
        &plan.effective_params,
        &["reference_panel", "min_overlap_snps"],
    )?;
    Ok(())
}
