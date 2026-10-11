# bijux-dna-process Boundary Contract

Owner: process lifecycle, cancellation and bounded raw stream capture.

## Owns

- Isolated Unix process-group setup and direct child reaping.
- One capture deadline spanning leader lifetime and inherited pipes.
- A shared stdout/stderr byte budget enforced during collection.
- Raw exit and cancellation evidence for consumer-owned interpretation.

## Allowed Effects

Spawn the explicitly supplied command; read its pipes; wait for its child;
signal its owned process group. The process owner does not select executable
versions, construct domain commands, interpret outputs, or publish evidence.

## Forbidden Dependencies

All workspace production, build and development dependencies are forbidden.
The crate is an independent transport leaf consumed by runner and containers.
It owns no scheduler, container, CLI, scientific or network policy.

## Validation

The process contracts exercise raw bytes, combined exact/overflow limits, leader
exit with inherited pipes, closed pipes with a live child, and pre-spawn rejection.
Workspace policies enforce the empty workspace dependency set, the exact process
spawn source and test harness command owners.

`cargo nextest run -p bijux-dna-process --test contracts --locked`

See [ARCHITECTURE.md](ARCHITECTURE.md) for platform and process-group limits.
