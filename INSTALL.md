---
title: "Installing PAI-Kernel SDK · v2.2.3"
slug: install
position: 0
hidden: false
excerpt: "Six install methods (one-line · Homebrew · Cargo · Docker · Manual · Source) · cross-platform · production + demo modes."
pai_cd:
  version: "2.2.3"
  status: "Canonical"
  source:
    file: "INSTALL.md"
    path: "docs/INSTALL.md"
    commit: "v2.2.3"
    authority_repo: "PAI-Kernel/pai-kernel"
  cite_as: "PAI-Kernel SDK v2.2.3 · Install Guide"
  last_amendment: "2026-05-09"
---

# Installing PAI-Kernel SDK

This guide covers installation of **PAI-Kernel SDK** (Constitutional Governance Framework for AI · v2.2.3) on macOS, Linux, and Windows. PAI-Kernel ships as a sidecar daemon (`pai_governance_daemon`) with HTTP API, integrated CLI, policy engine, and storage backend.

> **Quick start:**
>
> ```sh
> curl -fsSL https://paikernel.org/install.sh | sh
> ```
>
> This downloads the v2.2.3 binary for your platform, verifies SHA256, and installs to `$HOME/.local/pai-kernel/`.

---

## Table of Contents

1. [Installation methods](#installation-methods)
2. [Prerequisites](#prerequisites)
3. [Method 1: One-line installer (recommended)](#method-1-one-line-installer-recommended)
4. [Method 2: Homebrew (macOS · Linux)](#method-2-homebrew-macos--linux)
5. [Method 3: Cargo (Rust toolchain users)](#method-3-cargo-rust-toolchain-users)
6. [Method 4: Docker (any OS · Windows recommended)](#method-4-docker-any-os--windows-recommended)
7. [Method 5: Manual binary download](#method-5-manual-binary-download)
8. [Method 6: Build from source (developers · auditors)](#method-6-build-from-source-developers--auditors)
9. [Configuration (required for production)](#configuration-required-for-production)
10. [Demo mode (no configuration required)](#demo-mode-no-configuration-required)
11. [Verification](#verification)
12. [Updating](#updating)
13. [Uninstallation](#uninstallation)
14. [Troubleshooting](#troubleshooting)

---

## Installation methods

| Method | Audience | Prerequisites | Production-ready |
|---|---|---|---|
| **One-line installer** | All users · fastest path | `curl`, `tar`, `uname` | Yes |
| **Homebrew** | macOS · Linux users with brew | Homebrew | Yes |
| **Cargo** | Rust developers | Rust 1.88+ | Yes |
| **Docker** | Container deployments · Windows | Docker | Yes |
| **Manual binary** | Air-gapped · custom paths | None (after download) | Yes |
| **Build from source** | Auditors · contributors · custom builds | Rust 1.88+ · git | Yes |

All methods install the same `pai_governance_daemon` binary (v2.2.3 · GPG-signed releases · SHA256-verified).

---

## Prerequisites

### Operating system support

| OS | Architecture | Method support |
|---|---|---|
| macOS 11+ | Intel (x86_64) + Apple Silicon (aarch64) | All methods |
| Linux (glibc 2.31+) | x86_64 + aarch64 | All methods |
| Windows 10+ | x86_64 | Docker recommended · manual binary supported |
| WSL2 (Linux on Windows) | x86_64 + aarch64 | All methods (treated as Linux) |

### Tool requirements (Method 1 · 5)

- `curl` (or `wget`)
- `tar` (xz support recommended)
- `sha256sum` (or `shasum -a 256` on macOS)
- `uname`

### Tool requirements (Method 3 · 6)

- Rust toolchain 1.88+ (`rustup install stable` · `rustc --version`)
- `git` (Method 6 only)
- C compiler (for `rusqlite` native bindings · pre-installed on most systems)

### Disk space

- Compiled binary (after install): ~30-45 MB
- Installation directory (`$HOME/.local/pai-kernel/`): ~50 MB (binary + default config + policies)
- Build from source (Method 6): ~3-5 GB target/ directory

---

## Method 1: One-line installer (recommended)

**For most users** · cross-platform · auto-detects OS/arch · verifies SHA256 · no Rust toolchain required.

```sh
curl -fsSL https://paikernel.org/install.sh | sh
```

**What it does:**

1. Detects OS + architecture (`uname`)
2. Downloads `pai_governance_daemon-v2.2.3-{arch}-{os}.tar.gz` from GitHub Releases
3. Verifies SHA256 checksum against `.tar.gz.sha256`
4. Extracts to `$HOME/.local/pai-kernel/`
5. Optionally appends `$HOME/.local/pai-kernel/bin` to your `PATH`

**Environment overrides:**

```sh
# Install specific version
PAI_KERNEL_VERSION=v2.2.3 curl -fsSL https://paikernel.org/install.sh | sh

# Custom install directory
PAI_KERNEL_INSTALL_DIR=/opt/pai-kernel curl -fsSL https://paikernel.org/install.sh | sh

# Skip PATH modification (advanced users)
PAI_KERNEL_ADD_TO_PATH=no curl -fsSL https://paikernel.org/install.sh | sh
```

**Pinned-version variant (recommended for production):**

```sh
curl -fsSL https://raw.githubusercontent.com/PAI-Kernel/pai-kernel/v2.2.3/install.sh | sh
```

This pins the install script to v2.2.3 for reproducibility (vs `paikernel.org/install.sh` which always points to the current release).

---

## Method 2: Homebrew (macOS · Linux)

```sh
brew tap PAI-Kernel/tap
brew install pai-kernel
```

**Verification:**

```sh
brew info pai-kernel
pai_governance_daemon --version
```

**Updating:**

```sh
brew upgrade pai-kernel
```

---

## Method 3: Cargo (Rust toolchain users)

**Prerequisite:** Rust 1.88+ installed (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`).

```sh
cargo install pai_kernel --version 1.3.2
```

**What it does:**

1. Compiles `pai_kernel` binary from `crates.io` (full source build)
2. Installs to `$CARGO_HOME/bin/pai_kernel` (typically `~/.cargo/bin/`)
3. Pulls 18 published PAI-Kernel crates as dependencies (deterministic build)

**Verification:**

```sh
which pai_governance_daemon
pai_governance_daemon --version
# pai-kernel 1.3.2
```

Note: the `pai_kernel` crate provides the `pai_governance_daemon` binary (binary name differs from crate name).

**Updating:**

```sh
cargo install pai_kernel --version 1.3.3 --force   # adjust version as needed
```

---

## Method 4: Docker (any OS · Windows recommended)

```sh
docker pull ghcr.io/pai-kernel/pai-kernel:v2.2.3
```

**Run interactively:**

```sh
docker run --rm -it -p 9100:9100 \
  -e PAI_AUTHOR_API_KEY="your-api-key" \
  -e PAI_AUTHOR_SIGNING_KEY=$(openssl rand -hex 32) \
  -v $(pwd)/pai-data:/data \
  -w /data \
  ghcr.io/pai-kernel/pai-kernel:v2.2.3
```

**Run as daemon (production · systemd-friendly):**

```sh
docker run -d --name pai-kernel \
  --restart unless-stopped \
  -p 9100:9100 \
  -e PAI_AUTHOR_API_KEY="your-api-key" \
  -e PAI_AUTHOR_SIGNING_KEY="$(cat /etc/pai-kernel/signing.key)" \
  -v /var/lib/pai-kernel:/data \
  -w /data \
  ghcr.io/pai-kernel/pai-kernel:v2.2.3
```

(Volume mount provides persistent storage for `pai-kernel.db` + `policies/` · `-w /data` sets working directory. The daemon reads `pai-kernel.toml` from the working directory.)

**Tags available:**

- `:v2.2.3` (recommended · pinned)
- `:v2.2` (minor-version stream · auto-updates within 2.2.x)
- `:latest` (NOT recommended for production · always points to the newest release)
- All tags GPG-signed via Cosign (Sigstore) · verifiable via `cosign verify`

---

## Method 5: Manual binary download

For air-gapped systems · custom paths · or when scripts cannot run.

1. Visit https://github.com/PAI-Kernel/pai-kernel/releases/tag/v2.2.3
2. Download the appropriate archive for your platform:
   - `pai_governance_daemon-v2.2.3-x86_64-apple-darwin.tar.gz` (macOS Intel)
   - `pai_governance_daemon-v2.2.3-aarch64-apple-darwin.tar.gz` (macOS Apple Silicon)
   - `pai_governance_daemon-v2.2.3-x86_64-unknown-linux-gnu.tar.gz` (Linux x86_64)
   - `pai_governance_daemon-v2.2.3-aarch64-unknown-linux-gnu.tar.gz` (Linux ARM64)
3. Download the matching `.sha256` file
4. Verify SHA256:
   ```sh
   shasum -a 256 -c pai_governance_daemon-v2.2.3-x86_64-apple-darwin.tar.gz.sha256
   ```
5. Extract:
   ```sh
   mkdir -p $HOME/.local/pai-kernel
   tar -xzf pai_governance_daemon-v2.2.3-*.tar.gz -C $HOME/.local/pai-kernel
   ```
6. Add to PATH:
   ```sh
   echo 'export PATH="$HOME/.local/pai-kernel/bin:$PATH"' >> ~/.zshrc
   source ~/.zshrc
   ```

---

## Method 6: Build from source (developers · auditors)

For contributors, security auditors, or users requiring custom builds.

```sh
git clone https://github.com/PAI-Kernel/pai-kernel.git
cd pai-kernel
git checkout v2.2.3   # pin to release tag
cargo build --workspace --release --locked
```

**Verification:**

```sh
cargo test --workspace --locked
./target/release/pai_governance_daemon --version
```

**Reproducibility:**

PAI-Kernel SDK builds are reproducible bit-for-bit when:
- Rust toolchain matches MSRV (1.88+)
- Build flags omitted (no `RUSTFLAGS` overrides)
- `--locked` flag used (Cargo.lock pinned)
- Same target triple

The v2.2.3 release commit `a1bf95cf1ce2caf7b37640af4f7081f79441c843` provides a verified reproducibility anchor (captured 2026-05-06).

---

## Configuration (required for production)

PAI-Kernel SDK applies **fail-closed environment variable defaults**: the daemon **refuses to start** if required environment variables are missing. See `SECURITY.md` for security rationale.

### Required environment variables

The daemon recognizes **two** required environment variables. All other configuration goes via the TOML config file (see § Configuration file below).

| Variable | Description | Example |
|---|---|---|
| `PAI_AUTHOR_API_KEY` | API key string for author authentication | `PAI_AUTHOR_API_KEY="my-secret-key"` |
| `PAI_AUTHOR_SIGNING_KEY` | 32-byte hex string (Ed25519 signing key) | `PAI_AUTHOR_SIGNING_KEY=$(openssl rand -hex 32)` |

### Generating keys

```sh
# Generate 32-byte Ed25519 signing key (one-time setup)
export PAI_AUTHOR_SIGNING_KEY=$(openssl rand -hex 32)

# Choose your API key (any sufficiently random string · UUID example)
export PAI_AUTHOR_API_KEY=$(uuidgen)
```

For persistence across shells, add to `~/.zshrc` OR `~/.bashrc`:

```sh
echo "export PAI_AUTHOR_API_KEY='$(uuidgen)'" >> ~/.zshrc
echo "export PAI_AUTHOR_SIGNING_KEY='$(openssl rand -hex 32)'" >> ~/.zshrc
```

**SECURITY:** treat both keys as cryptographic secrets. Use a secret manager (1Password CLI, macOS Keychain, AWS Secrets Manager, etc) for production deployments. NEVER commit keys к version control.

### Optional environment variables

The current daemon (v2.2.3) recognizes only the two required env vars above. All other settings (bind address · port · storage backend · policy directory · log level · etc) are configured via the TOML config file (see next section).

### Configuration file

The daemon reads `./pai-kernel.toml` by default. Generate a starter config:

```sh
pai_governance_daemon init
```

This creates `./pai-kernel.toml` + `./policies/placeholder.rego` in the current directory. Default `pai-kernel.toml` structure:

```toml
[server]
bind = "127.0.0.1"
port = 9100
tls = false

[storage]
backend = "sqlite"
sqlite_path = "./pai-kernel.db"

[policy]
rego_dir = "./policies/"
reload_interval_secs = 30

[drift]
objective_changes_30d = 5
classification_changes_30d = 3
consecutive_upgrades_without_gap = 2

[logging]
level = "info"
format = "json"
```

Adjust values to match your deployment. Note: signing keys live ONLY in environment variables · NEVER in the TOML file (security).

Reference daemon с config file:

```sh
pai_governance_daemon --config ./pai-kernel.toml
```

Or with the brew-shipped default config:

```sh
pai_governance_daemon --config /opt/homebrew/opt/pai-kernel/share/pai-kernel/pai-kernel.toml
```

---

## Demo mode (no configuration required)

> **Quick start prerequisite:** to test the daemon без environment variable setup, the `--demo` flag is the **minimum** path. Running bare `pai_governance_daemon` without flags AND without env vars will exit immediately (fail-closed default).

For evaluation, testing, or quick exploration:

```sh
pai_governance_daemon --demo
```

**What `--demo` does:**

- Generates ephemeral signing key (lost on restart)
- Forces bind to `127.0.0.1:9100` (loopback only · NOT network-accessible)
- Uses in-memory storage (no SQLite · state lost on restart)
- Prints 4 prominent stderr WARNING lines:
  ```
  ⚠ DEMO MODE · ephemeral signing key (lost on restart)
  ⚠ DEMO MODE · 127.0.0.1 force-bind (not network-accessible)
  ⚠ DEMO MODE · in-memory storage (state not persisted)
  ⚠ DEMO MODE · NOT SUITABLE для production deployment
  ```

**When to use:**
- Initial exploration · `curl` API endpoints · understand surface
- Quick local testing · CI smoke tests
- Workshop · demonstration · educational settings

**When NOT to use:**
- Production deployments
- Multi-user environments
- Persistent state required
- Network-exposed services

---

## Verification

> **Prerequisite:** ensure the daemon is running first (via `--demo` OR exported env vars). Running these commands against a stopped daemon returns connection-refused.

### Daemon health check (HTTP API)

```sh
curl -s http://127.0.0.1:9100/api/v1/version
# {"version":"1.3.2","pai_cd_version":"3.1","rust_toolchain":"1.88"}
```

```sh
curl -s http://127.0.0.1:9100/api/v1/health
# {"status":"ok","witness_entries":0,"conservative_mode":false}
```

### Available API endpoints

| Endpoint | Method | Description |
|---|---|---|
| `/api/v1/version` | GET | Version + PAI-CD revision + Rust toolchain |
| `/api/v1/health` | GET | Health status + witness count + conservative mode flag |
| `/api/v1/log` | GET | Witness log entries |
| `/api/v1/log/verify` | GET | Verify witness log integrity |
| `/api/v1/export` | GET | Full governance bundle export (JSON) |

### Witness chain verification (CLI · standalone)

The verify subcommand works without a running daemon AND without env vars:

```sh
pai_governance_daemon verify
# Witness chain: empty (no entries). OK.
```

### Export state (CLI · requires env vars)

```sh
export PAI_AUTHOR_API_KEY="..."
export PAI_AUTHOR_SIGNING_KEY="$(openssl rand -hex 32)"
pai_governance_daemon export > state.json
# JSON bundle: witness_log + governance_state + capability_registry etc
```

### GPG signature verification (release artifacts)

```sh
# Import release signing key (one-time)
gpg --recv-keys 95C4B50ED56544DC033B130DB84B6C860ABAD0B1

# Verify release tag
git verify-tag v2.2.3

# Verify Docker image (Cosign)
cosign verify ghcr.io/pai-kernel/pai-kernel:v2.2.3 \
  --certificate-identity https://github.com/PAI-Kernel/pai-kernel/.github/workflows/release.yml@refs/tags/v2.2.3 \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
```

### Compliance test suite (CTS)

```sh
# Reference compliance fixtures (separate `pai_compliance` binary · cargo install pai_compliance)
pai_compliance
# ✓ T1..T6 invariants verified · 0 violations · CTS PASS
```

---

## Updating

### Method 1 (One-line installer)

```sh
curl -fsSL https://paikernel.org/install.sh | sh
# Auto-installs current release (overwrites $HOME/.local/pai-kernel/)
```

### Method 2 (Homebrew)

```sh
brew upgrade pai-kernel
```

### Method 3 (Cargo)

```sh
cargo install pai_kernel --version <NEW_VERSION> --force
```

### Method 4 (Docker)

```sh
docker pull ghcr.io/pai-kernel/pai-kernel:<NEW_VERSION>
docker stop pai-kernel && docker rm pai-kernel
# Re-run with new image (see Method 4 above)
```

**State preservation:** all upgrade methods preserve the working directory (`pai-kernel.db` + `policies/` + `pai-kernel.toml`) since storage paths are config-relative, not env-driven. Witness chain integrity is verified at daemon startup via hash-chain validation; tampering or corruption fails-closed (daemon refuses to start).

---

## Uninstallation

### Method 1 (One-line installer)

```sh
rm -rf $HOME/.local/pai-kernel
# Remove PATH entry from ~/.zshrc OR ~/.bashrc manually
```

### Method 2 (Homebrew)

```sh
brew uninstall pai-kernel
brew untap PAI-Kernel/tap   # optional · removes tap reference
```

### Method 3 (Cargo)

```sh
cargo uninstall pai_kernel
```

### Method 4 (Docker)

```sh
docker stop pai-kernel
docker rm pai-kernel
docker rmi ghcr.io/pai-kernel/pai-kernel:v2.2.3
# Remove data directory if no longer needed
rm -rf /var/lib/pai-kernel
```

**Data preservation:** uninstallation does NOT automatically remove the working directory containing `pai-kernel.db` + `policies/` + `pai-kernel.toml`. To preserve witness chain · policies · state for forensic analysis OR future re-deployment · keep this directory. To fully remove · delete it manually after uninstall.

---

## Troubleshooting

### Daemon exits immediately · "missing PAI_AUTHOR_API_KEY environment variable"

**Cause:** fail-closed default applied. The daemon refuses to start without both required env vars OR the `--demo` flag.

**Fix · production path:** set both env vars and re-run:

```sh
export PAI_AUTHOR_API_KEY="your-api-key"
export PAI_AUTHOR_SIGNING_KEY=$(openssl rand -hex 32)
pai_governance_daemon
```

**Fix · quick test path:** use demo mode (ephemeral keys · 127.0.0.1 only):

```sh
pai_governance_daemon --demo
```

### Browser cannot connect to 127.0.0.1:9100

**Cause:** daemon not running. Confirm with:

```sh
lsof -iTCP:9100 -sTCP:LISTEN
# Empty output = nothing listening = daemon not started
```

**Fix:** start the daemon via `--demo` OR with env vars (see entry above).

### `cargo install pai_kernel` fails · "could not find rusqlite"

**Cause:** missing C compiler OR libsqlite3 development headers.

**Fix (macOS):** `xcode-select --install`

**Fix (Linux):** `sudo apt install build-essential libsqlite3-dev` (Debian/Ubuntu) OR equivalent

### `curl https://paikernel.org/install.sh` returns 404

**Cause:** DNS not yet propagated OR temporary outage.

**Fix:** use direct GitHub URL:

```sh
curl -fsSL https://raw.githubusercontent.com/PAI-Kernel/pai-kernel/v2.2.3/install.sh | sh
```

### Witness chain verification fails post-upgrade

**Cause:** daemon detected hash-chain inconsistency (tampering OR partial write OR file system corruption).

**Investigation:**

```sh
pai_governance_daemon verify
# Reports specific entry that fails verification
# (--verbose flag not yet implemented · output is single-line summary in v2.2.3)
```

**Recovery:** restore working directory (`pai-kernel.db` + `policies/`) from backup. NEVER manually edit witness chain (immutability invariant violated · PAI-CD § Decision Log Principles).

### Docker image fails Cosign verification

**Cause:** image not from official `ghcr.io/pai-kernel/pai-kernel` namespace OR Cosign certificate issue.

**Fix:** verify image source · check Cosign `--certificate-identity` URL matches release workflow.

### High memory usage / slow startup

**Cause:** witness chain large (>100K entries) OR policy directory has many `.rego` files.

**Investigation:**

```sh
du -sh ./pai-kernel.db ./policies/
ls -la ./policies/
```

(Adjust paths if running from a different working directory than where `pai_governance_daemon init` was invoked.)

**Optimization:**
- Archive old witness entries (post-90-day retention) к separate storage
- Consolidate policy files (combine related `.rego` rules)
- Increase available memory (recommended: 200 MB RAM minimum)

---

## Further reading

- **Quick start tutorial:** `docs/quickstart.md`
- **HTTP API reference:** https://docs.rs/pai_api (auto-generated from source)
- **PAI Constitutional Document:** `PAI_Constitutional_Document.md` (foundational)
- **TLA+ verification:** `formal/VERIFICATION_MATRIX.md`
- **Release notes v2.2.3:** `docs/RELEASE_NOTES_v2.2.3.md`

## Support

- **Issues / Bug reports:** https://github.com/PAI-Kernel/pai-kernel/issues
- **Discussions:** https://github.com/PAI-Kernel/pai-kernel/discussions
- **Security disclosures:** see `SECURITY.md` (responsible disclosure protocol)

## Constitutional foundation

PAI-Kernel SDK enforces the **Constitutional Document for Personal AI** (`PAI_Constitutional_Document.md`). Adopters running pai-kernel preserve:

- **Authorial Sovereignty (P1):** state export available regardless of mode
- **Pre-Execution Classification (MP-2):** advisory-only outputs separated from structural decisions
- **Decision Log Principles (P2 · P4):** append-only witness chain · hash-linked tamper-evidence
- **Provider Independence (P4):** services-around-OSS license model · no provider lock-in
- **Six Core Invariants [NON-DEROGABLE]:** preserved through 4-layer enforcement (TLA+ · SPARK · Rego · Runtime)

For the benefit of all living beings.

---

*PAI-Kernel SDK · v2.2.3 · Constitutional Governance Framework for AI · MIT OR Apache-2.0 · Mikhail Sergeev / PAI-Kernel Initiative*
