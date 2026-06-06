#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_essential_pipeline_corpus_assets_writes_governed_tsv_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-essential-pipeline-corpus-assets"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "target/bench-readiness/essential-pipeline-corpus-assets.tsv");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read essential pipeline corpus/assets tsv");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "pipeline_id\tnode_id\tstage_id\tcorpus_id\tasset_profile_id\tinput_paths\toutput_paths\tstatus"
        )
    );
    assert!(lines.any(|line| {
        line.starts_with(
            "amplicon-asv-otu-no-vcf\tfastq.validate_reads\tfastq.validate_reads\tcorpus-03-amplicon-mini\tcorpus_only\t"
        ) && line.contains("external:corpus.raw_amplicon_fastq_reads")
            && line.ends_with("\tresolved_pipeline_bound")
    }));
    assert!(lines.any(|line| {
        line.starts_with(
            "edna-taxonomy-no-vcf\tfastq.screen_taxonomy\tfastq.screen_taxonomy\tcorpus-02\ttaxonomy_database.root+taxonomy_expected_truth_table\t"
        ) && line.contains("external:taxonomy_database.root")
            && line.contains("output:taxonomy_classification")
            && line.ends_with("\tresolved_fixture_bound")
    }));
    assert!(lines.any(|line| {
        line.starts_with(
            "reference-panel-imputation\tvcf.prepare_reference_panel\tvcf.prepare_reference_panel\tvcf_production_regression\tgenetic_map_contract+reference_dict_contract+reference_fai_contract+reference_fasta_contract+reference_panel_id_contract+reference_panel_lock_contract\t"
        ) && line.contains("external:corpus.reference_panel_vcf")
            && line.contains("output:prepared_panel_manifest_json")
            && line.ends_with("\tresolved_pipeline_bound")
    }));
}
