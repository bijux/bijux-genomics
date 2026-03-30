use std::collections::BTreeMap;
use std::sync::OnceLock;

use bijux_dna_core::ids::{StageId, ToolId};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
struct ToolExecutionContractRecord {
    #[serde(default)]
    required_inputs: Vec<String>,
    #[serde(default)]
    optional_inputs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ToolStageContractRecord {
    #[serde(default)]
    required_inputs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ToolLayoutManifestRecord {
    tool_id: String,
    #[serde(default)]
    execution_contract: ToolExecutionContractRecord,
    #[serde(default)]
    stage_contracts: BTreeMap<String, ToolStageContractRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct StageToolInputLayoutContract {
    pub(super) supports_single_end: bool,
    pub(super) supports_paired_end: bool,
}

fn tool_layout_manifests() -> &'static BTreeMap<(String, String), StageToolInputLayoutContract> {
    static CONTRACTS: OnceLock<BTreeMap<(String, String), StageToolInputLayoutContract>> =
        OnceLock::new();
    CONTRACTS.get_or_init(|| {
        let mut contracts = BTreeMap::new();
        for raw in [
            include_str!("../../../../domain/fastq/tools/adapterremoval.yaml"),
            include_str!("../../../../domain/fastq/tools/atropos.yaml"),
            include_str!("../../../../domain/fastq/tools/bayeshammer.yaml"),
            include_str!("../../../../domain/fastq/tools/bbduk.yaml"),
            include_str!("../../../../domain/fastq/tools/bbmerge.yaml"),
            include_str!("../../../../domain/fastq/tools/bowtie2.yaml"),
            include_str!("../../../../domain/fastq/tools/bowtie2_build.yaml"),
            include_str!("../../../../domain/fastq/tools/centrifuge.yaml"),
            include_str!("../../../../domain/fastq/tools/clumpify.yaml"),
            include_str!("../../../../domain/fastq/tools/cutadapt.yaml"),
            include_str!("../../../../domain/fastq/tools/dada2.yaml"),
            include_str!("../../../../domain/fastq/tools/diamond.yaml"),
            include_str!("../../../../domain/fastq/tools/dustmasker.yaml"),
            include_str!("../../../../domain/fastq/tools/fastp.yaml"),
            include_str!("../../../../domain/fastq/tools/fastq_scan.yaml"),
            include_str!("../../../../domain/fastq/tools/fastqc.yaml"),
            include_str!("../../../../domain/fastq/tools/fastqvalidator.yaml"),
            include_str!("../../../../domain/fastq/tools/fastuniq.yaml"),
            include_str!("../../../../domain/fastq/tools/fastx_clipper.yaml"),
            include_str!("../../../../domain/fastq/tools/flash2.yaml"),
            include_str!("../../../../domain/fastq/tools/fqtools.yaml"),
            include_str!("../../../../domain/fastq/tools/kaiju.yaml"),
            include_str!("../../../../domain/fastq/tools/kraken2.yaml"),
            include_str!("../../../../domain/fastq/tools/krakenuniq.yaml"),
            include_str!("../../../../domain/fastq/tools/leehom.yaml"),
            include_str!("../../../../domain/fastq/tools/lighter.yaml"),
            include_str!("../../../../domain/fastq/tools/multiqc.yaml"),
            include_str!("../../../../domain/fastq/tools/musket.yaml"),
            include_str!("../../../../domain/fastq/tools/pear.yaml"),
            include_str!("../../../../domain/fastq/tools/prinseq.yaml"),
            include_str!("../../../../domain/fastq/tools/rcorrector.yaml"),
            include_str!("../../../../domain/fastq/tools/seqfu.yaml"),
            include_str!("../../../../domain/fastq/tools/seqkit.yaml"),
            include_str!("../../../../domain/fastq/tools/seqkit_stats.yaml"),
            include_str!("../../../../domain/fastq/tools/seqpurge.yaml"),
            include_str!("../../../../domain/fastq/tools/seqtk.yaml"),
            include_str!("../../../../domain/fastq/tools/skewer.yaml"),
            include_str!("../../../../domain/fastq/tools/sortmerna.yaml"),
            include_str!("../../../../domain/fastq/tools/star.yaml"),
            include_str!("../../../../domain/fastq/tools/trim_galore.yaml"),
            include_str!("../../../../domain/fastq/tools/trimmomatic.yaml"),
            include_str!("../../../../domain/fastq/tools/umi_tools.yaml"),
            include_str!("../../../../domain/fastq/tools/vsearch.yaml"),
            include_str!("../../../../domain/fastq/tools/alientrimmer.yaml"),
        ] {
            let manifest: ToolLayoutManifestRecord = bijux_dna_infra::formats::parse_yaml(raw)
                .unwrap_or_else(|err| panic!("parse fastq tool layout manifest: {err}"));
            for (stage_id, stage_contract) in manifest.stage_contracts {
                let requires_reads_r2 = stage_contract
                    .required_inputs
                    .iter()
                    .chain(manifest.execution_contract.required_inputs.iter())
                    .any(|input| input == "reads_r2");
                let allows_reads_r2 = requires_reads_r2
                    || manifest
                        .execution_contract
                        .optional_inputs
                        .iter()
                        .any(|input| input == "reads_r2");
                contracts.insert(
                    (stage_id, manifest.tool_id.clone()),
                    StageToolInputLayoutContract {
                        supports_single_end: !requires_reads_r2,
                        supports_paired_end: allows_reads_r2,
                    },
                );
            }
        }
        contracts
    })
}

pub(super) fn stage_tool_input_layout_contract(
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Option<StageToolInputLayoutContract> {
    tool_layout_manifests()
        .get(&(stage_id.to_string(), tool_id.to_string()))
        .copied()
}
