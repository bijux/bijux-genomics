use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_core::ids::StageId;

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
}

#[test]
fn non_general_branch_contracts_remain_explicit_and_outside_generic_defaults() {
    let contracts = bijux_dna_domain_fastq::non_general_genomics_branch_contracts();
    let default_shotgun = bijux_dna_domain_fastq::default_shotgun_preprocess_stage_order()
        .into_iter()
        .map(|stage| stage.to_string())
        .collect::<Vec<_>>();

    assert_eq!(contracts.len(), 3);
    for contract in &contracts {
        assert!(contract.forbidden_from_generic_defaults);
        assert!(
            !default_shotgun.iter().any(|stage| stage == &contract.stage_id),
            "{} must stay out of generic shotgun defaults",
            contract.stage_id
        );
    }
}

#[test]
fn non_general_branch_lookup_tracks_stage_specific_assumptions() {
    let contract = bijux_dna_domain_fastq::non_general_genomics_branch_contract_for_stage(
        &StageId::from_static("fastq.infer_asvs"),
    )
    .expect("infer_asvs contract");
    assert_eq!(contract.governed_example_id, "fastq_edna_mini");
    assert!(contract.assay_assumptions.iter().any(|entry| entry.contains("marker-specific")));
}

#[test]
fn edna_example_docs_call_out_specialized_branch_selection() -> Result<()> {
    let readme =
        std::fs::read_to_string(workspace_root()?.join("examples/fastq/edna-mini/README.md"))?;
    assert!(
        readme.contains("generic FASTQ defaults intentionally exclude"),
        "eDNA example must explain why specialized stages are opt-in"
    );
    assert!(readme.contains("fastq.cluster_otus"));
    assert!(readme.contains("fastq.remove_chimeras"));
    Ok(())
}
