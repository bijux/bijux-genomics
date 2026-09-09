#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;
use std::path::{Path, PathBuf};

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn collect_sbatch_paths(root: &Path, paths: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }
    for entry in fs::read_dir(root).expect("read dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        let file_type = entry.file_type().expect("file type");
        if file_type.is_dir() {
            collect_sbatch_paths(&path, paths);
        } else if path.extension().and_then(|segment| segment.to_str()) == Some("sbatch") {
            paths.push(path);
        }
    }
}

#[test]
fn bench_local_render_all_domain_slurm_scripts_writes_governed_root() {
    let sandbox = support::RepoSandbox::new("slurm-script-files-").expect("repository sandbox");
    let repo_root = sandbox.path();
    let home = tempfile::tempdir().expect("home directory");
    let run_cli = |args: &[&str]| {
        sandbox
            .bijux_dna_command()
            .env("HOME", home.path())
            .env("BIJUX_SKIP_QA", "1")
            .env("BIJUX_ALLOW_SILVER", "1")
            .env("BIJUX_SKIP_IMAGE_CHECK", "1")
            .args(args)
            .output()
            .expect("run cli")
    };
    let manifest_path =
        repo_root.join("artifacts/runs/bench/slurm-dry-run/all-domains/submit-manifest.json");
    assert!(!manifest_path.exists(), "the test must start without another invocation's manifest");
    let setup = run_cli(&["bench", "local", "render-all-domain-slurm-submit-manifest"]);
    assert!(
        setup.status.success(),
        "manifest setup failed: {}",
        String::from_utf8_lossy(&setup.stderr)
    );
    let output = run_cli(&["bench", "local", "render-all-domain-slurm-scripts"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered_root = String::from_utf8_lossy(&output.stdout);
    assert_eq!(rendered_root.trim(), "artifacts/runs/bench/slurm-dry-run/all-domains");

    let output_root = repo_root.join("artifacts/runs/bench/slurm-dry-run/all-domains");
    let mut script_paths = Vec::new();
    collect_sbatch_paths(&output_root, &mut script_paths);
    script_paths.sort();
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(output_root.join("submit-manifest.json")).expect("read submit manifest"),
    )
    .expect("parse submit manifest");
    assert_eq!(script_paths.len(), manifest["jobs"].as_array().expect("manifest jobs").len());

    let vcf_script = output_root.join(
        "benchmark-results/vcf/vcf_production_regression/vcf.stats/vcf_cohort/bcftools/job.sbatch",
    );
    assert!(vcf_script.exists(), "expected governed VCF benchmark script");
    let vcf_body = fs::read_to_string(&vcf_script).expect("read VCF script");
    assert!(vcf_body.contains("#SBATCH --job-name="));
    assert!(vcf_body.contains("bcftools"));
    assert!(vcf_body.contains(
        "#SBATCH --output=artifacts/runs/bench/slurm-dry-run/all-domains/runs/all-domain-benchmark-dry-run/vcf/vcf.stats/bcftools/vcf_production_regression/vcf_cohort/stdout.log"
    ));
    assert!(vcf_body.contains(
        "RESULT_ROOT=artifacts/runs/bench/slurm-dry-run/all-domains/runs/all-domain-benchmark-dry-run/vcf/vcf.stats/bcftools/vcf_production_regression/vcf_cohort"
    ));

    let pipeline_script =
        output_root.join("essential-pipelines/relatedness-segments-vcf/vcf.ibd/job.sbatch");
    assert!(pipeline_script.exists(), "expected governed essential pipeline script");
    let pipeline_body = fs::read_to_string(&pipeline_script).expect("read pipeline script");
    assert!(pipeline_body.contains("# dependency_node_ids:"));
    assert!(pipeline_body.contains("germline"));
    assert!(pipeline_body.contains(
        "#SBATCH --output=artifacts/runs/bench/slurm-dry-run/all-domains/runs/all-domain-benchmark-dry-run/vcf/relatedness-segments-vcf/vcf.ibd/germline/vcf_production_regression/sample-set/stdout.log"
    ));
}
