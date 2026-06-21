use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::env::{load_image_catalog, load_platform, RuntimeKind};
use serde::{Deserialize, Serialize};

use super::all_domain_retained_tools::{
    collect_all_domain_retained_tool_rows, AllDomainRetainedToolRow,
};
use super::tool_execution_modes::load_runtime_probe_with_source;
use crate::commands::cli::env::expected_registry_digest_from_parts;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_APPTAINER_MAP_PATH: &str = "benchmarks/readiness/tools/apptainer-map.tsv";
const APPTAINER_MAP_SCHEMA_VERSION: &str = "bijux.bench.readiness.apptainer_map.v1";
const GOVERNED_TOOL_REGISTRY_PATHS: [&str; 3] = [
    "configs/ci/registry/tool_registry.toml",
    "configs/ci/registry/tool_registry_experimental.toml",
    "configs/ci/registry/tool_registry_vcf.toml",
];
const APPTAINER_CACHE_ROOT_TEMPLATE: &str = "${BIJUX_HPC_ROOT}/.cache";
const APPTAINER_CONTAINERS_ROOT_TEMPLATE: &str = "${BIJUX_HPC_ROOT}/.cache/bijux-dna-container";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ApptainerMapRow {
    pub(crate) tool_id: String,
    pub(crate) domains: Vec<String>,
    pub(crate) active_stage_ids: Vec<String>,
    pub(crate) docker_runtime: String,
    pub(crate) image_uri: String,
    pub(crate) local_image_name: String,
    pub(crate) dockerfile: String,
    pub(crate) apptainer_def: String,
    pub(crate) apptainer_cache_key: String,
    pub(crate) cache_root: String,
    pub(crate) expected_sif_path: String,
    pub(crate) conversion_command: String,
    pub(crate) runtime_probe_paths: Vec<String>,
    pub(crate) registry_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApptainerMapReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) docker_runtime: String,
    pub(crate) cache_root: String,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<ApptainerMapRow>,
}

#[derive(Debug, Deserialize)]
struct RegistryToolFile {
    tools: Vec<RegistryToolSourceRow>,
}

#[derive(Debug, Clone, Deserialize)]
struct RegistryToolSourceRow {
    id: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    pinned_commit: Option<String>,
    #[serde(default)]
    container_ref: Option<String>,
    #[serde(default)]
    dockerfile: Option<String>,
    #[serde(default)]
    apptainer_def: Option<String>,
    #[serde(default)]
    runtimes: Vec<String>,
}

#[derive(Debug, Clone)]
struct ApptainerRegistryRow {
    tool_id: String,
    version: Option<String>,
    pinned_commit: Option<String>,
    container_ref: Option<String>,
    dockerfile: String,
    apptainer_def: String,
    runtimes: Vec<String>,
    registry_paths: Vec<String>,
}

fn normalize_optional_registry_value(value: Option<String>) -> Option<String> {
    value.map(|entry| entry.trim().to_string()).filter(|entry| !entry.is_empty())
}

fn is_placeholder_registry_value(value: &str) -> bool {
    matches!(value.trim(), "" | "@" | "external" | "external@external" | "planned" | "pending")
}

