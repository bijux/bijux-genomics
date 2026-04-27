# LICENSING

## Purpose
Clarify workspace source licensing versus container/runtime tool licensing.

## Scope
Applies to repository source code, generated artifacts, and container recipes/images.

## Non-goals
This page does not replace legal review for downstream redistribution obligations.

## Contracts
- Workspace source code license is defined in [../../LICENSE](../../LICENSE).
- Container recipes may package third-party tools with different upstream licenses.
- Container image/tool licenses are tracked under:
  - [../../containers/licenses/README.md](../../containers/licenses/README.md)
  - [../../containers/versions/versions.toml](../../containers/versions/versions.toml) (`upstream_license`)
- Non-bijux packaged tools must declare provenance and license in:
  - [../../containers/apptainer/shared/NON_BIJUX_SOURCES.md](../../containers/apptainer/shared/NON_BIJUX_SOURCES.md)

## Examples
- A workspace crate can be Apache-2.0 while packaged upstream tool is GPL-3.0.
- Tool admission requires explicit license and provenance metadata.

## Failure modes
- Missing upstream license metadata blocks container admission.
- Ambiguous third-party licensing blocks promotion to production.
