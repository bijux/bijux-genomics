//! Owner: report output confinement and publication.
use anyhow::{ensure, Context, Result};
use bijux_dna_bench_model::{MicroBenchmarkReport, MicroBenchmarkReportDestinations};
use std::path::{Component, Path, PathBuf};
pub const DEFAULT_MICRO_BENCHMARK_REPORT_MARKDOWN_PATH: &str =
    "artifacts/benchmark-smoke/micro/MICRO_BENCHMARK_REPORT.md";
pub const DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH: &str =
    "artifacts/benchmark-smoke/micro/MICRO_BENCHMARK_REPORT.json";

pub(crate) struct Destinations {
    pub(crate) markdown: PathBuf,
    pub(crate) json: PathBuf,
    pub(crate) governed: bool,
}
impl Destinations {
    pub(crate) fn prepare(root: &Path, markdown: &Path, json: &Path) -> Result<Self> {
        let markdown = admit(root, markdown, "micro benchmark markdown report")?;
        let json = admit(root, json, "micro benchmark json report")?;
        ensure!(markdown != json, "micro benchmark report destinations must be distinct");
        let governed = display(root, &markdown) == DEFAULT_MICRO_BENCHMARK_REPORT_MARKDOWN_PATH
            && display(root, &json) == DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH;
        Ok(Self { markdown, json, governed })
    }
    pub(crate) fn display(&self, root: &Path) -> MicroBenchmarkReportDestinations {
        MicroBenchmarkReportDestinations {
            markdown: display(root, &self.markdown),
            json: display(root, &self.json),
        }
    }
    pub(crate) fn publish(&self, report: &MicroBenchmarkReport) -> Result<()> {
        for path in [&self.markdown, &self.json] {
            if let Some(parent) = path.parent() {
                bijux_dna_infra::ensure_dir(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
        }
        bijux_dna_infra::write_string(
            &self.markdown,
            &crate::artifacts::micro_report::render_micro_benchmark_report_markdown(report),
        )
        .with_context(|| format!("write {}", self.markdown.display()))?;
        bijux_dna_infra::atomic_write_json(&self.json, report)?;
        Ok(())
    }
}
fn admit(root: &Path, path: &Path, kind: &str) -> Result<PathBuf> {
    let path = if path.is_absolute() { path.to_path_buf() } else { root.join(path) };
    let relative = path
        .strip_prefix(root.join("artifacts/benchmark-smoke/micro"))
        .with_context(|| format!("{kind} must resolve inside `artifacts/benchmark-smoke/micro`"))?;
    ensure!(
        !relative.as_os_str().is_empty()
            && relative.components().all(|c| matches!(c, Component::Normal(_))),
        "{kind} must be a normalized file path inside `artifacts/benchmark-smoke/micro`"
    );
    Ok(path)
}
fn display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn report_destinations_reject_escape_aliases_and_directory_roots_before_writing() -> Result<()>
    {
        let temp = tempfile::tempdir()?;
        let root = temp.path();
        let good = Path::new(DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH);
        for path in [
            "report.md",
            "artifacts/benchmark-smoke/micro/../escaped.md",
            "artifacts/benchmark-smoke/micro",
            DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH,
        ] {
            assert!(Destinations::prepare(root, Path::new(path), good).is_err(), "{path}");
        }
        let paths = Destinations::prepare(
            root,
            Path::new(DEFAULT_MICRO_BENCHMARK_REPORT_MARKDOWN_PATH),
            good,
        )?;
        assert!(paths.governed);
        assert!(!root.join("artifacts").exists());
        Ok(())
    }
}
