use std::fs;
use std::path::Path;

use anyhow::Result;
use bijux_core::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};

fn dummy_tool(tool: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId(tool.to_string()),
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

fn snapshot_path(name: &str) -> Result<std::path::PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    Ok(Path::new(&manifest_dir)
        .join("tests")
        .join("snapshots")
        .join(name))
}

fn assert_snapshot(name: &str, plan: &bijux_core::StagePlanV1) -> Result<()> {
    let plan_json = bijux_stages_bam::StagePlanJson::from_plan(plan);
    let rendered = serde_json::to_string_pretty(&plan_json)?;
    let path = snapshot_path(name)?;
    if std::env::var("UPDATE_SNAPSHOTS")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        fs::write(path, rendered)?;
        return Ok(());
    }
    let snapshot = fs::read_to_string(path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}

#[test]
fn align_plan_json_is_emitted_and_stable() -> Result<()> {
    let params = bijux_domain_bam::params::AlignEffectiveParams {
        aligner: "bwa".to_string(),
        preset: "default".to_string(),
        threads: 1,
        reference: "reference.fasta".to_string(),
        reference_digest: "sha256:ref".to_string(),
        rg_policy: bijux_domain_bam::types::ReadGroupPolicy::Regenerate,
        read_group: bijux_domain_bam::params::ReadGroupSpec::with_defaults("sample"),
        build_indices: true,
        emit_stats: true,
    };
    let plan = bijux_stages_bam::bam::align::plan(
        &dummy_tool("bwa"),
        Path::new("reads_R1.fastq.gz"),
        Some(Path::new("reads_R2.fastq.gz")),
        Path::new("reference.fasta"),
        "sample",
        &params,
        Path::new("out"),
    )?;
    assert_snapshot("stage__bam__bam.align.json", &plan)
}

#[test]
fn validate_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_stages_bam::bam::validate::plan(
        &dummy_tool("samtools"),
        Path::new("reads.bam"),
        None,
        None,
        Path::new("out"),
    )?;
    assert_snapshot("stage__bam__bam.validate.json", &plan)
}

#[test]
fn qc_pre_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_stages_bam::bam::qc_pre::plan(
        &dummy_tool("samtools"),
        Path::new("reads.bam"),
        Path::new("out"),
    )?;
    assert_snapshot("stage__bam__bam.qc_pre.json", &plan)
}

