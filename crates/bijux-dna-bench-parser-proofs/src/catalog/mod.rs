use crate::reports::ParseKind;
mod families;
pub use families::family_probe_specs;

#[derive(Debug, Clone, Copy)]
pub struct FamilyProbeSpec {
    pub family_id: &'static str,
    pub representative_stage_id: &'static str,
    pub probe_kind: ProbeKind,
    pub parse_kind: ParseKind,
    pub snapshot_keys: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub enum ProbeKind {
    LocalStage { stage_id: &'static str, proof_pointer: Option<&'static str> },
    GeneratedIndexReference,
    GeneratedCorrectErrors,
    GeneratedDepleteHost,
    GeneratedDepleteRrna,
    GeneratedScreenTaxonomy,
    GeneratedBamAlign,
    GeneratedBamContamination,
    GeneratedBamDamage,
    GeneratedBamAuthenticity,
    GeneratedBamGenotyping,
    GeneratedBamKinship,
    GeneratedBamSex,
    GeneratedVcfCall,
    GeneratedVcfDemography,
    GeneratedVcfIbd,
    GeneratedVcfImpute,
    GeneratedVcfPhasing,
    GeneratedVcfPopulationStructure,
    GeneratedVcfQc,
}

mod bam;
mod fastq;
mod vcf;
