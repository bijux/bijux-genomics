//! Publication of a prepared report keeps its destination metadata and bytes together.
use anyhow::Result;
use serde::Serialize;
use std::path::Path;

pub(crate) trait AdapterReport: Serialize {
    fn set_output_path(&mut self, path: String);
}

pub(crate) fn publish<T: AdapterReport>(
    repo_root: &Path,
    output_path: &Path,
    mut report: T,
) -> Result<T> {
    let output_path = if output_path.is_absolute() {
        output_path.to_path_buf()
    } else {
        repo_root.join(output_path)
    };
    report.set_output_path(
        output_path
            .strip_prefix(repo_root)
            .unwrap_or(&output_path)
            .to_string_lossy()
            .replace('\\', "/"),
    );
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}

macro_rules! adapter_report {
    ($report:ty) => {
        impl AdapterReport for $report {
            fn set_output_path(&mut self, path: String) {
                self.output_path = path;
            }
        }
    };
}
adapter_report!(super::bcftools::VcfBcftoolsAdapterReport);
adapter_report!(super::angsd::VcfAngsdAdapterReport);
adapter_report!(super::plink::VcfPlinkFamilyAdapterReport);
adapter_report!(super::phasing::VcfPhasingFamilyAdapterReport);
adapter_report!(super::eigensoft::VcfEigensoftAdapterReport);
adapter_report!(super::imputation::VcfImputationFamilyAdapterReport);
adapter_report!(super::descent::VcfDescentFamilyAdapterReport);
