# Containers Release Checklist

Each checklist item must map to an executable gate in `scripts/containers/release-gate.sh`.

- [ ] HPC naming policy enforced (`scripts/containers/check-hpc-image-naming.sh`)
- [ ] Toolkit bundle coverage enforced (`scripts/containers/check-toolkit-bundles.sh`)
- [ ] Missing images gate enforced (`scripts/containers/check-missing-images.sh`)
- [ ] Registry-to-container coverage enforced (`scripts/containers/check-tool-container-coverage.sh`)
- [ ] Version lock up to date (`scripts/containers/check-version-lock.sh`)
- [ ] Version authority and hash pin checks (`scripts/containers/check-version-authority.sh`, `scripts/containers/check-version-hash-pin.sh`)
- [ ] Lock matches built output (`scripts/containers/check-lock-matches-built-output.sh`)
- [ ] Smoke contract + generated QA matrix checks (`scripts/containers/check-smoke-contract.sh`, `scripts/containers/check-qa-matrix-generated.sh`)
- [ ] Build provenance + digest policy checks (`scripts/containers/check-build-provenance.sh`, `scripts/containers/check-digest-output-policy.sh`)
- [ ] Runtime network and runtime-download policy checks (`scripts/containers/check-network-disclosure.sh`, `scripts/containers/check-runtime-downloads.sh`)
- [ ] SBOM + vuln hook checks (`scripts/containers/check-sbom-artifacts.sh`, `scripts/containers/check-vuln-hook.sh`)
- [ ] Owners/tool-id parity/tool docs checks (`scripts/containers/check-owners.sh`, `scripts/containers/check-tool-id-contract.sh`, `scripts/containers/check-tool-docs-generated.sh`)
- [ ] Time/locale and cache policy checks (`scripts/containers/check-time-locale-determinism.sh`, `scripts/containers/check-apptainer-cache-policy.sh`)

