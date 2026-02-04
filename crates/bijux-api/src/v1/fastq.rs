//! FASTQ domain helpers for v1.

pub use bijux_domain_fastq::banks as fastq_banks;
pub use bijux_domain_fastq::{
    adapter_bank_path, adapter_presets_path, benchmark_runs, contaminant_motifs_path,
    contaminant_presets_path, contaminant_references_dir, load_adapter_bank, load_adapter_presets,
    load_contaminant_motifs, load_contaminant_presets, load_polyx_bank, load_polyx_presets,
    polyx_bank_path, polyx_presets_path, qc_class_for_stage, write_benchmark_exports,
    AdapterPresetsV1, BenchCorpusId, EffectiveAdapterSet, QcClass, ReadScope, STAGES,
};

pub use bijux_stages_fastq::args as fastq_args;
