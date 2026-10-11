# configs/vcf/panels/locks

## What
Lock metadata for reference panels used in VCF workflows.

## Files
- `configs/vcf/panels/locks/panel_locks.toml`

Acquisition evidence is generated under the configured artifact root and is not checked in as a
source lock. The TOML lock is the repository authority; completed acquisition manifests bind the
observed bytes separately.
