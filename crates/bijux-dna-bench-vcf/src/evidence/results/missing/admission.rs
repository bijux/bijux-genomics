use super::paths::fixture_result_root;
use crate::inputs::VcfAdapterInputs;
use anyhow::{ensure, Result};
use bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow;
use std::{collections::BTreeSet, fs, path::Path};

pub(super) fn admit_expected_results(
    inputs: &VcfAdapterInputs,
    rows: &[VcfExpectedBenchmarkResultRow],
    repo: &Path,
    root: &Path,
) -> Result<()> {
    let serving = crate::evidence::serving::collect_vcf_tool_serving_map_rows(inputs)?;
    let ready = serving
        .iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(|row| (row.stage_id.as_str(), row.tool_id.as_str()))
        .collect::<BTreeSet<_>>();
    crate::evidence::expectations::ensure_vcf_expected_benchmark_result_contract(
        rows,
        ready.len(),
    )?;
    for row in rows {
        ensure!(
            ready.contains(&(row.stage_id.as_str(), row.tool_id.as_str())),
            "missing-result expectation is not a benchmark-ready binding"
        );
        ensure!(
            inputs.bindings().iter().any(|binding| binding.stage_id == row.stage_id
                && binding.tool_id == row.tool_id
                && binding.corpus_id == row.corpus_id
                && binding.asset_profile_id == row.asset_profile_id),
            "missing-result expectation disagrees with admitted binding identity"
        );
        for part in [&row.corpus_id, &row.stage_id, &row.asset_profile_id, &row.tool_id] {
            ensure!(
                safe_component(part),
                "missing-result fixture identity is not a safe path component"
            );
        }
        let outputs = row.expected_outputs.iter().collect::<BTreeSet<_>>();
        ensure!(
            !outputs.is_empty()
                && outputs.len() == row.expected_outputs.len()
                && outputs.iter().all(|id| safe_component(id)),
            "missing-result outputs must be nonempty, unique safe artifact identities"
        );
        let result = fixture_result_root(root, row);
        for name in ["stdout.log", "stderr.log", "stage-result.json"] {
            admit_target(repo, &result.join(name))?;
        }
        for id in outputs {
            admit_target(repo, &result.join("outputs").join(format!("{id}.json")))?;
        }
    }
    Ok(())
}
fn safe_component(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}
fn admit_target(repo: &Path, path: &Path) -> Result<()> {
    ensure!(path.starts_with(repo), "missing-result fixture target escapes repository");
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) => {
                ensure!(
                    !metadata.file_type().is_symlink(),
                    "missing-result fixture refuses symbolic links: {}",
                    ancestor.display()
                );
                if ancestor == path {
                    ensure!(
                        metadata.is_file(),
                        "missing-result fixture destination must be a regular file"
                    );
                } else {
                    ensure!(
                        metadata.is_dir(),
                        "missing-result fixture ancestry must contain directories"
                    );
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        if ancestor == repo {
            break;
        }
    }
    Ok(())
}
