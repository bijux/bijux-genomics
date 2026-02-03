//! Cross-domain FASTQ → BAM profile (aDNA).

use crate::bam::bam_adna_shotgun_profile;
use crate::fastq::{fastq_default_profile, DefaultPipelineOptions};
use crate::{Domain, EffectiveDefaults, PipelineProfile, StageNode};

#[must_use]
pub fn fastq_to_bam_adna_profile() -> PipelineProfile {
    let fastq_profile = fastq_default_profile(DefaultPipelineOptions {
        paired: true,
        enable_merge: true,
        enable_correct: false,
        enable_qc_post: true,
        enable_screen: false,
    });
    let bam_profile = bam_adna_shotgun_profile();

    let mut defaults = EffectiveDefaults::default();
    defaults.tools.extend(fastq_profile.defaults.tools.clone());
    defaults
        .params
        .extend(fastq_profile.defaults.params.clone());
    defaults.tools.extend(bam_profile.defaults.tools.clone());
    defaults.params.extend(bam_profile.defaults.params.clone());
    defaults
        .tools
        .insert("cross.align_stub".to_string(), "placeholder".to_string());
    defaults
        .params
        .insert("cross.align_stub".to_string(), serde_json::json!({}));

    PipelineProfile {
        id: "fastq-to-bam-adna",
        description: "FASTQ preprocess → alignment placeholder → BAM QC/damage",
        domains: vec![Domain::Fastq, Domain::Cross, Domain::Bam],
        graph: vec![
            StageNode {
                stage_id: "fastq.preprocess".to_string(),
            },
            StageNode {
                stage_id: "cross.align_stub".to_string(),
            },
            StageNode {
                stage_id: "bam.qc_pre".to_string(),
            },
            StageNode {
                stage_id: "bam.coverage".to_string(),
            },
            StageNode {
                stage_id: "bam.damage".to_string(),
            },
        ],
        defaults,
        invariants_preset: Some("adna"),
    }
}
