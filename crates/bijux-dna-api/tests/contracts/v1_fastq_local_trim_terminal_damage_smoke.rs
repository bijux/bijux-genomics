use anyhow::Result;
use flate2::read::MultiGzDecoder;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

struct RepoRootOverrideGuard {
    previous: Option<std::ffi::OsString>,
}

impl RepoRootOverrideGuard {
    fn install(root: &Path) -> Self {
        let previous = std::env::var_os("BIJUX_REPO_ROOT");
        std::env::set_var("BIJUX_REPO_ROOT", root);
        Self { previous }
    }
}

impl Drop for RepoRootOverrideGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.take() {
            std::env::set_var("BIJUX_REPO_ROOT", previous);
        } else {
            std::env::remove_var("BIJUX_REPO_ROOT");
        }
    }
}

fn repo_root() -> Result<PathBuf> {
    crate::support::repo_root()
}

fn read_gz_fastq_sequences(path: &Path) -> Result<Vec<String>> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(MultiGzDecoder::new(file));
    let mut lines = reader.lines();
    let mut sequences = Vec::new();
    while lines.next().transpose()?.is_some() {
        let sequence = lines
            .next()
            .transpose()?
            .unwrap_or_else(|| panic!("sequence line missing in {}", path.display()));
        let _plus = lines
            .next()
            .transpose()?
            .unwrap_or_else(|| panic!("plus line missing in {}", path.display()));
        let _quality = lines
            .next()
            .transpose()?
            .unwrap_or_else(|| panic!("quality line missing in {}", path.display()));
        sequences.push(sequence);
    }
    Ok(sequences)
}

#[test]
fn write_local_trim_terminal_damage_smoke_report_materializes_governed_outputs() -> Result<()> {
    let repo_root = repo_root()?;
    let _guard = RepoRootOverrideGuard::install(&repo_root);
    let output_dir = repo_root.join("target/local-smoke/fastq.trim_terminal_damage");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
    }

    let metrics_path =
        bijux_dna_api::v1::api::fastq::write_local_trim_terminal_damage_smoke_report()?;
    assert_eq!(
        metrics_path,
        repo_root.join("target/local-smoke/fastq.trim_terminal_damage/metrics.json")
    );
    assert!(metrics_path.is_file(), "local terminal-damage metrics must exist");

    let payload: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&metrics_path)?)?;
    assert_eq!(payload["stage_id"], serde_json::json!("fastq.trim_terminal_damage"));
    assert_eq!(payload["sample_id"], serde_json::json!("ancient-se"));
    assert_eq!(payload["tool_id"], serde_json::json!("cutadapt"));
    assert_eq!(payload["damage_mode"], serde_json::json!("ancient"));
    assert_eq!(payload["execution_policy"], serde_json::json!("explicit_terminal_trim"));
    assert_eq!(payload["input_reads"], serde_json::json!(2));
    assert_eq!(payload["output_reads"], serde_json::json!(2));
    assert_eq!(payload["trim_5p_bases"], serde_json::json!(2));
    assert_eq!(payload["trim_3p_bases"], serde_json::json!(1));
    assert_eq!(payload["input_bases"], serde_json::json!(58));
    assert_eq!(payload["output_bases"], serde_json::json!(52));
    assert_eq!(payload["bases_removed"], serde_json::json!(6));
    assert_eq!(payload["used_fallback"], serde_json::json!(true));

    let trimmed_fastq = repo_root.join(
        payload["trimmed_fastq_gz"].as_str().unwrap_or_else(|| panic!("trimmed_fastq_gz missing")),
    );
    assert!(trimmed_fastq.is_file(), "top-level trimmed FASTQ must exist");
    let sequences = read_gz_fastq_sequences(&trimmed_fastq)?;
    assert_eq!(
        sequences,
        vec!["GTAGATCGGAAGAGCTT".to_string(), "CAGTGACTGGAGTTCAGACGTGTGCTCTTCCGATC".to_string(),]
    );

    let report_json = repo_root
        .join(payload["report_json"].as_str().unwrap_or_else(|| panic!("report_json missing")));
    assert!(report_json.is_file(), "terminal-damage report must exist");
    let report: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report_json)?)?;
    assert_eq!(report["stage_id"], serde_json::json!("fastq.trim_terminal_damage"));
    assert_eq!(report["tool_id"], serde_json::json!("cutadapt"));
    assert_eq!(report["reads_in"], serde_json::json!(2));
    assert_eq!(report["reads_out"], serde_json::json!(2));
    assert_eq!(report["bases_in"], serde_json::json!(58));
    assert_eq!(report["bases_out"], serde_json::json!(52));
    assert_eq!(report["trim_5p_bases"], serde_json::json!(2));
    assert_eq!(report["trim_3p_bases"], serde_json::json!(1));
    assert_eq!(report["used_fallback"], serde_json::json!(true));

    Ok(())
}
