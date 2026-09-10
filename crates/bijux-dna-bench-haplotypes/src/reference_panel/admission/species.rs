//! reference panel admission species.
use bijux_dna_domain_vcf::contracts::ContigSpec;
use bijux_dna_domain_vcf::contracts::SpeciesContext;

pub(crate) fn governed_species_context() -> SpeciesContext {
    SpeciesContext {
        species_id: "Homo sapiens".to_string(),
        build_id: "GRCh38".to_string(),
        contig_set_digest: "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
            .to_string(),
        contigs: vec![
            ContigSpec { name: "1".to_string(), length_bp: 248_956_422 },
            ContigSpec { name: "2".to_string(), length_bp: 242_193_529 },
            ContigSpec { name: "chr1".to_string(), length_bp: 248_956_422 },
            ContigSpec { name: "chr2".to_string(), length_bp: 242_193_529 },
        ],
        sex_system: "xy".to_string(),
        par_policy: "grch38_par".to_string(),
        default_coverage_regime: None,
    }
}
