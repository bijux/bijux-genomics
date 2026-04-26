# bijux-dna Commands

`COMMANDS.md` is the single source of truth for command names managed by this crate. Parser types
live under `src/commands/cli/parse/`; routing lives under `src/commands/router/` and
domain-specific command adapters under `src/commands/*/`.

## Managed Command Inventory

### Stable operator commands

- `bijux-dna env images`
- `bijux-dna env info`
- `bijux-dna env doctor`
- `bijux-dna env list`
- `bijux-dna env export-json`
- `bijux-dna env export-containers --json`
- `bijux-dna env export-hpc`
- `bijux-dna env sif-inventory`
- `bijux-dna env ensure`
- `bijux-dna env apptainer-qa-matrix`
- `bijux-dna env ensure-images`
- `bijux-dna env lint-apptainer-defs`
- `bijux-dna env smoke`
- `bijux-dna env prep`
- `bijux-dna registry list-tools`
- `bijux-dna registry list-stages`
- `bijux-dna registry show-tool`
- `bijux-dna registry show-stage`
- `bijux-dna registry show`
- `bijux-dna registry export-json`
- `bijux-dna registry export-containers --json`
- `bijux-dna registry coverage-matrix`
- `bijux-dna registry validate-tool`
- `bijux-dna registry audit`
- `bijux-dna registry doctor`
- `bijux-dna registry promote`
- `bijux-dna registry lint`
- `bijux-dna corpus materialize`
- `bijux-dna corpus normalize`
- `bijux-dna corpus validate`
- `bijux-dna corpus list`
- `bijux-dna corpus diff`
- `bijux-dna status`
- `bijux-dna run filter`
- `bijux-dna run merge`
- `bijux-dna run trim`
- `bijux-dna run preprocess`
- `bijux-dna run run`
- `bijux-dna run stats-neutral`
- `bijux-dna run validate-pre`
- `bijux-dna run compare`
- `bijux-dna plan list`
- `bijux-dna plan explain`
- `bijux-dna plan plan`
- `bijux-dna analyze runs`
- `bijux-dna analyze summary`
- `bijux-dna analyze compare`
- `bijux-dna analyze rank`
- `bijux-dna analyze report`
- `bijux-dna analyze metrics`
- `bijux-dna analyze bench`
- `bijux-dna explain runs`
- `bijux-dna explain summary`
- `bijux-dna explain compare`
- `bijux-dna explain rank`
- `bijux-dna explain report`
- `bijux-dna explain metrics`
- `bijux-dna explain bench`
- `bijux-dna bench config validate`
- `bijux-dna bench run`
- `bijux-dna bench status`
- `bijux-dna bench workspace-value`
- `bijux-dna bench config-json`
- `bijux-dna bench repo-checks`
- `bijux-dna bench write-screen-taxonomy-database-lineage`
- `bijux-dna bench publication-targets`
- `bijux-dna bench corpus-fastq`
- `bijux-dna bench normalize-workspace-layout`
- `bijux-dna bench corpus-fastq-report`
- `bijux-dna bench corpus-fastq-publication-status`
- `bijux-dna bench corpus-fastq-published-dossiers`
- `bijux-dna bench fastq trim`
- `bijux-dna bench fastq trim-polyg-tails`
- `bijux-dna bench fastq trim-terminal-damage`
- `bijux-dna bench fastq validate`
- `bijux-dna bench fastq detect-adapters`
- `bijux-dna bench fastq profile-read-lengths`
- `bijux-dna bench fastq filter`
- `bijux-dna bench fastq filter-low-complexity`
- `bijux-dna bench fastq merge`
- `bijux-dna bench fastq remove-duplicates`
- `bijux-dna bench fastq remove-chimeras`
- `bijux-dna bench fastq normalize-primers`
- `bijux-dna bench fastq infer-asvs`
- `bijux-dna bench fastq cluster-otus`
- `bijux-dna bench fastq normalize-abundance`
- `bijux-dna bench fastq correct`
- `bijux-dna bench fastq report-qc`
- `bijux-dna bench fastq umi`
- `bijux-dna bench fastq index-reference`
- `bijux-dna bench fastq screen`
- `bijux-dna bench fastq deplete-host`
- `bijux-dna bench fastq deplete-reference-contaminants`
- `bijux-dna bench fastq deplete-rrna`
- `bijux-dna bench fastq stats`
- `bijux-dna bench fastq profile-overrepresented-sequences`
- `bijux-dna bench fastq preprocess`
- `bijux-dna bench bam stage`
- `bijux-dna bench bam pipeline`

### Debug/development commands

These commands are hidden in non-debug builds or exist for repository control-plane work:

- `bijux-dna ena select`
- `bijux-dna ena fetch`
- `bijux-dna tool validate`
- `bijux-dna domain validate`
- `bijux-dna domain coverage`
- `bijux-dna lab corpus list-fastq`
- `bijux-dna config init-hpc`
- `bijux-dna config doctor`
- `bijux-dna bam run`
- `bijux-dna bam list-stages`
- `bijux-dna bam explain`
- `bijux-dna vcf plan`
- `bijux-dna vcf explain`
- `bijux-dna vcf run`
- `bijux-dna validate-manifests`
- `bijux-dna platform`
- `bijux-dna image-qa`
- `bijux-dna replay`
- `bijux-dna compare`
- `bijux-dna policies audit`
- `bijux-dna ci validate`
- `bijux-dna debug`
- `bijux-dna collect`

## Ownership Rules

- Add a command here in the same change that adds a parser variant.
- Keep command implementation in the smallest matching `src/commands/*` owner.
- Use `bijux-dna-api` for planning, reporting, domain semantics, and execution contracts.
- Update help snapshots when visible help output changes.

## Verification

- Command parser changes: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --test contracts --no-default-features`
- Help text changes: update and review `tests/snapshots/*.txt`
- Command inventory changes: update this file and run the docs policy suite.
