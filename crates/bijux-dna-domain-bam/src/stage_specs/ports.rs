//! Complete authored output declarations, including optional reporting companions.
use super::BamStage;
use anyhow::{ensure, Context, Result};
use serde::Deserialize;
use std::{collections::BTreeSet, sync::OnceLock};

#[derive(Deserialize)]
struct OutputPort {
    name: String,
}

#[derive(Deserialize)]
struct Manifest {
    stage_id: String,
    outputs: Vec<OutputPort>,
}

/// Return every authored output name in declaration order, including optional outputs.
/// Required outputs remain available through `stage_output_ids_in_manifest_order`.
///
/// # Errors
/// Rejects unknown stages or malformed, misidentified and duplicate declarations.
pub fn stage_declared_output_ids(stage_id: &str) -> Result<&'static [String]> {
    let stage = BamStage::try_from(stage_id).context("unknown BAM stage output contract")?;
    macro_rules! output_names {
        ($file:literal) => {{
            static OUTPUTS: OnceLock<Result<Vec<String>, String>> = OnceLock::new();
            OUTPUTS
                .get_or_init(|| {
                    parse_outputs(
                        stage,
                        include_str!(concat!(
                            env!("CARGO_MANIFEST_DIR"),
                            "/../../domain/bam/stages/",
                            $file
                        )),
                    )
                    .map_err(|error| error.to_string())
                })
                .as_deref()
                .map_err(|error| anyhow::anyhow!(error.clone()))
        }};
    }
    match stage {
        BamStage::Align => output_names!("align.yaml"),
        BamStage::Validate => output_names!("validate.yaml"),
        BamStage::QcPre => output_names!("qc_pre.yaml"),
        BamStage::MappingSummary => output_names!("mapping_summary.yaml"),
        BamStage::Filter => output_names!("filter.yaml"),
        BamStage::MapqFilter => output_names!("mapq_filter.yaml"),
        BamStage::LengthFilter => output_names!("length_filter.yaml"),
        BamStage::Markdup => output_names!("markdup.yaml"),
        BamStage::DuplicationMetrics => output_names!("duplication_metrics.yaml"),
        BamStage::Complexity => output_names!("complexity.yaml"),
        BamStage::Coverage => output_names!("coverage.yaml"),
        BamStage::InsertSize => output_names!("insert_size.yaml"),
        BamStage::GcBias => output_names!("gc_bias.yaml"),
        BamStage::EndogenousContent => output_names!("endogenous_content.yaml"),
        BamStage::OverlapCorrection => output_names!("overlap_correction.yaml"),
        BamStage::Damage => output_names!("damage.yaml"),
        BamStage::Authenticity => output_names!("authenticity.yaml"),
        BamStage::Contamination => output_names!("contamination.yaml"),
        BamStage::Sex => output_names!("sex.yaml"),
        BamStage::BiasMitigation => output_names!("bias_mitigation.yaml"),
        BamStage::Recalibration => output_names!("recalibration.yaml"),
        BamStage::Haplogroups => output_names!("haplogroups.yaml"),
        BamStage::Genotyping => output_names!("genotyping.yaml"),
        BamStage::Kinship => output_names!("kinship.yaml"),
    }
}

fn parse_outputs(stage: BamStage, raw: &str) -> Result<Vec<String>> {
    let manifest: Manifest = bijux_dna_infra::formats::parse_yaml(raw)?;
    ensure!(manifest.stage_id == stage.as_str(), "BAM output manifest stage identity differs");
    let outputs = manifest.outputs.into_iter().map(|port| port.name).collect::<Vec<_>>();
    ensure!(!outputs.is_empty(), "BAM output manifest has no outputs");
    ensure!(
        outputs.iter().all(|id| !id.is_empty() && id.trim() == id),
        "BAM output manifest has a blank or padded name"
    );
    ensure!(
        outputs.iter().collect::<BTreeSet<_>>().len() == outputs.len(),
        "BAM output manifest repeats an artifact"
    );
    Ok(outputs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_outputs_admit_all_stages_and_retain_optional_qc_reports() -> Result<()> {
        for stage in BamStage::all() {
            assert!(!stage_declared_output_ids(stage.as_str())?.is_empty());
        }
        let outputs = stage_declared_output_ids("bam.qc_pre")?;
        assert_eq!(
            outputs,
            [
                "flagstat",
                "idxstats",
                "stats",
                "stage_metrics",
                "report_json",
                "multiqc_report",
                "multiqc_data",
                "governed_qc_inputs_manifest"
            ]
        );
        assert!(!super::super::stage_output_ids_in_manifest_order("bam.qc_pre")
            .context("required outputs")?
            .iter()
            .any(|id| id == "report_json"));
        assert!(std::ptr::eq(outputs, stage_declared_output_ids("bam.qc_pre")?));
        assert!(stage_declared_output_ids("bam.unknown").is_err());
        Ok(())
    }

    #[test]
    fn malformed_or_misbound_output_declarations_are_rejected() {
        for raw in [
            "stage_id: bam.validate\noutputs: [{name: report}, {name: report}]",
            "stage_id: bam.qc_pre\noutputs: [{name: report}]",
            "stage_id: bam.validate\noutputs: []",
            "stage_id: bam.validate\noutputs: [{name: ' report '}]",
            "stage_id: bam.validate\noutputs: [{data_type: json}]",
        ] {
            assert!(parse_outputs(BamStage::Validate, raw).is_err());
        }
    }
}
