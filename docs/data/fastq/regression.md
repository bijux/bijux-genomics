# FASTQ Regression Corpus

Minimal FASTQ fixtures used for regression checks. These are now embedded as
inline strings in tests rather than checked in as files.

SHA256 (inline fixtures):
- SE.fastq: 89c970df28ceaebed41cf01317b7372c979deaae60108e63368400d010253430
- PE_R1.fastq: 91183f7aa7a63b3a6d72fc9508cb7d02b9b83137c3fb0ff9158b374a830e4116
- PE_R2.fastq: 7ce13244d046be23ea8c08f291ae4733cc3db6f9ff43cdaa92d4c3f504dcc7c2

Expected counts (inline fixtures):
- SE.fastq: 2 reads, 16 bases
- PE_R1.fastq: 2 reads, 12 bases
- PE_R2.fastq: 2 reads, 12 bases
