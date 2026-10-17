# Parser proof public API

## Module Inventory

- catalog: governed FASTQ, BAM and VCF family declarations and probe kinds.
- model: persisted proof report and observation records.
- ports: `ProbeMaterializer` producer operations and `MaterializedProof` addresses/tool identity.
- reports: schema dispatch and compiled parser admission.

`build_report` computes the aggregate over explicit producer operations. `materialize_row` resolves one declared proof observation. `publish_report` validates counts and row identities before atomic per-file replacement.

The default feature set is empty. `fastq` admits the canonical FASTQ parser owner; `bam` admits the canonical BAM proof owner. Feature selection changes supported parsing, not persisted report identity. The caller resolves repository configuration and stage plans. A successful parser row proves the declared representation contract; it does not establish backend accuracy or equivalence.

BAM and VCF report admission and snapshot projection share one report
acquisition. VCF retains its presence-only governed-key contract; BAM retains
canonical typed admission and its domain parser's legacy interpretations.
