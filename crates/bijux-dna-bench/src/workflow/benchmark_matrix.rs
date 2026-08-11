//! Owner: bijux-dna-bench
//! Catalog-complete benchmark frontier derivation and publication.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_matrix_report, BenchmarkMatrixReport, BenchmarkMatrixRow,
    BenchmarkMatrixSummary, BenchmarkReadiness, BenchmarkSurfaceMatch,
    BENCHMARK_MATRIX_REPORT_SCHEMA_V1,
};
use bijux_dna_containers::api::{
    load_container_catalog, validate_container_catalog, ContainerCatalog,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkMatrixRequest {
    pub repository_root: PathBuf,
    pub campaign_id: String,
    pub domain: String,
    pub corpora_root: PathBuf,
    pub databases_root: PathBuf,
    pub images_root: PathBuf,
    pub generated_at: String,
}

#[derive(Debug, Clone, Copy)]
struct CrossBridge {
    id: &'static str,
    from_stage: &'static str,
    to_stage: &'static str,
}

#[must_use]
pub fn current_benchmark_matrix_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |delta| delta.as_secs())
        .to_string()
}

/// Derive the complete selected benchmark frontier from the product catalog.
///
/// # Errors
/// Returns an error when catalog, stage, input-surface, or persisted-model contracts are invalid.
pub fn derive_benchmark_matrix(request: &BenchmarkMatrixRequest) -> Result<BenchmarkMatrixReport> {
    let domains = resolve_matrix_domains(&request.domain)?;
    let validation = validate_container_catalog(&request.repository_root)?;
    if !validation.is_valid() {
        bail!("container catalog validation failed: {:?}", validation.errors);
    }
    let catalog = load_container_catalog(&request.repository_root)?;
    let stage_tools = supported_stage_tools(&catalog);
    let corpus_tokens = collect_name_tokens(&request.corpora_root, false)?;
    let database_tokens = collect_name_tokens(&request.databases_root, false)?;
    let image_tokens = collect_name_tokens(&request.images_root, true)?;
    let mut rows = Vec::new();

    for domain in domains.iter().filter(|domain| domain.as_str() != "cross") {
        append_domain_rows(
            &mut rows,
            &request.repository_root,
            domain,
            &catalog,
            &stage_tools,
            &corpus_tokens,
            &database_tokens,
            &image_tokens,
        )?;
    }
    if domains.iter().any(|domain| domain == "cross") {
        append_cross_rows(
            &mut rows,
            &request.repository_root,
            &stage_tools,
            &corpus_tokens,
            &database_tokens,
            &image_tokens,
        )?;
    }
    rows.sort_by(|left, right| left.row_id.cmp(&right.row_id));

    let report = BenchmarkMatrixReport {
        schema_version: BENCHMARK_MATRIX_REPORT_SCHEMA_V1.to_string(),
        campaign_id: request.campaign_id.clone(),
        domain: request.domain.clone(),
        domains,
        generated_at: request.generated_at.clone(),
        summary: summarize_rows(&rows),
        rows,
    };
    validate_benchmark_matrix_report(&report).context("validate derived benchmark matrix")?;
    Ok(report)
}

/// Validate and atomically publish a benchmark matrix.
///
/// # Errors
/// Returns an error when the report is invalid or cannot be serialized and written atomically.
pub fn publish_benchmark_matrix(path: &Path, report: &BenchmarkMatrixReport) -> Result<()> {
    validate_benchmark_matrix_report(report).context("validate benchmark matrix publication")?;
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    let payload = serde_json::to_vec_pretty(report)?;
    bijux_dna_infra::atomic_write_bytes(path, &payload)
        .with_context(|| format!("publish benchmark matrix {}", path.display()))
}

/// Load and validate a persisted benchmark matrix.
///
/// # Errors
/// Returns an error when the file cannot be read, parsed, or admitted.
pub fn load_benchmark_matrix_report(path: &Path) -> Result<BenchmarkMatrixReport> {
    let raw = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_slice::<BenchmarkMatrixReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    validate_benchmark_matrix_report(&report)
        .with_context(|| format!("validate benchmark matrix {}", path.display()))?;
    Ok(report)
}

