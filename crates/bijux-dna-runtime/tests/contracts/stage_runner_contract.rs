use bijux_dna_runtime::stage_runner_contract::{
    ensure_stage_supported_by_runner, RunnerContractKind,
};

#[test]
fn docker_runner_contract_covers_core_domains() {
    for stage_id in [
        "fastq.trim",
        "bam.align",
        "vcf.phasing",
        "core.prepare_reference",
        "cross.fastq_to_bam",
        "report.aggregate",
    ] {
        ensure_stage_supported_by_runner(RunnerContractKind::Docker, stage_id)
            .unwrap_or_else(|err| panic!("expected stage support for {stage_id}: {err}"));
    }
}

#[test]
fn docker_runner_contract_rejects_unknown_stage_prefixes() {
    let err = ensure_stage_supported_by_runner(RunnerContractKind::Docker, "toy.unknown_stage")
        .expect_err("unknown stage prefix must fail fast");
    assert!(
        err.to_string().contains("no stage-runner contract"),
        "unexpected error: {err}"
    );
}

