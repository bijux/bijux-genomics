# VCF adapter interface

`inputs::VcfAdapterInputs::new` admits explicit catalog and binding rows, rejects
unknown stages and repeated or blank binding identities, and takes a typed
`ReferenceFastaMaterializer`. The constructor performs no filesystem effects.
Callers may borrow the admitted catalog and bindings across adapter families.

`inputs::project_supported_vcf_bindings` projects caller-provided stage/tool pairs
onto stage templates. It rejects missing or duplicated templates and repeated
stage/tool/corpus/asset bindings. It neither reads registries nor infers tool scope.

Each family under `adapters` exports typed artifacts, command steps, rows and
reports. Collection services accept admitted inputs and an explicit repository
root; report publication also accepts an output path. Persisted schema identities
remain family-specific. Errors propagate admission, source and publication failures.

`fixtures` exposes local manifest and panel preparation used by those contracts.
Reference writing is delegated through the caller-supplied operation. Native VCF
indexing delegates to the stage execution owner. Panel metadata markers are
explicitly synthetic and are not interchangeable native reference panels.
