use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::commands::cli::BenchmarkMatrixArgs;
use crate::commands::cli::env::registry_tools_for_stage;
use crate::commands::hpc::campaign_dry_run;

const BENCHMARK_MATRIX_SCHEMA_VERSION: &str = "bijux.hpc.benchmark_matrix.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMatrixReport {
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub domains: Vec<String>,
    pub generated_at: String,
    pub summary: BenchmarkMatrixSummary,
    pub rows: Vec<BenchmarkMatrixRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMatrixSummary {
    pub total_rows: usize,
    pub readiness_counts: std::collections::BTreeMap<String, usize>,
    pub domain_counts: std::collections::BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMatrixRow {
    pub row_id: String,
    pub matrix_domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_match: BenchmarkSurfaceMatch,
    pub database_match: BenchmarkSurfaceMatch,
    pub image_match: BenchmarkSurfaceMatch,
    pub readiness: BenchmarkReadiness,
    pub repetitions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSurfaceMatch {
    pub required_profile: String,
    pub matched_profile: String,
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReadiness {
    pub class: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct CrossBridge {
    id: &'static str,
    from_stage: &'static str,
    to_stage: &'static str,
}

fn now_timestamp_compact() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |delta| delta.as_secs());
    secs.to_string()
}

fn workspace_root() -> Result<PathBuf> {
    let mut cursor = std::env::current_dir().context("resolve current directory")?;
    loop {
        let domain_dir = cursor.join("domain");
        let registry = cursor.join("configs").join("ci").join("registry").join("tool_registry.toml");
        if domain_dir.is_dir() && registry.is_file() {
            return Ok(cursor);
        }
        let Some(parent) = cursor.parent() else {
            break;
        };
        cursor = parent.to_path_buf();
    }
    Err(anyhow!(
        "unable to locate workspace root containing domain/ and configs/ci/registry/tool_registry.toml"
    ))
}

fn domain_stage_ids(root: &Path, domain: &str) -> Result<Vec<String>> {
    let stages_dir = root.join("domain").join(domain).join("stages");
    if !stages_dir.is_dir() {
        return Err(anyhow!("stage catalog not found: {}", stages_dir.display()));
    }
    let mut stages = Vec::new();
    for entry in std::fs::read_dir(&stages_dir)
        .with_context(|| format!("read {}", stages_dir.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if name.starts_with('_') {
            continue;
        }
        stages.push(format!("{domain}.{name}"));
    }
    stages.sort();
    stages.dedup();
    Ok(stages)
}

fn registry_path_from_root(root: &Path) -> PathBuf {
    bijux_dna_infra::configs_file(root, "ci/registry/tool_registry.toml")
}

fn normalize_token(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
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
            let name = entry.file_name().to_string_lossy().to_string();
            out.push(normalize_token(&name));
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
    if needs.iter().any(|needle| stage_id.contains(needle)) {
        if stage_id.contains("taxonomy") {
            return Some("taxonomy");
        }
        if stage_id.contains("rrna") {
            return Some("rrna");
        }
        if stage_id.starts_with("vcf.") || stage_id.contains("call") || stage_id.contains("genotyp") {
            return Some("vcf");
        }
        if stage_id.contains("align") {
            return Some("align");
        }
        return Some("general");
    }
    None
}

fn tool_match_tokens(tool_id: &str) -> Vec<String> {
    tool_id
        .split("=>")
        .flat_map(|part| part.split(','))
        .map(str::trim)
        .filter(|part| !part.is_empty())
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
    if let Some(required) = stage_database_profile(stage_id) {
        match_surface(required, tokens)
    } else {
        BenchmarkSurfaceMatch {
            required_profile: "not-required".to_string(),
            matched_profile: "not-required".to_string(),
            ready: true,
        }
    }
}

fn match_image_surface(tool_id: &str, image_tokens: &[String]) -> BenchmarkSurfaceMatch {
    let required = tool_match_tokens(tool_id);
    if required.is_empty() {
        return BenchmarkSurfaceMatch {
            required_profile: "unknown".to_string(),
            matched_profile: "<missing>".to_string(),
            ready: false,
        };
    }
    let mut matched = Vec::new();
    for token in required {
        if image_tokens.iter().any(|image| image.contains(&token)) {
            matched.push(token);
        }
    }
    let ready = !matched.is_empty();
    BenchmarkSurfaceMatch {
        required_profile: "tool-images".to_string(),
        matched_profile: if matched.is_empty() {
            "<missing>".to_string()
        } else {
            matched.join(",")
        },
        ready,
    }
}

fn classify_readiness(
    corpus_match: &BenchmarkSurfaceMatch,
    database_match: &BenchmarkSurfaceMatch,
    image_match: &BenchmarkSurfaceMatch,
) -> BenchmarkReadiness {
    let mut reasons = Vec::new();
    if !corpus_match.ready {
        reasons.push(format!(
            "corpus profile `{}` missing",
            corpus_match.required_profile
        ));
    }
    if !database_match.ready {
        reasons.push(format!(
            "database profile `{}` missing",
            database_match.required_profile
        ));
    }
    if !image_match.ready {
        reasons.push("image match missing for tool binding".to_string());
    }
    let class = if reasons.is_empty() {
        "ready"
    } else if reasons.len() == 1 {
        "degraded"
    } else {
        "refuse"
    };
    BenchmarkReadiness {
        class: class.to_string(),
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
    let mut readiness_counts = std::collections::BTreeMap::new();
    let mut domain_counts = std::collections::BTreeMap::new();
    for row in rows {
        *readiness_counts.entry(row.readiness.class.clone()).or_insert(0) += 1;
        *domain_counts.entry(row.matrix_domain.clone()).or_insert(0) += 1;
    }
    BenchmarkMatrixSummary {
        total_rows: rows.len(),
        readiness_counts,
        domain_counts,
    }
}

fn cross_bridges() -> &'static [CrossBridge] {
    &[
        CrossBridge {
            id: "fastq_to_bam",
            from_stage: "fastq.trim_reads",
            to_stage: "bam.align",
        },
        CrossBridge {
            id: "bam_to_vcf",
            from_stage: "bam.genotyping",
            to_stage: "vcf.call",
        },
        CrossBridge {
            id: "fastq_to_vcf",
            from_stage: "fastq.trim_reads",
            to_stage: "vcf.call_gl",
        },
    ]
}

fn resolve_matrix_domains(value: &str) -> Result<Vec<String>> {
    match value {
        "all" => Ok(vec![
            "fastq".to_string(),
            "bam".to_string(),
            "vcf".to_string(),
            "cross".to_string(),
        ]),
        "fastq" | "bam" | "vcf" => Ok(vec![value.to_string()]),
        "cross" => Ok(vec!["cross".to_string()]),
        other => Err(anyhow!(
            "benchmark-matrix supports --domain fastq|bam|vcf|cross|all; got `{other}`"
        )),
    }
}

pub fn benchmark_matrix(args: &BenchmarkMatrixArgs) -> Result<BenchmarkMatrixReport> {
    let domains = resolve_matrix_domains(&args.domain)?;
    let dry_run =
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_overrides.as_deref())?;
    let root = workspace_root()?;
    let registry_path = registry_path_from_root(&root);
    let corpus_tokens = collect_name_tokens(Path::new(&dry_run.layout.corpora_root), false)?;
    let database_tokens = collect_name_tokens(Path::new(&dry_run.layout.databases_root), false)?;
    let image_tokens = collect_name_tokens(Path::new(&dry_run.layout.images_root), true)?;
    let mut rows = Vec::new();
    for domain in &domains {
        if domain == "cross" {
            for bridge in cross_bridges() {
                let left_tools = registry_tools_for_stage(&registry_path, bridge.from_stage, None, "all")
                    .unwrap_or_default();
                let right_tools = registry_tools_for_stage(&registry_path, bridge.to_stage, None, "all")
                    .unwrap_or_default();
                for left in &left_tools {
                    for right in &right_tools {
                        let stage_binding = format!("{}=>{}", bridge.from_stage, bridge.to_stage);
                        let tool_binding = format!("{left}=>{right}");
                        let corpus_match = match_surface(
                            stage_corpus_profile(&stage_binding),
                            &corpus_tokens,
                        );
                        let database_match = match_database_surface(&stage_binding, &database_tokens);
                        let image_match = match_image_surface(&tool_binding, &image_tokens);
                        let readiness =
                            classify_readiness(&corpus_match, &database_match, &image_match);
                        rows.push(BenchmarkMatrixRow {
                            row_id: format!("cross.{}::{}::{}", bridge.id, stage_binding, tool_binding),
                            matrix_domain: "cross".to_string(),
                            stage_id: stage_binding,
                            tool_id: tool_binding,
                            repetitions: repetition_policy("cross", bridge.to_stage, &readiness.class),
                            corpus_match,
                            database_match,
                            image_match,
                            readiness,
                        });
                    }
                }
            }
            continue;
        }
        let stages = domain_stage_ids(&root, domain)?;
        for stage_id in stages {
            let tools = match registry_tools_for_stage(&registry_path, &stage_id, None, "all") {
                Ok(value) if !value.is_empty() => value,
                _ => vec!["<unbound>".to_string()],
            };
            for tool_id in tools {
                let corpus_match = match_surface(stage_corpus_profile(&stage_id), &corpus_tokens);
                let database_match = match_database_surface(&stage_id, &database_tokens);
                let image_match = match_image_surface(&tool_id, &image_tokens);
                let readiness =
                    classify_readiness(&corpus_match, &database_match, &image_match);
                rows.push(BenchmarkMatrixRow {
                    row_id: format!("{stage_id}::{tool_id}"),
                    matrix_domain: domain.clone(),
                    stage_id: stage_id.clone(),
                    tool_id,
                    repetitions: repetition_policy(domain, &stage_id, &readiness.class),
                    corpus_match,
                    database_match,
                    image_match,
                    readiness,
                });
            }
        }
    }
    Ok(BenchmarkMatrixReport {
        schema_version: BENCHMARK_MATRIX_SCHEMA_VERSION.to_string(),
        campaign_id: dry_run.campaign_id,
        domain: args.domain.clone(),
        domains,
        generated_at: now_timestamp_compact(),
        summary: summarize_rows(&rows),
        rows,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{
        classify_readiness, cross_bridges, domain_stage_ids, match_database_surface, match_surface,
        repetition_policy, resolve_matrix_domains, summarize_rows, BenchmarkMatrixRow,
        BenchmarkReadiness, BenchmarkSurfaceMatch,
    };
    use crate::commands::cli::BenchmarkMatrixArgs;
    use crate::commands::hpc::benchmark_matrix;

    fn write_matrix_campaign(root: &std::path::Path) -> std::path::PathBuf {
        for name in [
            "corpora",
            "databases",
            "images",
            "scratch",
            "logs",
            "results",
            "code",
            "imports",
            "baselines",
        ] {
            std::fs::create_dir_all(root.join(name)).expect("create dir");
        }
        std::fs::write(root.join("corpora/modern_wgs"), b"x").expect("seed corpus token");
        std::fs::write(root.join("databases/vcf_reference"), b"x").expect("seed db token");
        std::fs::create_dir_all(root.join("images/apptainer")).expect("seed image dir");
        std::fs::write(root.join("images/apptainer/seqkit.sif"), b"x").expect("seed image token");
        let env_path = root.join("campaign.env");
        std::fs::write(&env_path, "BIJUX_SLURM_ACCOUNT=a\nBIJUX_SLURM_PROJECT=p\n")
            .expect("write env");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&env_path).expect("env metadata").permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&env_path, perms).expect("set env perms");
        }
        let config_path = root.join("campaign.toml");
        let config = format!(
            r#"
[campaign]
id = "matrix-mini"
domain = "fastq"

[layout]
corpora_root = "{root}/corpora"
databases_root = "{root}/databases"
images_root = "{root}/images"
scratch_root = "{root}/scratch"
logs_root = "{root}/logs"
encrypted_results_root = "{root}/results"
encrypted_code_root = "{root}/code"
appraiser_imports_root = "{root}/imports"
baselines_root = "{root}/baselines"

[slurm]
site_profile = "generic"

[resources]
default = "standard"

[resources.templates.standard]
cpus = 1
mem_gb = 1
walltime = "00:05:00"
scratch_gb = 1

[security]
encryption_recipients = ["alice"]
env_file = "{root}/campaign.env"

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#,
            root = root.display()
        );
        std::fs::write(&config_path, config).expect("write config");
        config_path
    }

    #[test]
    fn stage_catalog_lists_non_schema_fastq_entries() {
        let root = super::workspace_root().expect("workspace root");
        let stages = domain_stage_ids(&root, "fastq").expect("stages");
        assert!(stages.iter().all(|stage| stage.starts_with("fastq.")));
        assert!(stages.iter().any(|stage| stage == "fastq.validate_reads"));
        assert!(!stages.iter().any(|stage| stage.ends_with("._schema")));
    }

    #[test]
    fn stage_catalog_lists_non_schema_bam_entries() {
        let root = super::workspace_root().expect("workspace root");
        let stages = domain_stage_ids(&root, "bam").expect("stages");
        assert!(stages.iter().all(|stage| stage.starts_with("bam.")));
        assert!(stages.iter().any(|stage| stage == "bam.align"));
    }

    #[test]
    fn stage_catalog_lists_non_schema_vcf_entries() {
        let root = super::workspace_root().expect("workspace root");
        let stages = domain_stage_ids(&root, "vcf").expect("stages");
        assert!(stages.iter().all(|stage| stage.starts_with("vcf.")));
        assert!(stages.iter().any(|stage| stage == "vcf.call"));
    }

    #[test]
    fn matrix_domain_selector_supports_all_and_single_domains() {
        assert_eq!(
            resolve_matrix_domains("all").expect("all"),
            vec![
                "fastq".to_string(),
                "bam".to_string(),
                "vcf".to_string(),
                "cross".to_string()
            ]
        );
        assert_eq!(
            resolve_matrix_domains("bam").expect("bam"),
            vec!["bam".to_string()]
        );
        assert_eq!(
            resolve_matrix_domains("cross").expect("cross"),
            vec!["cross".to_string()]
        );
        assert!(resolve_matrix_domains("unknown").is_err());
    }

    #[test]
    fn cross_bridge_catalog_is_populated() {
        let bridges = cross_bridges();
        assert!(bridges.len() >= 3);
        assert!(bridges.iter().any(|bridge| bridge.id == "fastq_to_bam"));
    }

    #[test]
    fn readiness_classifies_missing_surfaces_as_refuse() {
        let corpus = BenchmarkSurfaceMatch {
            required_profile: "wgs".to_string(),
            matched_profile: "<missing>".to_string(),
            ready: false,
        };
        let db = BenchmarkSurfaceMatch {
            required_profile: "vcf".to_string(),
            matched_profile: "<missing>".to_string(),
            ready: false,
        };
        let image = BenchmarkSurfaceMatch {
            required_profile: "tool-images".to_string(),
            matched_profile: "<missing>".to_string(),
            ready: false,
        };
        let readiness = classify_readiness(&corpus, &db, &image);
        assert_eq!(readiness.class, "refuse");
        assert!(readiness.reasons.len() >= 2);
    }

    #[test]
    fn repetition_policy_increases_for_cross_and_call_paths() {
        assert_eq!(repetition_policy("fastq", "fastq.validate_reads", "ready"), 3);
        assert_eq!(repetition_policy("vcf", "vcf.call", "ready"), 5);
        assert_eq!(repetition_policy("cross", "fastq.trim_reads=>bam.align", "degraded"), 4);
        assert_eq!(repetition_policy("bam", "bam.align", "refuse"), 0);
    }

    #[test]
    fn database_surface_marks_not_required_when_stage_is_independent() {
        let match_result = match_database_surface("fastq.profile_reads", &[]);
        assert_eq!(match_result.required_profile, "not-required");
        assert!(match_result.ready);
    }

    #[test]
    fn corpus_surface_uses_profile_matching_tokens() {
        let match_result = match_surface("edna", &["modern_wgs".to_string(), "edna_sweden".to_string()]);
        assert_eq!(match_result.required_profile, "edna");
        assert!(match_result.ready);
        assert!(match_result.matched_profile.contains("edna"));
    }

    #[test]
    fn benchmark_matrix_generates_fastq_rows() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_matrix_campaign(root.path());
        let report = benchmark_matrix(&BenchmarkMatrixArgs {
            config,
            env_file: None,
            user_overrides: None,
            domain: "fastq".to_string(),
            out: None,
            fail_on_refuse: false,
            json: false,
        })
        .expect("matrix");
        assert_eq!(report.domain, "fastq");
        assert!(report.rows.iter().any(|row| row.matrix_domain == "fastq"));
    }

    #[test]
    fn benchmark_matrix_generates_bam_and_vcf_rows() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_matrix_campaign(root.path());
        let bam = benchmark_matrix(&BenchmarkMatrixArgs {
            config: config.clone(),
            env_file: None,
            user_overrides: None,
            domain: "bam".to_string(),
            out: None,
            fail_on_refuse: false,
            json: false,
        })
        .expect("bam matrix");
        let vcf = benchmark_matrix(&BenchmarkMatrixArgs {
            config,
            env_file: None,
            user_overrides: None,
            domain: "vcf".to_string(),
            out: None,
            fail_on_refuse: false,
            json: false,
        })
        .expect("vcf matrix");
        assert!(bam.rows.iter().any(|row| row.matrix_domain == "bam"));
        assert!(vcf.rows.iter().any(|row| row.matrix_domain == "vcf"));
    }

    #[test]
    fn benchmark_matrix_generates_cross_rows_and_all_domains() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_matrix_campaign(root.path());
        let cross = benchmark_matrix(&BenchmarkMatrixArgs {
            config: config.clone(),
            env_file: None,
            user_overrides: None,
            domain: "cross".to_string(),
            out: None,
            fail_on_refuse: false,
            json: false,
        })
        .expect("cross matrix");
        assert!(cross.rows.iter().all(|row| row.matrix_domain == "cross"));
        assert!(cross.rows.iter().all(|row| row.stage_id.contains("=>")));

        let all = benchmark_matrix(&BenchmarkMatrixArgs {
            config,
            env_file: None,
            user_overrides: None,
            domain: "all".to_string(),
            out: None,
            fail_on_refuse: false,
            json: false,
        })
        .expect("all matrix");
        assert!(all.domains.contains(&"fastq".to_string()));
        assert!(all.domains.contains(&"bam".to_string()));
        assert!(all.domains.contains(&"vcf".to_string()));
        assert!(all.domains.contains(&"cross".to_string()));
        assert!(all.rows.iter().any(|row| row.matrix_domain == "cross"));
    }

    #[test]
    fn summary_aggregates_readiness_and_domain_counts() {
        let rows = vec![
            BenchmarkMatrixRow {
                row_id: "r1".to_string(),
                matrix_domain: "fastq".to_string(),
                stage_id: "fastq.validate_reads".to_string(),
                tool_id: "seqkit".to_string(),
                corpus_match: BenchmarkSurfaceMatch {
                    required_profile: "general".to_string(),
                    matched_profile: "general".to_string(),
                    ready: true,
                },
                database_match: BenchmarkSurfaceMatch {
                    required_profile: "not-required".to_string(),
                    matched_profile: "not-required".to_string(),
                    ready: true,
                },
                image_match: BenchmarkSurfaceMatch {
                    required_profile: "tool-images".to_string(),
                    matched_profile: "seqkit".to_string(),
                    ready: true,
                },
                readiness: BenchmarkReadiness {
                    class: "ready".to_string(),
                    reasons: Vec::new(),
                },
                repetitions: 3,
            },
            BenchmarkMatrixRow {
                row_id: "r2".to_string(),
                matrix_domain: "vcf".to_string(),
                stage_id: "vcf.call".to_string(),
                tool_id: "<unbound>".to_string(),
                corpus_match: BenchmarkSurfaceMatch {
                    required_profile: "wgs".to_string(),
                    matched_profile: "<missing>".to_string(),
                    ready: false,
                },
                database_match: BenchmarkSurfaceMatch {
                    required_profile: "vcf".to_string(),
                    matched_profile: "<missing>".to_string(),
                    ready: false,
                },
                image_match: BenchmarkSurfaceMatch {
                    required_profile: "tool-images".to_string(),
                    matched_profile: "<missing>".to_string(),
                    ready: false,
                },
                readiness: BenchmarkReadiness {
                    class: "refuse".to_string(),
                    reasons: vec!["missing".to_string()],
                },
                repetitions: 0,
            },
        ];
        let summary = summarize_rows(&rows);
        assert_eq!(summary.total_rows, 2);
        assert_eq!(summary.domain_counts.get("fastq"), Some(&1));
        assert_eq!(summary.domain_counts.get("vcf"), Some(&1));
        assert_eq!(summary.readiness_counts.get("ready"), Some(&1));
        assert_eq!(summary.readiness_counts.get("refuse"), Some(&1));
    }
}
