# star Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_STAR=2.7.11b
ENV VERSION_STAR=${VERSION_STAR}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        wget ca-certificates build-essential \
        zlib1g-dev libbz2-dev xxd && \
    wget -q https://github.com/alexdobin/STAR/archive/${VERSION_STAR}.tar.gz \
         -O /tmp/star.tar.gz && \
    tar -xzf /tmp/star.tar.gz -C /opt && \
    cd /opt/STAR-${VERSION_STAR}/source && \
    find . -name Makefile -exec \
        sed -Ei 's/-mavx2|-msse[0-9.]*|-mfpmath=sse//g' {} \; && \
    echo 'CFLAGS  += -march=armv8-a+simd' >> Makefile && \
    echo 'CXXFLAGS+= -march=armv8-a+simd' >> Makefile && \
    make -j"$(nproc)" STAR && \
    mv STAR /usr/local/bin/star-bin && \
    rm -rf /tmp/star.tar.gz /opt/STAR-${VERSION_STAR} /var/lib/apt/lists/* && \
    apt-get purge -y wget ca-certificates build-essential xxd && \
    apt-get autoremove -y

# Create a wrapper script for unified STAR execution
RUN echo '#!/bin/sh' > /usr/local/bin/star && \
    echo 'if [ "$1" = "--version" ]; then' >> /usr/local/bin/star && \
    echo '    echo "$VERSION_STAR"' >> /usr/local/bin/star && \
    echo '    exit 0' >> /usr/local/bin/star && \
    echo 'elif [ "$1" = "--help" ]; then' >> /usr/local/bin/star && \
    echo '    /usr/local/bin/star-bin --help 2>&1' >> /usr/local/bin/star && \
    echo '    exit 0' >> /usr/local/bin/star && \
    echo 'else' >> /usr/local/bin/star && \
    echo '    /usr/local/bin/star-bin "$@"' >> /usr/local/bin/star && \
    echo 'fi' >> /usr/local/bin/star && \
    chmod +x /usr/local/bin/star

WORKDIR /data
ENTRYPOINT ["star"]
CMD ["--help"]
