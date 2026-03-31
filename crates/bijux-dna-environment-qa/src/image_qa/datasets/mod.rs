mod discovery;
mod hydration;

use super::{QaDataset, QaStage};
pub(crate) use discovery::discover_qa_datasets;
pub(crate) use hydration::hydrate_datasets;

pub(crate) fn datasets_for_stage(stage: QaStage, datasets: &[QaDataset]) -> Vec<QaDataset> {
    match stage {
        QaStage::Merge => datasets
            .iter()
            .filter(|dataset| dataset.r2.is_some())
            .cloned()
            .collect(),
        QaStage::Trim => {
            let pe: Vec<QaDataset> = datasets
                .iter()
                .filter(|dataset| dataset.r2.is_some())
                .cloned()
                .collect();
            if pe.is_empty() {
                datasets.to_vec()
            } else {
                pe
            }
        }
        _ => datasets.to_vec(),
    }
}

pub(crate) fn dataset_input_hash(stage: QaStage, dataset: &QaDataset) -> String {
    match stage {
        QaStage::Merge => {
            let r1 = dataset.input_hash_r1.as_str();
            let r2 = dataset.input_hash_r2.as_deref().unwrap_or("missing");
            format!("{r1},{r2}")
        }
        _ => dataset.input_hash_r1.clone(),
    }
}
