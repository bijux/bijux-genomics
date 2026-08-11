# configs/ci

Purpose: CI contract configuration root.

Layout:
- `artifacts/planning/`: backlog cards, scoreboard, and issue-routing labels.
- `configs/ci/compatibility/`: compatibility release inputs and deprecation dashboard sources.
- `configs/ci/registry/`: tool/domain registries and lockfile.
- `configs/ci/stages/`: stage catalogs.
- `configs/ci/tools/`: required tools and image catalog.
- `configs/ci/params/`: parameter schema registries.
- `configs/ci/lints/`: clippy allowlist contracts.
- `configs/ci/budgets/`: fast-tier command and wall-clock budget policy.
- `configs/ci/dependency-graph.json`: source, crate, generated-asset, contract, and report
  dependency graph used for fail-closed change-impact planning.
- `configs/ci/build-authority.json`: the single Rust toolchain, pinned test executor, lockfile,
  compiler-cache, and target-isolation authority.
- `configs/ci/full-shards.json`: complete-workspace nextest coverage and exact hash partitions;
  the runner retains bounded logs and timing receipts while discarding scratch archives, build
  targets, extracted shard artifacts, and isolated source clones before returning
  executed concurrently by the Rust CI supervisor with disjoint per-shard compile, test,
  temporary, coverage, and profile-output roots under one declared build authority. Its
  `artifacts/`-relative output declarations rebase as one closed tree beneath `ARTIFACT_ROOT`.
- `configs/ci/fast-groups.json`: bounded independent fast-lane groups, reference runner and
  warm-cache timing target executed by the same event-driven supervisor.

Rule:
- Keep governed config collections in named subdirectories with their own `index.md`.
- The root-level `*.toml` files in `configs/ci/` remain legacy governed surfaces and must not grow workflow truth that belongs in the subdirectories.
- Validate the dependency graph against locked Cargo metadata with
  `cargo run -q -p bijux-dna-dev -- checks run check-ci-dependency-graph`.
- An unmatched path or empty impact set selects `ci-full`; nightly full runs validate that reduced
  plans have not hidden undeclared dependencies.
