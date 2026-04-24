use anyhow::Result;
use bijux_dna_domain_fastq::{preflight_stage, FastqArtifactKind};

#[test]
fn preflight_accepts_paired_input_for_pair_preserving_transforms() -> Result<()> {
    for stage_id in [
        "fastq.trim_reads",
        "fastq.filter_reads",
        "fastq.filter_low_complexity",
        "fastq.trim_polyg_tails",
        "fastq.trim_terminal_damage",
        "fastq.deplete_host",
        "fastq.deplete_reference_contaminants",
        "fastq.deplete_rrna",
        "fastq.validate_reads",
        "fastq.detect_adapters",
        "fastq.profile_read_lengths",
        "fastq.profile_overrepresented_sequences",
        "fastq.profile_reads",
    ] {
        preflight_stage(stage_id, FastqArtifactKind::PairedEnd)?;
    }
    Ok(())
}

#[test]
fn preflight_rejects_single_end_input_for_paired_only_stages() {
    for stage_id in ["fastq.merge_pairs", "fastq.correct_errors", "fastq.extract_umis"] {
        let Err(err) = preflight_stage(stage_id, FastqArtifactKind::SingleEnd) else {
            panic!("single-end inputs must be rejected for {stage_id}");
        };
        assert!(err.to_string().contains("accepted kinds"));
    }
}
