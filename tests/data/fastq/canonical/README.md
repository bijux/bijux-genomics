# Bijux canonical FASTQ dataset

Tiny FASTQ fixtures used across QA, benchmarks, regression tests, and CI.

Files (SHA-256):
- BIJUX_SE_R1.fastq.gz: aa0d377ec155f3205f02fb4fa9cb9bc9f1216b15e1ae4e047679184ae1f53af2
- BIJUX_PE_R1.fastq.gz: ea09b95a1563c7cdf8b15d56318f2be224a9ec45697f1706291e442ee8293887
- BIJUX_PE_R2.fastq.gz: 131c44a3052d518046d52f75bfa4745468cf77972bbfb04280c9c5b14149f540

Intended usage:
- QA: `bijux image-qa`
- Bench: `make bench-all` (sample inputs)
- Regression: stage-level tests in `crates/bijux-cli/tests`
- CI: deterministic fixtures
