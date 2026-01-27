# umi_tools Dockerfile (ARM64)
# License: Apache-2.0
FROM python:3.10-slim
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_UMI_TOOLS=1.1.6
ENV VERSION_UMI_TOOLS=${VERSION_UMI_TOOLS}

RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        liblzma-dev && \
    rm -rf /var/lib/apt/lists/*

RUN python3 -m venv /opt/venv && \
    /opt/venv/bin/pip install --no-cache-dir --upgrade pip && \
    /opt/venv/bin/pip install --no-cache-dir umi_tools==${VERSION_UMI_TOOLS} && \
    mv /opt/venv/bin/umi_tools /usr/local/bin/umi_tools-bin && \
    printf '%s\n' \
      '#!/bin/sh' \
      'case "$1" in' \
      '  --version) echo "${VERSION_UMI_TOOLS}"; exit 0;;' \
      '  --help|-h) exec /usr/local/bin/umi_tools-bin --help;;' \
      '  *) exec /usr/local/bin/umi_tools-bin "$@";; esac' \
      > /usr/local/bin/umi_tools && \
    chmod +x /usr/local/bin/umi_tools

ENV PATH="/opt/venv/bin:${PATH}"

WORKDIR /data
ENTRYPOINT ["umi_tools"]
CMD ["--help"]
