# Campaign contract tests

`tests/contracts/campaign.rs` exercises the complete lifecycle with two small valid
gzip FASTQ records and an injected executor. It covers bounded scheduling, exact
argv/cwd/environment propagation, failed tool observations, post-execution time,
dry-run evidence retention, transport failure retention, path overlap and resource
admission, symlink rejection, argument precedence and cache namespace confinement.
The symlink case runs on Unix. These fixtures prove orchestration contracts; they
do not execute native tools or establish scientific accuracy.

Adjacent `src/*/contracts.rs` cases retain stage mapping, resume validation,
SortMeRNA bundle/cache behavior and QC contributor/alternate-backend contracts.
The CLI retains its canonical workspace output and platform admission cases.

Use the repository nextest configuration with this crate's all-targets selection;
repeat with `--all-features` when qualifying its feature closure. CLI public corpus
compatibility and workspace dependency policies cover the composed application.
