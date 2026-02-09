# fqtools Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04 AS builder
ENV DEBIAN_FRONTEND=noninteractive

# Bring in the version
ARG VERSION_FQTOOLS=v2.3
ENV FQTOOLS_VERSION=${VERSION_FQTOOLS}

# Install build dependencies
RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends \
      git build-essential zlib1g-dev libhts-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Clone the exact tag and build
RUN git clone --branch "${FQTOOLS_VERSION}" --depth 1 \
      https://github.com/alastair-droop/fqtools /opt/fqtools

WORKDIR /opt/fqtools

RUN make -j"$(nproc)" HTSDIR=/usr CFLAGS="-O2 -g -Wall -Wextra -Wno-unused-parameter -fcommon"

# ---------- runtime stage ----------
FROM ubuntu:24.04

# Bring in the version again
ARG VERSION_FQTOOLS=v2.3
ENV FQTOOLS_VERSION=${VERSION_FQTOOLS}

# Install only runtime libraries
RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends \
      zlib1g libhts3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the built binary
COPY --from=builder /opt/fqtools/bin/fqtools /usr/local/bin/fqtools-bin

# Wrapper for unified CLI and version reporting
RUN printf '%s\n' \
  '#!/bin/sh' \
  'case "$1" in' \
  '  --version|-v)' \
  '    echo "${FQTOOLS_VERSION#v}"' \
  '    exit 0;;' \
  '  *)' \
  '    exec /usr/local/bin/fqtools-bin "$@";;' \
  'esac' \
  > /usr/local/bin/fqtools && chmod +x /usr/local/bin/fqtools

WORKDIR /data
CMD ["fqtools", "--help"]
