//! Request-local adapter observations. Acquisition and publication are distinct operations.
use super::angsd::VcfAngsdAdapterReport;
use super::bcftools::VcfBcftoolsAdapterReport;
use super::descent::VcfDescentFamilyAdapterReport;
use super::eigensoft::VcfEigensoftAdapterReport;
use super::imputation::VcfImputationFamilyAdapterReport;
use super::phasing::VcfPhasingFamilyAdapterReport;
use super::plink::VcfPlinkFamilyAdapterReport;
use crate::inputs::VcfAdapterInputs;
use anyhow::{anyhow, Result};
use std::cell::OnceCell;
use std::path::Path;

/// Reuse validated adapter observations within one request and admitted input snapshot.
/// Creating this value performs no effects. The first requested family/tool may
/// materialize fixture inputs; publishing its prepared report does not repeat acquisition.
/// A report's output path is its planned destination until a publish operation succeeds.
#[derive(Debug)]
pub struct VcfAdapterObservations<'a> {
    inputs: &'a VcfAdapterInputs,
    repo_root: &'a Path,
    bcftools: OnceCell<VcfBcftoolsAdapterReport>,
    angsd: OnceCell<VcfAngsdAdapterReport>,
    eigensoft: OnceCell<VcfEigensoftAdapterReport>,
    imputation: OnceCell<VcfImputationFamilyAdapterReport>,
    descent: OnceCell<VcfDescentFamilyAdapterReport>,
    plink: OnceCell<VcfPlinkFamilyAdapterReport>,
    plink2: OnceCell<VcfPlinkFamilyAdapterReport>,
    shapeit: OnceCell<VcfPhasingFamilyAdapterReport>,
    shapeit5: OnceCell<VcfPhasingFamilyAdapterReport>,
    eagle: OnceCell<VcfPhasingFamilyAdapterReport>,
    beagle: OnceCell<VcfPhasingFamilyAdapterReport>,
}

impl<'a> VcfAdapterObservations<'a> {
    /// Bind a request to immutable admitted declarations and its repository context.
    #[must_use]
    pub fn new(inputs: &'a VcfAdapterInputs, repo_root: &'a Path) -> Self {
        Self {
            inputs,
            repo_root,
            bcftools: OnceCell::new(),
            angsd: OnceCell::new(),
            eigensoft: OnceCell::new(),
            imputation: OnceCell::new(),
            descent: OnceCell::new(),
            plink: OnceCell::new(),
            plink2: OnceCell::new(),
            shapeit: OnceCell::new(),
            shapeit5: OnceCell::new(),
            eagle: OnceCell::new(),
            beagle: OnceCell::new(),
        }
    }

    /// Borrow the exact admitted declarations associated with these observations.
    #[must_use]
    pub fn inputs(&self) -> &VcfAdapterInputs {
        self.inputs
    }

    /// Borrow the repository context associated with these observations.
    #[must_use]
    pub fn repo_root(&self) -> &Path {
        self.repo_root
    }

    /// Borrow a prepared report, acquiring this family only on its first request.
    ///
    /// # Errors
    /// Returns family contract, registry or fixture acquisition errors; failures are not cached.
    pub fn bcftools(&self) -> Result<&VcfBcftoolsAdapterReport> {
        prepared(&self.bcftools, || {
            super::bcftools::service::prepare_vcf_bcftools_adapter(
                self.inputs,
                self.repo_root,
                Path::new(super::bcftools::DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH),
            )
        })
    }

    /// Publish the acquired report at the requested destination without reacquiring inputs.
    ///
    /// # Errors
    /// Returns acquisition errors on first demand or an atomic publication error.
    pub fn publish_bcftools(&self, output_path: &Path) -> Result<VcfBcftoolsAdapterReport> {
        super::report::publish(self.repo_root, output_path, self.bcftools()?.clone())
    }

    /// Borrow a prepared report, acquiring this family only on its first request.
    ///
    /// # Errors
    /// Returns family contract, registry or fixture acquisition errors; failures are not cached.
    pub fn angsd(&self) -> Result<&VcfAngsdAdapterReport> {
        prepared(&self.angsd, || {
            super::angsd::service::prepare_vcf_angsd_adapter(
                self.inputs,
                self.repo_root,
                Path::new(super::angsd::DEFAULT_VCF_ANGSD_ADAPTER_PATH),
            )
        })
    }

    /// Publish the acquired report at the requested destination without reacquiring inputs.
    ///
    /// # Errors
    /// Returns acquisition errors on first demand or an atomic publication error.
    pub fn publish_angsd(&self, output_path: &Path) -> Result<VcfAngsdAdapterReport> {
        super::report::publish(self.repo_root, output_path, self.angsd()?.clone())
    }

    /// Borrow a prepared report, acquiring this family only on its first request.
    ///
    /// # Errors
    /// Returns family contract, registry or fixture acquisition errors; failures are not cached.
    pub fn eigensoft(&self) -> Result<&VcfEigensoftAdapterReport> {
        prepared(&self.eigensoft, || {
            super::eigensoft::service::prepare_vcf_eigensoft_adapter(
                self.inputs,
                self.repo_root,
                Path::new(super::eigensoft::DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH),
            )
        })
    }

