---
title: "Audit Checklist — PAI-Kernel SDK"
slug: audit-checklist
position: 5
hidden: false
excerpt: "Adopter-runnable verification procedures · reproducible build · test execution · GPG signature verification · binary checksums · constitutional compliance gate."
pai_cd:
  version: "2.2.3"
  status: "Canonical"
  source:
    file: "audit_checklist.md"
    path: "docs/audit_checklist.md"
    commit: "main"
    authority_repo: "PAI-Kernel/pai-kernel"
  cite_as: "PAI-Kernel SDK v2.2.3 · Audit Checklist"
  last_amendment: "2026-05-09"
---

# PAI-Kernel SDK · Audit Checklist

Adopter-runnable verification procedures for PAI-Kernel SDK v2.2.3. Each section is independently executable · all commands assume Rust 1.88+ toolchain installed.

> **Purpose:** enable independent third-party verification of build reproducibility · test correctness · cryptographic signature integrity · binary identity · constitutional compliance behavior. No internal access OR credentials required.

---

## Prerequisites

```sh
# Verify toolchain
rustc --version  # should be >= 1.88.0
cargo --version

# Optional but recommended
gpg --version
shasum --version  # macOS · OR sha256sum on Linux
jq --version
```

---

## 1 · Reproducible build verification

### 1.1 Clone to pinned tag

```sh
git clone https://github.com/PAI-Kernel/pai-kernel.git
cd pai-kernel
git checkout v2.2.3  # immutable tag · sha 77d564790c
```

### 1.2 Build full workspace · locked

```sh
cargo build --workspace --release --locked
# Expected: "Finished `release` profile [optimized] target(s)" with no errors
```

`--locked` flag ensures Cargo.lock matches expectations exactly · any drift causes immediate failure (build determinism gate).

### 1.3 Verify build artifacts

```sh
# Daemon binary should exist
ls -la target/release/pai_governance_daemon

# Compliance binary should exist
ls -la target/release/pai_compliance

# All workspace crates compiled to .rlib
ls target/release/deps/libpai_*.rlib | wc -l
# Expected: ~25-28 .rlib files (depending on workspace member count)
```

### 1.4 Cross-check version metadata

```sh
./target/release/pai_governance_daemon --version
# Expected: "pai-kernel 1.3.2"
```

---

## 2 · Test suite execution

### 2.1 Run all tests · workspace · all features · locked

```sh
cargo test --workspace --release --locked --all-features
# Expected: "test result: ok" for each crate · zero failures
```

Test execution time: ~2-5 minutes on typical adopter machine (depending on CPU).

### 2.2 Run constitutional compliance binary

```sh
cargo run -p pai_compliance --release --locked
# Expected: JSON output of T1..T6 invariant test results · all "pass": true
```

Save output for audit:

```sh
cargo run -p pai_compliance --release --locked --quiet > compliance_report.json
jq '.[] | select(.pass == false)' compliance_report.json
# Expected: empty (no failures)
```

### 2.3 Doc tests

```sh
cargo test --workspace --release --locked --doc
# Expected: "test result: ok"
```

---

## 3 · GPG signature verification

### 3.1 Import release signing key

```sh
gpg --recv-keys 95C4B50ED56544DC033B130DB84B6C860ABAD0B1
# OR import from keys/release_pubkey.asc if present in repo
```

Key fingerprint: `95C4 B50E D565 44DC 033B  130D B84B 6C86 0ABA D0B1`
Key holder: Mikhail Sergeev <Mikhail.Sergeev@PAIkernel.org>

### 3.2 Verify v2.2.3 release tag

```sh
git verify-tag v2.2.3
# Expected: "Good signature from Mikhail A. Sergeev"
```

### 3.3 Verify recent commits on main

```sh
# Verify last 5 commits all signed by release key
git log --show-signature -5 main 2>&1 | grep -E "^(commit|gpg:)" | head -20
```

Each commit should show `gpg: Good signature from "Mikhail A. Sergeev"` immediately after `commit <sha>` line.

---

## 4 · Binary checksum verification

### 4.1 Download release binary

