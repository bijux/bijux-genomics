#![cfg(feature = "bam_downstream")]
#![allow(clippy::too_many_lines)]

use anyhow::Result;
use std::path::{Path, PathBuf};

struct RepoRootOverrideGuard {
    previous: Option<std::ffi::OsString>,
}

impl RepoRootOverrideGuard {
    fn install(root: &Path) -> Self {
        let previous = std::env::var_os("BIJUX_REPO_ROOT");
        std::env::set_var("BIJUX_REPO_ROOT", root);
        Self { previous }
    }
}

impl Drop for RepoRootOverrideGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.take() {
            std::env::set_var("BIJUX_REPO_ROOT", previous);
        } else {
            std::env::remove_var("BIJUX_REPO_ROOT");
        }
    }
}

fn repo_root() -> Result<PathBuf> {
    crate::support::repo_root()
}

#[test]
fn write_local_genotyping_plan_materializes_governed_target_output() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("benchmarks/readiness/local-ready/bam.genotyping");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let plan_path = bijux_dna_api::v1::api::bam::write_local_genotyping_plan()?;
    assert_eq!(
        plan_path,
        repo_root.join("benchmarks/readiness/local-ready/bam.genotyping/plan.json")
    );
    assert!(plan_path.is_file(), "local-ready plan artifact must exist");

    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&plan_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("bam.genotyping"));
    assert_eq!(payload["tool_id"], serde_json::json!("angsd"));
    assert_eq!(payload["resources"]["threads"], serde_json::json!(2));
    assert_eq!(payload["resources"]["mem_gb"], serde_json::json!(8));
    assert_eq!(
        payload["params"]["reference"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
        )
    );
    assert_eq!(
        payload["params"]["sites"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_genotyping_candidate_sites.vcf"
        )
    );
    assert_eq!(
        payload["params"]["regions"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_genotyping_target_regions.txt"
        )
    );
    assert_eq!(
        payload["params"]["producer_contract"]["bcf"],
        serde_json::json!("benchmarks/readiness/local-ready/bam.genotyping/genotyping.bcf")
    );
    assert_eq!(
        payload["params"]["producer_contract"]["vcf"],
        serde_json::json!("benchmarks/readiness/local-ready/bam.genotyping/genotyping.vcf.gz")
    );
    assert_eq!(
        payload["params"]["sample_id"],
        serde_json::json!("human_like_genotyping_candidate_panel")
    );
    assert_eq!(payload["params"]["tool"], serde_json::json!("angsd"));

    let inputs = payload["io"]["inputs"]
        .as_array()
        .unwrap_or_else(|| panic!("plan inputs must serialize as an array"));
    let bam = inputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("bam"))
        .unwrap_or_else(|| panic!("bam input missing from local-ready genotyping payload"));
    assert_eq!(
        bam["path"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_genotyping_candidate_panel.sam"
        )
    );
    let input_index = inputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("bam_bai"))
        .unwrap_or_else(|| panic!("bam_bai input missing from local-ready genotyping payload"));
    assert_eq!(
        input_index["path"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_genotyping_candidate_panel.sam.bai"
        )
    );
    let reference = inputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("reference"))
        .unwrap_or_else(|| panic!("reference input missing from local-ready genotyping payload"));
    assert_eq!(
        reference["path"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
        )
    );
    let sites = inputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("sites"))
        .unwrap_or_else(|| panic!("sites input missing from local-ready genotyping payload"));
    assert_eq!(
        sites["path"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_genotyping_candidate_sites.vcf"
        )
    );
    let regions = inputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("regions"))
        .unwrap_or_else(|| panic!("regions input missing from local-ready genotyping payload"));
    assert_eq!(
        regions["path"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_genotyping_target_regions.txt"
        )
    );

    let outputs = payload["io"]["outputs"]
        .as_array()
        .unwrap_or_else(|| panic!("plan outputs must serialize as an array"));
    let bcf = outputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("genotyping_bcf"))
        .unwrap_or_else(|| panic!("genotyping_bcf output missing from local-ready payload"));
    assert_eq!(
        bcf["path"],
        serde_json::json!("benchmarks/readiness/local-ready/bam.genotyping/genotyping.bcf")
    );
    let vcf = outputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("genotyping_vcf"))
        .unwrap_or_else(|| panic!("genotyping_vcf output missing from local-ready payload"));
    assert_eq!(
        vcf["path"],
        serde_json::json!("benchmarks/readiness/local-ready/bam.genotyping/genotyping.vcf.gz")
    );
    let tbi = outputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("genotyping_vcf_tbi"))
        .unwrap_or_else(|| panic!("genotyping_vcf_tbi output missing from local-ready payload"));
    assert_eq!(
        tbi["path"],
        serde_json::json!("benchmarks/readiness/local-ready/bam.genotyping/genotyping.vcf.gz.tbi")
    );
    let gl = outputs
        .iter()
        .find(|artifact| artifact["name"] == serde_json::json!("genotyping_gl"))
        .unwrap_or_else(|| panic!("genotyping_gl output missing from local-ready payload"));
    assert_eq!(
        gl["path"],
        serde_json::json!("benchmarks/readiness/local-ready/bam.genotyping/genotyping.gl.json")
    );

    assert!(
        payload["command"]["template"].as_array().is_some_and(|command| command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains(
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_genotyping_candidate_panel.sam.bai"
                )
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains(
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
                )
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains(
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_genotyping_candidate_sites.vcf"
                )
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains(
                    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_genotyping_target_regions.txt"
                )
            })
        ) && command.iter().any(
            |part| part.as_str().is_some_and(|shell| {
                shell.contains("benchmarks/readiness/local-ready/bam.genotyping/genotyping.bcf")
            })
        )),
        "local-ready genotyping command must carry the governed BAI, reference, sites, regions, and BCF output"
    );
    Ok(())
}

#[test]
fn write_local_genotyping_plan_preserves_governed_command_metadata() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("benchmarks/readiness/local-ready/bam.genotyping");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let plan_path = bijux_dna_api::v1::api::bam::write_local_genotyping_plan()?;
    let payload: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&plan_path)?)?;

    assert_eq!(
        payload["out_dir"],
        serde_json::json!("benchmarks/readiness/local-ready/bam.genotyping")
    );
    assert_eq!(payload["effective_params"]["caller"], serde_json::json!("angsd"));
    assert_eq!(payload["effective_params"]["min_posterior"], serde_json::json!(0.9));
    assert_eq!(payload["effective_params"]["min_call_rate"], serde_json::json!(0.5));

    let command = payload["command"]["template"]
        .as_array()
        .and_then(|template| template.last())
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("local-ready genotyping plan must serialize a shell command"));
    assert!(
        command.contains(
            "angsd -i benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_genotyping_candidate_panel.sam"
        )
            && command.contains(
                "-sites benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_genotyping_candidate_sites.vcf"
            )
            && command.contains(
                "-rf benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_genotyping_target_regions.txt"
            )
            && command.contains("benchmarks/readiness/local-ready/bam.genotyping/genotyping.summary.json")
            && command.contains("\"min_posterior\": 0.9")
            && command.contains("\"min_call_rate\": 0.5")
            && command.contains("\"bcf_source\": \"benchmarks/readiness/local-ready/bam.genotyping/genotyping.bcf\""),
        "local-ready genotyping command must preserve the governed caller, sites, regions, summary, threshold metadata, and BCF source contract"
    );

    Ok(())
}
