use std::path::{Path, PathBuf};

use bijux_dna_science::compile::compile_workspace;
use bijux_dna_science::render::{
    binding_resolution_tsv, claim_evidence_tsv, decision_reasoning_tsv,
    fastq_container_reference_tsv, fastq_environment_tsv, index_json, source_archive_gaps_tsv,
    source_inventory_tsv, to_pretty_json,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn fastq_environment_slice_matches_committed_outputs() {
    let root = repo_root();
    let compiled = compile_workspace(&root).expect("compile science workspace");

    assert!(compiled.fastq_environment_rows.iter().any(|row| {
        row.stage_id == "fastq.trim_reads" && row.tool_id == "fastp" && row.is_default
    }));
    assert!(compiled.fastq_environment_rows.iter().any(|row| {
        row.stage_id == "fastq.trim_reads"
            && row.tool_id == "seqpurge"
            && row.tool_status == "disallowed"
    }));
    assert!(compiled
        .source_inventory
        .iter()
        .any(|row| row.source_id == "source.fastq.tool-registry"));
    assert!(compiled
        .fastq_container_reference_rows
        .iter()
        .any(|row| row.tool_id == "fastp" && row.version == "0.23.4"));

    assert_eq!(
        std::fs::read_to_string(
            root.join("science/generated/current/evidence/source_inventory.tsv")
        )
        .expect("read committed source inventory"),
        source_inventory_tsv(&compiled.source_inventory)
    );
    assert_eq!(
        std::fs::read_to_string(
            root.join("science/generated/current/evidence/source_archive_gaps.tsv")
        )
        .expect("read committed source archive gaps"),
        source_archive_gaps_tsv(&compiled.source_archive_gaps)
    );
    assert_eq!(
        std::fs::read_to_string(
            root.join("science/generated/current/evidence/fastq_container_reference_matrix.tsv")
        )
        .expect("read committed fastq container matrix"),
        fastq_container_reference_tsv(&compiled.fastq_container_reference_rows)
    );
    assert_eq!(
        std::fs::read_to_string(
            root.join("science/generated/current/evidence/claim_evidence_map.tsv")
        )
        .expect("read committed claim map"),
        claim_evidence_tsv(&compiled.claim_evidence_map)
    );
    assert_eq!(
        std::fs::read_to_string(
            root.join("science/generated/current/evidence/decision_reasoning_map.tsv")
        )
        .expect("read committed decision map"),
        decision_reasoning_tsv(&compiled.decision_reasoning_map)
    );
    assert_eq!(
        std::fs::read_to_string(
            root.join("science/generated/current/evidence/binding_resolution.tsv")
        )
        .expect("read committed binding map"),
        binding_resolution_tsv(&compiled.binding_resolution)
    );
    assert_eq!(
        std::fs::read_to_string(
            root.join("science/generated/current/evidence/fastq_stage_tool_environment_matrix.tsv"),
        )
        .expect("read committed fastq matrix"),
        fastq_environment_tsv(&compiled.fastq_environment_rows)
    );
    assert_eq!(
        std::fs::read_to_string(
            root.join("science/generated/current/evidence/unresolved_refs.json")
        )
        .expect("read committed unresolved refs"),
        to_pretty_json(&compiled.unresolved_refs).expect("render unresolved refs")
    );
    assert_eq!(
        std::fs::read_to_string(root.join("science/generated/indexes/science_index.json"))
            .expect("read committed science index"),
        index_json(&compiled.index).expect("render science index")
    );
}
