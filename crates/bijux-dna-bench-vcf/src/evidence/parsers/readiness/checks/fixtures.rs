use super::super::fixtures::validate_parser_fixture_family;
use anyhow::Result;
use bijux_dna_domain_vcf::VcfDomainStage;
use std::path::Path;
pub(super) fn bcftools(repo_root: &Path) -> Result<String> {
    validate_parser_fixture_family(
        repo_root,
        "bcftools",
        &[
            VcfDomainStage::Call,
            VcfDomainStage::CallDiploid,
            VcfDomainStage::CallGl,
            VcfDomainStage::CallPseudohaploid,
            VcfDomainStage::DamageFilter,
            VcfDomainStage::Filter,
            VcfDomainStage::GlPropagation,
            VcfDomainStage::Postprocess,
            VcfDomainStage::PrepareReferencePanel,
            VcfDomainStage::Stats,
        ],
        "validated 10 governed bcftools VCF parser fixture rows",
    )
}

pub(super) fn angsd(repo_root: &Path) -> Result<String> {
    validate_parser_fixture_family(
        repo_root,
        "angsd",
        &[VcfDomainStage::CallGl, VcfDomainStage::CallPseudohaploid],
        "validated 2 governed ANGSD VCF parser fixture rows",
    )
}

pub(super) fn plink(repo_root: &Path) -> Result<String> {
    validate_parser_fixture_family(
        repo_root,
        "plink",
        &[VcfDomainStage::Qc, VcfDomainStage::Admixture],
        "validated governed plink VCF parser fixture rows",
    )?;
    validate_parser_fixture_family(
        repo_root,
        "plink2",
        &[
            VcfDomainStage::Qc,
            VcfDomainStage::Pca,
            VcfDomainStage::Admixture,
            VcfDomainStage::PopulationStructure,
            VcfDomainStage::Roh,
        ],
        "validated governed plink2 VCF parser fixture rows",
    )?;
    Ok("validated 7 governed PLINK-family VCF parser fixture rows".to_string())
}

pub(super) fn eigensoft(repo_root: &Path) -> Result<String> {
    validate_parser_fixture_family(
        repo_root,
        "eigensoft",
        &[VcfDomainStage::Pca, VcfDomainStage::PopulationStructure],
        "validated 2 governed EIGENSOFT VCF parser fixture rows",
    )
}

pub(super) fn phasing(repo_root: &Path) -> Result<String> {
    validate_parser_fixture_family(
        repo_root,
        "shapeit5",
        &[VcfDomainStage::Phasing],
        "validated governed shapeit5 phasing parser fixture rows",
    )?;
    validate_parser_fixture_family(
        repo_root,
        "eagle",
        &[VcfDomainStage::Phasing],
        "validated governed eagle phasing parser fixture rows",
    )?;
    validate_parser_fixture_family(
        repo_root,
        "beagle",
        &[VcfDomainStage::Phasing],
        "validated governed beagle phasing parser fixture rows",
    )?;
    Ok("validated 3 governed VCF phasing parser fixture rows".to_string())
}

pub(super) fn imputation(repo_root: &Path) -> Result<String> {
    for tool_id in ["beagle", "glimpse", "impute5", "minimac4"] {
        validate_parser_fixture_family(
            repo_root,
            tool_id,
            &[VcfDomainStage::Impute, VcfDomainStage::ImputationMetrics],
            "validated governed VCF imputation parser fixture rows",
        )?;
    }
    Ok("validated 8 governed VCF imputation parser fixture rows".to_string())
}

pub(super) fn segments(repo_root: &Path) -> Result<String> {
    validate_parser_fixture_family(
        repo_root,
        "plink2",
        &[VcfDomainStage::Roh],
        "validated governed plink2 ROH parser fixture rows",
    )?;
    validate_parser_fixture_family(
        repo_root,
        "germline",
        &[VcfDomainStage::Ibd],
        "validated governed germline IBD parser fixture rows",
    )?;
    validate_parser_fixture_family(
        repo_root,
        "ibdseq",
        &[VcfDomainStage::Ibd],
        "validated governed ibdseq IBD parser fixture rows",
    )?;
    validate_parser_fixture_family(
        repo_root,
        "ibdhap",
        &[VcfDomainStage::Ibd],
        "validated governed ibdhap IBD parser fixture rows",
    )?;
    validate_parser_fixture_family(
        repo_root,
        "ibdne",
        &[VcfDomainStage::Demography],
        "validated governed ibdne demography parser fixture rows",
    )?;
    Ok("validated 5 governed VCF segment parser fixture rows".to_string())
}
