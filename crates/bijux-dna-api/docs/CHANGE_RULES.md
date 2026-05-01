# Change Rules

These rules define safe changes for `bijux-dna-api`.

## Compatible Changes

- Add a new helper under `src/internal/`, `src/runtime/`, or `src/support/`
  without exposing it through `v1::api`.
- Add a new public operation behind the v1 front door when it has documented
  request/response contracts, command docs, and tests.
- Add optional serialized fields with stable defaults.
- Add new schema snapshots for new response types.
- Strengthen validation when valid existing requests keep the same response
  shape or receive clearer errors.
- Add governed workflow/plan manifest fields when they are documented, tested,
  and emitted deterministically.

## Breaking Changes

- Rename, remove, or change the meaning of a public operation.
- Rename, remove, or change the serialized meaning of public schema fields.
- Change `PlanResponse`, `ExecuteResponse`, `DryRunResponse`, `RunStatus`,
  `RenderReportResult`, or `ExplainResponse` shape without snapshot review.
- Move public exports out of `bijux_dna_api::v1::api`.
- Add undeclared filesystem, process, container, network, or global-state effects.
- Add a lower-level dependency that inverts crate boundaries.

## Required Updates

For public API changes, update all relevant files in the same change:

- `docs/API.md`
- `docs/PUBLIC_API.md`
- `docs/COMMANDS.md`
- `docs/REQUEST_FLOW.md` when behavior changes
- `tests/schemas/` and `tests/snapshots/` when schema shape changes
- `tests/contracts/` when workflow behavior changes
- `tests/boundaries/architecture.rs` when layout changes intentionally

## Review Rule

When a change is plausibly breaking, treat it as breaking until the schema,
contract, and boundary tests prove otherwise.
