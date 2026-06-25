use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use sha2::{Digest, Sha256};

use super::local_stage_commands::local_stage_plans;

const GENERATED_BENCHMARK_OUTPUT_ROOTS: [&str; 1] = ["benchmarks/readiness/stage-tool-commands/"];
const OUTPUT_FLAG_ARGS: [&str; 12] = [
    "-o",
    "-O",
    "--out",
    "--output",
    "--output-dir",
    "--output-root",
    "--output-prefix",
    "--prefix",
    "--stdout",
    "--stderr",
    "--report",
    "--summary",
];
const SOURCE_ROOTS: [&str; 3] = ["assets/", "benchmarks/", "runs/"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalHpcStageInputHint {
    pub(crate) artifact_id: String,
    pub(crate) artifact_role: String,
    pub(crate) source_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalHpcResolvedSourceAsset {
    pub(crate) source_kind: String,
    pub(crate) source_path: String,
    pub(crate) checksum_sha256: String,
    pub(crate) size_bytes: u64,
    pub(crate) member_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalHpcDiscoveredSourceInput {
    pub(crate) artifact_id: Option<String>,
    pub(crate) artifact_role: Option<String>,
    pub(crate) source_asset: LocalHpcResolvedSourceAsset,
}

#[derive(Debug, Default)]
struct StepPathUsage {
    consumed: BTreeSet<String>,
    produced: BTreeSet<String>,
}

pub(crate) fn collect_local_hpc_stage_input_hints(
    repo_root: &Path,
    stage_ids: &BTreeSet<String>,
) -> Result<BTreeMap<String, Vec<LocalHpcStageInputHint>>> {
    let mut hints = BTreeMap::<String, Vec<LocalHpcStageInputHint>>::new();
    for stage_id in stage_ids {
        let stage_plans = match local_stage_plans(repo_root, stage_id) {
            Ok(stage_plans) => stage_plans,
            Err(error) if error.to_string().starts_with("unsupported local benchmark stage `") => {
                continue;
            }
            Err(error)
                if error.to_string().contains(
                    "requires the `bam_downstream` feature for rendered local benchmark metadata",
                ) =>
            {
                hints.insert(stage_id.clone(), bam_downstream_stage_input_hints(stage_id));
                continue;
            }
            Err(error) => return Err(error),
        };
        let mut stage_hints = stage_plans
            .into_iter()
            .flat_map(|plan| plan.io.inputs.into_iter())
            .map(|input| LocalHpcStageInputHint {
                artifact_id: input.name.as_str().to_string(),
                artifact_role: input.role.as_str().to_string(),
                source_path: path_relative_to_repo(repo_root, &input.path),
            })
            .collect::<Vec<_>>();
        stage_hints.sort_by(|left, right| {
            left.source_path
                .cmp(&right.source_path)
                .then_with(|| left.artifact_id.cmp(&right.artifact_id))
                .then_with(|| left.artifact_role.cmp(&right.artifact_role))
        });
        stage_hints.dedup_by(|left, right| {
            left.source_path == right.source_path
                && left.artifact_id == right.artifact_id
                && left.artifact_role == right.artifact_role
        });
        hints.insert(stage_id.clone(), stage_hints);
    }
    Ok(hints)
}

fn bam_downstream_stage_input_hints(stage_id: &str) -> Vec<LocalHpcStageInputHint> {
    let hints = match stage_id {
        "bam.bias_mitigation" => vec![
            ("bam", "bam", "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam"),
            (
                "reference",
                "reference",
                "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta",
            ),
        ],
        "bam.genotyping" => vec![
            (
                "bam",
                "bam",
                "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_genotyping_candidate_panel.sam",
            ),
            (
                "bam_bai",
                "index",
                "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_genotyping_candidate_panel.sam.bai",
            ),
            (
                "reference",
                "reference",
                "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta",
            ),
            (
                "regions",
                "reference",
                "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_genotyping_target_regions.txt",
            ),
            (
                "sites",
                "variant",
                "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_genotyping_candidate_sites.vcf",
            ),
        ],
        "bam.haplogroups" => vec![
            (
                "bam",
                "bam",
                "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_y_haplogroup_panel.sam",
            ),
            (
                "bam_bai",
                "index",
                "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_y_haplogroup_panel.sam.bai",
            ),
            (
                "reference",
                "reference",
                "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta",
            ),
            (
                "reference_panel",
                "reference",
                "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_y_haplogroup_panel.tsv",
            ),
        ],
        "bam.kinship" => vec![
            (
                "bam",
                "bam",
                "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_kinship_low_overlap_pair.sam",
            ),
            (
                "bam",
                "bam",
                "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_kinship_related_pair.sam",
            ),
        ],
        _ => Vec::new(),
    };
    hints
        .into_iter()
        .map(|(artifact_id, artifact_role, source_path)| LocalHpcStageInputHint {
            artifact_id: artifact_id.to_string(),
            artifact_role: artifact_role.to_string(),
            source_path: source_path.to_string(),
        })
        .collect()
}

pub(crate) fn collect_local_hpc_job_source_inputs<'a, I>(
    repo_root: &Path,
    consumer_label: &str,
    stage_id: &str,
    command_steps: I,
    stage_input_hints: &BTreeMap<String, Vec<LocalHpcStageInputHint>>,
) -> Result<Vec<LocalHpcDiscoveredSourceInput>>
where
    I: IntoIterator<Item = &'a [String]>,
{
    let mut discovered_by_source = BTreeMap::<String, LocalHpcDiscoveredSourceInput>::new();
    let mut produced_paths = BTreeSet::<String>::new();

    for argv in command_steps {
        if let Some(materialized_stage_id) = materialize_stage_id(argv) {
            let hints = stage_input_hints.get(&materialized_stage_id).ok_or_else(|| {
                anyhow!(
                    "missing local stage input hints for materialized stage `{materialized_stage_id}`"
                )
            })?;
            for hint in hints {
                let resolved =
                    resolve_local_hpc_source_asset(repo_root, &hint.source_path)?.ok_or_else(|| {
                        anyhow!(
                            "materialized stage `{materialized_stage_id}` depends on unresolved source `{}`",
                            hint.source_path
                        )
                    })?;
                merge_discovered_source_input(
                    &mut discovered_by_source,
                    resolved,
                    Some(hint.artifact_id.clone()),
                    Some(hint.artifact_role.clone()),
                );
            }
            continue;
        }

        let usage = collect_step_path_usage(argv)?;
        for source_path in usage.consumed.difference(&usage.produced) {
            if produced_paths.contains(source_path.as_str()) {
                continue;
            }
            if let Some(resolved) = resolve_local_hpc_source_asset(repo_root, source_path)? {
                merge_discovered_source_input(&mut discovered_by_source, resolved, None, None);
            }
        }
        produced_paths.extend(usage.produced);
    }

    if discovered_by_source.is_empty() {
        if let Some(hints) = stage_input_hints.get(stage_id) {
            for hint in hints {
                let resolved = resolve_local_hpc_source_asset(repo_root, &hint.source_path)?
                    .ok_or_else(|| {
                        anyhow!(
                            "stage `{stage_id}` depends on unresolved source `{}`",
                            hint.source_path
                        )
                    })?;
                merge_discovered_source_input(
                    &mut discovered_by_source,
                    resolved,
                    Some(hint.artifact_id.clone()),
                    Some(hint.artifact_role.clone()),
                );
            }
        }
    }

    if discovered_by_source.is_empty() {
        return Err(anyhow!(
            "local HPC input discovery found no source inputs for `{consumer_label}`"
        ));
    }

    Ok(discovered_by_source.into_values().collect())
}

pub(crate) fn materialize_stage_id(argv: &[String]) -> Option<String> {
    if !argv.iter().any(|arg| arg == "materialize-stage") {
        return None;
    }
    argv.windows(2).find(|window| window[0] == "--stage-id").map(|window| window[1].clone())
}

pub(crate) fn resolve_local_hpc_source_asset(
    repo_root: &Path,
    source_path: &str,
) -> Result<Option<LocalHpcResolvedSourceAsset>> {
    let absolute_source = repo_root.join(source_path);
    if absolute_source.is_file() {
        let checksum_sha256 = bijux_dna_infra::hash_file_sha256(&absolute_source)
            .map_err(|err| anyhow!(err.to_string()))?;
        let size_bytes = absolute_source
            .metadata()
            .with_context(|| format!("read metadata for {}", absolute_source.display()))?
            .len();
        return Ok(Some(LocalHpcResolvedSourceAsset {
            source_kind: "file".to_string(),
            source_path: source_path.to_string(),
            checksum_sha256,
            size_bytes,
            member_count: 1,
        }));
    }
    if absolute_source.is_dir() {
        let (checksum_sha256, size_bytes, member_count) =
            digest_directory_members(&absolute_source, &absolute_source)?;
        return Ok(Some(LocalHpcResolvedSourceAsset {
            source_kind: "directory".to_string(),
            source_path: source_path.to_string(),
            checksum_sha256,
            size_bytes,
            member_count,
        }));
    }

    resolve_prefix_bundle(&absolute_source, source_path)
}

pub(crate) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).map_or_else(
        |_| path.to_string_lossy().replace('\\', "/"),
        |relative| relative.to_string_lossy().replace('\\', "/"),
    )
}

