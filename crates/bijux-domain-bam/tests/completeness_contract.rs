use std::path::PathBuf;

use bijux_domain_bam::metrics::{
    parse_mapdamage2_misincorporation, parse_mosdepth_summary, parse_pydamage_json,
    parse_samtools_flagstat, parse_samtools_idxstats, parse_samtools_stats,
};
use bijux_domain_bam::params::BamEffectiveParams;
use bijux_domain_bam::{required_audit_artifacts, stage_spec, BamStage};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("bam")
        .join(name)
}

fn params_to_value(params: &BamEffectiveParams) -> anyhow::Result<serde_json::Value> {
    let value = match params {
        BamEffectiveParams::Align(inner) => serde_json::to_value(inner),
        BamEffectiveParams::Validate(inner) => serde_json::to_value(inner),
        BamEffectiveParams::QcPre(inner) => serde_json::to_value(inner),
        BamEffectiveParams::Filter(inner) => serde_json::to_value(inner),
        BamEffectiveParams::Markdup(inner) => serde_json::to_value(inner),
        BamEffectiveParams::Complexity(inner) => serde_json::to_value(inner),
        BamEffectiveParams::Coverage(inner) => serde_json::to_value(inner),
        BamEffectiveParams::Damage(inner) => serde_json::to_value(inner),
        BamEffectiveParams::Authenticity(inner) => serde_json::to_value(inner),
        BamEffectiveParams::Contamination(inner) => serde_json::to_value(inner),
        BamEffectiveParams::Sex(inner) => serde_json::to_value(inner),
        BamEffectiveParams::BiasMitigation(inner) => serde_json::to_value(inner),
        BamEffectiveParams::Recalibration(inner) => serde_json::to_value(inner),
        BamEffectiveParams::Haplogroups(inner) => serde_json::to_value(inner),
        BamEffectiveParams::Genotyping(inner) => serde_json::to_value(inner),
        BamEffectiveParams::Kinship(inner) => serde_json::to_value(inner),
    }?;
    Ok(value)
}

#[test]
fn bam_stages_meet_completeness_contract() -> anyhow::Result<()> {
    for stage in BamStage::all() {
        let spec = stage_spec(*stage);
        let audit = required_audit_artifacts(*stage);
        assert!(
            !audit.is_empty(),
            "stage {} missing audit artifacts",
            stage.as_str()
        );
        assert!(
            !spec.artifact_policy.required_outputs.is_empty(),
            "stage {} missing required outputs",
            stage.as_str()
        );
        let params_value = params_to_value(&spec.default_params)?;
        stage
            .parse_effective_params(&params_value)
            .unwrap_or_else(|_| panic!("default params invalid for {}", stage.as_str()));
    }
    Ok(())
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
