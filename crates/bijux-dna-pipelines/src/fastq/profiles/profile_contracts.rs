use bijux_dna_core::ids::{AssayKind, LibraryLayout, LibraryModel, PlatformHint, UdgTreatment};

use crate::{ArtifactType, Domain, MetricsBundle, PipelineCapabilities, ReportSection};

pub(super) fn fastq_library_model(
    layout: LibraryLayout,
    udg_treatment: UdgTreatment,
    assay_kind: AssayKind,
) -> LibraryModel {
    LibraryModel {
        layout,
        udg_treatment,
        platform_hint: PlatformHint::Illumina,
        assay_kind,
    }
}

pub(super) fn fastq_capabilities(required_stages: Vec<String>) -> PipelineCapabilities {
    PipelineCapabilities {
        input_domains: vec![Domain::Fastq],
        output_domains: vec![Domain::Fastq],
        input_artifacts: vec![ArtifactType::FastqReads],
        output_artifacts: vec![ArtifactType::FastqReads, ArtifactType::MetricsBundle],
        required_inputs: vec!["fastq"],
        produces_outputs: vec!["fastq", "fastq.metrics"],
        report_sections: vec!["fastq"],
        required_report_sections: vec![ReportSection::Fastq, ReportSection::PipelineDefaults],
        required_metrics_bundles: vec![MetricsBundle::FastqCore],
        required_stages,
        required_metrics: vec!["fastq.metrics"],
        required_artifacts: vec![
            "report.json",
            "run_manifest.json",
            "stage_summaries.json",
            "invariants_report.json",
        ],
        supports_benchmarks: true,
    }
}
