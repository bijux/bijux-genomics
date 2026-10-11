//! Owner: bijux-dna-bench-population
//! Population structure observations records.
use crate::population_structure::model::{AdmixtureSampleRow, PcaSampleRow};
use anyhow::anyhow;
use anyhow::Result;

pub(crate) fn parse_pca_rows(payload: &serde_json::Value) -> Result<Vec<PcaSampleRow>> {
    payload
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("consumed PCA report missing `rows`"))?
        .iter()
        .map(|row| {
            Ok(PcaSampleRow {
                sample_id: row
                    .get("sample_id")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed PCA row missing `sample_id`"))?
                    .to_string(),
                population_id: row
                    .get("population_id")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed PCA row missing `population_id`"))?
                    .to_string(),
                population_label: row
                    .get("population_label")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed PCA row missing `population_label`"))?
                    .to_string(),
                sex: row
                    .get("sex")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed PCA row missing `sex`"))?
                    .to_string(),
                pc1: row
                    .get("pc1")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or_else(|| anyhow!("consumed PCA row missing `pc1`"))?,
                pc2: row
                    .get("pc2")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or_else(|| anyhow!("consumed PCA row missing `pc2`"))?,
            })
        })
        .collect::<Result<Vec<_>>>()
}

pub(crate) fn parse_admixture_rows(payload: &serde_json::Value) -> Result<Vec<AdmixtureSampleRow>> {
    let cluster_headers = payload
        .get("cluster_headers")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("consumed admixture report missing `cluster_headers`"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("consumed admixture cluster headers must be strings"))
        })
        .collect::<Result<Vec<_>>>()?;
    payload
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("consumed admixture report missing `rows`"))?
        .iter()
        .map(|row| {
            let mut clusters = Vec::new();
            for header in &cluster_headers {
                clusters.push((
                    header.clone(),
                    row.get(header)
                        .and_then(serde_json::Value::as_f64)
                        .ok_or_else(|| anyhow!("consumed admixture row missing `{header}`"))?,
                ));
            }
            Ok(AdmixtureSampleRow {
                sample_id: row
                    .get("sample_id")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed admixture row missing `sample_id`"))?
                    .to_string(),
                population_id: row
                    .get("population_id")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed admixture row missing `population_id`"))?
                    .to_string(),
                population_label: row
                    .get("population_label")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed admixture row missing `population_label`"))?
                    .to_string(),
                sex: row
                    .get("sex")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed admixture row missing `sex`"))?
                    .to_string(),
                status: row
                    .get("status")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| anyhow!("consumed admixture row missing `status`"))?
                    .to_string(),
                clusters,
            })
        })
        .collect::<Result<Vec<_>>>()
}
