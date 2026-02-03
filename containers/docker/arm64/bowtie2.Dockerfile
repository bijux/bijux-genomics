# bowtie2 Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_BOWTIE2=2.5.4
ENV VERSION_BOWTIE2=${VERSION_BOWTIE2}

# Install dependencies and precompiled Bowtie2 ARM64 binary
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        perl \
        curl \
        wget \
        unzip \
        ca-certificates \
        make \
        gcc && \
    curl -L https://cpanmin.us | perl - --self-upgrade && \
    cpanm Sys::Hostname && \
    wget -q https://github.com/BenLangmead/bowtie2/releases/download/v${VERSION_BOWTIE2}/bowtie2-${VERSION_BOWTIE2}-linux-aarch64.zip -O /tmp/bowtie2.zip && \
    unzip -q /tmp/bowtie2.zip -d /opt && \
    cp /opt/bowtie2-${VERSION_BOWTIE2}-linux-aarch64/bowtie2* /usr/local/bin/ && \
    rm -rf /tmp/bowtie2.zip /opt/bowtie2-${VERSION_BOWTIE2}-linux-aarch64 && \
    apt-get purge -y curl unzip wget && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /data
ENTRYPOINT ["/bin/sh", "-c", "bowtie2 \"$@\""]
CMD []
