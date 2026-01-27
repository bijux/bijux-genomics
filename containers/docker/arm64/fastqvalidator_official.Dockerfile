# fastqvalidator_official Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04 AS builder
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_FASTQVALIDATOR=v0.1.1
ENV VERSION_FASTQVALIDATOR=${VERSION_FASTQVALIDATOR}

RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends \
        git \
        build-essential \
        zlib1g-dev \
        libbz2-dev \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN git clone --depth 1 https://github.com/statgen/libStatGen.git /opt/libStatGen && \
    git clone --branch "${VERSION_FASTQVALIDATOR}" --depth 1 \
        https://github.com/statgen/fastQValidator.git /opt/fastQValidator

WORKDIR /opt/fastQValidator

RUN make -j"$(nproc)" LIB_PATH_FASTQ_VALIDATOR=/opt/libStatGen && \
    make install PREFIX=/usr/local

FROM ubuntu:24.04
ARG VERSION_FASTQVALIDATOR=v0.1.1
ENV VERSION_FASTQVALIDATOR=${VERSION_FASTQVALIDATOR}

RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends \
        zlib1g \
        libbz2-1.0 \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/fastQValidator /usr/local/bin/fq-validator-bin

RUN printf '%s\n' \
    '#!/bin/sh' \
    'case "$1" in' \
    '  --version) echo "${VERSION_FASTQVALIDATOR#v}"; exit 0;;' \
    '  --help|-h|"") exec /usr/local/bin/fq-validator-bin --help;;' \
    '  *) exec /usr/local/bin/fq-validator-bin "$@";; esac' \
  > /usr/local/bin/fastq-validator && \
  chmod +x /usr/local/bin/fastq-validator

WORKDIR /data

LABEL org.opencontainers.image.source="https://github.com/statgen/fastQValidator" \
      org.opencontainers.image.version="${VERSION_FASTQVALIDATOR}"

CMD ["fastq-validator", "--help"]
