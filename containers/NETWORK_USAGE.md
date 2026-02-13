# Container Runtime Network Usage

Authoritative runtime-network disclosure index.

Runtime network is denied by default. Any tool that requires runtime network access must:
- set `runtime_network = true` in `containers/network/<tool>.network.toml`
- be listed here with a rationale

Build-time network access is tracked separately via `build_network` in the same metadata files.

Detailed policy guidance is documented in `containers/docs/NETWORK_USAGE.md`.

