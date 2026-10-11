# Upstream source provenance

Every container recipe in this repository is maintained by Bijux Genomics.
That recipe ownership does not change the authorship or license of the tool it
packages.

The [canonical catalog](../contracts/catalog.toml) owns each tool's upstream
project, documentation, license, supported runtimes, source checksum or pinned
commit, and support disposition. The version lock resolves the build source and
version, while the generated [license index](../../docs/30-operations/CONTAINER_LICENSE_INDEX.md)
records redistribution facts. A separate handwritten table is intentionally
not maintained because it can drift from those authorities.

Validate all tools together:

```console
cargo run -q -p bijux-dna-dev -- containers run check-upstream-provenance
cargo run -q -p bijux-dna-dev -- containers run check-license-metadata
cargo run -q -p bijux-dna-dev -- containers run check-license-index-generated
```

The provenance gate requires HTTPS upstream and documentation URLs plus an
explicit license field for every catalog tool. Retained tools also require at
least one resolved immutable source checksum or commit; excluded tools require
an explicit support reason. A supported Apptainer entry must resolve to the
canonical Bijux-maintained definition path.
