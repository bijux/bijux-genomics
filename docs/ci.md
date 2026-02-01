# CI Gates

## Test tiers

- **default**: PR gate; fast deterministic suite.
- **slow**: ignored tests with `_slow_` in name.
- **science**: ignored tests with `_science_` in name.
- **e2e**: ignored tests with `_e2e_` in name; requires `BIJUX_E2E=1`.

We tag slow tests with `#[ignore]` and a note (e.g. "slow e2e").
Run them locally via:

```
make test-fast
make test-slow
make test-e2e
```

## make test-images

This gate verifies that Docker images are **present and runnable** for the
current platform. It is a smoke test, not a full functional test.

### What PASS means

- Image exists locally
- Required executable exists (if declared)
- Probe command runs and exits with an allowed exit code
- Probe output contains the expected version

### What FAIL means

- **image not found**: Docker image tag does not exist locally
- **executable missing**: expected binary not found in image
- **missing runtime dependency**: binary fails due to shared library issues
- **unexpected exit code**: probe returned a non-allowed exit code
- **probe failed**: probe ran but did not report the expected version

### Output levels

- **INFO (default):** one line per image + final summary
- **DEBUG:** full commands and stdout/stderr

Enable DEBUG with:

- `DEBUG=1 make test-images`
- `cargo run --bin test_docker_images -- --debug`

Quiet mode:

- `QUIET=1 make test-images`
- `cargo run --bin test_docker_images -- --quiet`
