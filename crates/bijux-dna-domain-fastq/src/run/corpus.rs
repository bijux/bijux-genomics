use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchCorpusId {
    Fastq5Set,
}

impl BenchCorpusId {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            BenchCorpusId::Fastq5Set => "fastq_5set",
        }
    }
}

impl std::str::FromStr for BenchCorpusId {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "fastq_5set" => Ok(BenchCorpusId::Fastq5Set),
            _ => Err(format!("unknown bench corpus: {value}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchDataset {
    pub id: &'static str,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub sha256_r1: &'static str,
    pub sha256_r2: Option<&'static str>,
    pub paired: bool,
}

#[derive(Debug, Clone)]
pub struct BenchCorpus {
    pub id: BenchCorpusId,
    pub datasets: Vec<BenchDataset>,
}

impl BenchCorpus {
    #[must_use]
    pub fn new(id: BenchCorpusId, datasets: Vec<BenchDataset>) -> Self {
        Self { id, datasets }
    }
}

#[must_use]
pub fn bench_corpus(id: BenchCorpusId) -> BenchCorpus {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let datasets = match id {
        BenchCorpusId::Fastq5Set => vec![
            BenchDataset {
                id: "ERR2112797",
                r1: root.join("scripts/lab/corpus/fastq/ERR2112797/ERR2112797_1.fastq.gz"),
                r2: Some(root.join("scripts/lab/corpus/fastq/ERR2112797/ERR2112797_2.fastq.gz")),
                sha256_r1: "158c3d487dd55a6f914e860eca2eebe744346fcdb75b53a1adb9137194451239",
                sha256_r2: Some("ff5c74ae4a8aab317709908c065fba3196a7c331a77e94d8d56991e4ad5e2c61"),
                paired: true,
            },
            BenchDataset {
                id: "ERR769587",
                r1: root.join("scripts/lab/corpus/fastq/ERR769587/ERR769587.fastq.gz"),
                r2: None,
                sha256_r1: "928e0b976934f4a41de7b04ba6fefe8dec9a2db257b348afb958335c3421f7dc",
                sha256_r2: None,
                paired: false,
            },
            BenchDataset {
                id: "ERR769592",
                r1: root.join("scripts/lab/corpus/fastq/ERR769592/ERR769592.fastq.gz"),
                r2: None,
                sha256_r1: "e0be169a0607fb365f23421a87bb5780c94f8dbabcf06d1de978410f4a82c293",
                sha256_r2: None,
                paired: false,
            },
            BenchDataset {
                id: "SYNTHETIC_SE",
                r1: root.join("scripts/lab/corpus/fastq/synthetic/SE.fastq.gz"),
                r2: None,
                sha256_r1: "aa0d377ec155f3205f02fb4fa9cb9bc9f1216b15e1ae4e047679184ae1f53af2",
                sha256_r2: None,
                paired: false,
            },
            BenchDataset {
                id: "SYNTHETIC_PE",
                r1: root.join("scripts/lab/corpus/fastq/synthetic/PE_R1.fastq.gz"),
                r2: Some(root.join("scripts/lab/corpus/fastq/synthetic/PE_R2.fastq.gz")),
                sha256_r1: "ea09b95a1563c7cdf8b15d56318f2be224a9ec45697f1706291e442ee8293887",
                sha256_r2: Some("131c44a3052d518046d52f75bfa4745468cf77972bbfb04280c9c5b14149f540"),
                paired: true,
            },
        ],
    };
    BenchCorpus { id, datasets }
}
