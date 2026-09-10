# Public corpus services

Public modules are `fastq`, `amplicon`, `edna`, `damage`, and feature-gated `bam`
and `vcf`. Each exposes its corpus manifest/report types, schema identity,
governed default fixture path and named loading/validation functions.

The main entrypoints are `validate_fastq_corpus_fixture_manifest_path`,
`validate_amplicon_corpus_fixture_manifest_path`,
`validate_edna_corpus_fixture_manifest_path`,
`validate_bam_damage_fixture_manifest_path`,
`validate_bam_corpus_fixture_manifest_path` and
`validate_vcf_corpus_fixture_manifest_path`. Their arguments are explicit paths;
errors preserve the source file and violated corpus rule.

Specialized amplicon truth-table and eDNA taxon loaders support downstream
micro-pipeline consumers. VCF sample metadata supports cohort compatibility.
Implementation modules remain private and have no public deep-import contract.

Persisted schema identities, field names, default path values, and observed
validation behavior remain unchanged. Absolute manifest paths remain supported;
these APIs do not claim path containment, bounded total input size, external
provenance verification or complete tool substitution.

With `panel-metadata`, `panel_metadata` exposes governed panel/map constants,
`ResolvedVcfPanelAssetContract`, `ResolvedVcfPanelDerivationContract`,
`LocalVcfPanelFixtureReport`, authority resolution, metadata materialization and
fixture exclusion. Existing `bijux-dna-bench-vcf::fixtures::panel` imports remain
compatible. Only byte-identical existing markers are reusable.
