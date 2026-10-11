#![allow(clippy::expect_used)]
use super::warmup::supports_apptainer_warmup;
use super::{
    prepare_sortmerna_sample_workdir, promote_sortmerna_sample_index_cache,
    resolve_deplete_rrna_stage_options,
};
use std::fs;
#[test]
fn supports_apptainer_warmup_accepts_apptainer_platform_names() {
    assert!(supports_apptainer_warmup("apptainer-amd64"));
    assert!(supports_apptainer_warmup("cluster-apptainer"));
    assert!(!supports_apptainer_warmup("docker-amd64"));
}

#[test]
fn deplete_rrna_stage_options_require_existing_rrna_bundle() {
    let temp = tempfile::tempdir().expect("tempdir");
    let rrna_db = temp.path().join("sortmerna_v4_3_default_db.fasta");
    fs::write(&rrna_db, ">rrna\nACGT\n").expect("write rrna fasta");

    let options = resolve_deplete_rrna_stage_options(&[
        "--rrna-db".to_string(),
        rrna_db.display().to_string(),
        "--rrna-bundle-id".to_string(),
        "sortmerna_v4_3_default_db".to_string(),
        "--min-identity".to_string(),
        "0.95".to_string(),
    ])
    .expect("resolve rrna stage options");
    assert_eq!(options.rrna_db, rrna_db);
    assert_eq!(options.rrna_bundle_id, "sortmerna_v4_3_default_db");
    assert!((options.min_identity - 0.95).abs() < f64::EPSILON);
}

#[test]
fn deplete_rrna_stage_options_require_declared_bundle_id() {
    let temp = tempfile::tempdir().expect("tempdir");
    let rrna_db = temp.path().join("sortmerna_v4_3_default_db.fasta");
    fs::write(&rrna_db, ">rrna\nACGT\n").expect("write rrna fasta");

    let error = resolve_deplete_rrna_stage_options(&[
        "--rrna-db".to_string(),
        rrna_db.display().to_string(),
    ])
    .expect_err("missing rrna bundle id must fail");
    assert!(error
        .to_string()
        .contains("fastq.deplete_rrna requires --rrna-bundle-id in stage arguments"));
}

#[test]
fn sortmerna_workdir_preparation_clones_and_promotes_shared_cache() {
    let temp = tempfile::tempdir().expect("tempdir");
    let out_root = temp.path().join("results");

    let shared_idx_dir =
        prepare_sortmerna_sample_workdir(&out_root, "sample_0001", "sortmerna_v4_3_default_db")
            .expect("prepare empty workdir");
    assert!(shared_idx_dir.is_dir());

    let sample_one_idx = out_root
        .join("bench")
        .join("deplete_rrna")
        .join("sample_0001")
        .join("tools")
        .join("sortmerna")
        .join("sortmerna_workdir")
        .join("idx")
        .join("reference.stats");
    fs::write(&sample_one_idx, "seed").expect("write sample idx seed");
    let promoted =
        promote_sortmerna_sample_index_cache(&out_root, "sample_0001", "sortmerna_v4_3_default_db")
            .expect("promote shared idx");
    assert!(promoted.join("reference.stats").is_file());

    prepare_sortmerna_sample_workdir(&out_root, "sample_0002", "sortmerna_v4_3_default_db")
        .expect("prepare seeded workdir");
    let sample_two_idx = out_root
        .join("bench")
        .join("deplete_rrna")
        .join("sample_0002")
        .join("tools")
        .join("sortmerna")
        .join("sortmerna_workdir")
        .join("idx")
        .join("reference.stats");
    assert!(sample_two_idx.is_file());
}

#[test]
fn rrna_bundle_identity_cannot_escape_its_cache_namespace() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    let reference = directory.path().join("rrna.fasta");
    fs::write(&reference, b">rrna\nACGT\n")?;
    for bundle in ["", ".", "..", "../foreign", "/foreign", "a\\b"] {
        assert!(
            resolve_deplete_rrna_stage_options(&[
                "--rrna-db".into(),
                reference.display().to_string(),
                "--rrna-bundle-id".into(),
                bundle.into()
            ])
            .is_err(),
            "{bundle}"
        );
    }
    assert_eq!(fs::read(&reference)?, b">rrna\nACGT\n");
    Ok(())
}
