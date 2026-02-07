# EVENTS

## Schema
Runtime emits structured events for:
- Step start/end
- Artifact verification
- Cache hits

## Stability
Event fields are versioned through the contract version. Additive changes bump minor; breaking changes bump major.

## Sinks
Events are recorded through the runtime recorder interface. Sinks configure storage location and filtering.