#[test]
fn filter_plan_json_is_emitted_and_stable() -> Result<()> {
    let params = bijux_domain_bam::params::FilterEffectiveParams {
        mapq_threshold: 30,
        include_flags: Vec::new(),
        exclude_flags: Vec::new(),
        min_length: 30,
        remove_duplicates: false,
        base_quality_threshold: 20,
    };
    let plan = bijux_stages_bam::bam::filter::plan(
        &dummy_tool("samtools"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_snapshot("stage__bam__bam.filter.json", &plan)
}

#[test]
fn markdup_plan_json_is_emitted_and_stable() -> Result<()> {
    let params = bijux_domain_bam::params::MarkDupEffectiveParams {
        optical_duplicates: bijux_domain_bam::params::OpticalDuplicatePolicy::MarkOnly,
        umi_policy: bijux_domain_bam::params::UmiPolicy::Ignore,
        duplicate_action: bijux_domain_bam::params::DuplicateAction::Mark,
    };
    let plan = bijux_stages_bam::bam::markdup::plan(
        &dummy_tool("gatk"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_snapshot("stage__bam__bam.markdup.json", &plan)
}

#[test]
fn complexity_plan_json_is_emitted_and_stable() -> Result<()> {
    let params = bijux_domain_bam::params::ComplexityEffectiveParams {
        min_reads: 100_000,
        projection_points: vec![1_000_000, 2_000_000],
    };
    let plan = bijux_stages_bam::bam::complexity::plan(
        &dummy_tool("preseq"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_snapshot("stage__bam__bam.complexity.json", &plan)
}

#[test]
fn coverage_plan_json_is_emitted_and_stable() -> Result<()> {
    let params = bijux_domain_bam::params::CoverageEffectiveParams {
        regions: None,
        depth_thresholds: vec![1, 3, 5],
    };
    let plan = bijux_stages_bam::bam::coverage::plan(
        &dummy_tool("mosdepth"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_snapshot("stage__bam__bam.coverage.json", &plan)
}

#[test]
fn damage_plan_json_is_emitted_and_stable() -> Result<()> {
    let params = bijux_domain_bam::params::DamageEffectiveParams {
        udg_model: bijux_domain_bam::params::UdgModel::NonUdg,
        pmd_threshold_5p: 0.3,
        pmd_threshold_3p: 0.3,
        trim_5p: 2,
        trim_3p: 2,
    };
    let plan = bijux_stages_bam::bam::damage::plan(
        &dummy_tool("pydamage"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_snapshot("stage__bam__bam.damage.json", &plan)
}

#[test]
fn authenticity_plan_json_is_emitted_and_stable() -> Result<()> {
    let params = bijux_domain_bam::params::AuthenticityEffectiveParams {
        mode: "aggregate".to_string(),
    };
    let plan = bijux_stages_bam::bam::authenticity::plan(
        &dummy_tool("auth"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_snapshot("stage__bam__bam.authenticity.json", &plan)
}

#[test]
fn contamination_plan_json_is_emitted_and_stable() -> Result<()> {
    let params = bijux_domain_bam::params::ContaminationEffectiveParams {
        reference_panels: vec!["panel.vcf".to_string()],
        scope: bijux_domain_bam::params::ContaminationScope::Both,
        prior: None,
        sex_specific: false,
        assumptions: None,
    };
    let plan = bijux_stages_bam::bam::contamination::plan(
        &dummy_tool("authentic"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_snapshot("stage__bam__bam.contamination.json", &plan)
}

#[test]
fn sex_plan_json_is_emitted_and_stable() -> Result<()> {
    let params = bijux_domain_bam::params::SexEffectiveParams {
        expected_sex: None,
        method: "rxy".to_string(),
    };
    let plan = bijux_stages_bam::bam::sex::plan(
        &dummy_tool("rxy"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_snapshot("stage__bam__bam.sex.json", &plan)
}

#[test]
#[cfg(feature = "bam_downstream")]
fn bias_mitigation_plan_json_is_emitted_and_stable() -> Result<()> {
    let params = bijux_domain_bam::params::BiasMitigationEffectiveParams {
        gc_bias_correction: true,
        map_bias_correction: false,
    };
    let plan = bijux_stages_bam::bam::bias_mitigation::plan(
        &dummy_tool("angsd"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_snapshot("stage__bam__bam.bias_mitigation.json", &plan)
}

#[test]
fn recalibration_plan_json_is_emitted_and_stable() -> Result<()> {
    let params = bijux_domain_bam::params::BqsrEffectiveParams {
        known_sites: vec!["known.vcf".to_string()],
        mode: bijux_domain_bam::params::BqsrMode::Skip,
        skip_criteria: bijux_domain_bam::params::RecalibrationSkipCriteria {
            min_mean_coverage: 1.0,
            min_breadth_1x: 0.1,
        },
    };
    let plan = bijux_stages_bam::bam::recalibration::plan(
        &dummy_tool("gatk"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_snapshot("stage__bam__bam.recalibration.json", &plan)
}

#[test]
#[cfg(feature = "bam_downstream")]
fn haplogroups_plan_json_is_emitted_and_stable() -> Result<()> {
    let params = bijux_domain_bam::params::HaplogroupEffectiveParams {
        reference_panel: "mito_default".to_string(),
        min_coverage: Some(1.0),
    };
    let plan = bijux_stages_bam::bam::haplogroups::plan(
        &dummy_tool("haplogrep"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_snapshot("stage__bam__bam.haplogroups.json", &plan)
}

#[test]
#[cfg(feature = "bam_downstream")]
fn genotyping_plan_json_is_emitted_and_stable() -> Result<()> {
    let params = bijux_domain_bam::params::GenotypingEffectiveParams {
        caller: "angsd".to_string(),
        min_posterior: Some(0.9),
        min_call_rate: Some(0.5),
    };
    let plan = bijux_stages_bam::bam::genotyping::plan(
        &dummy_tool("angsd"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_snapshot("stage__bam__bam.genotyping.json", &plan)
}

#[test]
#[cfg(feature = "bam_downstream")]
fn kinship_plan_json_is_emitted_and_stable() -> Result<()> {
    let params = bijux_domain_bam::params::KinshipEffectiveParams {
        reference_panel: "king_default".to_string(),
        min_overlap_snps: 1000,
    };
    let plan = bijux_stages_bam::bam::kinship::plan(
        &dummy_tool("king"),
        Path::new("reads.bam"),
        Path::new("out"),
        &params,
    )?;
    assert_snapshot("stage__bam__bam.kinship.json", &plan)
}
