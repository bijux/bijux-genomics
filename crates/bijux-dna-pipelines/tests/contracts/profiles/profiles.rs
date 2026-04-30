/// Snapshot intent: verifies stable, reviewed output for this contract.
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use bijux_dna_core::id_catalog;
use bijux_dna_pipelines::bam::{
    bam_adna_capture_profile, bam_adna_shotgun_profile, bam_default_profile,
    bam_reference_adna_profile,
};
use bijux_dna_pipelines::cross::{
    bam_to_vcf_default_profile, cross_workflow_templates, fastq_to_bam_adna_shotgun_profile,
    fastq_to_bam_default_profile, fastq_to_vcf_minimal_profile,
};
use bijux_dna_pipelines::fastq::{
    fastq_amplicon_standard_profile, fastq_amplicon_umi_profile,
    fastq_contaminant_depletion_profile, fastq_default_profile, fastq_edna_metabarcoding_profile,
    fastq_host_depletion_profile, fastq_minimal_profile, fastq_qc_only_profile,
    fastq_reference_adna_profile, fastq_rrna_depletion_profile, fastq_trim_qc_profile,
    fastq_umi_profile,
};
use bijux_dna_pipelines::vcf::vcf_reference_basic_profile;
use bijux_dna_testkit::snapshot_name;
use insta::assert_json_snapshot;

