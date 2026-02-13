# VCF Tool Admission Criteria (Downstream)

Purpose: define minimum criteria for admitting downstream VCF tools from planned to production.

Scope: `vcf` downstream tools (`plink`, `plink2`, `beagle`, `eigensoft`, `germline`, `ibdseq`, `ibdhap`, `ibdne`, and peers).

Contracts:
- Tool must be present in registry with explicit `status`.
- Tool must have pinned version/hash and container parity (or explicit external policy).
- Tool must have fixture coverage in relevant stage directories.
- Tool defaults and rationale must be documented in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- Tool must pass smoke/contract/policy gates before promotion.

Failure modes:
- Planned tool promoted without fixture coverage causes policy failure.
- Stage uses tool without container/external declaration causes parity failure.
