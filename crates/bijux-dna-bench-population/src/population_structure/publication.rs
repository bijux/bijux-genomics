//! Owner: bijux-dna-bench-population
//! Population structure publication.
use crate::execution::stage_result::path_relative_to_repo;
use crate::population_structure::model::{
    AdmixtureSampleRow, LocalVcfPopulationStructureDistanceSummary,
    LocalVcfPopulationStructureSampleGroup, PcaSampleRow,
};
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use std::path::Path;

pub(crate) fn rebase_population_structure_paths(
    payload: &mut serde_json::Value,
    repo_root: &Path,
    published_stage_root: &Path,
    population_labels_manifest: &Path,
) -> Result<()> {
    for (pointer, published_path) in [
        ("/sample_labels/manifest", population_labels_manifest.to_path_buf()),
        ("/pca/eigenvec_tsv", published_stage_root.join("eigenvec.tsv")),
        ("/pca/eigenval_tsv", published_stage_root.join("eigenval.tsv")),
        ("/pca/manifest_json", published_stage_root.join("pca_manifest.json")),
        ("/admixture/q_matrix_tsv", published_stage_root.join("admixture_q_matrix.tsv")),
        ("/admixture/k_selection_json", published_stage_root.join("admixture_k_selection.json")),
        ("/outputs/pruned_variants_tsv", published_stage_root.join("pruned_variants.tsv")),
        (
            "/outputs/population_structure_json",
            published_stage_root.join("population_structure.json"),
        ),
        ("/outputs/population_mds", published_stage_root.join("population_structure_mds.mds")),
        ("/outputs/tool_log", published_stage_root.join("population_structure_mds.log")),
    ] {
        let Some(slot) = payload.pointer_mut(pointer) else {
            continue;
        };
        if slot.is_null() {
            continue;
        }
        if !slot.is_string() {
            bail!("population-structure source path `{pointer}` must be a string or null");
        }
        *slot = serde_json::Value::String(path_relative_to_repo(repo_root, &published_path));
    }
    Ok(())
}

pub(crate) fn build_sample_groups(
    pca_rows: &[PcaSampleRow],
    admixture_rows: &[AdmixtureSampleRow],
) -> Result<Vec<LocalVcfPopulationStructureSampleGroup>> {
    let mut groups = Vec::new();
    for (pca_row, admixture_row) in pca_rows.iter().zip(admixture_rows.iter()) {
        if pca_row.sample_id != admixture_row.sample_id {
            bail!(
                "consumed PCA/admixture sample order drifted: `{}` vs `{}`",
                pca_row.sample_id,
                admixture_row.sample_id
            );
        }
        let (dominant_cluster, dominant_fraction) = admixture_row
            .clusters
            .iter()
            .max_by(|left, right| left.1.partial_cmp(&right.1).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .ok_or_else(|| anyhow!("admixture row is missing cluster values"))?;
        groups.push(LocalVcfPopulationStructureSampleGroup {
            sample_id: pca_row.sample_id.clone(),
            population_id: pca_row.population_id.clone(),
            population_label: pca_row.population_label.clone(),
            sex: pca_row.sex.clone(),
            dominant_cluster,
            dominant_fraction,
            pc1: pca_row.pc1,
            pc2: pca_row.pc2,
            status: admixture_row.status.clone(),
        });
    }
    Ok(groups)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "Mean PC distance is an approximate floating-point observation; integer pair counts are retained separately"
)]
pub(crate) fn build_distance_summary(
    sample_groups: &[LocalVcfPopulationStructureSampleGroup],
) -> Result<LocalVcfPopulationStructureDistanceSummary> {
    if sample_groups.is_empty() {
        bail!("population-structure smoke requires at least one sample group");
    }
    let mut distances = Vec::new();
    let mut within_population_pair_count = 0_u64;
    let mut cross_population_pair_count = 0_u64;
    for left_index in 0..sample_groups.len() {
        for right_index in (left_index + 1)..sample_groups.len() {
            let left = &sample_groups[left_index];
            let right = &sample_groups[right_index];
            let delta_pc1 = left.pc1 - right.pc1;
            let delta_pc2 = left.pc2 - right.pc2;
            let distance = (delta_pc1.powi(2) + delta_pc2.powi(2)).sqrt();
            distances.push(distance);
            if left.population_id == right.population_id {
                within_population_pair_count += 1;
            } else {
                cross_population_pair_count += 1;
            }
        }
    }
    let pair_count = u64::try_from(distances.len()).map_err(|_| anyhow!("pair count overflow"))?;
    let sample_count =
        u64::try_from(sample_groups.len()).map_err(|_| anyhow!("sample count overflow"))?;
    let (min_pc_distance, max_pc_distance, mean_pc_distance) = if distances.is_empty() {
        (0.0, 0.0, 0.0)
    } else {
        let min = distances.iter().copied().fold(f64::INFINITY, f64::min);
        let max = distances.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let mean = distances.iter().copied().sum::<f64>() / distances.len() as f64;
        (min, max, mean)
    };
    Ok(LocalVcfPopulationStructureDistanceSummary {
        sample_count,
        pair_count,
        within_population_pair_count,
        cross_population_pair_count,
        min_pc_distance,
        max_pc_distance,
        mean_pc_distance,
    })
}
