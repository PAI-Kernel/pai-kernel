# OSS-Fuzz Dockerfile · PAI-Kernel
#
# Built by OSS-Fuzz CI · base image provides Rust nightly + cargo-fuzz +
# sanitizer harnesses. We clone the canonical public repo and run build.sh
# which delegates to cargo-fuzz for compiling fuzz targets.

FROM gcr.io/oss-fuzz-base/base-builder-rust

RUN git clone --depth 1 https://github.com/PAI-Kernel/pai-kernel /src/pai-kernel

WORKDIR /src/pai-kernel

COPY build.sh $SRC/
