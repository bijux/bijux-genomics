use bijux_dna_core::id_catalog;

use super::{contract, ObserverSpecializationContract};

pub(super) const CONTRACTS: &[ObserverSpecializationContract] = &[
    contract(
        "fastq.normalize_primers",
        id_catalog::TOOL_CUTADAPT,
        "report_json",
    ),
    contract("fastq.normalize_abundance", "seqkit", "report_json"),
    contract("fastq.infer_asvs", "dada2", "report_json"),
    contract("fastq.cluster_otus", "vsearch", "report_json"),
    contract("fastq.remove_chimeras", "vsearch", "report_json"),
];
