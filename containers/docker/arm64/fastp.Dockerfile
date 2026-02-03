# fastp Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_FASTP=0.23.4
ENV VERSION_FASTP=${VERSION_FASTP}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        wget ca-certificates build-essential zlib1g-dev libdeflate-dev libisal-dev && \
    wget -q https://github.com/OpenGene/fastp/archive/v${VERSION_FASTP}.tar.gz -O /tmp/fastp.tar.gz && \
    tar -xzf /tmp/fastp.tar.gz -C /opt && \
    cd /opt/fastp-${VERSION_FASTP} && \
    make && \
    cp fastp /usr/local/bin/ && \
    rm -rf /tmp/fastp.tar.gz /opt/fastp-${VERSION_FASTP} && \
    apt-get purge -y wget build-essential && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /data
ENTRYPOINT ["/bin/sh", "-c", "fastp \"$@\""]
CMD []
