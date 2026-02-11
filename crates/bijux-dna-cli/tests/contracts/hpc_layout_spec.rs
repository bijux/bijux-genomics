use std::path::Path;

#[test]
fn hpc_run_layout_spec_rejects_adhoc_paths() {
    let bad = Path::new("/tmp/adhoc/run-1");
    let res = bijux_dna::commands::hpc::enforce_hpc_results_layout(bad);
    assert!(res.is_err());
}

#[test]
fn hpc_run_layout_spec_accepts_results_naming() {
    let good = Path::new(
        "/home/bijan/bijux/bijux-dna-results/results/corpus/pipeline/stage/tool/20260211T120001Z/run-id",
    );
    let res = bijux_dna::commands::hpc::enforce_hpc_results_layout(good);
    assert!(res.is_ok());
}
