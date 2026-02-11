use std::path::PathBuf;

use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_planner_vcf::{plan_vcf_minimal, VcfPipelineInputs};

#[test]
fn vcf_minimal_plan_contains_call_filter_stats_chain() {
    let input = VcfPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        vcf: PathBuf::from("sample.vcf.gz"),
        out_dir: PathBuf::from("out"),
    };
    let graph = plan_vcf_minimal(&input).unwrap_or_else(|err| panic!("plan graph: {err}"));
    let ids = graph
        .steps()
        .iter()
        .map(|s| s.stage_id.as_str().to_string())
        .collect::<Vec<_>>();
    assert!(ids.contains(&"vcf.call".to_string()));
    assert!(ids.contains(&"vcf.filter".to_string()));
    assert!(ids.contains(&"vcf.stats".to_string()));
}
