# scripts/hpc/lunarc

## What
Lunarc-specific HPC synchronization and helper scripts.

## Philosophy
Provider-specific behavior stays scoped here so Lunarc conventions do not spread into generic scripts.

Requires: bash, rg, coreutils (plus script-specific tools documented inline).
Exit codes: 0 success; 1 policy/validation failure; 2 usage/config error; 127 missing dependency.
