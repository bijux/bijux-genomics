# Container Publish-Ready Metadata

`bijux-dna` treats OCI image labels as the canonical metadata surface for publishable containers.

## Purpose
Define the metadata contract containers must satisfy before publication.

## Scope
This document covers OCI label expectations for Docker and Apptainer image definitions.

## Non-goals
- Publishing images.
- Defining tool runtime behavior inside an image.

## Contracts
- Container definitions must expose the canonical OCI label set.
- Release automation must stamp source revision and creation metadata before publication.

This policy is designed so the same container definition can stay valid for:

- Apptainer builds on Lunarc
- Docker image builds
- future publication to GHCR
- future publication to Docker Hub

## Canonical Metadata

The required canonical labels are:

- `org.opencontainers.image.source`
- `org.opencontainers.image.revision`
- `org.opencontainers.image.created`
- `org.opencontainers.image.licenses`
- `org.opencontainers.image.version`
- `org.opencontainers.image.tool`
- `org.opencontainers.image.title`

These labels are the metadata that downstream tooling, registries, and policy checks should read.

## What We Do Not Do

We do not treat container-internal self-report files as authoritative metadata.

In particular, container definitions should not add duplicate metadata channels such as:

- `/opt/bijux/VERSION.json`
- `/usr/local/bin/bijux-tool-info`

Those patterns duplicate OCI labels, drift easily, and do not improve registry publication quality.

## Build-Time Stamping

Definitions may keep sentinel values such as `unknown` for:

- `org.opencontainers.image.revision`
- `org.opencontainers.image.created`

The release pipeline is expected to stamp those fields with the exact source revision and build timestamp at publish time.

## Publish Direction

When we prepare registry publication work, the preferred direction is:

1. Build images from digest-pinned bases.
2. Stamp OCI labels during the release build.
3. Publish immutable digests first.
4. Add human-facing tags only after digest publication succeeds.
5. Keep repository, source, license, and version metadata aligned with the OCI labels.

## Registry Notes

For GHCR and Docker Hub readiness, the most important metadata surface is the OCI label set, not an in-container JSON file.

For multi-architecture publication, image description should be attached at the OCI manifest/index annotation layer in the release pipeline when needed.