#[allow(clippy::too_many_arguments)]
fn append_domain_rows(
    rows: &mut Vec<BenchmarkMatrixRow>,
    repository_root: &Path,
    domain: &str,
    catalog: &ContainerCatalog,
    stage_tools: &BTreeMap<String, Vec<String>>,
    corpus_tokens: &[String],
    database_tokens: &[String],
    image_tokens: &[String],
) -> Result<()> {
    let stages = domain_stage_ids(repository_root, domain)?;
    for stage_id in &stages {
        if let Some(tools) = stage_tools.get(stage_id) {
            for tool_id in tools {
                rows.push(bound_row(
                    domain,
                    stage_id,
                    tool_id,
                    corpus_tokens,
                    database_tokens,
                    image_tokens,
                ));
            }
        } else {
            rows.push(refused_row(
                domain,
                stage_id,
                "<unbound>",
                "stage contract has no supported catalog tool binding",
                image_tokens,
            ));
        }
    }

    for (stage_id, tools) in stage_tools {
        if stage_domain(stage_id) != Some(domain) || stages.contains(stage_id) {
            continue;
        }
        for tool_id in tools {
            rows.push(refused_row(
                domain,
                stage_id,
                tool_id,
                "supported catalog pair references a missing stage contract",
                image_tokens,
            ));
        }
    }

    for tool in catalog
        .tools
        .iter()
        .filter(|tool| tool.is_product_supported() && tool.stage_ids.is_empty())
        .filter(|tool| tool.domains.iter().any(|candidate| candidate == domain))
    {
        rows.push(refused_row(
            domain,
            &format!("{domain}.<unbound-stage>"),
            &tool.id,
            "supported catalog tool has no stage binding",
            image_tokens,
        ));
    }
    Ok(())
}

fn append_cross_rows(
    rows: &mut Vec<BenchmarkMatrixRow>,
    repository_root: &Path,
    stage_tools: &BTreeMap<String, Vec<String>>,
    corpus_tokens: &[String],
    database_tokens: &[String],
    image_tokens: &[String],
) -> Result<()> {
    let stage_universe = ["fastq", "bam", "vcf"]
        .into_iter()
        .map(|domain| domain_stage_ids(repository_root, domain))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<BTreeSet<_>>();
    for bridge in cross_bridges() {
        let left_tools = stage_tools
            .get(bridge.from_stage)
            .cloned()
            .unwrap_or_else(|| vec!["<unbound-left>".to_string()]);
        let right_tools = stage_tools
            .get(bridge.to_stage)
            .cloned()
            .unwrap_or_else(|| vec!["<unbound-right>".to_string()]);
        for left in &left_tools {
            for right in &right_tools {
                let stage_binding = format!("{}=>{}", bridge.from_stage, bridge.to_stage);
                let tool_binding = format!("{left}=>{right}");
                let missing_stage_contract = !stage_universe.contains(bridge.from_stage)
                    || !stage_universe.contains(bridge.to_stage);
                if missing_stage_contract
                    || left.starts_with("<unbound")
                    || right.starts_with("<unbound")
                {
                    let mut row = refused_row(
                        "cross",
                        &stage_binding,
                        &tool_binding,
                        if missing_stage_contract {
                            "cross-stage bridge references a missing stage contract"
                        } else {
                            "cross-stage bridge has an unbound supported-tool endpoint"
                        },
                        image_tokens,
                    );
                    row.row_id = format!("cross.{}::{stage_binding}::{tool_binding}", bridge.id);
                    rows.push(row);
                } else {
                    let mut row = bound_row(
                        "cross",
                        &stage_binding,
                        &tool_binding,
                        corpus_tokens,
                        database_tokens,
                        image_tokens,
                    );
                    row.row_id = format!("cross.{}::{stage_binding}::{tool_binding}", bridge.id);
                    rows.push(row);
                }
            }
        }
    }
    Ok(())
}

fn bound_row(
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    corpus_tokens: &[String],
    database_tokens: &[String],
    image_tokens: &[String],
) -> BenchmarkMatrixRow {
    let corpus_match = match_surface(stage_corpus_profile(stage_id), corpus_tokens);
    let database_match = match_database_surface(stage_id, database_tokens);
    let image_match = match_image_surface(tool_id, image_tokens);
    let readiness = classify_readiness(&corpus_match, &database_match, &image_match);
    BenchmarkMatrixRow {
        row_id: format!("{stage_id}::{tool_id}"),
        matrix_domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        repetitions: repetition_policy(domain, stage_id, &readiness.class),
        corpus_match,
        database_match,
        image_match,
        readiness,
    }
}

