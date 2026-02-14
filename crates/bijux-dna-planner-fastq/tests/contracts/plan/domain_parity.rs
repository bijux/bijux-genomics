use anyhow::Result;
use bijux_dna_domain_fastq::FastqPipelineMode;
use bijux_dna_domain_fastq::FASTQ_STAGE_ID_CATALOG;
use bijux_dna_planner_fastq::{default_pipeline_spec, DefaultPipelineOptions};
use std::collections::BTreeSet;

fn domain_fastq_stage_ids() -> Result<BTreeSet<String>> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("workspace root"))?;
    let path = root.join("domain/fastq/index.yaml");
    let raw = std::fs::read_to_string(path)?;
    let mut in_stage_ids = false;
    let mut out = BTreeSet::new();
    for line in raw.lines() {
        if line.trim() == "stage_ids:" {
            in_stage_ids = true;
            continue;
        }
        if in_stage_ids {
            if !line.starts_with("  - ") {
                break;
            }
            out.insert(line.trim_start_matches("  - ").trim().to_string());
        }
    }
    if out.is_empty() {
        return Err(anyhow::anyhow!(
            "domain/fastq/index.yaml missing or empty stage_ids"
        ));
    }
    Ok(out)
}

#[test]
fn fastq_domain_yaml_matches_rust_stage_catalog() -> Result<()> {
    let yaml = domain_fastq_stage_ids()?;
    let rust = FASTQ_STAGE_ID_CATALOG
        .iter()
        .map(|s| s.to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        yaml, rust,
        "domain fastq stage IDs drifted from Rust catalog"
    );
    Ok(())
}

#[test]
fn fastq_planner_registry_covers_new_amplicon_stages() {
    let stages = bijux_dna_planner_fastq::stage_api::fastq::registry()
        .into_iter()
        .map(|s| s.id.to_string())
        .collect::<BTreeSet<_>>();
    for required in [
        "fastq.primer_normalization",
        "fastq.chimera_detection",
        "fastq.asv_inference",
        "fastq.otu_clustering",
        "fastq.abundance_normalization",
    ] {
        assert!(
            stages.contains(required),
            "planner registry missing stage {required}"
        );
    }
}

#[test]
fn amplicon_mode_pipeline_emits_amplicon_stages() {
    let spec = default_pipeline_spec(DefaultPipelineOptions {
        paired: false,
        enable_merge: false,
        enable_correct: false,
        enable_qc_post: true,
        enable_screen: false,
        mode: FastqPipelineMode::Amplicon,
    });
    for required in [
        "fastq.primer_normalization",
        "fastq.chimera_detection",
        "fastq.abundance_normalization",
    ] {
        assert!(
            spec.stages.iter().any(|stage| stage == required),
            "amplicon mode missing stage {required}"
        );
    }
}
