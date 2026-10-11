//! Explicit tiny BAM parser-proof generation, independent of planner/configuration discovery.
mod alignment;
mod authenticity;
mod contamination;
mod damage;
mod genotyping;
mod inputs;
mod kinship;
mod model;
pub mod reports;
mod sex;
pub use alignment::generate_bam_align_probe;
pub use authenticity::generate_bam_authenticity_probe;
pub use contamination::generate_bam_contamination_probe;
pub use damage::generate_bam_damage_probe;
pub use genotyping::generate_bam_genotyping_probe;
pub use kinship::generate_bam_kinship_probe;
pub use model::*;
pub use sex::generate_bam_sex_probe;
