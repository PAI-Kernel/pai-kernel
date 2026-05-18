#!/bin/bash -eu
#
# OSS-Fuzz build script · PAI-Kernel
#
# Builds 4 cargo-fuzz targets via cargo-fuzz · copies binaries +
# corresponding sanitizer info to $OUT/ for ClusterFuzz scheduler.
#
# Triggered automatically by OSS-Fuzz infrastructure on schedule
# (typically daily) AND on demand for adopter-reported issues.

cd $SRC/pai-kernel

# Build fuzz targets · release mode + sanitizer flags managed by OSS-Fuzz env
cargo +nightly fuzz build --release

# Copy fuzz binaries to OSS-Fuzz output directory
FUZZ_TARGET_OUTPUT_DIR="fuzz/target/x86_64-unknown-linux-gnu/release"

for target in export_bundle_parse witness_entry_parse pii_scan_text boundary_declaration_parse; do
    cp "${FUZZ_TARGET_OUTPUT_DIR}/${target}" "$OUT/"
done

# Note: OSS-Fuzz expects optional fuzz_target.options file per target with
# ClusterFuzz tuning · skipped for initial submission · standard defaults applied.
