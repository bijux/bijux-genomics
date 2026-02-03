//! Cross-domain FASTQ → BAM profile (aDNA).

use crate::bam::bam_adna_shotgun_profile;
use crate::fastq::{fastq_default_profile, DefaultPipelineOptions};
use crate::{Domain, EffectiveDefaults, PipelineCapabilities, PipelineProfile, StageNode};
use bijux_domain_bam::params::{AlignEffectiveParams, ReadGroupSpec};
use bijux_domain_bam::types::ReadGroupPolicy;

fn base_defaults() -> (PipelineProfile, PipelineProfile, EffectiveDefaults) {
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
    (fastq_profile, bam_profile, defaults)
}

fn align_defaults(preset: &str) -> serde_json::Value {
    serde_json::to_value(AlignEffectiveParams {
        aligner: "bwa".to_string(),
        preset: preset.to_string(),
        threads: 1,
        reference: "reference.fasta".to_string(),
        reference_digest: "unknown".to_string(),
        rg_policy: ReadGroupPolicy::Regenerate,
        read_group: ReadGroupSpec::with_defaults("sample"),
        build_indices: false,
        emit_stats: true,
    })
    .unwrap_or(serde_json::Value::Null)
}

#[must_use]
pub fn fastq_to_bam_adna_shotgun_profile() -> PipelineProfile {
    let (_fastq_profile, _bam_profile, mut defaults) = base_defaults();
    defaults
        .tools
        .insert("core.prepare_reference".to_string(), "samtools".to_string());
    defaults
        .params
        .insert("core.prepare_reference".to_string(), serde_json::json!({}));
    defaults
        .tools
        .insert("bam.align".to_string(), "bwa".to_string());
    defaults
        .params
        .insert("bam.align".to_string(), align_defaults("adna_short"));

    PipelineProfile {
        id: "fastq-to-bam-adna-shotgun",
        description: "FASTQ preprocess → align → BAM QC/damage (aDNA shotgun)",
        domains: vec![Domain::Fastq, Domain::Cross, Domain::Bam],
        graph: vec![
            StageNode {
                stage_id: "fastq.preprocess".to_string(),
            },
            StageNode {
                stage_id: "core.prepare_reference".to_string(),
            },
            StageNode {
                stage_id: "bam.align".to_string(),
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
        capabilities: PipelineCapabilities {
            required_inputs: vec!["fastq", "reference"],
            produces_outputs: vec!["fastq", "bam", "bam.metrics"],
            supports_benchmarking: false,
        },
    }
}

#[must_use]
pub fn fastq_to_bam_default_profile() -> PipelineProfile {
    let (_fastq_profile, _bam_profile, mut defaults) = base_defaults();
    defaults
        .tools
        .insert("core.prepare_reference".to_string(), "samtools".to_string());
    defaults
        .params
        .insert("core.prepare_reference".to_string(), serde_json::json!({}));
    defaults
        .tools
        .insert("bam.align".to_string(), "bwa".to_string());
    defaults
        .params
        .insert("bam.align".to_string(), align_defaults("default"));

    PipelineProfile {
        id: "fastq-to-bam-default",
        description: "FASTQ preprocess → align → BAM QC/damage (modern defaults)",
        domains: vec![Domain::Fastq, Domain::Cross, Domain::Bam],
        graph: vec![
            StageNode {
                stage_id: "fastq.preprocess".to_string(),
            },
            StageNode {
                stage_id: "core.prepare_reference".to_string(),
            },
            StageNode {
                stage_id: "bam.align".to_string(),
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
        invariants_preset: None,
        capabilities: PipelineCapabilities {
            required_inputs: vec!["fastq", "reference"],
            produces_outputs: vec!["fastq", "bam", "bam.metrics"],
            supports_benchmarking: false,
        },
    }
}
