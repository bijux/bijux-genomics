use anyhow::Result;
use bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

#[test]
fn local_merge_pairs_smoke_plans_use_governed_overlap_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_fastq::stage_api::local_merge_pairs_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed merge smoke should keep one overlap fixture");

    let [case] = plans.as_slice() else {
        panic!("expected exactly one merge smoke case");
    };
    assert_eq!(case.sample_id, "human_like_pe_merge_overlap");
    assert_eq!(
        case.r1,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz"
        )
    );
    assert_eq!(
        case.r2,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz"
        )
    );
    assert_eq!(case.merge_overlap, 8);
    assert_eq!(case.min_length, 12);
    assert_eq!(case.unmerged_read_policy, UnmergedReadPolicy::EmitUnmergedPairs);

    assert_eq!(case.plan.stage_id.as_str(), "fastq.merge_pairs");
    assert_eq!(case.plan.tool_id.as_str(), "pear");
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("runs/bench/local-smoke/fastq.merge_pairs/human_like_pe_merge_overlap/pear")
    );
    assert_eq!(case.plan.resources.threads, 1);
    assert_eq!(case.plan.effective_params["paired_mode"], serde_json::json!("paired_end"));
    assert_eq!(case.plan.effective_params["merge_overlap"], serde_json::json!(8));
    assert_eq!(case.plan.effective_params["min_len"], serde_json::json!(12));
    assert_eq!(
        case.plan.effective_params["unmerged_read_policy"],
        serde_json::json!("emit_unmerged_pairs")
    );
    assert_eq!(
        case.plan.params["r1"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz"
        )
    );
    assert_eq!(
        case.plan.params["r2"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz"
        )
    );
    assert_eq!(
        case.plan.params["merged_reads"],
        serde_json::json!(
            "runs/bench/local-smoke/fastq.merge_pairs/human_like_pe_merge_overlap/pear/pear.assembled.fastq"
        )
    );
    assert_eq!(
        case.plan.params["unmerged_reads_r1"],
        serde_json::json!(
            "runs/bench/local-smoke/fastq.merge_pairs/human_like_pe_merge_overlap/pear/pear.unassembled.forward.fastq"
        )
    );
    assert_eq!(
        case.plan.params["unmerged_reads_r2"],
        serde_json::json!(
            "runs/bench/local-smoke/fastq.merge_pairs/human_like_pe_merge_overlap/pear/pear.unassembled.reverse.fastq"
        )
    );

    Ok(())
}

#[test]
fn local_merge_pairs_output_contract_plans_cover_every_supported_tool() -> Result<()> {
    let repo_root = repo_root();
    let plans =
        bijux_dna_planner_fastq::stage_api::local_merge_pairs_output_contract_plans(&repo_root)?;
    let tool_ids = plans
        .iter()
        .map(|case| case.plan.tool_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        tool_ids,
        std::collections::BTreeSet::from([
            "adapterremoval",
            "bbmerge",
            "flash2",
            "leehom",
            "pear",
            "seqprep",
            "vsearch",
        ])
    );
    assert_eq!(plans.len(), tool_ids.len());

    let Some(seqprep) =
        plans.iter().find(|case| case.plan.tool_id.as_str() == "seqprep")
    else {
        panic!("missing SeqPrep output-contract plan");
    };
    assert_eq!(seqprep.plan.effective_params["merge_overlap"], serde_json::Value::Null);
    assert_eq!(seqprep.plan.effective_params["min_len"], serde_json::Value::Null);
    assert!(seqprep
        .plan
        .io
        .outputs
        .iter()
        .any(|artifact| artifact.name.as_str() == "merged_reads"));
    Ok(())
}

#[test]
fn local_merge_pairs_smoke_stage_api_surface_stays_callable() {
    let _: fn(&Path) -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalMergePairsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_merge_pairs_smoke_plans;
    let _: fn(&Path) -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalMergePairsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_merge_pairs_output_contract_plans;
}