fn profile_drift_components(profile: &bijux_dna_pipelines::PipelineProfile) -> serde_json::Value {
    use bijux_dna_core::prelude::hashing::params_hash;

    let tool_map = profile
        .defaults
        .tools
        .iter()
        .map(|(stage, tool)| (stage.as_str().to_string(), tool.as_str().to_string()))
        .collect::<BTreeMap<_, _>>();
    let param_map = profile
        .defaults
        .params
        .iter()
        .map(|(stage, params)| {
            (
                stage.as_str().to_string(),
                params_hash(&params.to_json()).expect("hash profile params"),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut tool_bindings = serde_json::to_value(&tool_map).expect("serialize tools map");
    let mut param_bindings = serde_json::to_value(&param_map).expect("serialize params hash map");
    prune_bam_downstream(&mut tool_bindings);
    prune_bam_downstream(&mut param_bindings);

    let tool_hash = params_hash(&tool_bindings)
        .expect("hash tools map");
    let params_hash_value = params_hash(&param_bindings).expect("hash params map");

    serde_json::json!({
        "tool_bindings_hash": tool_hash,
        "param_bindings_hash": params_hash_value,
        "tool_bindings": tool_bindings,
        "param_bindings": param_bindings,
    })
}

fn canonical_sha256_hex(value: &serde_json::Value) -> String {
    use sha2::Digest;
    use std::fmt::Write as _;

    let bytes = bijux_dna_core::contract::canonical::to_canonical_json_bytes(value)
        .expect("canonicalize contract payload");
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}

fn prune_bam_downstream(value: &mut serde_json::Value) {
    let banned = ["bam.genotyping", "bam.haplogroups", "bam.kinship"];
    match value {
        serde_json::Value::Array(items) => {
            items.retain(|item| !item.as_str().is_some_and(|entry| banned.contains(&entry)));
            for item in items {
                prune_bam_downstream(item);
            }
        }
        serde_json::Value::Object(map) => {
            for key in banned {
                map.remove(key);
            }
            for value in map.values_mut() {
                prune_bam_downstream(value);
            }
        }
        _ => {}
    }
}

fn snapshot_settings() -> insta::Settings {
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    settings
}

#[test]
fn bam_default_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "bam_default_profile");
    let mut json = serde_json::to_value(bam_default_profile()).expect("serialize profile");
    prune_bam_downstream(&mut json);
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn bam_adna_shotgun_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "bam_adna_shotgun_profile");
    let mut json = serde_json::to_value(bam_adna_shotgun_profile()).expect("serialize profile");
    prune_bam_downstream(&mut json);
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn bam_adna_capture_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "bam_adna_capture_profile");
    let mut json = serde_json::to_value(bam_adna_capture_profile()).expect("serialize profile");
    prune_bam_downstream(&mut json);
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn bam_reference_adna_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "bam_reference_adna_profile");
    let mut json = serde_json::to_value(bam_reference_adna_profile()).expect("serialize profile");
    prune_bam_downstream(&mut json);
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_default_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_default_profile");
    let json = serde_json::to_value(fastq_default_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_amplicon_standard_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_amplicon_standard_profile");
    let json = serde_json::to_value(fastq_amplicon_standard_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_amplicon_umi_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_amplicon_umi_profile");
    let json = serde_json::to_value(fastq_amplicon_umi_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_contaminant_depletion_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_contaminant_depletion_profile");
    let json =
        serde_json::to_value(fastq_contaminant_depletion_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_edna_metabarcoding_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_edna_metabarcoding_profile");
    let json = serde_json::to_value(fastq_edna_metabarcoding_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_host_depletion_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_host_depletion_profile");
    let json = serde_json::to_value(fastq_host_depletion_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_minimal_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_minimal_profile");
    let json = serde_json::to_value(fastq_minimal_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_reference_adna_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_reference_adna_profile");
    let json = serde_json::to_value(fastq_reference_adna_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_qc_only_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_qc_only_profile");
    let json = serde_json::to_value(fastq_qc_only_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_rrna_depletion_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_rrna_depletion_profile");
    let json = serde_json::to_value(fastq_rrna_depletion_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_trim_qc_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_trim_qc_profile");
    let json = serde_json::to_value(fastq_trim_qc_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_umi_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_umi_profile");
    let json = serde_json::to_value(fastq_umi_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_reference_adna_profile_golden_stage_tool_and_param_hashes_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_reference_adna_profile_golden");
    let profile = fastq_reference_adna_profile();
    let stage_list = profile
        .capabilities
        .required_stages
        .iter()
        .map(|stage| (*stage).to_string())
        .collect::<Vec<_>>();
    let mut tool_ids = std::collections::BTreeMap::new();
    let mut param_hashes = std::collections::BTreeMap::new();
    for (stage, tool) in &profile.defaults.tools {
        tool_ids.insert(stage.as_str().to_string(), tool.as_str().to_string());
    }
    for (stage, params) in &profile.defaults.params {
        let hash = bijux_dna_core::prelude::hashing::params_hash(&params.to_json())
            .expect("hash default params");
        param_hashes.insert(stage.as_str().to_string(), hash);
    }
    let json = serde_json::json!({
        "profile_id": profile.id,
        "required_stages": stage_list,
        "tool_ids": tool_ids,
        "param_hashes": param_hashes,
    });
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn bam_to_vcf_default_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "bam_to_vcf_default_profile");
    let json = serde_json::to_value(bam_to_vcf_default_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_to_vcf_minimal_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_to_vcf_minimal_profile");
    let json = serde_json::to_value(fastq_to_vcf_minimal_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn cross_workflow_templates_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "cross_workflow_templates");
    let json = serde_json::to_value(cross_workflow_templates()).expect("serialize templates");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn vcf_reference_basic_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "vcf_reference_basic_profile");
    let json = serde_json::to_value(vcf_reference_basic_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn cross_fastq_to_bam_adna_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_to_bam_adna_shotgun_profile");
    let mut json =
        serde_json::to_value(fastq_to_bam_adna_shotgun_profile()).expect("serialize profile");
    prune_bam_downstream(&mut json);
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn cross_fastq_to_bam_default_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_to_bam_default_profile");
    let mut json = serde_json::to_value(fastq_to_bam_default_profile()).expect("serialize profile");
    prune_bam_downstream(&mut json);
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn profile_hash_contracts_are_manifest_backed_and_unique() {
    let mut seen_hashes = HashSet::new();
    let mut profiles = Vec::new();
    profiles.extend(bijux_dna_pipelines::registry::fastq_profiles());
    profiles.extend(bijux_dna_pipelines::registry::bam_profiles());
    profiles.extend(bijux_dna_pipelines::registry::cross_profiles());
    profiles.extend(bijux_dna_pipelines::registry::vcf_profiles());

    for profile in profiles {
        let manifest = profile.profile_manifest();
        let profile_hash = profile.profile_hash();
        let drift_components = profile_drift_components(&profile);

        assert_eq!(
            manifest.pipeline_id,
            profile.id.as_str(),
            "profile manifest must preserve the profile id"
        );

        let manifest_json = serde_json::to_value(&manifest).expect("serialize profile manifest");
        assert_eq!(
            profile_hash,
            canonical_sha256_hex(&manifest_json),
            "profile hash must be derived from the canonical profile manifest for {}",
            profile.id.as_str()
        );

        assert!(
            seen_hashes.insert(profile_hash),
            "profile hashes must stay unique across the registry; duplicate for {}",
            profile.id.as_str()
        );

        let mut expected_tool_bindings =
            serde_json::to_value(&manifest.tool_ids).expect("serialize tool bindings");
        let mut expected_param_bindings =
            serde_json::to_value(&manifest.param_hashes).expect("serialize param bindings");
        prune_bam_downstream(&mut expected_tool_bindings);
        prune_bam_downstream(&mut expected_param_bindings);

        assert_eq!(
            drift_components["tool_bindings"],
            expected_tool_bindings,
            "tool binding drift surface must mirror manifest tool ids for {}",
            profile.id.as_str()
        );
        assert_eq!(
            drift_components["param_bindings"],
            expected_param_bindings,
            "parameter drift surface must mirror manifest param hashes for {}",
            profile.id.as_str()
        );

        let expected_tool_hash = canonical_sha256_hex(&drift_components["tool_bindings"]);
        let expected_param_hash = canonical_sha256_hex(&drift_components["param_bindings"]);
        assert_eq!(
            drift_components["tool_bindings_hash"],
            serde_json::Value::String(expected_tool_hash),
            "tool binding drift hash must match the canonical tool binding map for {}",
            profile.id.as_str()
        );
        assert_eq!(
            drift_components["param_bindings_hash"],
            serde_json::Value::String(expected_param_hash),
            "parameter drift hash must match the canonical param binding map for {}",
            profile.id.as_str()
        );
    }
}

#[test]
fn profile_hash_depends_on_manifest_only_not_capability_ordering() {
    let mut profile = fastq_default_profile();
    let original_hash = profile.profile_hash();
    profile.capabilities.required_stages.reverse();
    profile.capabilities.report_sections.reverse();
    profile.capabilities.required_metrics.reverse();
    assert_eq!(
        original_hash,
        profile.profile_hash(),
        "profile hash must stay stable when non-manifest capability ordering changes"
    );
}

#[test]
fn production_fastq_profiles_cover_iteration_14_templates() {
    let profiles = [
        fastq_qc_only_profile(),
        fastq_trim_qc_profile(),
        fastq_umi_profile(),
        fastq_host_depletion_profile(),
        fastq_rrna_depletion_profile(),
        fastq_contaminant_depletion_profile(),
        fastq_amplicon_standard_profile(),
        fastq_amplicon_umi_profile(),
        fastq_edna_metabarcoding_profile(),
    ];

    let required_stage_sets = profiles
        .iter()
        .map(|profile| {
            (
                profile.id.as_str(),
                profile.capabilities.required_stages.iter().map(String::as_str).collect::<Vec<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    assert!(required_stage_sets[id_catalog::PIPELINE_FASTQ_QC_ONLY].contains(&"fastq.report_qc"));
    assert!(required_stage_sets[id_catalog::PIPELINE_FASTQ_TRIM_QC].contains(&"fastq.trim_reads"));
    assert!(required_stage_sets[id_catalog::PIPELINE_FASTQ_UMI].contains(&"fastq.extract_umis"));
    assert!(required_stage_sets[id_catalog::PIPELINE_FASTQ_HOST_DEPLETION]
        .contains(&"fastq.deplete_host"));
    assert!(required_stage_sets[id_catalog::PIPELINE_FASTQ_RRNA_DEPLETION]
        .contains(&"fastq.deplete_rrna"));
    assert!(required_stage_sets[id_catalog::PIPELINE_FASTQ_CONTAMINANT_DEPLETION]
        .contains(&"fastq.deplete_reference_contaminants"));
    assert!(required_stage_sets[id_catalog::PIPELINE_FASTQ_AMPLICON_STANDARD]
        .contains(&"fastq.infer_asvs"));
    assert!(required_stage_sets[id_catalog::PIPELINE_FASTQ_AMPLICON_UMI]
        .contains(&"fastq.extract_umis"));
    assert!(required_stage_sets[id_catalog::PIPELINE_FASTQ_EDNA_METABARCODING]
        .contains(&"fastq.cluster_otus"));
}
