use std::path::PathBuf;

use bijux_dna_domain_bam::metrics::{
    parse_mapdamage2_misincorporation, parse_mosdepth_summary, parse_pydamage_json,
    parse_samtools_flagstat, parse_samtools_idxstats, parse_samtools_stats,
};
use bijux_dna_domain_bam::params::BamEffectiveParams;
use bijux_dna_domain_bam::{required_audit_artifacts, stage_spec, BamStage};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("bam")
        .join("default")
        .join(name)
}

fn params_to_value(params: &BamEffectiveParams) -> anyhow::Result<serde_json::Value> {
    Ok(serde_json::to_value(params)?)
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
