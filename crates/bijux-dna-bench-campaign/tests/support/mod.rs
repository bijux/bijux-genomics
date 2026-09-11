#![allow(clippy::expect_used)]
use anyhow::{Context, Result};
use bijux_dna_bench_campaign::{
    ports::{CampaignClock, CampaignCommand, CampaignExecution, CampaignPaths},
    request::{CorpusCampaignContract, CorpusCampaignRequest, StageArgumentDefaults},
};
use bijux_dna_bench_corpus::{
    normalized::load_fastq_corpus_source_spec, source::CorpusAcquisitionLayout,
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

// Two deterministic gzip FASTQ records, generated from @a/@b, ACGT and IIII.
// Synthetic scheduler fixtures do not establish native backend or scientific equivalence.
const READS: [(&str, &[u8], &str); 2] = [
    (
        "a",
        &[
            31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 115, 72, 228, 114, 116, 118, 15, 225, 210, 230, 242,
            4, 2, 46, 0, 107, 44, 42, 113, 15, 0, 0, 0,
        ],
        "6392ae83e6ffed516399ea8ddd2ed5d4757b58a5d3bb45c4b53443378832a1aa",
    ),
    (
        "b",
        &[
            31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 115, 72, 226, 114, 116, 118, 15, 225, 210, 230, 242,
            4, 2, 46, 0, 176, 9, 75, 13, 15, 0, 0, 0,
        ],
        "262d81b3ff3d5dcb57077e42ddc7a1ac587e9ea4a646e70833b69b7fa0ea50d7",
    ),
];

pub struct Fixture {
    pub _directory: tempfile::TempDir,
    pub request: CorpusCampaignRequest,
}
impl Fixture {
    pub fn create() -> Result<Self> {
        let directory = tempfile::tempdir()?;
        let corpus_root = directory.path().join("corpus");
        fs::create_dir_all(corpus_root.join("normalized"))?;
        let mut source = load_fastq_corpus_source_spec(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../benchmarks/corpora/human-short-read-diversity.toml"),
        )?;
        source.target_total = 2;
        source.target_ancient_se = Some(0);
        source.target_ancient_pe = Some(0);
        source.target_modern_se = Some(2);
        source.target_modern_pe = Some(0);
        let template =
            source.samples.first().context("source declaration has no sample template")?.clone();
        source.samples.clear();
        let mut files = BTreeMap::new();
        for (sample, bytes, digest) in READS {
            let accession = format!("accession_{sample}");
            let mut metadata = template.clone();
            metadata.accession.clone_from(&accession);
            metadata.layout = CorpusAcquisitionLayout::Se;
            metadata.era = "modern".into();
            source.samples.push(metadata);
            fs::create_dir_all(corpus_root.join("raw").join(&accession))?;
            for relative in [
                format!("normalized/{sample}_R1.fastq.gz"),
                format!("raw/{accession}/{sample}_R1.fastq.gz"),
            ] {
                fs::write(corpus_root.join(&relative), bytes)?;
                files.insert(relative, digest);
            }
        }
        fs::write(
            corpus_root.join("MANIFEST.json"),
            serde_json::to_vec(&serde_json::json!({"files":files}))?,
        )?;
        let request = CorpusCampaignRequest {
            corpus_id: source.corpus_id.clone(),
            source,
            contract: CorpusCampaignContract {
                stage_id: "fastq.filter_reads".into(),
                scenario_id: "scheduler-fixture".into(),
                sample_scope: "full".into(),
                tools: vec!["fastp".into()],
            },
            repo_root: directory.path().to_path_buf(),
            corpus_root,
            out_root: directory.path().join(".cache/results/run"),
            config_path: directory.path().join("config.toml"),
            publication_config_path: directory.path().join("publication.toml"),
            program: PathBuf::from("/fixture/bin/bijux-dna"),
            platform: "fixture".into(),
            tools: vec![],
            threads: 2,
            jobs: 1,
            sample_jobs: 2,
            sample_limit: 0,
            dry_run: false,
            resume: false,
            stage_args: vec![],
            manifest_args: vec![],
            defaults: StageArgumentDefaults::default(),
        };
        Ok(Self { _directory: directory, request })
    }
}
impl CampaignPaths for Fixture {
    fn stage_out_root(&self, _corpus_id: &str, _stage_id: &str) -> Result<PathBuf> {
        anyhow::bail!("filter execution must not resolve unrelated stage outputs")
    }
    fn apptainer_bind_root(&self) -> Option<PathBuf> {
        None
    }
}
#[derive(Debug, Clone)]
pub struct Call {
    pub program: String,
    pub arguments: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub environment: Option<BTreeMap<String, String>>,
}
#[derive(Default)]
pub struct Recorder {
    pub calls: Mutex<Vec<Call>>,
    pub fail_sample: Option<String>,
    pub transport_error: bool,
}
impl CampaignExecution for Recorder {
    fn execute(&self, command: CampaignCommand<'_>) -> Result<i32> {
        let sample = command
            .arguments
            .windows(2)
            .find(|pair| pair[0] == "--sample-id")
            .map(|pair| pair[1].as_str())
            .context("missing sample identity")?;
        self.calls.lock().map_err(|_| anyhow::anyhow!("recording poisoned"))?.push(Call {
            program: command.program.into(),
            arguments: command.arguments.to_vec(),
            cwd: command.cwd.map(Path::to_path_buf),
            environment: command.environment.cloned(),
        });
        if self.transport_error {
            anyhow::bail!("fixture transport refused")
        }
        Ok(if self.fail_sample.as_deref() == Some(sample) { 7 } else { 0 })
    }
}
pub struct PublicationClock<'a> {
    pub recorder: &'a Recorder,
    pub expected_calls: usize,
}
impl CampaignClock for PublicationClock<'_> {
    fn now(&self) -> SystemTime {
        assert_eq!(
            self.recorder.calls.lock().expect("recording").len(),
            self.expected_calls,
            "publication clock must be sampled after all executions"
        );
        UNIX_EPOCH
    }
}