fn refused_row(
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    reason: &str,
    image_tokens: &[String],
) -> BenchmarkMatrixRow {
    BenchmarkMatrixRow {
        row_id: format!("{stage_id}::{tool_id}"),
        matrix_domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        corpus_match: unavailable_surface("stage-contract"),
        database_match: unavailable_surface("tool-binding"),
        image_match: match_image_surface(tool_id, image_tokens),
        readiness: BenchmarkReadiness {
            class: "refuse".to_string(),
            reasons: vec![reason.to_string()],
        },
        repetitions: 0,
    }
}

fn unavailable_surface(required_profile: &str) -> BenchmarkSurfaceMatch {
    BenchmarkSurfaceMatch {
        required_profile: required_profile.to_string(),
        matched_profile: "<unavailable>".to_string(),
        ready: false,
    }
}

fn supported_stage_tools(catalog: &ContainerCatalog) -> BTreeMap<String, Vec<String>> {
    let mut stage_tools = BTreeMap::<String, Vec<String>>::new();
    for tool in catalog.tools.iter().filter(|tool| tool.is_product_supported()) {
        for stage_id in &tool.stage_ids {
            stage_tools.entry(stage_id.clone()).or_default().push(tool.id.clone());
        }
    }
    for tools in stage_tools.values_mut() {
        tools.sort();
        tools.dedup();
    }
    stage_tools
}

fn domain_stage_ids(root: &Path, domain: &str) -> Result<BTreeSet<String>> {
    let stages_dir = root.join("domain").join(domain).join("stages");
    if !stages_dir.is_dir() {
        bail!("stage catalog not found: {}", stages_dir.display());
    }
    let mut stages = BTreeSet::new();
    for entry in
        std::fs::read_dir(&stages_dir).with_context(|| format!("read {}", stages_dir.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if !name.starts_with('_') {
            stages.insert(format!("{domain}.{name}"));
        }
    }
    Ok(stages)
}

fn stage_domain(stage_id: &str) -> Option<&str> {
    stage_id.split_once('.').map(|(domain, _)| domain)
}

fn resolve_matrix_domains(value: &str) -> Result<Vec<String>> {
    match value {
        "all" => {
            Ok(vec!["fastq".to_string(), "bam".to_string(), "vcf".to_string(), "cross".to_string()])
        }
        "fastq" | "bam" | "vcf" | "cross" => Ok(vec![value.to_string()]),
        other => Err(anyhow!(
            "benchmark-matrix supports --domain fastq|bam|vcf|cross|all; got `{other}`"
        )),
    }
}

fn normalize_token(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { '_' })
        .collect()
}

fn collect_name_tokens(root: &Path, recursive: bool) -> Result<Vec<String>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            out.push(normalize_token(&entry.file_name().to_string_lossy()));
            if recursive && metadata.is_dir() {
                stack.push(path);
            }
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn stage_corpus_profile(stage_id: &str) -> &'static str {
    if stage_id.contains("damage") || stage_id.contains("authenticity") {
        return "ancient";
    }
    if stage_id.contains("taxonomy")
        || stage_id.contains("asv")
        || stage_id.contains("otu")
        || stage_id.contains("metabarcoding")
    {
        return "edna";
    }
    if stage_id.starts_with("bam.") || stage_id.starts_with("vcf.") || stage_id.contains("align") {
        return "wgs";
    }
    "general"
}

fn stage_database_profile(stage_id: &str) -> Option<&'static str> {
    let needs = [
        "align",
        "index",
        "deplete",
        "screen",
        "call",
        "genotyp",
        "imput",
        "phase",
        "taxonomy",
        "reference",
        "panel",
    ];
    if !needs.iter().any(|needle| stage_id.contains(needle)) {
        return None;
    }
    if stage_id.contains("taxonomy") {
        Some("taxonomy")
    } else if stage_id.contains("rrna") {
        Some("rrna")
    } else if stage_id.starts_with("vcf.")
        || stage_id.contains("call")
        || stage_id.contains("genotyp")
    {
        Some("vcf")
    } else if stage_id.contains("align") {
        Some("align")
    } else {
        Some("general")
    }
}

fn tool_match_tokens(tool_id: &str) -> Vec<String> {
    tool_id
        .split("=>")
        .flat_map(|part| part.split(','))
        .map(str::trim)
        .filter(|part| !part.is_empty() && !part.starts_with('<'))
        .map(|part| part.split('_').next().unwrap_or(part))
        .map(normalize_token)
        .collect()
}

