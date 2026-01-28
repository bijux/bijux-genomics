use std::path::Path;

use sha2::Digest;

fn sha256(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

fn count_reads_and_bases(path: &Path) -> Result<(u64, u64), Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(path)?;
    let mut reads = 0u64;
    let mut bases = 0u64;
    for (idx, line) in data.lines().enumerate() {
        if idx % 4 == 1 {
            reads += 1;
            bases += line.trim().len() as u64;
        }
    }
    Ok((reads, bases))
}

#[test]
fn regression_corpus_hashes_match() -> Result<(), Box<dyn std::error::Error>> {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("data")
        .join("fastq")
        .join("regression");
    let expected = [
        (
            "SE.fastq",
            "9ee11c942ee3cba7185a2f2ad42586e83d0adf3d0e9ceba123a66bbdf96718e3",
        ),
        (
            "PE_R1.fastq",
            "53b6d2f590fd69d9efa112cf2ac3b55d5a2f5b04f9936383dbe9d03ce16b3712",
        ),
        (
            "PE_R2.fastq",
            "6174d36b9d4ad7e587a5d66bc3793a5153f5cb92f394454a5a89da353cad70bd",
        ),
    ];
    for (name, hash) in expected {
        let actual = sha256(&base.join(name))?;
        assert_eq!(actual, hash, "hash mismatch for {name}");
    }
    Ok(())
}

#[test]
fn regression_corpus_counts_match() -> Result<(), Box<dyn std::error::Error>> {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("data")
        .join("fastq")
        .join("regression");
    let expected = [
        ("SE.fastq", 2, 14),
        ("PE_R1.fastq", 2, 12),
        ("PE_R2.fastq", 2, 12),
    ];
    for (name, reads, bases) in expected {
        let (actual_reads, actual_bases) = count_reads_and_bases(&base.join(name))?;
        assert_eq!(actual_reads, reads, "reads mismatch for {name}");
        assert_eq!(actual_bases, bases, "bases mismatch for {name}");
    }
    Ok(())
}