fn merge_registry_value(
    field: &str,
    tool_id: &str,
    current: &mut Option<String>,
    incoming: Option<String>,
) -> Result<()> {
    let incoming = normalize_optional_registry_value(incoming);
    match (current.as_deref(), incoming.as_deref()) {
        (None, Some(_)) => {
            *current = incoming;
        }
        (Some(existing), Some(candidate)) if existing == candidate => {}
        (Some(existing), Some(candidate))
            if is_placeholder_registry_value(existing)
                && !is_placeholder_registry_value(candidate) =>
        {
            *current = incoming;
        }
        (Some(existing), Some(candidate))
            if !is_placeholder_registry_value(existing)
                && is_placeholder_registry_value(candidate) => {}
        (Some(existing), Some(candidate))
            if is_placeholder_registry_value(existing)
                && is_placeholder_registry_value(candidate) => {}
        (Some(_), Some(_)) => {
            return Err(anyhow!(
                "apptainer readiness map found conflicting concrete `{field}` values for `{tool_id}`"
            ));
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn run_render_apptainer_map(
    args: &parse::BenchReadinessRenderApptainerMapArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_apptainer_map(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_APPTAINER_MAP_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_apptainer_map(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<ApptainerMapReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_apptainer_map_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_apptainer_map_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut domain_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        for domain in &row.domains {
            *domain_counts.entry(domain.clone()).or_default() += 1;
        }
    }

    Ok(ApptainerMapReport {
        schema_version: APPTAINER_MAP_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        docker_runtime: rows
            .first()
            .map(|row| row.docker_runtime.clone())
            .unwrap_or_else(|| "docker-unknown".to_string()),
        cache_root: APPTAINER_CACHE_ROOT_TEMPLATE.to_string(),
        domain_counts,
        rows,
    })
}

pub(crate) fn collect_apptainer_map_rows(repo_root: &Path) -> Result<Vec<ApptainerMapRow>> {
    let retained_rows = collect_all_domain_retained_tool_rows(repo_root)?;
    let retained_by_tool =
        retained_rows.into_iter().map(|row| (row.tool_id.clone(), row)).collect::<BTreeMap<_, _>>();
    let registry_by_tool = load_apptainer_registry_rows(repo_root)?;
    let image_catalog = load_image_catalog().context("load docker image catalog")?;
    let platform = load_platform(None).context("load default docker platform")?;
    if platform.runner != RuntimeKind::Docker {
        return Err(anyhow!(
            "apptainer readiness map requires a docker default platform, found {}",
            platform.runner
        ));
    }
    let docker_runtime = format!("docker-{}", platform.arch);

    let expected_tool_ids = retained_by_tool
        .keys()
        .filter(|tool_id| registry_by_tool.get(*tool_id).is_some_and(is_docker_backed))
        .cloned()
        .collect::<BTreeSet<_>>();
    if expected_tool_ids.is_empty() {
        return Err(anyhow!(
            "apptainer readiness map expected at least one retained docker-backed tool"
        ));
    }

    let mut rows = Vec::with_capacity(expected_tool_ids.len());
    for tool_id in expected_tool_ids {
        let retained_row = retained_by_tool
            .get(tool_id.as_str())
            .ok_or_else(|| anyhow!("missing retained-scope row for `{tool_id}`"))?;
        let registry_row = registry_by_tool.get(tool_id.as_str()).ok_or_else(|| {
            anyhow!("missing registry row for retained docker-backed tool `{tool_id}`")
        })?;
        rows.push(build_apptainer_map_row(
            repo_root,
            retained_row,
            registry_row,
            &image_catalog,
            platform.image_prefix.as_str(),
            platform.arch.as_str(),
            &docker_runtime,
        )?);
    }

    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    ensure_apptainer_map_contract(&rows, &registry_by_tool, &retained_by_tool)?;
    Ok(rows)
}

fn build_apptainer_map_row<S: ::std::hash::BuildHasher>(
    repo_root: &Path,
    retained_row: &AllDomainRetainedToolRow,
    registry_row: &ApptainerRegistryRow,
    image_catalog: &std::collections::HashMap<
        String,
        bijux_dna_api::v1::api::env::ToolImageSpec,
        S,
    >,
    image_prefix: &str,
    arch: &str,
    docker_runtime: &str,
) -> Result<ApptainerMapRow> {
    let dockerfile_path = repo_root.join(&registry_row.dockerfile);
    if !dockerfile_path.is_file() {
        return Err(anyhow!(
            "apptainer readiness map dockerfile missing for `{}` at {}",
            registry_row.tool_id,
            dockerfile_path.display()
        ));
    }
    let apptainer_def_path = repo_root.join(&registry_row.apptainer_def);
    if !apptainer_def_path.is_file() {
        return Err(anyhow!(
            "apptainer readiness map apptainer_def missing for `{}` at {}",
            registry_row.tool_id,
            apptainer_def_path.display()
        ));
    }

    let image_spec = image_catalog.get(&registry_row.tool_id).ok_or_else(|| {
        anyhow!("apptainer readiness map image catalog is missing `{}`", registry_row.tool_id)
    })?;
    let version = image_spec.version.trim();
    if version.is_empty() {
        return Err(anyhow!(
            "apptainer readiness map image catalog version is empty for `{}`",
            registry_row.tool_id
        ));
    }
    let local_image_name = format!("{image_prefix}/{}:{version}-{arch}", registry_row.tool_id);
    let image_uri = format!("docker-daemon://{local_image_name}");

    let apptainer_cache_key = expected_registry_digest_from_parts(
        registry_row.tool_id.as_str(),
        registry_row.version.as_deref(),
        registry_row.pinned_commit.as_deref(),
        registry_row.container_ref.as_deref(),
        Some(registry_row.apptainer_def.as_str()),
    )
    .ok_or_else(|| {
        anyhow!(
            "apptainer readiness map could not derive a stable cache key for `{}`",
            registry_row.tool_id
        )
    })?;
    let expected_sif_path = format!(
        "{}/{}/{}.sif",
        APPTAINER_CONTAINERS_ROOT_TEMPLATE, registry_row.tool_id, apptainer_cache_key
    );
    let conversion_command = format!("apptainer build --force '{expected_sif_path}' '{image_uri}'");
    let runtime_probe_paths = collect_runtime_probe_paths(repo_root, retained_row)?;

    Ok(ApptainerMapRow {
        tool_id: retained_row.tool_id.clone(),
        domains: retained_row.domains.clone(),
        active_stage_ids: retained_row.active_stage_ids.clone(),
        docker_runtime: docker_runtime.to_string(),
        image_uri,
        local_image_name,
        dockerfile: registry_row.dockerfile.clone(),
        apptainer_def: registry_row.apptainer_def.clone(),
        apptainer_cache_key,
        cache_root: APPTAINER_CACHE_ROOT_TEMPLATE.to_string(),
        expected_sif_path,
        conversion_command,
        runtime_probe_paths,
        registry_paths: registry_row.registry_paths.clone(),
    })
}

fn collect_runtime_probe_paths(
    repo_root: &Path,
    retained_row: &AllDomainRetainedToolRow,
) -> Result<Vec<String>> {
    let mut paths = Vec::new();
    for domain in &retained_row.domains {
        let loaded = load_runtime_probe_with_source(repo_root, domain, &retained_row.tool_id)
            .with_context(|| {
                format!("load runtime probe for `{}` in domain `{}`", retained_row.tool_id, domain)
            })?;
        let relative = path_relative_to_repo(repo_root, &loaded.path);
        if !paths.iter().any(|existing| existing == &relative) {
            paths.push(relative);
        }
    }
    paths.sort();
    Ok(paths)
}

fn load_apptainer_registry_rows(
    repo_root: &Path,
) -> Result<BTreeMap<String, ApptainerRegistryRow>> {
    let mut rows = BTreeMap::<String, ApptainerRegistryRow>::new();
    for relative_path in GOVERNED_TOOL_REGISTRY_PATHS {
        let registry_path = repo_root.join(relative_path);
        let raw = fs::read_to_string(&registry_path)
            .with_context(|| format!("read {}", registry_path.display()))?;
        let parsed = toml::from_str::<RegistryToolFile>(&raw)
            .with_context(|| format!("parse {}", registry_path.display()))?;
        for source in parsed.tools {
            let dockerfile = source
                .dockerfile
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or_default()
                .to_string();
            let apptainer_def = source
                .apptainer_def
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or_default()
                .to_string();
            let row = ApptainerRegistryRow {
                tool_id: source.id.clone(),
                version: normalize_optional_registry_value(source.version),
                pinned_commit: normalize_optional_registry_value(source.pinned_commit),
                container_ref: normalize_optional_registry_value(source.container_ref),
                dockerfile,
                apptainer_def,
                runtimes: source.runtimes,
                registry_paths: vec![relative_path.to_string()],
            };
            if let Some(existing) = rows.get_mut(source.id.as_str()) {
                merge_registry_value("version", &source.id, &mut existing.version, row.version)?;
                merge_registry_value(
                    "pinned_commit",
                    &source.id,
                    &mut existing.pinned_commit,
                    row.pinned_commit,
                )?;
                merge_registry_value(
                    "container_ref",
                    &source.id,
                    &mut existing.container_ref,
                    row.container_ref,
                )?;
                if existing.dockerfile != row.dockerfile
                    || existing.apptainer_def != row.apptainer_def
                {
                    return Err(anyhow!(
                        "apptainer readiness map found inconsistent duplicate registry rows for `{}`",
                        source.id
                    ));
                }
                for runtime in row.runtimes {
                    if !existing.runtimes.iter().any(|current| current == &runtime) {
                        existing.runtimes.push(runtime);
                    }
                }
                existing.runtimes.sort();
                if !existing.registry_paths.iter().any(|path| path == relative_path) {
                    existing.registry_paths.push(relative_path.to_string());
                    existing.registry_paths.sort();
                }
            } else {
                rows.insert(source.id.clone(), row);
            }
        }
    }
    Ok(rows)
}

fn is_docker_backed(row: &ApptainerRegistryRow) -> bool {
    row.runtimes.iter().any(|runtime| runtime == "docker") && !row.dockerfile.trim().is_empty()
}

fn ensure_apptainer_map_contract(
    rows: &[ApptainerMapRow],
    registry_by_tool: &BTreeMap<String, ApptainerRegistryRow>,
    retained_by_tool: &BTreeMap<String, AllDomainRetainedToolRow>,
) -> Result<()> {
    let expected_tool_ids = retained_by_tool
        .keys()
        .filter(|tool_id| registry_by_tool.get(*tool_id).is_some_and(is_docker_backed))
        .cloned()
        .collect::<BTreeSet<_>>();
    let actual_tool_ids = rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>();
    if actual_tool_ids != expected_tool_ids {
        return Err(anyhow!(
            "apptainer readiness map drifted from the retained docker-backed tool scope"
        ));
    }
    if rows.len() != actual_tool_ids.len() {
        return Err(anyhow!("apptainer readiness map must keep exactly one row per tool"));
    }

    for row in rows {
        if row.tool_id.trim().is_empty()
            || row.domains.is_empty()
            || row.active_stage_ids.is_empty()
            || row.image_uri.trim().is_empty()
            || row.local_image_name.trim().is_empty()
            || row.apptainer_def.trim().is_empty()
            || row.dockerfile.trim().is_empty()
            || row.apptainer_cache_key.trim().is_empty()
            || row.cache_root != APPTAINER_CACHE_ROOT_TEMPLATE
            || row.expected_sif_path.trim().is_empty()
            || row.conversion_command.trim().is_empty()
            || row.runtime_probe_paths.is_empty()
            || row.registry_paths.is_empty()
        {
            return Err(anyhow!(
                "apptainer readiness map row `{}` is missing a required mapping field",
                row.tool_id
            ));
        }
        if !row.image_uri.starts_with("docker-daemon://") {
            return Err(anyhow!(
                "apptainer readiness map row `{}` must publish a docker-daemon image URI",
                row.tool_id
            ));
        }
        if !row.expected_sif_path.ends_with(".sif") {
            return Err(anyhow!(
                "apptainer readiness map row `{}` must publish a `.sif` target path",
                row.tool_id
            ));
        }
        if !row.expected_sif_path.contains(&format!("/{}/", row.tool_id)) {
            return Err(anyhow!(
                "apptainer readiness map row `{}` must keep the tool-scoped container cache directory",
                row.tool_id
            ));
        }
        if !row.expected_sif_path.contains(row.apptainer_cache_key.as_str()) {
            return Err(anyhow!(
                "apptainer readiness map row `{}` drifted from its stable Apptainer cache key",
                row.tool_id
            ));
        }
        if !row.conversion_command.contains(row.expected_sif_path.as_str())
            || !row.conversion_command.contains(row.image_uri.as_str())
        {
            return Err(anyhow!(
                "apptainer readiness map row `{}` must keep its image URI and SIF target in the conversion command",
                row.tool_id
            ));
        }
    }

    Ok(())
}

fn render_apptainer_map_tsv(rows: &[ApptainerMapRow]) -> String {
    let mut rendered = String::from(
        "tool_id\tdomains\tactive_stage_ids\tdocker_runtime\timage_uri\tlocal_image_name\tdockerfile\tapptainer_def\tapptainer_cache_key\tcache_root\texpected_sif_path\tconversion_command\truntime_probe_paths\tregistry_paths\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.domains.join(",")),
            sanitize_tsv(&row.active_stage_ids.join(",")),
            sanitize_tsv(&row.docker_runtime),
            sanitize_tsv(&row.image_uri),
            sanitize_tsv(&row.local_image_name),
            sanitize_tsv(&row.dockerfile),
            sanitize_tsv(&row.apptainer_def),
            sanitize_tsv(&row.apptainer_cache_key),
            sanitize_tsv(&row.cache_root),
            sanitize_tsv(&row.expected_sif_path),
            sanitize_tsv(&row.conversion_command),
            sanitize_tsv(&row.runtime_probe_paths.join(",")),
            sanitize_tsv(&row.registry_paths.join(",")),
        ));
    }
    rendered
}

fn sanitize_tsv(value: &str) -> String {
    value.replace('\t', " ").replace('\n', " ").replace('\r', " ")
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{render_apptainer_map_tsv, ApptainerMapRow, APPTAINER_CACHE_ROOT_TEMPLATE};

    #[test]
    fn render_apptainer_map_tsv_keeps_expected_columns() {
        let rendered = render_apptainer_map_tsv(&[ApptainerMapRow {
            tool_id: "adapterremoval".to_string(),
            domains: vec!["fastq".to_string()],
            active_stage_ids: vec!["fastq.trim_reads".to_string()],
            docker_runtime: "docker-arm64".to_string(),
            image_uri: "docker-daemon://bijuxdna/adapterremoval:2.3.3-arm64".to_string(),
            local_image_name: "bijuxdna/adapterremoval:2.3.3-arm64".to_string(),
            dockerfile: "containers/docker/arm64/Dockerfile.adapterremoval".to_string(),
            apptainer_def: "containers/apptainer/shared/adapterremoval.def".to_string(),
            apptainer_cache_key: "abc123".to_string(),
            cache_root: APPTAINER_CACHE_ROOT_TEMPLATE.to_string(),
            expected_sif_path:
                "${BIJUX_HPC_ROOT}/.cache/bijux-dna-container/adapterremoval/abc123.sif"
                    .to_string(),
            conversion_command: "apptainer build --force '${BIJUX_HPC_ROOT}/.cache/bijux-dna-container/adapterremoval/abc123.sif' 'docker-daemon://bijuxdna/adapterremoval:2.3.3-arm64'".to_string(),
            runtime_probe_paths: vec!["domain/fastq/tools/adapterremoval.yaml".to_string()],
            registry_paths: vec!["configs/ci/registry/tool_registry.toml".to_string()],
        }]);

        assert!(rendered.starts_with("tool_id\tdomains\tactive_stage_ids\tdocker_runtime\t"));
        assert!(rendered.contains("docker-daemon://bijuxdna/adapterremoval:2.3.3-arm64"));
        assert!(rendered
            .contains("${BIJUX_HPC_ROOT}/.cache/bijux-dna-container/adapterremoval/abc123.sif"));
    }
}
