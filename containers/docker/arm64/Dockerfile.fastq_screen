# fastq_screen Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive
ARG VERSION_FASTQ_SCREEN=0.15.3
ENV VERSION_FASTQ_SCREEN=${VERSION_FASTQ_SCREEN}

RUN --mount=type=cache,target=/var/cache/apt \
    apt-get update -qq -o Acquire::Retries=5 -o Acquire::http::Timeout=30 && \
    apt-get install -y --no-install-recommends \
        perl \
        curl \
        ca-certificates \
        bzip2 \
        gzip && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /opt
RUN curl -fsSL -o fastq_screen.tar.gz "https://github.com/StevenWingett/FastQ-Screen/archive/refs/tags/v${VERSION_FASTQ_SCREEN}.tar.gz" && \
    tar -xzf fastq_screen.tar.gz && \
    mv FastQ-Screen-${VERSION_FASTQ_SCREEN} fastq_screen && \
    ln -s /opt/fastq_screen/fastq_screen /usr/local/bin/fastq_screen

WORKDIR /data

LABEL org.opencontainers.image.source="https://www.bioinformatics.babraham.ac.uk/projects/fastq_screen/" \
      org.opencontainers.image.version="${VERSION_FASTQ_SCREEN}"

CMD ["fastq_screen", "--help"]
