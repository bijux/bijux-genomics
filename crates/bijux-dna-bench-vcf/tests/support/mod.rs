//! Resolve the runtime checkout so native test archives remain relocatable.
use std::path::PathBuf;
pub fn repo_root() -> PathBuf {
    let declared =
        std::env::var_os("BIJUX_REPO_ROOT").filter(|value| !value.is_empty()).map(PathBuf::from);
    let cwd = std::env::current_dir()
        .unwrap_or_else(|error| panic!("resolve VCF adapter test root: {error}"));
    let candidate = declared.or_else(|| {
        cwd.ancestors()
            .find(|path| path.join("Cargo.lock").is_file() && path.join("configs").is_dir())
            .map(PathBuf::from)
    });
    let root = candidate.unwrap_or_else(|| panic!("declare BIJUX_REPO_ROOT for VCF adapter tests"));
    assert!(
        root.join("Cargo.lock").is_file()
            && root.join("configs").is_dir()
            && root.join("crates").is_dir(),
        "invalid VCF adapter test repository root"
    );
    root
}

/// Test composition consumes canonical declarations without compiling the workflow library.
pub fn inputs(root: &std::path::Path) -> bijux_dna_bench_vcf::inputs::VcfAdapterInputs {
    let catalog: bijux_dna_bench_model::LocalVcfStageCatalogConfig = toml::from_str(
        &std::fs::read_to_string(root.join("configs/bench/local/vcf-stage-catalog.toml"))
            .expect("catalog declaration"),
    )
    .expect("catalog");
    let matrix: bijux_dna_bench_model::LocalVcfStageMatrixConfig = toml::from_str(
        &std::fs::read_to_string(root.join("configs/bench/local/vcf-stage-matrix.toml"))
            .expect("matrix declaration"),
    )
    .expect("matrix");
    bijux_dna_bench_model::validate_local_vcf_stage_matrix_config(&matrix)
        .expect("matrix admission");
    let domain: DomainBindings = serde_yaml::from_str(
        &std::fs::read_to_string(root.join("domain/vcf/index.yaml")).expect("domain declaration"),
    )
    .expect("domain bindings");
    let pairs = domain
        .stage_tool_compatibility
        .into_iter()
        .flat_map(|(stage, tools)| tools.into_iter().map(move |tool| (stage.clone(), tool)));
    let bindings = bijux_dna_bench_vcf::inputs::project_supported_vcf_bindings(&matrix.rows, pairs)
        .expect("bindings");
    bijux_dna_bench_vcf::inputs::VcfAdapterInputs::new(catalog.rows, bindings, reference_fixture)
        .expect("admitted inputs")
}

#[derive(serde::Deserialize)]
struct DomainBindings {
    stage_tool_compatibility: std::collections::BTreeMap<String, Vec<String>>,
}

/// Exercise the injected operation with genuine FASTA/index bytes. Production provider
/// reuse/publication policy is independently covered by benchmark and public CLI tests.
fn reference_fixture(
    source: &std::path::Path,
    output: &std::path::Path,
) -> anyhow::Result<(PathBuf, PathBuf)> {
    use anyhow::Context;
    let fasta = output.join(source.file_name().context("reference fixture name")?);
    let raw = std::fs::read(source)?;
    let payload = bijux_dna_db_ref::build_reference_fasta_index(&raw)?;
    let mut name = fasta.as_os_str().to_os_string();
    name.push(".fai");
    let index = PathBuf::from(name);
    bijux_dna_infra::atomic_write_bytes(&fasta, &raw)?;
    bijux_dna_infra::atomic_write_bytes(&index, payload.as_bytes())?;
    Ok((fasta, index))
}
