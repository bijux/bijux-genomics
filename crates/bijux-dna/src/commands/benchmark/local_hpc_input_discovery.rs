use super::local_stage_commands::local_stage_plans;
use super::readiness::benchmark_command_rows::DEFAULT_STAGE_TOOL_COMMAND_ROOT;
use anyhow::Result;
pub(crate) use bijux_dna_api::v1::api::bench::{
    materialize_stage_id, HpcDiscoveredSourceInput as LocalHpcDiscoveredSourceInput,
    HpcStageInputHint as LocalHpcStageInputHint,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
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

pub(crate) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).map_or_else(
        |_| path.to_string_lossy().replace('\\', "/"),
        |relative| relative.to_string_lossy().replace('\\', "/"),
    )
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
    bijux_dna_api::v1::api::bench::collect_hpc_job_source_inputs(
        repo_root,
        consumer_label,
        stage_id,
        command_steps,
        stage_input_hints,
        DEFAULT_STAGE_TOOL_COMMAND_ROOT,
    )
}
