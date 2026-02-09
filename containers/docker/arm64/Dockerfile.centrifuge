# centrifuge Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04

ARG VERSION_CENTRIFUGE=1.0.4
ENV VERSION_CENTRIFUGE=${VERSION_CENTRIFUGE} \
    DEBIAN_FRONTEND=noninteractive

# 1) System dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      wget build-essential ca-certificates \
      zlib1g-dev libbz2-dev liblzma-dev && \
    rm -rf /var/lib/apt/lists/*

# 2) Fetch, patch & build
RUN wget -qO /tmp/centrifuge.tar.gz \
      https://github.com/DaehwanKimLab/centrifuge/archive/v${VERSION_CENTRIFUGE}-beta.tar.gz && \
    tar -xf /tmp/centrifuge.tar.gz -C /opt && \
    cd /opt/centrifuge-${VERSION_CENTRIFUGE}-beta && \
    \
    # a) Stub out x86-only CPUID routines
    printf '%s\n' \
      '#pragma once' \
      '#include <stdint.h>' \
      'static inline void __cpuid(int32_t, int32_t*, int32_t*, int32_t*, int32_t*) {}' \
      'static inline void __get_cpuid(unsigned int, unsigned int*, unsigned int*, unsigned int*, unsigned int*) {}' \
      'static inline void __get_cpuid_count(unsigned int, unsigned int, unsigned int*, unsigned int*, unsigned int*, unsigned int*) {}' \
    > third_party/cpuid.h && \
    \
    # b) Fix the "-1" narrowing issue
    sed -i 's/-1/(char)-1/g' alphabet.cpp && \
    \
    # c) Remove unsupported x86 flags from Makefile
    sed -i 's/-m32//g; s/-msse2//g; s/-DPOPCNT_CAPABILITY//g' Makefile && \
    \
    # c.1) Avoid std <version> header shadowing
    if [ -f version ] && [ ! -f VERSION ]; then mv version VERSION; fi && \
    \
    # d) Build all three bins (capture log for easier debugging)
    make -j1 CXXFLAGS="-O3" 2>&1 | tee /tmp/centrifuge-build.log && \
    \
    # e) Install
    install -m0755 centrifuge-class      /usr/local/bin/centrifuge && \
    install -m0755 centrifuge-build-bin  /usr/local/bin/centrifuge-build && \
    install -m0755 centrifuge-inspect-bin /usr/local/bin/centrifuge-inspect && \
    \
    # f) Clean up
    rm -rf /tmp/centrifuge.tar.gz /opt/centrifuge-${VERSION_CENTRIFUGE}-beta

WORKDIR /data
ENTRYPOINT ["centrifuge"]
CMD ["--help"]
