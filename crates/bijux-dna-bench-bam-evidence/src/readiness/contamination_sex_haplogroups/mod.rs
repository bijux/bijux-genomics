mod declarations;
mod input;
mod models;
mod observations;
mod projection;
mod proofs;
mod publication;
mod service;
pub use models::{
    BamContaminationSexHaplogroupsReadyReport, BamContaminationSexHaplogroupsReadyRow,
};
pub use publication::render_bam_contamination_sex_haplogroups_ready;
mod checks;
mod explanation;
