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
