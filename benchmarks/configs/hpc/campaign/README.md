# HPC Campaign Profiles

These are baseline campaign profiles for Slurm dry-run and preflight validation.

- `lunarc-small.toml`: Lunarc-oriented defaults.
- `lunarc-fastq-bam-local-ready.toml`: Lunarc-oriented cross-domain dry-run profile that expects prepared corpus, database, and image roots.
- `lunarc-fastq-bam-vcf-local-ready.toml`: Lunarc-oriented cross-domain dry-run profile that expects prepared corpus, database, and image roots for FASTQ→BAM→VCF execution without reference or panel build jobs on compute nodes.
- `generic-small.toml`: generic Slurm defaults.
- `cross-mini.toml`: cross-domain fixture with explicit handoff dependency.
- `site-profiles/lunarc.toml`: Lunarc site defaults.
- `site-profiles/generic.toml`: generic site defaults.
- `sharing/*.toml`: collaborator recipient profiles for bundle sharing.

Secrets must not be committed in these profiles. Use `security.env_file` and user overrides.
Use `security.encryption_backend` to choose `mock-envelope-v1` (local fixture backend) or `age-cli`
for recipient-based encryption with real identities.
Set `security.encrypt_operator_outputs = true` only when `.log/.out/.err` must also be encrypted.

Sharing flow:

1. Define collaborator recipients in `sharing/*.toml`.
2. Re-encrypt and redact with:
   `bijux-dna slurm share-bundle --bundle <path> --profile benchmarks/configs/hpc/campaign/sharing/<profile>.toml --out-dir <dir>`
3. Verify policy coverage with:
   `bijux-dna slurm verify-results-policy --results-bundle <results> --code-bundle <code>`

Resource templates can be selected globally with `resources.default`, or by stage family via
`resources.stage_defaults`.
