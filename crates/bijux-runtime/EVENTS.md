# EVENTS

Runtime emits structured events for observability and diagnostics. Events are
transport-neutral and are consumed by adapters (e.g., OpenTelemetry).

Event payloads live under `telemetry::events` and should remain stable across
minor releases.
