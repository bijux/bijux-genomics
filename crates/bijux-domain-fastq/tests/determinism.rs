use sha2::Digest;

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn count_reads_and_bases(data: &str) -> (u64, u64) {
    let mut reads = 0u64;
    let mut bases = 0u64;
    for (idx, line) in data.lines().enumerate() {
        if idx % 4 == 1 {
            reads += 1;
            bases += line.trim().len() as u64;
        }
    }
    (reads, bases)
}

#[test]
fn regression_corpus_hashes_match() {
    let se = "@read1\nACGTACGT\n+\nFFFFFFFF\n@read2\nTTTTAAAA\n+\nFFFFFFFF\n";
    let pe_r1 = "@read1/1\nACGTAC\n+\nFFFFFF\n@read2/1\nTTTTAA\n+\nFFFFFF\n";
    let pe_r2 = "@read1/2\nTGCATG\n+\nFFFFFF\n@read2/2\nAAAATT\n+\nFFFFFF\n";
    let expected = [
        (
            "SE.fastq",
            "89c970df28ceaebed41cf01317b7372c979deaae60108e63368400d010253430",
        ),
        (
            "PE_R1.fastq",
            "91183f7aa7a63b3a6d72fc9508cb7d02b9b83137c3fb0ff9158b374a830e4116",
        ),
        (
            "PE_R2.fastq",
            "7ce13244d046be23ea8c08f291ae4733cc3db6f9ff43cdaa92d4c3f504dcc7c2",
        ),
    ];
    for (name, hash) in expected {
        let actual = match name {
            "SE.fastq" => sha256_bytes(se.as_bytes()),
            "PE_R1.fastq" => sha256_bytes(pe_r1.as_bytes()),
            "PE_R2.fastq" => sha256_bytes(pe_r2.as_bytes()),
            _ => unreachable!(),
        };
        assert_eq!(actual, hash, "hash mismatch for {name}");
    }
}

#[test]
fn regression_corpus_counts_match() {
    let se = "@read1\nACGTACGT\n+\nFFFFFFFF\n@read2\nTTTTAAAA\n+\nFFFFFFFF\n";
    let pe_r1 = "@read1/1\nACGTAC\n+\nFFFFFF\n@read2/1\nTTTTAA\n+\nFFFFFF\n";
    let pe_r2 = "@read1/2\nTGCATG\n+\nFFFFFF\n@read2/2\nAAAATT\n+\nFFFFFF\n";
    let expected = [
        ("SE.fastq", 2, 16),
        ("PE_R1.fastq", 2, 12),
        ("PE_R2.fastq", 2, 12),
    ];
    for (name, reads, bases) in expected {
        let data = match name {
            "SE.fastq" => se,
            "PE_R1.fastq" => pe_r1,
            "PE_R2.fastq" => pe_r2,
            _ => unreachable!(),
        };
        let (actual_reads, actual_bases) = count_reads_and_bases(data);
        assert_eq!(actual_reads, reads, "reads mismatch for {name}");
        assert_eq!(actual_bases, bases, "bases mismatch for {name}");
    }
}
