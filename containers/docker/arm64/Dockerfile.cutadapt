# cutadapt Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04 AS runtime
ENV DEBIAN_FRONTEND=noninteractive

ARG CUTADAPT_VERSION=4.9
ENV CUTADAPT_VERSION=${CUTADAPT_VERSION}

RUN --mount=type=cache,target=/var/cache/apt \
    apt-get update -qq && \
    apt-get install -y --no-install-recommends \
      python3 \
      python3-pip \
      python3-dev \
      build-essential \
      ca-certificates && \
    python3 -m pip install --no-cache-dir --break-system-packages \
      "cutadapt==${CUTADAPT_VERSION}" \
      numpy && \
    apt-get purge -y \
      python3-dev \
      build-essential && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /data
ENTRYPOINT ["/bin/sh", "-c", "cutadapt \"$@\""]
CMD []
