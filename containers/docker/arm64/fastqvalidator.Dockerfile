# fastqvalidator Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04 AS builder
ENV DEBIAN_FRONTEND=noninteractive

# Re-declare the arg so it's available in this stage
ARG VERSION_FASTQVALIDATOR=v0.1.1
ENV VERSION_FASTQVALIDATOR=${VERSION_FASTQVALIDATOR}

# Install build tools
RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends \
        git \
        build-essential \
        zlib1g-dev \
        libbz2-dev \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Fetch and build
RUN git clone --depth 1 https://github.com/statgen/libStatGen.git /opt/libStatGen && \
    git clone --branch "${VERSION_FASTQVALIDATOR}" --depth 1 \
        https://github.com/statgen/fastQValidator.git /opt/fastQValidator

WORKDIR /opt/fastQValidator

RUN make -j"$(nproc)" LIB_PATH_FASTQ_VALIDATOR=/opt/libStatGen && \
    make install PREFIX=/usr/local

# ---------- runtime stage ----------
FROM ubuntu:24.04

# Re-declare for the runtime stage
ARG VERSION_FASTQVALIDATOR=v0.1.1
ENV VERSION_FASTQVALIDATOR=${VERSION_FASTQVALIDATOR}

# Only runtime libs
RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends \
        zlib1g \
        libbz2-1.0 \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the validator binary
COPY --from=builder /usr/local/bin/fastQValidator /usr/local/bin/fq-validator-bin

# Wrapper for consistent CLI and clean version output
RUN printf '%s\n' \
    '#!/bin/sh' \
    'case "$1" in' \
    '  --version) echo "${VERSION_FASTQVALIDATOR#v}"; exit 0;;' \
    '  *) exec /usr/local/bin/fq-validator-bin "$@";; esac' \
  > /usr/local/bin/fastq-validator && \
  chmod +x /usr/local/bin/fastq-validator

WORKDIR /data

# Default to showing help
CMD ["fastq-validator", "--help"]
