use bijux_dna_core::ids::{AssayKind, LibraryLayout, LibraryModel, PlatformHint, UdgTreatment};

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