```sh
PLATFORM="x86_64-apple-darwin"  # OR aarch64-apple-darwin · x86_64-unknown-linux-gnu · aarch64-unknown-linux-gnu
URL="https://github.com/PAI-Kernel/pai-kernel/releases/download/v2.2.3"

curl -L -o "pai_governance_daemon-v2.2.3-${PLATFORM}.tar.gz" \
  "${URL}/pai_governance_daemon-v2.2.3-${PLATFORM}.tar.gz"

curl -L -o "pai_governance_daemon-v2.2.3-${PLATFORM}.tar.gz.sha256" \
  "${URL}/pai_governance_daemon-v2.2.3-${PLATFORM}.tar.gz.sha256"
```

### 4.2 Verify SHA256 checksum

```sh
# macOS:
shasum -a 256 -c "pai_governance_daemon-v2.2.3-${PLATFORM}.tar.gz.sha256"

# Linux:
sha256sum -c "pai_governance_daemon-v2.2.3-${PLATFORM}.tar.gz.sha256"

# Expected: "OK"
```

### 4.3 Compare to built artifact

```sh
# Extract release tarball
tar -xzf "pai_governance_daemon-v2.2.3-${PLATFORM}.tar.gz" -C /tmp/release-binary

# Get hashes
shasum -a 256 /tmp/release-binary/pai_governance_daemon
shasum -a 256 ./target/release/pai_governance_daemon
```

If both hashes match · the released binary corresponds bit-for-bit to your locally-built binary. Note: matching requires identical Rust toolchain · same target triple · `--release --locked` build flags · no `RUSTFLAGS` overrides.

If hashes differ · this does NOT necessarily indicate tampering. Build determinism in Rust requires matching:

- Rust toolchain version (we use 1.88.0 per `rust-toolchain.toml`)
- Target triple (e.g., `x86_64-apple-darwin` · `aarch64-unknown-linux-gnu`)
- Build environment (some toolchain versions embed build timestamps)

---

## 5 · Container image verification (Cosign)

### 5.1 Verify Docker image signature via Sigstore Cosign

```sh
cosign verify ghcr.io/pai-kernel/pai-kernel:v2.2.3 \
  --certificate-identity https://github.com/PAI-Kernel/pai-kernel/.github/workflows/release.yml@refs/tags/v2.2.3 \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
```

Expected: `Verification for ghcr.io/pai-kernel/pai-kernel:v2.2.3 -- The following checks were performed: ... Verified OK`

### 5.2 Inspect image manifest

```sh
docker manifest inspect ghcr.io/pai-kernel/pai-kernel:v2.2.3 | jq
# Verify: media type · layers · architecture
```

### 5.3 Run smoke test in container

```sh
docker run --rm ghcr.io/pai-kernel/pai-kernel:v2.2.3 --version
# Expected: "pai-kernel 1.3.2"

docker run --rm -p 9100:9100 ghcr.io/pai-kernel/pai-kernel:v2.2.3 --demo &
sleep 2
curl -s http://127.0.0.1:9100/api/v1/health
# Expected: {"status":"ok",...}
kill %1
```

---

## 6 · Constitutional compliance runtime test

### 6.1 Daemon health check

```sh
# Start daemon in demo mode
pai_governance_daemon --demo > /tmp/daemon.log 2>&1 &
sleep 2

# Verify endpoints
curl -s http://127.0.0.1:9100/api/v1/version | jq
# Expected: {"version":"1.3.2","pai_cd_version":"3.1","rust_toolchain":"1.88"}

curl -s http://127.0.0.1:9100/api/v1/health | jq
# Expected: {"status":"ok","witness_entries":0,"conservative_mode":false}

# Cleanup
pkill -f pai_governance_daemon
```

### 6.2 Witness chain verification (CLI · standalone)

```sh
pai_governance_daemon verify
# Expected: "Witness chain: empty (no entries). OK." for fresh daemon
# OR: chain validation summary with entry count for daemon with history
```

This subcommand works without env vars OR running daemon · suitable for CI / batch verification.

### 6.3 Export full governance bundle

```sh
# Requires env vars (production setup)
export PAI_AUTHOR_API_KEY="$(uuidgen)"
export PAI_AUTHOR_SIGNING_KEY="$(openssl rand -hex 32)"

pai_governance_daemon export > governance_bundle.json
jq '.witness_log' governance_bundle.json  # entries
jq '.governance_state' governance_bundle.json  # state
```

