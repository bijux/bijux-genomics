use serde::Serialize;

use crate::taxonomy::VcfDomainStage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfParserFixtureInventoryRow {
    pub tool_id: &'static str,
    pub stage: VcfDomainStage,
    pub parser_id: &'static str,
    pub fixture_path: &'static str,
}

const VCF_PARSER_FIXTURE_INVENTORY: &[VcfParserFixtureInventoryRow] = &[
    VcfParserFixtureInventoryRow {
        tool_id: "bcftools",
        stage: VcfDomainStage::Call,
        parser_id: "parse_bcftools_call_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/bcftools/vcf.call",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "bcftools",
        stage: VcfDomainStage::CallDiploid,
        parser_id: "parse_bcftools_call_diploid_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/bcftools/vcf.call_diploid",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "bcftools",
        stage: VcfDomainStage::CallGl,
        parser_id: "parse_bcftools_call_gl_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/bcftools/vcf.call_gl",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "bcftools",
        stage: VcfDomainStage::CallPseudohaploid,
        parser_id: "parse_bcftools_call_pseudohaploid_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/bcftools/vcf.call_pseudohaploid",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "bcftools",
        stage: VcfDomainStage::DamageFilter,
        parser_id: "parse_bcftools_damage_filter_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/bcftools/vcf.damage_filter",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "bcftools",
        stage: VcfDomainStage::Filter,
        parser_id: "parse_bcftools_filter_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/bcftools/vcf.filter",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "bcftools",
        stage: VcfDomainStage::GlPropagation,
        parser_id: "parse_bcftools_gl_propagation_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/bcftools/vcf.gl_propagation",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "bcftools",
        stage: VcfDomainStage::Postprocess,
        parser_id: "parse_bcftools_postprocess_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/bcftools/vcf.postprocess",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "bcftools",
        stage: VcfDomainStage::PrepareReferencePanel,
        parser_id: "parse_bcftools_prepare_reference_panel_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/bcftools/vcf.prepare_reference_panel",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "bcftools",
        stage: VcfDomainStage::Stats,
        parser_id: "parse_bcftools_stats_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/bcftools/vcf.stats",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "angsd",
        stage: VcfDomainStage::CallGl,
        parser_id: "parse_angsd_call_gl_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/angsd/vcf.call_gl",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "angsd",
        stage: VcfDomainStage::CallPseudohaploid,
        parser_id: "parse_angsd_call_pseudohaploid_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/angsd/vcf.call_pseudohaploid",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "angsd",
        stage: VcfDomainStage::DamageFilter,
        parser_id: "parse_angsd_damage_filter_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/angsd/vcf.damage_filter",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "angsd",
        stage: VcfDomainStage::GlPropagation,
        parser_id: "parse_angsd_gl_propagation_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/angsd/vcf.gl_propagation",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "plink",
        stage: VcfDomainStage::Qc,
        parser_id: "parse_plink_qc_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/plink/vcf.qc",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "plink",
        stage: VcfDomainStage::Admixture,
        parser_id: "parse_plink_admixture_prep_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/plink/vcf.admixture",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "plink2",
        stage: VcfDomainStage::Qc,
        parser_id: "parse_plink2_qc_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/plink2/vcf.qc",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "plink2",
        stage: VcfDomainStage::Pca,
        parser_id: "parse_plink2_pca_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/plink2/vcf.pca",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "plink2",
        stage: VcfDomainStage::Admixture,
        parser_id: "parse_plink2_admixture_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/plink2/vcf.admixture",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "plink2",
        stage: VcfDomainStage::PopulationStructure,
        parser_id: "parse_plink2_population_structure_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/plink2/vcf.population_structure",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "eigensoft",
        stage: VcfDomainStage::Pca,
        parser_id: "parse_eigensoft_pca_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/eigensoft/pca",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "eigensoft",
        stage: VcfDomainStage::PopulationStructure,
        parser_id: "parse_eigensoft_population_structure_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/eigensoft/population_structure",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "shapeit5",
        stage: VcfDomainStage::Phasing,
        parser_id: "parse_shapeit5_phasing_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/phasing/shapeit5",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "eagle",
        stage: VcfDomainStage::Phasing,
        parser_id: "parse_eagle_phasing_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/phasing/eagle",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "beagle",
        stage: VcfDomainStage::Phasing,
        parser_id: "parse_beagle_phasing_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/phasing/beagle",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "beagle",
        stage: VcfDomainStage::Impute,
        parser_id: "parse_beagle_impute_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/imputation/beagle/vcf.impute",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "beagle",
        stage: VcfDomainStage::Imputation,
        parser_id: "parse_beagle_imputation_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/imputation/beagle/vcf.imputation",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "glimpse",
        stage: VcfDomainStage::Impute,
        parser_id: "parse_glimpse_impute_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/imputation/glimpse/vcf.impute",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "glimpse",
        stage: VcfDomainStage::Imputation,
        parser_id: "parse_glimpse_imputation_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/imputation/glimpse/vcf.imputation",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "impute5",
        stage: VcfDomainStage::Impute,
        parser_id: "parse_impute5_impute_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/imputation/impute5/vcf.impute",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "impute5",
        stage: VcfDomainStage::Imputation,
        parser_id: "parse_impute5_imputation_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/imputation/impute5/vcf.imputation",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "minimac4",
        stage: VcfDomainStage::Impute,
        parser_id: "parse_minimac4_impute_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/imputation/minimac4/vcf.impute",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "minimac4",
        stage: VcfDomainStage::Imputation,
        parser_id: "parse_minimac4_imputation_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/imputation/minimac4/vcf.imputation",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "plink2",
        stage: VcfDomainStage::Roh,
        parser_id: "parse_plink2_roh_segment_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/segments/plink2/vcf.roh/complete",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "germline",
        stage: VcfDomainStage::Ibd,
        parser_id: "parse_germline_ibd_segment_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/segments/germline/vcf.ibd/complete",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "ibdseq",
        stage: VcfDomainStage::Ibd,
        parser_id: "parse_ibdseq_ibd_segment_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/segments/ibdseq/vcf.ibd/complete",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "ibdhap",
        stage: VcfDomainStage::Ibd,
        parser_id: "parse_ibdhap_ibd_segment_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/segments/ibdhap/vcf.ibd/complete",
    },
    VcfParserFixtureInventoryRow {
        tool_id: "ibdne",
        stage: VcfDomainStage::Demography,
        parser_id: "parse_ibdne_demography_metrics",
        fixture_path: "tests/fixtures/bench/parsers/vcf/segments/ibdne/vcf.demography/complete",
    },
];

#[must_use]
pub fn vcf_parser_fixture_inventory() -> &'static [VcfParserFixtureInventoryRow] {
    VCF_PARSER_FIXTURE_INVENTORY
}

#[must_use]
pub fn find_vcf_parser_fixture_inventory_row(
    tool_id: &str,
    stage: VcfDomainStage,
) -> Option<&'static VcfParserFixtureInventoryRow> {
    VCF_PARSER_FIXTURE_INVENTORY.iter().find(|row| row.tool_id == tool_id && row.stage == stage)
}
