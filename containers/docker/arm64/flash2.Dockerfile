# flash2 Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_FLASH2=2.2.00
ENV VERSION_FLASH2=${VERSION_FLASH2}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        build-essential zlib1g-dev git ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN git clone https://github.com/dstreett/FLASH2.git /opt/FLASH2 && \
    cd /opt/FLASH2 && \
    git checkout tags/${VERSION_FLASH2} -b build-branch

RUN cd /opt/FLASH2 && \
    make -j"$(nproc)" && \
    cp flash2 /usr/local/bin/flash2 && \
    rm -rf /opt/FLASH2

WORKDIR /data
ENTRYPOINT ["/bin/sh", "-c", "flash2 \"$@\""]
CMD []
