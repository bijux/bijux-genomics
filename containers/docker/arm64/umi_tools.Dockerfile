# umi_tools Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_UMI_TOOLS=1.1.6
ENV VERSION_UMI_TOOLS=${VERSION_UMI_TOOLS}

RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends --no-install-suggests -o Acquire::Retries=3 \
        python3 python3-venv python3-dev python3-pip liblzma-dev ca-certificates build-essential && \
    python3 -m venv /opt/venv && \
    /opt/venv/bin/pip install --no-cache-dir --upgrade pip && \
    /opt/venv/bin/pip install --no-cache-dir umi_tools==${VERSION_UMI_TOOLS} && \
    apt-get purge -y python3-dev build-essential && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*

ENV PATH="/opt/venv/bin:${PATH}"

WORKDIR /data
ENTRYPOINT ["umi_tools"]
CMD ["--help"]
