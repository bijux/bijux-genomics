# pear Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_PEAR=0.9.6
ENV VERSION_PEAR=${VERSION_PEAR}

RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends --no-install-suggests -o Acquire::Retries=3 \
        wget \
        ca-certificates \
        build-essential \
        zlib1g-dev \
        libbz2-dev \
        liblzma-dev \
        autoconf \
        automake \
        libtool \
        m4 \
        pkg-config && \
    wget -q https://depot.galaxyproject.org/software/pear/pear_${VERSION_PEAR}_src_all.tar.gz -O /tmp/pear.tar.gz && \
    tar -xzf /tmp/pear.tar.gz -C /opt && \
    cd /opt/pear-${VERSION_PEAR}-src && \
    autoreconf -i && \
    ./configure --prefix=/usr/local && \
    make -j"$(nproc)" && \
    make install && \
    mv /usr/local/bin/pear /usr/local/bin/pear-bin && \
    rm -rf /opt/pear-${VERSION_PEAR}-src /tmp/pear.tar.gz /var/lib/apt/lists/* && \
    apt-get purge -y wget ca-certificates build-essential autoconf automake libtool m4 pkg-config && \
    apt-get autoremove -y

# Create a wrapper script for unified PEAR execution
RUN echo '#!/bin/sh' > /usr/local/bin/pear && \
    echo 'if [ "$1" = "--version" ]; then' >> /usr/local/bin/pear && \
    echo '    echo "$VERSION_PEAR"' >> /usr/local/bin/pear && \
    echo '    exit 0' >> /usr/local/bin/pear && \
    echo 'elif [ "$1" = "--help" ]; then' >> /usr/local/bin/pear && \
    echo '    /usr/local/bin/pear-bin --help' >> /usr/local/bin/pear && \
    echo '    exit 0' >> /usr/local/bin/pear && \
    echo 'else' >> /usr/local/bin/pear && \
    echo '    /usr/local/bin/pear-bin "$@"' >> /usr/local/bin/pear && \
    echo 'fi' >> /usr/local/bin/pear && \
    chmod +x /usr/local/bin/pear

WORKDIR /data
ENTRYPOINT ["pear"]
CMD ["--help"]
