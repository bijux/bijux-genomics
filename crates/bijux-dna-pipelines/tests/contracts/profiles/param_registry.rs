use std::collections::{BTreeMap, BTreeSet};

use bijux_dna_core::ids::StageId;
use bijux_dna_domain_bam::BamStage;
use bijux_dna_domain_fastq::stage_param_descriptor;
use bijux_dna_pipelines::registry::{bam_profiles, cross_profiles, fastq_profiles};
use bijux_dna_pipelines::DefaultParams;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct StagesConfig {
    stages: Vec<StageEntry>,
}

#[derive(Debug, Deserialize)]
struct StageEntry {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ParamRegistryFile {
    entries: Vec<ParamRegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct ParamRegistryEntry {
    stage_id: String,
    param_type_id: String,
    schema_version: String,
}

fn expected_registry(stage_id: &str) -> (String, String) {
    let stage = StageId::new(stage_id.to_string());
    if stage_id.starts_with("fastq.") {
        if let Some(descriptor) = stage_param_descriptor(&stage) {
            let schema_version = if descriptor.schema_version == "v1"
                && descriptor.param_type_id.starts_with("fastq.")
            {
                "bijux.fastq.params.v1"
            } else {
                descriptor.schema_version
            };
            return (descriptor.param_type_id.to_string(), schema_version.to_string());
        }
        if matches!(
            stage_id,
            "fastq.normalize_abundance"
                | "fastq.infer_asvs"
                | "fastq.remove_chimeras"
                | "fastq.cluster_otus"
                | "fastq.normalize_primers"
        ) {
            return (stage_id.to_string(), "bijux.fastq.params.v1".to_string());
        }
        return ("paramless".to_string(), "none".to_string());
    }
    if stage_id.starts_with("bam.") {
        if BamStage::try_from(stage_id).is_ok() {
            return (stage_id.to_string(), "bijux.bam.params.v1".to_string());
        }
        return ("paramless".to_string(), "none".to_string());
    }
    ("paramless".to_string(), "none".to_string())
}

fn fastq_param_kind_compatible(expected_kind: &str, actual_kind: &str) -> bool {
    expected_kind == actual_kind
        || (expected_kind.starts_with("fastq.") && actual_kind.starts_with("fastq."))
}

fn profile_param_kind(params: &DefaultParams) -> &'static str {
    match params {
        DefaultParams::FastqValidate(_) => "fastq.validate",
        DefaultParams::FastqStats(_) => "fastq.stats",
        DefaultParams::FastqReadLengthProfile(_) => "fastq.profile_read_lengths",
        DefaultParams::FastqCorrect(_) => "fastq.correct_errors",
        DefaultParams::FastqUmi(_) => "fastq.extract_umis",
        DefaultParams::FastqDetectAdapters(_) => "fastq.detect_adapters",
        DefaultParams::FastqTrim(_) => "fastq.trim_reads",
        DefaultParams::FastqTrimTerminalDamage(_) => "fastq.trim_terminal_damage",
        DefaultParams::FastqTrimPolygTails(_) => "fastq.trim_polyg_tails",
        DefaultParams::FastqFilter(_) => "fastq.filter_reads",
        DefaultParams::FastqOverrepresentedProfile(_) => "fastq.profile_overrepresented_sequences",
        DefaultParams::FastqQcPost(_) => "fastq.report_qc",
        DefaultParams::FastqPreprocess(_) => "fastq.preprocess",
        DefaultParams::FastqMerge(_) => "fastq.merge_pairs",
        DefaultParams::FastqScreen(_) => "fastq.screen_taxonomy",
        DefaultParams::FastqHostDepletion(_) => "fastq.deplete_host",
        DefaultParams::FastqReferenceContaminantDepletion(_) => {
            "fastq.deplete_reference_contaminants"
        }
        DefaultParams::FastqRrna(_) => "fastq.deplete_rrna",
        DefaultParams::FastqPrimerNormalization(_) => "fastq.normalize_primers",
        DefaultParams::FastqChimeraDetection(_) => "fastq.remove_chimeras",
        DefaultParams::FastqAsvInference(_) => "fastq.infer_asvs",
        DefaultParams::FastqOtuClustering(_) => "fastq.cluster_otus",
        DefaultParams::FastqAbundanceNormalization(_) => "fastq.normalize_abundance",
        DefaultParams::Bam(_) => "bam.effective",
        DefaultParams::Vcf(_) => "vcf.effective",
        DefaultParams::Empty(_) => "paramless",
    }
}

fn registry_entries() -> BTreeMap<String, (String, String)> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("configs")
        .join("ci");
    let registry_text = std::fs::read_to_string(root.join("params").join("param_registry.toml"))
        .unwrap_or_else(|err| panic!("read param_registry.toml failed: {err}"));
    let registry: ParamRegistryFile = toml::from_str(&registry_text)
        .unwrap_or_else(|err| panic!("parse param_registry.toml failed: {err}"));
    registry
        .entries
        .into_iter()
        .map(|entry| (entry.stage_id, (entry.param_type_id, entry.schema_version)))
        .collect()
}

