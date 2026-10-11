use super::{FamilyProbeSpec, ProbeKind};
use crate::reports::ParseKind;

pub(super) const VCF: &[FamilyProbeSpec] = &[
    FamilyProbeSpec {
        family_id: "vcf_calling_and_curation",
        representative_stage_id: "vcf.call",
        probe_kind: ProbeKind::GeneratedVcfCall,
        parse_kind: ParseKind::VcfCallMetrics,
        snapshot_keys: &["stage_id", "tool_id", "variant_count", "snp_count", "sample_count"],
    },
    FamilyProbeSpec {
        family_id: "vcf_demographic_history",
        representative_stage_id: "vcf.demography",
        probe_kind: ProbeKind::GeneratedVcfDemography,
        parse_kind: ParseKind::VcfDemographyReport,
        snapshot_keys: &["stage_id", "tool_id", "method", "inference_status", "time_bins"],
    },
    FamilyProbeSpec {
        family_id: "vcf_identity_by_descent",
        representative_stage_id: "vcf.ibd",
        probe_kind: ProbeKind::GeneratedVcfIbd,
        parse_kind: ParseKind::VcfIbdReport,
        snapshot_keys: &["stage_id", "tool_id", "execution_mode", "pair_count", "status"],
    },
    FamilyProbeSpec {
        family_id: "vcf_imputation",
        representative_stage_id: "vcf.impute",
        probe_kind: ProbeKind::GeneratedVcfImpute,
        parse_kind: ParseKind::VcfImputeMetrics,
        snapshot_keys: &[
            "stage_id",
            "tool_id",
            "variant_count",
            "imputed_genotypes",
            "masked_truth_match_count",
        ],
    },
    FamilyProbeSpec {
        family_id: "vcf_ordination_and_population_structure",
        representative_stage_id: "vcf.population_structure",
        probe_kind: ProbeKind::GeneratedVcfPopulationStructure,
        parse_kind: ParseKind::VcfPopulationStructureReport,
        snapshot_keys: &["tool_id", "status", "sample_groups", "distance_summary"],
    },
    FamilyProbeSpec {
        family_id: "vcf_phasing",
        representative_stage_id: "vcf.phasing",
        probe_kind: ProbeKind::GeneratedVcfPhasing,
        parse_kind: ParseKind::VcfPhasingMetrics,
        snapshot_keys: &[
            "stage_id",
            "tool_id",
            "input_genotypes",
            "phased_genotypes",
            "phase_set_count",
        ],
    },
    FamilyProbeSpec {
        family_id: "vcf_quality_control",
        representative_stage_id: "vcf.qc",
        probe_kind: ProbeKind::GeneratedVcfQc,
        parse_kind: ParseKind::VcfQcMetrics,
        snapshot_keys: &[
            "tool_id",
            "sample_missingness",
            "variant_missingness",
            "heterozygosity",
            "hwe_summary",
        ],
    },
];