fn match_surface(required: &str, tokens: &[String]) -> BenchmarkSurfaceMatch {
    let matched = if required == "general" {
        tokens.first().cloned()
    } else {
        tokens
            .iter()
            .find(|token| token.contains(required))
            .cloned()
            .or_else(|| tokens.first().cloned())
    };
    BenchmarkSurfaceMatch {
        required_profile: required.to_string(),
        matched_profile: matched.unwrap_or_else(|| "<missing>".to_string()),
        ready: if required == "general" {
            !tokens.is_empty()
        } else {
            tokens.iter().any(|token| token.contains(required))
        },
    }
}

fn match_database_surface(stage_id: &str, tokens: &[String]) -> BenchmarkSurfaceMatch {
    stage_database_profile(stage_id).map_or_else(
        || BenchmarkSurfaceMatch {
            required_profile: "not-required".to_string(),
            matched_profile: "not-required".to_string(),
            ready: true,
        },
        |required| match_surface(required, tokens),
    )
}

fn match_image_surface(tool_id: &str, image_tokens: &[String]) -> BenchmarkSurfaceMatch {
    let required = tool_match_tokens(tool_id);
    let matched = required
        .iter()
        .filter(|token| image_tokens.iter().any(|image| image.contains(token.as_str())))
        .cloned()
        .collect::<Vec<_>>();
    BenchmarkSurfaceMatch {
        required_profile: if required.is_empty() { "unknown" } else { "tool-images" }.to_string(),
        matched_profile: if matched.is_empty() {
            "<missing>".to_string()
        } else {
            matched.join(",")
        },
        ready: !matched.is_empty(),
    }
}

fn classify_readiness(
    corpus_match: &BenchmarkSurfaceMatch,
    database_match: &BenchmarkSurfaceMatch,
    image_match: &BenchmarkSurfaceMatch,
) -> BenchmarkReadiness {
    let mut reasons = Vec::new();
    if !corpus_match.ready {
        reasons.push(format!("corpus profile `{}` missing", corpus_match.required_profile));
    }
    if !database_match.ready {
        reasons.push(format!("database profile `{}` missing", database_match.required_profile));
    }
    if !image_match.ready {
        reasons.push("image match missing for tool binding".to_string());
    }
    BenchmarkReadiness {
        class: match reasons.len() {
            0 => "ready",
            1 => "degraded",
            _ => "refuse",
        }
        .to_string(),
        reasons,
    }
}

fn repetition_policy(matrix_domain: &str, stage_id: &str, readiness_class: &str) -> u32 {
    if readiness_class == "refuse" {
        return 0;
    }
    let mut repeats = if readiness_class == "degraded" { 2 } else { 3 };
    if matrix_domain == "cross" || stage_id.contains("call") || stage_id.contains("genotyp") {
        repeats += 2;
    }
    if stage_id.contains("validate") || stage_id.contains("qc") {
        repeats = repeats.max(2);
    }
    repeats
}

fn summarize_rows(rows: &[BenchmarkMatrixRow]) -> BenchmarkMatrixSummary {
    let mut readiness_counts = BTreeMap::new();
    let mut domain_counts = BTreeMap::new();
    for row in rows {
        *readiness_counts.entry(row.readiness.class.clone()).or_insert(0) += 1;
        *domain_counts.entry(row.matrix_domain.clone()).or_insert(0) += 1;
    }
    BenchmarkMatrixSummary { total_rows: rows.len(), readiness_counts, domain_counts }
}

