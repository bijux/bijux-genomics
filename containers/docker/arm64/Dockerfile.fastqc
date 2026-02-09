# fastqc Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_FASTQC=0.12.1
ENV VERSION_FASTQC=${VERSION_FASTQC}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        bash \
        perl \
        fontconfig \
        fonts-dejavu-core \
        libharfbuzz0b \
        unzip \
        wget \
        openjdk-11-jre-headless && \
    wget -q https://www.bioinformatics.babraham.ac.uk/projects/fastqc/fastqc_v${VERSION_FASTQC}.zip -O /tmp/fastqc.zip && \
    unzip -q /tmp/fastqc.zip -d /opt && \
    chmod +x /opt/FastQC/fastqc && \
    ln -sf /opt/FastQC/fastqc /usr/local/bin/fastqc && \
    rm /tmp/fastqc.zip && \
    apt-get remove -y unzip wget && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /data
CMD ["fastqc", "--help"]
