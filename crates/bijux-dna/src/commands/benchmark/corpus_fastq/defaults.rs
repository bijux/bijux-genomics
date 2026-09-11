use crate::commands::benchmark_workspace::BenchmarkConfig;
use bijux_dna_bench_campaign::request::StageArgumentDefaults;
pub(super) fn stage_defaults(config: &BenchmarkConfig, stage_id: &str) -> StageArgumentDefaults {
    match stage_id {
        "fastq.deplete_rrna" => {
            let inputs = &config.stage_inputs.fastq_deplete_rrna;
            StageArgumentDefaults {
                stage: supplied(&[
                    ("rrna_db", inputs.rrna_db.as_deref()),
                    ("rrna_bundle_id", inputs.rrna_bundle_id.as_deref()),
                    ("min_identity", inputs.min_identity.as_deref()),
                ]),
                manifest: supplied(&[]),
            }
        }
        "fastq.deplete_host" => {
            let inputs = &config.stage_inputs.fastq_deplete_host;
            StageArgumentDefaults {
                stage: supplied(&[("reference_index", inputs.reference_index.as_deref())]),
                manifest: supplied(&[
                    ("reference_catalog_id", inputs.reference_catalog_id.as_deref()),
                    ("reference_index_backend", inputs.reference_index_backend.as_deref()),
                ]),
            }
        }
        "fastq.deplete_reference_contaminants" => {
            let inputs = &config.stage_inputs.fastq_deplete_reference_contaminants;
            StageArgumentDefaults {
                stage: supplied(&[("reference_index", inputs.reference_index.as_deref())]),
                manifest: supplied(&[
                    ("reference_catalog_id", inputs.reference_catalog_id.as_deref()),
                    ("reference_index_backend", inputs.reference_index_backend.as_deref()),
                ]),
            }
        }
        "fastq.screen_taxonomy" => {
            let inputs = &config.stage_inputs.fastq_screen_taxonomy;
            StageArgumentDefaults {
                stage: supplied(&[("database_root", inputs.database_root.as_deref())]),
                manifest: supplied(&[
                    ("database_catalog_id", inputs.database_catalog_id.as_deref()),
                    ("database_artifact_id", inputs.database_artifact_id.as_deref()),
                    ("database_namespace", inputs.database_namespace.as_deref()),
                    ("database_scope", inputs.database_scope.as_deref()),
                ]),
            }
        }
        _ => StageArgumentDefaults::default(),
    }
}
fn supplied(values: &[(&str, Option<&str>)]) -> Vec<(String, String)> {
    values
        .iter()
        .filter_map(|(key, value)| value.map(|value| ((*key).to_string(), value.to_string())))
        .collect()
}
