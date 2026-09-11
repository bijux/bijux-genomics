# Parser proof ownership

`catalog/` groups governed family declarations by FASTQ, BAM and VCF. A declaration
binds a family, representative stage, materialization kind, parser kind and ordered
snapshot keys. `reports/` owns schema dispatch; FASTQ and BAM features select their
canonical native parser owners. VCF currently retains its declared required-key
admission. Presence of those keys does not establish value-level scientific validity.

`admission.rs` resolves all active families, uniqueness and compiled parser support
before invoking producers. `materialization.rs` interprets local-stage proof pointers
and observed tool identity; the port supplies canonical producer operations and
fallback tool declarations. `service.rs` sorts observations and computes counts.
`publication.rs` refuses contradictory counts or failed/duplicate rows before atomic
per-file replacement. `model.rs` owns the persisted report and row records.

Applications resolve governed configuration and stage plans. BAM native/tiny fixture
production lives in the BAM proof owner, FASTQ production in the FASTQ evidence
owner, and existing VCF smoke services supply VCF artifacts. No native engine is
needed to compile the default aggregate service. Enabling a family explicitly admits
its native compilation closure; this is verified from Cargo metadata and trees.

A complete run reuses its configured probe namespace without deleting the whole
previous tree. Individual generators retain their owned output semantics. Failed
builds leave the prior aggregate report untouched, but generation is not a transaction
across all artifacts. Atomic publication does not protect against concurrent writers
to parent directories. Synthetic contamination/genotyping fixture values remain
synthetic; a passing parser row is a representation contract, not an accuracy claim.
