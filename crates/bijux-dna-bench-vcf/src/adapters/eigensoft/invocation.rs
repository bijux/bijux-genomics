//! Owner: bijux-dna-bench-vcf
//! EIGENSOFT parameter-file and PCA command steps.
use super::commands::step;
use super::{VcfEigensoftAdapterCommandStep, GOVERNED_COHORT_VCF_PATH};

pub(super) struct CommandFiles<'a> {
    pub(super) convertf_par: &'a str,
    pub(super) eigenval: &'a str,
    pub(super) eigenvec: &'a str,
    pub(super) geno: &'a str,
    pub(super) ind: &'a str,
    pub(super) smartpca_log: &'a str,
    pub(super) smartpca_par: &'a str,
    pub(super) snp: &'a str,
}

impl CommandFiles<'_> {
    pub(super) fn steps(&self) -> Vec<VcfEigensoftAdapterCommandStep> {
        let Self {
            convertf_par: convertf_par_path,
            eigenval: eigenval_path,
            eigenvec: eigenvec_path,
            geno: geno_path,
            ind: ind_path,
            smartpca_log: smartpca_log_path,
            smartpca_par: smartpca_par_path,
            snp: snp_path,
        } = self;
        vec![
            step(
                "write_convertf_par",
                "config_render",
                vec![
                    "sh",
                    "-lc",
                    &format!(
                        "cat > '{convertf_par_path}' <<'EOF'\n\
genotypename: {GOVERNED_COHORT_VCF_PATH}\n\
snpname: {snp_path}\n\
indivname: {ind_path}\n\
outputformat: EIGENSTRAT\n\
genotypeoutname: {geno_path}\n\
snpoutname: {snp_path}\n\
indivoutname: {ind_path}\n\
familynames: NO\n\
EOF"
                    ),
                ],
                &["convertf_par"],
            ),
            step(
                "convertf",
                "format_conversion",
                vec!["convertf", "-p", convertf_par_path],
                &["eigensoft_geno", "eigensoft_snp", "eigensoft_ind"],
            ),
            step(
                "write_smartpca_par",
                "config_render",
                vec![
                    "sh",
                    "-lc",
                    &format!(
                        "cat > '{smartpca_par_path}' <<'EOF'\n\
genotypename: {geno_path}\n\
snpname: {snp_path}\n\
indivname: {ind_path}\n\
evecoutname: {eigenvec_path}\n\
evaloutname: {eigenval_path}\n\
numoutevec: 10\n\
familynames: NO\n\
EOF"
                    ),
                ],
                &["smartpca_par"],
            ),
            step(
                "smartpca",
                "principal_components",
                vec![
                    "sh",
                    "-lc",
                    &format!("smartpca -p '{smartpca_par_path}' > '{smartpca_log_path}' 2>&1"),
                ],
                &["smartpca_eigenvec", "smartpca_eigenval", "smartpca_log"],
            ),
        ]
    }
}
