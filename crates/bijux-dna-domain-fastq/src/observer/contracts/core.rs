use bijux_dna_core::id_catalog;

use super::{contract, ObserverSpecializationContract};

pub(super) const CONTRACTS: &[ObserverSpecializationContract] = &[
    contract("fastq.index_reference", "bowtie2_build", "report_json"),
    contract("fastq.index_reference", "star", "report_json"),
    contract("fastq.validate_reads", "fastqvalidator", "validation_report"),
    contract("fastq.validate_reads", "fastqc", "validation_report"),
    contract("fastq.validate_reads", "fastq_scan", "validation_report"),
    contract("fastq.validate_reads", "seqtk", "validation_report"),
    contract("fastq.validate_reads", "fqtools", "validation_report"),
    contract("fastq.profile_read_lengths", "seqkit_stats", "report_json"),
    contract("fastq.detect_adapters", "fastqc", "report_json"),
    contract("fastq.profile_overrepresented_sequences", "fastqc", "report_json"),
    contract("fastq.profile_overrepresented_sequences", "fastq_scan", "report_json"),
    contract("fastq.profile_overrepresented_sequences", "seqkit", "report_json"),
    contract("fastq.profile_reads", "seqfu", "qc_json"),
    contract("fastq.profile_reads", "seqkit", "qc_json"),
    contract("fastq.profile_reads", "seqkit_stats", "qc_json"),
    contract("fastq.report_qc", "multiqc", "multiqc_data"),
    contract("fastq.screen_taxonomy", id_catalog::TOOL_KRAKEN2, "classification_report_json"),
    contract("fastq.screen_taxonomy", "krakenuniq", "classification_report_json"),
    contract("fastq.screen_taxonomy", "centrifuge", "classification_report_json"),
    contract("fastq.screen_taxonomy", "kaiju", "classification_report_json"),
];
