//! Owner: source observation freshness and produced-output exclusion.
use super::*;
use std::{collections::BTreeMap, fs};
#[test]
fn observed_inputs_exclude_intermediates_and_reacquire_current_bytes() -> anyhow::Result<()> {
    let temp = bijux_dna_testkit::temp::tempdir_for("hpc-input-observation");
    fs::create_dir_all(temp.path().join("benchmarks"))?;
    let input = temp.path().join("benchmarks/input.txt");
    fs::write(&input, b"first")?;
    fs::write(temp.path().join("benchmarks/intermediate.txt"), b"previous output")?;
    let steps = [
        vec![
            "tool".into(),
            "benchmarks/input.txt".into(),
            "--output".into(),
            "benchmarks/intermediate.txt".into(),
        ],
        vec!["tool".into(), "benchmarks/intermediate.txt".into()],
    ];
    let collect = || {
        collect_hpc_job_source_inputs(
            temp.path(),
            "observed-job",
            "stage",
            steps.iter().map(Vec::as_slice),
            &BTreeMap::new(),
            "artifacts/generated",
        )
    };
    let first = collect()?;
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].source_asset.source_path, "benchmarks/input.txt");
    assert_eq!(first[0].source_asset.checksum_sha256, bijux_dna_infra::hash_bytes_sha256(b"first"));
    fs::write(&input, b"replacement")?;
    let fresh = collect()?;
    assert_eq!(
        fresh[0].source_asset.checksum_sha256,
        bijux_dna_infra::hash_bytes_sha256(b"replacement")
    );
    assert_ne!(first[0].source_asset.checksum_sha256, fresh[0].source_asset.checksum_sha256);
    fs::remove_file(&input)?;
    assert!(collect().is_err());
    Ok(())
}
