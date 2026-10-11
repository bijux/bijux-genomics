use anyhow::Result;
use bijux_dna_bench_bam_proofs::parser_probes::{
    generate_bam_authenticity_probe, generate_bam_damage_probe, DamageProbe,
};
use std::path::Path;

#[test]
fn invalid_damage_observations_reject_before_probe_effects() -> Result<()> {
    let dir = tempfile::tempdir()?;
    for (c_to_t_5p, g_to_a_3p) in [(f64::NAN, 0.1), (0.1, f64::INFINITY), (-0.1, 0.2), (0.1, 1.1)] {
        let request = DamageProbe {
            tool_id: "declared-tool",
            bam: Path::new("unread.bam"),
            c_to_t_5p,
            g_to_a_3p,
            strict_profile_upgraded: false,
        };
        let out = dir.path().join("proofs");
        assert!(generate_bam_damage_probe(dir.path(), &out, &request).is_err());
        assert!(generate_bam_authenticity_probe(dir.path(), &out, &request).is_err());
        assert!(!out.exists());
    }
    Ok(())
}

#[test]
fn unknown_alignment_backend_refuses_before_reads_or_output_creation() -> Result<()> {
    use bijux_dna_bench_bam_proofs::parser_probes::{generate_bam_align_probe, AlignmentProbe};
    let dir = tempfile::tempdir()?;
    let read_group = bijux_dna_domain_bam::params::ReadGroupSpec::with_defaults("sample");
    for tool_id in ["", "unknown", "bowtie2-extra"] {
        let out = dir.path().join("proofs");
        let request = AlignmentProbe {
            tool_id,
            reference_fasta: Path::new("unread.fa"),
            input_r1: Path::new("unread.fq"),
            input_r2: None,
            sample_id: "sample",
            read_group: &read_group,
            sensitivity_profile: None,
            preset: None,
            seed_length: None,
        };
        let error = generate_bam_align_probe(dir.path(), &out, &request)
            .err()
            .ok_or_else(|| anyhow::anyhow!("unknown tool accepted"))?;
        assert!(error.to_string().contains("unsupported alignment parser proof tool"));
        assert!(!out.exists());
    }
    Ok(())
}

#[test]
fn contamination_tool_scope_conflicts_preserve_prior_proof() -> Result<()> {
    use bijux_dna_bench_bam_proofs::parser_probes::{
        generate_bam_contamination_probe, ContaminationProbe,
    };
    use bijux_dna_domain_bam::params::ContaminationScope;
    let dir = tempfile::tempdir()?;
    let output = dir.path().join("bam_contamination/contamination.summary.json");
    std::fs::create_dir_all(output.parent().ok_or_else(|| anyhow::anyhow!("proof parent"))?)?;
    std::fs::write(&output, b"prior proof")?;
    for (tool_id, scope) in [
        ("unknown", ContaminationScope::Both),
        ("schmutzi", ContaminationScope::Nuclear),
        ("contammix", ContaminationScope::Mito),
        ("verifybamid2", ContaminationScope::Mito),
    ] {
        let request = ContaminationProbe { tool_id, scope, assumptions: "declared fixture" };
        assert!(generate_bam_contamination_probe(dir.path(), &request).is_err());
        assert_eq!(std::fs::read(&output)?, b"prior proof");
        let fresh = dir.path().join("fresh");
        assert!(generate_bam_contamination_probe(&fresh, &request).is_err());
        assert!(!fresh.exists());
    }
    Ok(())
}

#[test]
fn contamination_parser_proof_keeps_method_and_tool_identity_aligned() -> Result<()> {
    use bijux_dna_bench_bam_proofs::parser_probes::{
        generate_bam_contamination_probe, ContaminationProbe,
    };
    use bijux_dna_domain_bam::params::ContaminationScope;
    let dir = tempfile::tempdir()?;
    for (tool_id, scope, scope_label) in [
        ("schmutzi", ContaminationScope::Mito, "scope:mito"),
        ("contammix", ContaminationScope::Nuclear, "scope:nuclear"),
        ("verifybamid2", ContaminationScope::Both, "scope:both"),
    ] {
        let request =
            ContaminationProbe { tool_id, scope, assumptions: "synthetic parser fixture" };
        let proof = generate_bam_contamination_probe(dir.path(), &request)?;
        let value: serde_json::Value = serde_json::from_slice(&std::fs::read(&proof.proof_path)?)?;
        assert_eq!(proof.observed_tool_id, tool_id);
        assert_eq!(value.get("method").and_then(serde_json::Value::as_str), Some(tool_id));
        assert_eq!(
            value
                .get("assumptions")
                .and_then(serde_json::Value::as_array)
                .and_then(|rows| rows.get(1))
                .and_then(serde_json::Value::as_str),
            Some(scope_label)
        );
        assert!(bijux_dna_domain_bam::metrics::parse_contamination_json(&proof.proof_path).is_ok());
    }
    Ok(())
}

#[test]
fn selected_alignment_backend_matches_native_provenance_and_declared_reference() -> Result<()> {
    use bijux_dna_bench_bam_proofs::parser_probes::{generate_bam_align_probe, AlignmentProbe};
    let dir = tempfile::tempdir()?;
    std::fs::write(dir.path().join("reference.fa"), ">chr1\nAACCGGTTAACCGGTT\n")?;
    std::fs::write(
        dir.path().join("reads.fq"),
        "@read-local\nGGTTAA\n+\nIIIIII\n@read-unmapped\nTTTTTTTTTT\n+\nIIIIIIIIII\n",
    )?;
    let read_group = bijux_dna_domain_bam::params::ReadGroupSpec::with_defaults("sample");
    let cwd = std::env::current_dir()?;
    for (tool_id, mode) in [("bwa", "end_to_end"), ("bowtie2", "local")] {
        let request = AlignmentProbe {
            tool_id,
            reference_fasta: Path::new("reference.fa"),
            input_r1: Path::new("reads.fq"),
            input_r2: None,
            sample_id: "sample",
            read_group: &read_group,
            sensitivity_profile: Some("very_sensitive_local"),
            preset: Some("default"),
            seed_length: Some(12),
        };
        let proof = generate_bam_align_probe(dir.path(), &dir.path().join(tool_id), &request)?;
        let provenance: bijux_dna_domain_bam::BamAlignmentProvenanceV1 =
            serde_json::from_slice(&std::fs::read(&proof.proof_path)?)?;
        assert_eq!(proof.observed_tool_id, tool_id);
        assert_eq!(provenance.backend_tool_id, tool_id);
        assert_eq!(provenance.mode.as_deref(), Some(mode));
        assert_eq!(provenance.reference_fasta, Path::new("reference.fa"));
        assert!(dir.path().join(tool_id).join("bam_align/align.bam").is_file());
        assert_eq!(std::env::current_dir()?, cwd);
    }
    Ok(())
}
