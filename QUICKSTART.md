---
title: "Quickstart — PAI-Kernel SDK in 5 minutes"
slug: quickstart
position: 0
hidden: false
excerpt: "Five-minute path to running PAI-Kernel SDK locally · brew install · --demo mode · browser test · NO env setup required."
pai_cd:
  version: "2.2.3"
  status: "Canonical"
  source:
    file: "QUICKSTART.md"
    path: "QUICKSTART.md"
    commit: "v2.2.3"
    authority_repo: "PAI-Kernel/pai-kernel"
  cite_as: "PAI-Kernel SDK v2.2.3 · Quickstart"
  last_amendment: "2026-05-09"
---

> **Note:** this file = mirror of [`docs/quickstart.md`](docs/quickstart.md) · placed at root for discoverability (alongside `README.md` · `INSTALL.md` · `LICENSE`). Both files content-identical · either can be edited but should sync with the other.



# PAI-Kernel SDK · Quickstart

Five-minute path to running PAI-Kernel SDK locally with **no environment setup required**.

> **Goal:** evaluate fit · run a working daemon · query the API · understand surface. Total time: ~5 minutes wall-clock.

For comprehensive coverage (production setup · all install methods · troubleshooting · upgrades), see [`docs/INSTALL.md`](docs/INSTALL.md).

---

## Prerequisites

- macOS · Linux · OR Windows (Docker recommended for Windows)
- One of: `brew` · `cargo` · `docker` · `curl`

## Step 1 · Install (~30 seconds)

Pick the fastest path for your environment:

### macOS · Linux · Homebrew

```sh
brew install PAI-Kernel/tap/pai-kernel
```

### macOS · Linux · One-line installer (no Homebrew)

```sh
curl -fsSL https://paikernel.org/install.sh | sh
```

### Windows · Docker (recommended)

Native Windows binary support is provided through Docker (Docker Desktop OR Docker Engine via WSL2). This is the recommended Windows path — avoids Rust toolchain configuration on Windows and provides parity with Linux container runtime adopters use in production.

```powershell
# PowerShell · Windows 10+ with Docker Desktop installed
docker pull ghcr.io/pai-kernel/pai-kernel:v2.2.3
```

Alternative for Windows · WSL2 + Linux instructions (treats WSL2 as Linux):

```bash
# Within WSL2 Ubuntu/Debian:
curl -fsSL https://paikernel.org/install.sh | sh
```

### Any OS · Docker (cross-platform)

```sh
docker pull ghcr.io/pai-kernel/pai-kernel:v2.2.3
```

## Step 2 · Run in demo mode (~5 seconds)

Demo mode requires NO environment variables · uses ephemeral keys · binds to 127.0.0.1:9100 only · prints prominent stderr warnings.

```sh
pai_governance_daemon --demo
```

You will see four `WARNING` lines on stderr (this is expected · demo mode is NOT for production), followed by:

```json
{"level":"INFO","fields":{"message":"PAI-Kernel Governance Sidecar v1.3.2 listening on 127.0.0.1:9100","addr":"127.0.0.1:9100"},...}
```

Leave this terminal open.

### Docker variant

```sh
docker run --rm -it -p 9100:9100 ghcr.io/pai-kernel/pai-kernel:v2.2.3 --demo
```

## Step 3 · Query the API (~10 seconds)

In a new terminal, verify the daemon responds:

```sh
curl -s http://127.0.0.1:9100/api/v1/version
```

Expected response:

```json
{"version":"1.3.2","pai_cd_version":"3.1","rust_toolchain":"1.88"}
```

```sh
curl -s http://127.0.0.1:9100/api/v1/health
```

Expected response:

```json
{"status":"ok","witness_entries":0,"conservative_mode":false}
```

## Step 4 · Explore the API surface (~2 minutes)

Five endpoints are exposed in v2.2.3:

| Endpoint | Description |
|---|---|
| `GET /api/v1/version` | SDK version, PAI-CD revision, Rust toolchain |
| `GET /api/v1/health` | Health status, witness entries count, conservative mode flag |
| `GET /api/v1/log` | Witness log entries (chronological) |
| `GET /api/v1/log/verify` | Verify witness log integrity (hash chain check) |
| `GET /api/v1/export` | Full governance bundle export as JSON |

Try the witness log verification:

```sh
curl -s http://127.0.0.1:9100/api/v1/log/verify | jq
```

## Step 5 · Try the example binaries (~2 minutes)

Five reference examples demonstrate individual constitutional invariants. Each runs in seconds and prints structured output:

```sh
# After cargo install pai_kernel OR brew install:
cargo run -p pai_examples --bin classify_output --release
cargo run -p pai_examples --bin causal_graph --release
cargo run -p pai_examples --bin evasion_audit --release
cargo run -p pai_examples --bin vulnerability_check --release
cargo run -p pai_examples --bin compliance_identity --release
```

See [`examples/README.md`](examples/README.md) for what each demonstrates.

---

## Stop the daemon

In the terminal where `pai_governance_daemon --demo` is running, press `Ctrl-C`. Demo mode state is in-memory only · nothing persists.

## Next steps

You have just verified PAI-Kernel SDK installs · runs · responds to API queries. From here:

- **Configure for production:** see [`docs/INSTALL.md`](docs/INSTALL.md) § Configuration · set required `PAI_AUTHOR_API_KEY` and `PAI_AUTHOR_SIGNING_KEY` environment variables (32-byte hex via `openssl rand -hex 32`).
- **Initialize a working directory:** `pai_governance_daemon init` creates `./pai-kernel.toml` + `./policies/` for persistent deployment.
- **Read the constitutional foundation:** [`corpus/PAI_Constitutional_Document.md`](corpus/PAI_Constitutional_Document.md) describes the framework PAI-Kernel SDK enforces.
- **Review release notes:** [`docs/RELEASE_NOTES_v2.2.3.md`](docs/RELEASE_NOTES_v2.2.3.md) for v2.2.3 specifics.
- **Check known limitations:** [`docs/KNOWN_LIMITATIONS.md`](docs/KNOWN_LIMITATIONS.md) describes scope and current constraints.

## Troubleshooting

### `pai_governance_daemon: command not found` after `brew install`

`brew install` should add `/opt/homebrew/bin` to PATH automatically. If not, run:

```sh
echo 'export PATH="/opt/homebrew/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

### Daemon exits immediately without `--demo`

Expected behavior · fail-closed default. Either use `--demo` for testing OR set `PAI_AUTHOR_API_KEY` + `PAI_AUTHOR_SIGNING_KEY` environment variables. See [`docs/INSTALL.md`](docs/INSTALL.md) § Configuration.

### Browser cannot connect to 127.0.0.1:9100

Verify daemon is running:

```sh
lsof -iTCP:9100 -sTCP:LISTEN
```

Empty output = daemon not started. Start with `--demo` OR with required env vars. See [`docs/INSTALL.md`](docs/INSTALL.md) § Troubleshooting.

---

*PAI-Kernel SDK · v2.2.3 · Constitutional Governance Framework for AI · MIT OR Apache-2.0 · Mikhail Sergeev / PAI-Kernel Initiative*