fn merge_discovered_source_input(
    discovered_by_source: &mut BTreeMap<String, LocalHpcDiscoveredSourceInput>,
    resolved: LocalHpcResolvedSourceAsset,
    artifact_id: Option<String>,
    artifact_role: Option<String>,
) {
    discovered_by_source
        .entry(resolved.source_path.clone())
        .and_modify(|entry| {
            if entry.artifact_id.is_none() {
                entry.artifact_id.clone_from(&artifact_id);
            }
            if entry.artifact_role.is_none() {
                entry.artifact_role.clone_from(&artifact_role);
            }
        })
        .or_insert(LocalHpcDiscoveredSourceInput {
            artifact_id,
            artifact_role,
            source_asset: resolved,
        });
}

fn shell_step_body(argv: &[String]) -> Option<&str> {
    let [shell, flag, body, ..] = argv else {
        return None;
    };
    let shell_name = Path::new(shell).file_name()?.to_str()?;
    if !matches!(shell_name, "sh" | "bash" | "zsh") {
        return None;
    }
    if !flag.starts_with('-') || !flag.contains('c') {
        return None;
    }

    Some(body)
}

fn collect_step_path_usage(argv: &[String]) -> Result<StepPathUsage> {
    if let Some(body) = shell_step_body(argv) {
        return collect_shell_step_path_usage(body);
    }
    Ok(collect_tokenized_step_path_usage(argv))
}