    /// Publish the acquired report at the requested destination without reacquiring inputs.
    ///
    /// # Errors
    /// Returns acquisition errors on first demand or an atomic publication error.
    pub fn publish_eigensoft(&self, output_path: &Path) -> Result<VcfEigensoftAdapterReport> {
        super::report::publish(self.repo_root, output_path, self.eigensoft()?.clone())
    }

    /// Borrow a prepared report, acquiring this family only on its first request.
    ///
    /// # Errors
    /// Returns family contract, registry or fixture acquisition errors; failures are not cached.
    pub fn imputation(&self) -> Result<&VcfImputationFamilyAdapterReport> {
        prepared(&self.imputation, || {
            super::imputation::service::prepare_vcf_imputation_family_adapter(
                self.inputs,
                self.repo_root,
                Path::new(super::imputation::DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH),
            )
        })
    }

    /// Publish the acquired report at the requested destination without reacquiring inputs.
    ///
    /// # Errors
    /// Returns acquisition errors on first demand or an atomic publication error.
    pub fn publish_imputation(
        &self,
        output_path: &Path,
    ) -> Result<VcfImputationFamilyAdapterReport> {
        super::report::publish(self.repo_root, output_path, self.imputation()?.clone())
    }

    /// Borrow a prepared report, acquiring this family only on its first request.
    ///
    /// # Errors
    /// Returns family contract, registry or fixture acquisition errors; failures are not cached.
    pub fn descent(&self) -> Result<&VcfDescentFamilyAdapterReport> {
        prepared(&self.descent, || {
            super::descent::service::prepare_vcf_descent_family_adapter(
                self.inputs,
                self.repo_root,
                Path::new(super::descent::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH),
            )
        })
    }

    /// Publish the acquired report at the requested destination without reacquiring inputs.
    ///
    /// # Errors
    /// Returns acquisition errors on first demand or an atomic publication error.
    pub fn publish_descent(&self, output_path: &Path) -> Result<VcfDescentFamilyAdapterReport> {
        super::report::publish(self.repo_root, output_path, self.descent()?.clone())
    }

    /// Borrow a prepared report for a supported tool in this family.
    ///
    /// # Errors
    /// Rejects an unsupported tool or propagates its contract/fixture acquisition error.
    pub fn plink(&self, tool_id: &str) -> Result<&VcfPlinkFamilyAdapterReport> {
        let (slot, path) = match tool_id {
            "plink" => (&self.plink, super::plink::DEFAULT_VCF_PLINK_ADAPTER_PATH),
            "plink2" => (&self.plink2, super::plink::DEFAULT_VCF_PLINK2_ADAPTER_PATH),
            _ => return Err(anyhow!("unsupported VCF plink observation tool `{tool_id}`")),
        };
        prepared(slot, || {
            super::plink::service::prepare_vcf_plink_family_adapter(
                self.inputs,
                self.repo_root,
                tool_id,
                Path::new(path),
            )
        })
    }

    /// Publish a cached tool report with destination metadata matching the written file.
    ///
    /// # Errors
    /// Returns tool admission/acquisition or atomic publication errors.
    pub fn publish_plink(
        &self,
        tool_id: &str,
        output_path: &Path,
    ) -> Result<VcfPlinkFamilyAdapterReport> {
        super::report::publish(self.repo_root, output_path, self.plink(tool_id)?.clone())
    }

    /// Borrow a prepared report for a supported tool in this family.
    ///
    /// # Errors
    /// Rejects an unsupported tool or propagates its contract/fixture acquisition error.
    pub fn phasing(&self, tool_id: &str) -> Result<&VcfPhasingFamilyAdapterReport> {
        let (slot, path) = match tool_id {
            "shapeit" => (&self.shapeit, super::phasing::DEFAULT_VCF_SHAPEIT_ADAPTER_PATH),
            "shapeit5" => (&self.shapeit5, super::phasing::DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH),
            "eagle" => (&self.eagle, super::phasing::DEFAULT_VCF_EAGLE_ADAPTER_PATH),
            "beagle" => (&self.beagle, super::phasing::DEFAULT_VCF_BEAGLE_ADAPTER_PATH),
            _ => return Err(anyhow!("unsupported VCF phasing observation tool `{tool_id}`")),
        };
        prepared(slot, || {
            super::phasing::service::prepare_vcf_phasing_family_adapter(
                self.inputs,
                self.repo_root,
                tool_id,
                Path::new(path),
            )
        })
    }

    /// Publish a cached tool report with destination metadata matching the written file.
    ///
    /// # Errors
    /// Returns tool admission/acquisition or atomic publication errors.
    pub fn publish_phasing(
        &self,
        tool_id: &str,
        output_path: &Path,
    ) -> Result<VcfPhasingFamilyAdapterReport> {
        super::report::publish(self.repo_root, output_path, self.phasing(tool_id)?.clone())
    }
}

fn prepared<T>(slot: &OnceCell<T>, load: impl FnOnce() -> Result<T>) -> Result<&T> {
    if let Some(value) = slot.get() {
        return Ok(value);
    }
    let value = load()?;
    slot.set(value).map_err(|_| anyhow!("recursive adapter report acquisition"))?;
    slot.get().ok_or_else(|| anyhow!("adapter observation was not retained"))
}
