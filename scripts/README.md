# scripts

Purpose: strict index of supported script areas and allowed usage.

Allowed subdirectories:
- `scripts/tooling`: repo tooling wrappers; python logic lives in `scripts/tooling/python/`.

Internal-only:
- `scripts/_lib`: shared shell helpers only.
- `scripts/experimental`: quarantined non-supported scripts.
- `scripts/assets`: asset refresh helpers.

The checks control plane is `cargo run -p bijux-dev-dna -- checks ...`.
The compatibility entrypoint remains `./scripts/run.sh checks <check-id>`.
The docs control plane is `cargo run -p bijux-dev-dna -- docs ...`.
The compatibility entrypoint remains `./scripts/run.sh docs <command>`.
The examples control plane is `cargo run -p bijux-dev-dna -- examples ...`.
The compatibility entrypoint remains `./scripts/run.sh examples <command>`.
The hpc control plane is `cargo run -p bijux-dev-dna -- hpc ...`.
The compatibility entrypoint remains `./scripts/run.sh hpc <command>`.
The lab control plane is `cargo run -p bijux-dev-dna -- lab ...`.
The compatibility entrypoint remains `./scripts/run.sh lab <command>`.
The smoke control plane is `cargo run -p bijux-dev-dna -- smoke ...`.
The compatibility entrypoint remains `./scripts/run.sh smoke <command>`.
The test control plane is `cargo run -p bijux-dev-dna -- test ...`.
The compatibility entrypoint remains `./scripts/run.sh test <command>`.
The containers control plane is `cargo run -p bijux-dev-dna -- containers ...`.
The canonical entrypoints are `./bijux-dev-dna/containers`.
The compatibility entrypoint remains `./scripts/run.sh containers <command>`.
The domain control plane is `cargo run -p bijux-dev-dna -- domain ...`.
The compatibility entrypoint remains `./scripts/run.sh domain <command>`.

Use `./scripts/run.sh --list` to print the supported compatibility surface and native command groups.

Requires: bash, rg, coreutils (plus script-specific tools documented inline).
Exit codes: 0 success; 1 policy/validation failure; 2 usage/config error; 127 missing dependency.