#[test]
fn param_registry_covers_all_stage_ids() {
    let entries = registry_entries();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("configs")
        .join("ci");
    let stages_text = std::fs::read_to_string(root.join("stages").join("stages.toml"))
        .unwrap_or_else(|err| panic!("read stages.toml failed: {err}"));
    let stages: StagesConfig = toml::from_str(&stages_text)
        .unwrap_or_else(|err| panic!("parse stages.toml failed: {err}"));

    let stage_ids: BTreeSet<String> = stages.stages.into_iter().map(|stage| stage.id).collect();
    let registry_ids: BTreeSet<String> = entries.keys().cloned().collect();
    assert!(
        stage_ids.is_subset(&registry_ids),
        "param registry must include every stage id from configs/ci/stages/stages.toml",
    );
}

#[test]
fn param_registry_matches_domain_mappings() {
    let entries = registry_entries();
    for (stage_id, (param_type_id, schema_version)) in entries {
        let (expected_type, expected_schema) = expected_registry(&stage_id);
        assert_eq!(param_type_id, expected_type, "registry param type mismatch for {stage_id}",);
        assert_eq!(
            schema_version, expected_schema,
            "registry schema version mismatch for {stage_id}",
        );
    }
}

#[test]
fn supported_stages_do_not_use_wrong_param_kind() {
    let entries = registry_entries();
    let mut profiles = Vec::new();
    profiles.extend(fastq_profiles());
    profiles.extend(bam_profiles());
    profiles.extend(cross_profiles());

    for profile in profiles {
        for stage in &profile.capabilities.required_stages {
            let stage_id = StageId::new((*stage).to_string());
            let default_expected = expected_registry(stage);
            let (expected_kind, _schema) = entries.get(stage).cloned().unwrap_or(default_expected);
            if expected_kind == "paramless" {
                continue;
            }
            let params = profile.defaults.params.get(&stage_id).unwrap_or_else(|| {
                panic!("profile {} missing params for stage {stage}", profile.id)
            });
            let actual_kind = profile_param_kind(params);
            if stage.starts_with("bam.") {
                assert!(
                    matches!(params, DefaultParams::Bam(_)),
                    "profile {} stage {} must use BAM param kind, got {}",
                    profile.id,
                    stage,
                    actual_kind
                );
            } else if stage.starts_with("vcf.") {
                assert!(
                    matches!(params, DefaultParams::Vcf(_)),
                    "profile {} stage {} must use VCF param kind, got {}",
                    profile.id,
                    stage,
                    actual_kind
                );
            } else {
                assert!(
                    fastq_param_kind_compatible(&expected_kind, actual_kind),
                    "profile {} stage {} uses wrong param kind",
                    profile.id,
                    stage
                );
            }
        }
    }
}