---

## 7 · Crates.io verification

### 7.1 Verify all 18 published crates available

```sh
for crate in pai_api pai_classify pai_compliance pai_config pai_delegation pai_drift pai_export pai_gate pai_governance_daemon pai_harness pai_influence pai_interface pai_kernel pai_mcp pai_openai_adapter pai_policy pai_storage pai_witness; do
  echo "=== $crate ==="
  cargo search "$crate" --limit 1 2>&1 | grep "^$crate "
done
# Expected: all 18 show "= \"1.3.2\""
```

### 7.2 Install via cargo (independent path)

```sh
cargo install pai_kernel --version 1.3.2
which pai_governance_daemon
# Should be in $CARGO_HOME/bin
```

---

## 8 · Optional · Homebrew formula verification

### 8.1 Verify formula source

```sh
brew tap PAI-Kernel/tap
cat /opt/homebrew/Library/Taps/pai-kernel/homebrew-tap/Formula/pai-kernel.rb | head -40
```

### 8.2 Install + verify

```sh
brew install PAI-Kernel/tap/pai-kernel
brew test pai-kernel
# Expected: smoke test passes
brew info pai-kernel
# Expected: shows Caveats with environment setup guidance
```

---

## Summary checklist

After completing relevant sections, you should have verified:

- [ ] Reproducible source build from immutable v2.2.3 tag
- [ ] All workspace tests pass (`cargo test --workspace --release --locked --all-features`)
- [ ] Constitutional compliance gate passes (T1..T6 invariants)
- [ ] GPG signature on v2.2.3 tag matches release key
- [ ] Binary SHA256 checksum matches release artifact (or your built binary)
- [ ] Container image verifies via Cosign
- [ ] Daemon responds to API queries in demo mode
- [ ] All 18 crates available on crates.io at version 1.3.2
- [ ] (Optional) Homebrew formula installs cleanly

If all checks pass, you have independently verified PAI-Kernel SDK v2.2.3 corresponds to the published release artifacts · binary signatures match · runtime behavior matches documented surface.

---

## What this checklist does NOT cover

- **TLA+ formal verification reproduction** — TLA+ model and TLC results currently maintained in internal governance repository. Public TLA+ artifact publication remains a sustained item (depends on `formal/` directory public-vs-internal disposition).
- **Full audit-grade reproducibility** — bit-for-bit binary reproducibility across heterogeneous build environments requires additional pinning beyond `--locked` flag (toolchain version · build host · environment variables).
- **Adversarial security audit** — independent code review focused on vulnerability discovery is separate scope · contact maintainer for arrangements.
- **Constitutional document review** — auditing PAI-CD framework itself (six invariants · constitutional principles) is separate scope · see [`corpus/PAI_Constitutional_Document.md`](../corpus/PAI_Constitutional_Document.md).

---

## Reporting verification failures

If any checklist item fails on your system:

1. Re-verify prerequisites (Rust 1.88+ · cleanup any old toolchain)
2. Re-run failing step with verbose output (`cargo build --verbose` · `RUST_LOG=debug` for daemon)
3. Open GitHub issue with: failing step · OS + toolchain · full error output · steps taken to reproduce
4. Reference [`SUPPORT.md`](SUPPORT.md) for response expectations

---

## See also

- [`docs/INSTALL.md`](INSTALL.md) — installation guide (covers each install path in depth)
- [`docs/quickstart.md`](quickstart.md) — 5-minute getting-started
- [`docs/upgrade.md`](upgrade.md) — version upgrade procedures
- [`docs/RELEASE_NOTES_v2.2.3.md`](RELEASE_NOTES_v2.2.3.md) — v2.2.3 release notes
- [`docs/KNOWN_LIMITATIONS.md`](KNOWN_LIMITATIONS.md) — known limitations and scope
- [`SECURITY.md`](../SECURITY.md) — security policy + reporting
- [`SUPPORT.md`](SUPPORT.md) — support tiers and response expectations

---

*PAI-Kernel SDK · v2.2.3 · Constitutional Governance Framework for AI · MIT OR Apache-2.0 · Mikhail Sergeev / PAI-Kernel Initiative*
