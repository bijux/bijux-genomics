use bijux_dna_core::ids::StageId;

use super::BoundaryInvariant;

pub const STAGE_BOUNDARY_INVARIANTS: [BoundaryInvariant; 6] = [
    BoundaryInvariant {
        from: StageId::from_static("fastq.validate_reads"),
        to: StageId::from_static("fastq.detect_adapters"),
        rule: "validation does not modify reads; adapter detection consumes validated reads",
    },
    BoundaryInvariant {
        from: StageId::from_static("fastq.detect_adapters"),
        to: StageId::from_static("fastq.trim_terminal_damage"),
        rule: "damage-aware pretrim consumes unchanged reads from report-only adapter detection",
    },
    BoundaryInvariant {
        from: StageId::from_static("fastq.trim_terminal_damage"),
        to: StageId::from_static("fastq.trim_reads"),
        rule: "damage-aware pretrim output remains FASTQ and preserves pairing semantics",
    },
    BoundaryInvariant {
        from: StageId::from_static("fastq.trim_reads"),
        to: StageId::from_static("fastq.filter_reads"),
        rule: "trim output must remain FASTQ and preserve pairing",
    },
    BoundaryInvariant {
        from: StageId::from_static("fastq.filter_reads"),
        to: StageId::from_static("fastq.profile_reads"),
        rule: "filter output remains FASTQ; stats is report-only",
    },
    BoundaryInvariant {
        from: StageId::from_static("fastq.merge_pairs"),
        to: StageId::from_static("fastq.profile_reads"),
        rule: "merge produces merged reads; stats accepts merged FASTQ",
    },
];