fn cross_bridges() -> &'static [CrossBridge] {
    &[
        CrossBridge { id: "fastq_to_bam", from_stage: "fastq.trim_reads", to_stage: "bam.align" },
        CrossBridge {
            id: "fastq_validate_to_bam",
            from_stage: "fastq.validate_reads",
            to_stage: "bam.align",
        },
        CrossBridge {
            id: "fastq_adna_to_bam_damage",
            from_stage: "fastq.trim_terminal_damage",
            to_stage: "bam.damage",
        },
        CrossBridge { id: "bam_to_vcf", from_stage: "bam.genotyping", to_stage: "vcf.call" },
        CrossBridge { id: "fastq_to_vcf", from_stage: "fastq.trim_reads", to_stage: "vcf.call_gl" },
        CrossBridge {
            id: "bam_summary_to_vcf_stats",
            from_stage: "bam.mapping_summary",
            to_stage: "vcf.stats",
        },
        CrossBridge {
            id: "fastq_profile_to_bam_summary",
            from_stage: "fastq.profile_reads",
            to_stage: "bam.mapping_summary",
        },
        CrossBridge {
            id: "fastq_validate_to_bam_contamination",
            from_stage: "fastq.validate_reads",
            to_stage: "bam.contamination",
        },
        CrossBridge {
            id: "bam_contamination_to_vcf_filter",
            from_stage: "bam.contamination",
            to_stage: "vcf.filter",
        },
        CrossBridge {
            id: "vcf_filter_to_vcf_stats",
            from_stage: "vcf.filter",
            to_stage: "vcf.stats",
        },
    ]
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    fn repository_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("repository root")
            .to_path_buf()
    }

    fn request(surface_root: &Path) -> BenchmarkMatrixRequest {
        BenchmarkMatrixRequest {
            repository_root: repository_root(),
            campaign_id: "matrix-contract".to_string(),
            domain: "all".to_string(),
            corpora_root: surface_root.join("corpora"),
            databases_root: surface_root.join("databases"),
            images_root: surface_root.join("images"),
            generated_at: "1700000000".to_string(),
        }
    }

    fn catalog_pairs(catalog: &ContainerCatalog) -> BTreeSet<(String, String)> {
        catalog
            .tools
            .iter()
            .filter(|tool| tool.is_product_supported())
            .flat_map(|tool| {
                tool.stage_ids.iter().map(|stage_id| (stage_id.clone(), tool.id.clone()))
            })
            .collect()
    }

    #[test]
    fn stage_catalog_and_domain_selector_contracts_are_preserved() {
        let stage_count = ["fastq", "bam", "vcf"]
            .into_iter()
            .map(|domain| domain_stage_ids(&repository_root(), domain).expect("stages").len())
            .sum::<usize>();
        assert_eq!(stage_count, 89);
        assert_eq!(
            resolve_matrix_domains("all").expect("all"),
            vec!["fastq", "bam", "vcf", "cross"]
        );
        assert_eq!(resolve_matrix_domains("bam").expect("bam"), vec!["bam"]);
        assert_eq!(resolve_matrix_domains("cross").expect("cross"), vec!["cross"]);
        assert!(resolve_matrix_domains("unknown").is_err());
    }

    #[test]
    fn readiness_and_repetition_policies_preserve_planning_semantics() {
        let corpus = unavailable_surface("wgs");
        let database = unavailable_surface("vcf");
        let image = unavailable_surface("tool-images");
        let readiness = classify_readiness(&corpus, &database, &image);
        assert_eq!(readiness.class, "refuse");
        assert_eq!(repetition_policy("fastq", "fastq.validate_reads", "ready"), 3);
        assert_eq!(repetition_policy("vcf", "vcf.call", "ready"), 5);
        assert_eq!(repetition_policy("cross", "fastq.trim_reads=>bam.align", "degraded"), 4);
        assert_eq!(repetition_policy("bam", "bam.align", "refuse"), 0);

        let no_database = match_database_surface("fastq.profile_reads", &[]);
        assert_eq!(no_database.required_profile, "not-required");
        assert!(no_database.ready);
        let edna = match_surface("edna", &["modern_wgs".to_string(), "edna_sweden".to_string()]);
        assert!(edna.ready);
        assert!(edna.matched_profile.contains("edna"));
    }

    #[test]
    fn cross_bridge_and_single_domain_frontiers_remain_selectable() {
        assert!(cross_bridges().len() >= 10);
        assert!(cross_bridges().iter().any(|bridge| bridge.id == "fastq_to_bam"));
        assert!(cross_bridges()
            .iter()
            .any(|bridge| bridge.id == "bam_contamination_to_vcf_filter"));

        let surfaces = tempfile::tempdir().expect("surface root");
        let mut fastq_request = request(surfaces.path());
        fastq_request.domain = "fastq".to_string();
        let report = derive_benchmark_matrix(&fastq_request).expect("fastq matrix");
        assert_eq!(report.domains, vec!["fastq"]);
        assert!(report.rows.iter().all(|row| row.matrix_domain == "fastq"));
    }

    #[test]
    fn matrix_retains_complete_catalog_stage_tool_and_licensed_closure() {
        let surfaces = tempfile::tempdir().expect("surface root");
        let report = derive_benchmark_matrix(&request(surfaces.path())).expect("matrix");
        let catalog = load_container_catalog(&repository_root()).expect("catalog");
        let expected_pairs = catalog_pairs(&catalog);
        let actual_pairs = report
            .rows
            .iter()
            .filter(|row| row.matrix_domain != "cross")
            .filter(|row| !row.stage_id.ends_with(".<unbound-stage>"))
            .filter(|row| !row.tool_id.starts_with('<'))
            .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
            .collect::<BTreeSet<_>>();
        assert_eq!(expected_pairs.len(), 159);
        assert_eq!(actual_pairs.len(), 159);
        assert_eq!(actual_pairs, expected_pairs);

        let expected_tools = catalog
            .tools
            .iter()
            .filter(|tool| tool.is_product_supported())
            .map(|tool| tool.id.clone())
            .collect::<BTreeSet<_>>();
        let actual_tools = report
            .rows
            .iter()
            .filter(|row| !row.tool_id.contains("=>") && !row.tool_id.starts_with('<'))
            .map(|row| row.tool_id.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(expected_tools.len(), 89);
        assert_eq!(actual_tools, expected_tools);

        let licensed_pairs = BTreeSet::from([
            ("bam.damage".to_string(), "ngsbriggs".to_string()),
            ("bam.kinship".to_string(), "angsd".to_string()),
            ("bam.sex".to_string(), "angsd".to_string()),
            ("fastq.remove_duplicates".to_string(), "fastuniq".to_string()),
            ("vcf.call_gl".to_string(), "angsd".to_string()),
            ("vcf.call_pseudohaploid".to_string(), "angsd".to_string()),
            ("vcf.imputation_metrics".to_string(), "impute5".to_string()),
            ("vcf.impute".to_string(), "impute5".to_string()),
        ]);
        let external_delivery_pairs = catalog
            .tools
            .iter()
            .filter(|tool| tool.is_product_supported() && tool.permits_external_artifact())
            .flat_map(|tool| {
                tool.stage_ids.iter().map(|stage_id| (stage_id.clone(), tool.id.clone()))
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(external_delivery_pairs, licensed_pairs);
        assert!(licensed_pairs.is_subset(&actual_pairs));
    }

    #[test]
    fn matrix_covers_all_stage_contracts_and_refuses_catalog_drift() {
        let surfaces = tempfile::tempdir().expect("surface root");
        let report = derive_benchmark_matrix(&request(surfaces.path())).expect("matrix");
        let expected_stages = ["fastq", "bam", "vcf"]
            .into_iter()
            .map(|domain| domain_stage_ids(&repository_root(), domain).expect("stages"))
            .fold(BTreeSet::new(), |mut all, stages| {
                all.extend(stages);
                all
            });
        let actual_stages = report
            .rows
            .iter()
            .map(|row| row.stage_id.clone())
            .filter(|stage_id| expected_stages.contains(stage_id))
            .collect::<BTreeSet<_>>();
        assert_eq!(expected_stages.len(), 89);
        assert_eq!(actual_stages, expected_stages);

        let unbound_tools = report
            .rows
            .iter()
            .filter(|row| row.stage_id.ends_with(".<unbound-stage>"))
            .collect::<Vec<_>>();
        assert_eq!(unbound_tools.len(), 8);
        assert!(unbound_tools.iter().all(|row| {
            row.readiness.class == "refuse"
                && row.repetitions == 0
                && row.readiness.reasons.iter().any(|reason| reason.contains("no stage binding"))
        }));
    }

    #[test]
    fn publication_validates_before_atomic_write() {
        let surfaces = tempfile::tempdir().expect("surface root");
        let report = derive_benchmark_matrix(&request(surfaces.path())).expect("matrix");
        let output = surfaces.path().join("publication/matrix.json");
        publish_benchmark_matrix(&output, &report).expect("publish");
        let persisted: BenchmarkMatrixReport =
            serde_json::from_slice(&std::fs::read(&output).expect("read publication"))
                .expect("parse publication");
        assert_eq!(persisted, report);
        assert_eq!(load_benchmark_matrix_report(&output).expect("load publication"), report);

        let mut invalid = report;
        invalid.summary.total_rows += 1;
        let refused_output = surfaces.path().join("publication/refused.json");
        assert!(publish_benchmark_matrix(&refused_output, &invalid).is_err());
        assert!(!refused_output.exists());
    }
}
