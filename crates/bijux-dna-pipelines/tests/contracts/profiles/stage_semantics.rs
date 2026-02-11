use bijux_dna_pipelines::registry::{bam_profiles, cross_profiles, fastq_profiles, vcf_profiles};
use bijux_dna_runtime::manifests::load_manifests;

fn kind_rank(kind: bijux_dna_core::contract::ArtifactKind) -> u8 {
    use bijux_dna_core::contract::ArtifactKind;
    match kind {
        ArtifactKind::Fastq => 1,
        ArtifactKind::Bam => 2,
        ArtifactKind::Vcf => 3,
        ArtifactKind::Report
        | ArtifactKind::Metrics
        | ArtifactKind::Index
        | ArtifactKind::Unknown => 255,
    }
}

fn inferred_stage_kind(stage_id: &str) -> bijux_dna_core::contract::ArtifactKind {
    if stage_id.starts_with("fastq.") {
        bijux_dna_core::contract::ArtifactKind::Fastq
    } else if stage_id.starts_with("bam.") {
        bijux_dna_core::contract::ArtifactKind::Bam
    } else if stage_id.starts_with("vcf.") {
        bijux_dna_core::contract::ArtifactKind::Vcf
    } else {
        bijux_dna_core::contract::ArtifactKind::Unknown
    }
}

#[test]
fn stage_registry_declares_semantic_io_and_versioning_metadata() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let registry =
        load_manifests(&root.join("configs").join("tool_registry.toml")).expect("load manifests");

    for (stage_id, stage) in registry.stages() {
        assert!(
            !stage.produced_artifacts.is_empty(),
            "stage {} must declare stable produced artifact names",
            stage_id
        );
        for name in &stage.produced_artifacts {
            assert!(
                !name.is_empty()
                    && name
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "stage {} produced artifact name must be stable snake_case: {}",
                stage_id,
                name
            );
        }
        assert!(
            stage.stage_semver.split('.').count() == 3
                && stage
                    .stage_semver
                    .split('.')
                    .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())),
            "stage {} must declare semver version",
            stage_id
        );
    }
}

#[test]
fn pipeline_stage_sequences_are_type_correct_across_domains() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let registry =
        load_manifests(&root.join("configs").join("tool_registry.toml")).expect("load manifests");

    let mut profiles = Vec::new();
    profiles.extend(fastq_profiles());
    profiles.extend(bam_profiles());
    profiles.extend(cross_profiles());
    profiles.extend(vcf_profiles());

    for profile in profiles {
        let mut prev_output: Option<bijux_dna_core::contract::ArtifactKind> = None;
        for stage_id in &profile.capabilities.required_stages {
            let key = bijux_dna_core::ids::StageId::new((*stage_id).to_string());
            let (input_kind, output_kind) = registry
                .stages()
                .get(&key)
                .map(|stage| (stage.input_kind, stage.output_kind))
                .unwrap_or_else(|| {
                    let inferred = inferred_stage_kind(stage_id);
                    (inferred, inferred)
                });
            if let Some(prev) = prev_output {
                if kind_rank(prev) != 255 && kind_rank(input_kind) != 255 {
                    assert_eq!(
                        prev, input_kind,
                        "pipeline {} has incompatible stage chain: previous output {:?} -> {} input {:?}",
                        profile.id, prev, stage_id, input_kind
                    );
                }
            }
            prev_output = Some(output_kind);
        }
    }
}