fn collect_tokenized_step_path_usage(argv: &[String]) -> StepPathUsage {
    let mut usage = StepPathUsage::default();
    let mut previous_flag: Option<&str> = None;

    for arg in argv {
        if let Some((assignment_key, path)) = assignment_repo_path(arg) {
            if is_generated_benchmark_output_path(path) || is_output_flag(assignment_key) {
                usage.produced.insert(path.to_string());
            } else {
                usage.consumed.insert(path.to_string());
            }
        } else if let Some(path) = candidate_repo_path(arg) {
            if is_generated_benchmark_output_path(path) || previous_flag.is_some_and(is_output_flag)
            {
                usage.produced.insert(path.to_string());
            } else {
                usage.consumed.insert(path.to_string());
            }
        }

        previous_flag = if arg.starts_with('-') { Some(arg.as_str()) } else { None };
    }

    usage
}

fn collect_shell_step_path_usage(shell_command: &str) -> Result<StepPathUsage> {
    let matcher = Regex::new(
        r#"'((?:assets|benchmarks|runs)/[^']+)'|"((?:assets|benchmarks|runs)/[^"]+)"|((?:assets|benchmarks|runs)/[A-Za-z0-9_./@+-]+)"#,
    )
    .context("compile local HPC input-discovery path matcher")?;
    let mut usage = StepPathUsage::default();

    for captures in matcher.captures_iter(shell_command) {
        let full_match = captures.get(0).expect("shell path capture must include full match");
        let source_path = captures
            .get(1)
            .or_else(|| captures.get(2))
            .or_else(|| captures.get(3))
            .expect("shell path capture must include a repo-relative path")
            .as_str()
            .trim_end_matches('.');
        if !is_repo_source_path(source_path) {
            continue;
        }
        if is_generated_benchmark_output_path(source_path)
            || is_output_context(shell_command, full_match.start())
        {
            usage.produced.insert(source_path.to_string());
        } else {
            usage.consumed.insert(source_path.to_string());
        }
    }

    Ok(usage)
}

