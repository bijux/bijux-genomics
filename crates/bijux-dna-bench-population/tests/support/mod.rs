//! Explicit test composition with isolated outputs and canonical declaration fixtures.
use crate::inputs::PopulationBenchmarkInputs;
use bijux_dna_bench_model::{AllDomainActiveStageToolMatrixRow, LocalVcfStageMatrixConfig};
use bijux_dna_core::id_catalog as stage;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn workspace() -> PathBuf {
    let declared = std::env::var_os("BIJUX_REPO_ROOT").map(PathBuf::from);
    let cwd = std::env::current_dir().expect("test cwd");
    declared
        .or_else(|| {
            cwd.ancestors()
                .find(|p| p.join("Cargo.lock").is_file() && p.join("configs").is_dir())
                .map(PathBuf::from)
        })
        .expect("population test repository")
}

#[derive(serde::Deserialize)]
struct DomainBindings {
    stage_tool_compatibility: BTreeMap<String, Vec<String>>,
}

pub(crate) fn inputs() -> PopulationBenchmarkInputs {
    let root = workspace();
    let matrix: LocalVcfStageMatrixConfig = toml::from_str(
        &std::fs::read_to_string(root.join("configs/bench/local/vcf-stage-matrix.toml"))
            .expect("matrix"),
    )
    .expect("matrix declaration");
    bijux_dna_bench_model::validate_local_vcf_stage_matrix_config(&matrix)
        .expect("matrix admission");
    let domain: DomainBindings = serde_yaml::from_str(
        &std::fs::read_to_string(root.join("domain/vcf/index.yaml")).expect("domain"),
    )
    .expect("domain bindings");
    let mut supported = Vec::new();
    let mut governed = Vec::new();
    let mut active = Vec::new();
    for row in matrix.rows {
        if [stage::VCF_PCA, stage::VCF_ADMIXTURE, stage::VCF_POPULATION_STRUCTURE]
            .contains(&row.stage_id.as_str())
        {
            for tool in domain.stage_tool_compatibility.get(&row.stage_id).expect("supported tools")
            {
                let mut binding = row.clone();
                binding.tool_id.clone_from(tool);
                supported.push(binding);
            }
        }
        if row.stage_id == stage::VCF_IBD {
            // This explicit admitted readiness fixture exercises the service. Actual
            // readiness acquisition is covered by public CLI composition consumers.
            for tool in domain.stage_tool_compatibility.get(&row.stage_id).expect("IBD tools") {
                active.push(AllDomainActiveStageToolMatrixRow {
                    domain: "vcf".into(),
                    stage_id: row.stage_id.clone(),
                    tool_id: tool.clone(),
                    corpus_id: row.corpus_id.clone(),
                    asset_profile_id: row.asset_profile_id.clone(),
                    adapter_id: row.adapter_id.clone(),
                    parser_id: row.parser_id.clone(),
                    schema_id: "admitted-ibd-test-observation".into(),
                    status: "benchmark_ready".into(),
                });
            }
        }
        if [stage::VCF_ROH, stage::VCF_IBD, stage::VCF_DEMOGRAPHY].contains(&row.stage_id.as_str())
        {
            governed.push(row);
        }
    }
    PopulationBenchmarkInputs::new(supported, governed, active).expect("population declarations")
}

pub(crate) fn fixture() -> tempfile::TempDir {
    let root = workspace();
    let fixture =
        bijux_dna_infra::temp_dir_in(&root.join("artifacts/rust/test/population"), "cohort-")
            .expect("isolated cohort root");
    let manifest =
        std::path::Path::new(bijux_dna_bench_corpus::vcf::DEFAULT_VCF_MINI_MANIFEST_PATH);
    let source = root.join(manifest);
    let text = std::fs::read_to_string(&source).expect("VCF manifest");
    let declaration: toml::Value = toml::from_str(&text).expect("VCF declaration");
    let parent = manifest.parent().expect("manifest parent");
    let target = fixture.path().join(manifest);
    std::fs::create_dir_all(target.parent().expect("target parent")).expect("fixture directory");
    std::fs::write(target, text).expect("fixture manifest");
    for (key, value) in declaration.as_table().expect("manifest table") {
        if key.ends_with("_path") {
            let relative = std::path::Path::new(value.as_str().expect("fixture path"));
            assert!(relative
                .components()
                .all(|part| matches!(part, std::path::Component::Normal(_))));
            let target = fixture.path().join(parent).join(relative);
            std::fs::create_dir_all(target.parent().expect("fixture parent"))
                .expect("fixture directory");
            std::fs::copy(root.join(parent).join(relative), target)
                .expect("copy declared cohort input");
        }
    }
    fixture
}
