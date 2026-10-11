# LICENSING

## Purpose
Clarify workspace source licensing versus container/runtime tool licensing.

## Scope
Applies to repository source code, generated artifacts, and container recipes/images.

## Non-goals
This page does not replace legal review for downstream redistribution obligations.

## Contracts
- Workspace source code license is defined in [../../LICENSE](https://github.com/bijux/bijux-genomics/blob/main/LICENSE).
- Container recipes may package third-party tools with different upstream licenses.
- Container image/tool licenses are tracked under:
  - [../../containers/licenses/README.md](https://github.com/bijux/bijux-genomics/blob/main/containers/licenses/README.md)
  - [../../containers/versions/versions.toml](https://github.com/bijux/bijux-genomics/blob/main/containers/versions/versions.toml) (`upstream_license`)
- Every packaged tool must declare catalog-owned provenance and license in:
  - [../../containers/docs/UPSTREAM_SOURCES.md](https://github.com/bijux/bijux-genomics/blob/main/containers/docs/UPSTREAM_SOURCES.md)

## Examples
- A workspace crate can be Apache-2.0 while packaged upstream tool is GPL-3.0.
- Tool admission requires explicit license and provenance metadata.

## Failure modes
- Missing upstream license metadata blocks container admission.
- Ambiguous third-party licensing blocks promotion to production.
