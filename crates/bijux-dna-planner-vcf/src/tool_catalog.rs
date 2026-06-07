use std::path::{Path, PathBuf};

use bijux_dna_core::prelude::ContainerImageRefV1;
use bijux_dna_domain_vcf::taxonomy::{CoverageRegime, VcfDomainStage};

pub(crate) fn stage_compat_tools(stage: VcfDomainStage) -> &'static [&'static str] {
    match stage {
        VcfDomainStage::Call => &["bcftools"],
        VcfDomainStage::CallDiploid => &["bcftools"],
        VcfDomainStage::CallGl => &["angsd", "bcftools"],
        VcfDomainStage::CallPseudohaploid => &["angsd", "bcftools"],
        VcfDomainStage::DamageFilter => &["bcftools", "angsd"],
        VcfDomainStage::Filter => &["bcftools"],
        VcfDomainStage::GlPropagation => &["bcftools", "angsd"],
        VcfDomainStage::PrepareReferencePanel => &["bcftools"],
        VcfDomainStage::Phasing => &["beagle", "eagle", "shapeit5"],
        VcfDomainStage::ImputationMetrics | VcfDomainStage::Impute => {
            &["glimpse", "impute5", "minimac4", "beagle"]
        }
        VcfDomainStage::Postprocess => &["bcftools"],
        VcfDomainStage::PopulationStructure | VcfDomainStage::Pca => &["plink2", "eigensoft"],
        VcfDomainStage::Admixture => &["plink2", "plink"],
        VcfDomainStage::Ibd => &["germline", "ibdseq", "ibdhap"],
        VcfDomainStage::Roh => &["plink2"],
        VcfDomainStage::Demography => &["ibdne"],
        VcfDomainStage::Qc => &["plink2", "plink"],
        VcfDomainStage::Stats => &["bcftools"],
    }
}

pub(crate) fn default_tool(stage: VcfDomainStage, coverage: CoverageRegime) -> &'static str {
    match stage {
        VcfDomainStage::CallGl => "angsd",
        VcfDomainStage::CallPseudohaploid => "angsd",
        VcfDomainStage::Phasing => match coverage {
            CoverageRegime::Diploid => "shapeit5",
            CoverageRegime::LowCovGl => "beagle",
            CoverageRegime::Pseudohaploid => "beagle",
        },
        VcfDomainStage::ImputationMetrics | VcfDomainStage::Impute => match coverage {
            CoverageRegime::Diploid => "minimac4",
            CoverageRegime::LowCovGl => "glimpse",
            CoverageRegime::Pseudohaploid => "beagle",
        },
        VcfDomainStage::PopulationStructure | VcfDomainStage::Pca | VcfDomainStage::Roh => "plink2",
        VcfDomainStage::Ibd => "germline",
        VcfDomainStage::Demography => "ibdne",
        _ => "bcftools",
    }
}

pub(crate) fn image_for_tool(tool: &str) -> ContainerImageRefV1 {
    let image = match tool {
        "angsd" => "quay.io/biocontainers/angsd:0.940--h2e03b76_2",
        "shapeit5" => "quay.io/biocontainers/shapeit5:5.1.1--h9948957_0",
        "eagle" => "quay.io/biocontainers/eagle:2.4.1--h8b12597_2",
        "beagle" => "quay.io/biocontainers/beagle:5.4--hdfd78af_0",
        "glimpse" => "quay.io/biocontainers/glimpse:2.0.0--h9ee0642_0",
        "impute5" => "quay.io/biocontainers/impute5:1.2.0--h43eeafb_4",
        "minimac4" => "quay.io/biocontainers/minimac4:4.1.6--h7d875b9_4",
        "plink" => "quay.io/biocontainers/plink:1.90b6.21--h0a44026_2",
        "plink2" => "quay.io/biocontainers/plink2:2.00a3.7--h5ef6573_0",
        "eigensoft" => "quay.io/biocontainers/eigensoft:7.2.1--h9ee0642_4",
        "germline" => "quay.io/biocontainers/germline:1.5.3--hdfd78af_0",
        "ibdseq" => "quay.io/bijux/ibdseq:3.0-planned",
        "ibdhap" => "quay.io/biocontainers/ibdhap:1.0.0--h9ee0642_0",
        "ibdne" => "quay.io/biocontainers/ibdne:23.05.23.ae9f5b3--hdfd78af_0",
        _ => "quay.io/biocontainers/bcftools:1.20--h8b25389_0",
    };
    ContainerImageRefV1 { image: image.to_string(), digest: None }
}

pub(crate) fn phasing_backend_supports_gl_only_input(tool: &str) -> bool {
    tool == "beagle"
}

pub(crate) fn eagle_license_metadata_present() -> bool {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("containers/licenses/eagle.license.toml")
        .exists()
}