fn is_output_context(body: &str, start: usize) -> bool {
    let prefix = body.get(..start).unwrap_or(body).trim_end();
    if prefix.ends_with('>') || prefix.ends_with("1>") || prefix.ends_with("2>") {
        return true;
    }

    last_shell_token(prefix).is_some_and(is_output_flag)
}

fn last_shell_token(prefix: &str) -> Option<&str> {
    prefix.rsplit(|ch: char| ch.is_whitespace()).find(|segment| !segment.is_empty())
}

fn is_output_flag(flag: &str) -> bool {
    OUTPUT_FLAG_ARGS.contains(&flag)
}

fn candidate_repo_path(value: &str) -> Option<&str> {
    if is_repo_source_path(value) {
        Some(value)
    } else {
        None
    }
}

fn assignment_repo_path(value: &str) -> Option<(&str, &str)> {
    let (key, path) = value.split_once('=')?;
    candidate_repo_path(path).map(|path| (key, path))
}

fn is_repo_source_path(value: &str) -> bool {
    SOURCE_ROOTS.iter().any(|root| value.starts_with(root))
}

fn is_generated_benchmark_output_path(value: &str) -> bool {
    GENERATED_BENCHMARK_OUTPUT_ROOTS.iter().any(|root| value.starts_with(root))
}

fn resolve_prefix_bundle(
    absolute_source: &Path,
    source_path: &str,
) -> Result<Option<LocalHpcResolvedSourceAsset>> {
    let Some(parent) = absolute_source.parent() else {
        return Ok(None);
    };
    if !parent.exists() {
        return Ok(None);
    }
    let Some(prefix) = absolute_source.file_name().and_then(|value| value.to_str()) else {
        return Ok(None);
    };

    let mut members = fs::read_dir(parent)
        .with_context(|| format!("read {}", parent.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.starts_with(&format!("{prefix}.")))
        })
        .collect::<Vec<_>>();
    members.sort();

    if members.is_empty() {
        return Ok(None);
    }

    let (checksum_sha256, size_bytes, member_count) = digest_explicit_members(&members, parent)?;
    Ok(Some(LocalHpcResolvedSourceAsset {
        source_kind: "prefix_bundle".to_string(),
        source_path: source_path.to_string(),
        checksum_sha256,
        size_bytes,
        member_count,
    }))
}

fn digest_directory_members(root: &Path, path: &Path) -> Result<(String, u64, usize)> {
    let mut members = Vec::new();
    collect_directory_files(path, &mut members)?;
    members.sort();
    digest_explicit_members(&members, root)
}

fn collect_directory_files(path: &Path, members: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("read {}", path.display()))? {
        let entry = entry.with_context(|| format!("read entry in {}", path.display()))?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_directory_files(&entry_path, members)?;
        } else if entry_path.is_file() {
            members.push(entry_path);
        }
    }
    Ok(())
}

fn digest_explicit_members(
    members: &[PathBuf],
    relative_root: &Path,
) -> Result<(String, u64, usize)> {
    let mut hasher = Sha256::new();
    let mut size_bytes = 0u64;

    for member in members {
        let digest =
            bijux_dna_infra::hash_file_sha256(member).map_err(|err| anyhow!(err.to_string()))?;
        let metadata =
            member.metadata().with_context(|| format!("read metadata for {}", member.display()))?;
        let relative_member = member
            .strip_prefix(relative_root)
            .unwrap_or(member)
            .to_string_lossy()
            .replace('\\', "/");
        size_bytes += metadata.len();
        hasher.update(relative_member.as_bytes());
        hasher.update([0]);
        hasher.update(digest.as_bytes());
        hasher.update([0]);
        hasher.update(metadata.len().to_string().as_bytes());
        hasher.update([b'\n']);
    }

    Ok((hex_sha256_digest(hasher), size_bytes, members.len()))
}

fn hex_sha256_digest(hasher: Sha256) -> String {
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}
