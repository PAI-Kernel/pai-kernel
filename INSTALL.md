---
title: "Install Guide — PAI-Kernel v2.2.1 Early Preview"
slug: install
position: 0
hidden: false
excerpt: "Cross-platform build + run guide; includes Ollama side-by-side demo. ~30-60 min first install."
pai_cd:
  version: "2.2.1"
  status: "Canonical"
  source:
    file: "INSTALL.md"
    path: "INSTALL.md"
    commit: "v2.2.1"
    authority_repo: "PAI-Kernel/pai-kernel"
  cite_as: "PAI-CD v2.2.1 · Install Guide"
  last_amendment: "2026-04-23"
---

# Install Guide — PAI-Kernel v2.2.1 Early Preview

> **Early preview · invitation-only distribution.** This is not production-ready software. It is a governance substrate for AI systems; AI-model wiring is v3.1 roadmap. Adopters running this release observe PAI-Kernel and Ollama running side-by-side — see [§ 10 · What you're seeing](#-10--what-youre-seeing-level-1-demo-mode) for the honest L1 framing.

---

## TL;DR — 30-second view (experienced users)

```bash
# Prerequisites: Rust 1.86+ toolchain, 8 GB RAM, 20 GB disk, working internet.

# 1. Get source
git clone https://github.com/PAI-Kernel/pai-kernel.git
cd pai-kernel
git checkout v2.2.1

# 2. Build SDK
cargo build --workspace --release

# 3. Run daemon (Terminal 1)
./target/release/pai_governance_daemon

# 4. (optional, Terminal 2) Install Ollama + run chat for side-by-side demo
ollama pull llama3.2
ollama run llama3.2

# 5. (optional, Terminal 3) Inspect SDK state while chat is running
curl http://127.0.0.1:9100/api/v1/state | jq
```

If everything above ran without errors you're done. Continue reading if any step failed, or for the full walk-through.

**Install via crates.io** is an alternative for people who want SDK only (no source tree):

```bash
cargo install pai-kernel
pai_governance_daemon --config /path/to/pai-kernel.toml
```

---

## § 1 · Overview

### 1.1 What you'll have at the end

Running the steps in this guide produces:

- A compiled PAI-Kernel governance daemon binary listening on `127.0.0.1:9100`
- A local SQLite database (`./pai-kernel.db`) storing witness-chain entries and governance state
- An Ollama installation with at least one open-weights language model (default recommendation: `llama3.2`)
- Three parallel processes running side-by-side: (a) governance daemon, (b) Ollama chat, (c) inspection tool (browser or `curl`)
- Optionally: the PAI-Console React UI running at `http://127.0.0.1:3000`, visualizing governance state

### 1.2 What this release does NOT do

- **PAI-Kernel does NOT mediate Ollama's responses.** Chat messages flow directly between you and the Ollama process; they are not routed through the governance daemon in v2.2.1.
- **No witness-chain entries are auto-populated from Ollama chat.** Witness entries come from direct SDK calls you make.
- **No Conservative Mode blocking** of AI output. That's a future SDK-integration feature.
- **No production-hardened defaults.** Bind is localhost-only; no TLS by default; no multi-tenant.

See `KNOWN_LIMITATIONS.md` in the repository root for the full scope statement.

### 1.3 Time required

| Stage | First-time | Subsequent |
|---|---|---|
| Prerequisites install (Rust, build tools) | 15-30 min | 0 (already installed) |
| Clone + build SDK | 5-10 min | 2-3 min |
| Configure + run daemon | 2-5 min | 1 min |
| Install Ollama + pull model | 10-20 min (download size) | 0 |
| First successful test | 5 min | 2 min |
| **Total (first time)** | **30-60 min** | **5 min** |

### 1.4 Prerequisites summary

- **OS:** macOS 12+ · Windows 10/11 · Linux (Ubuntu 22+, Fedora 38+, Debian 11+)
- **CPU:** x86_64 or aarch64 (Apple Silicon works natively)
- **RAM:** 8 GB minimum (16 GB recommended for running 8B-parameter Ollama models)
- **Disk:** 20 GB free (≈ 5 GB for Rust toolchain + build artifacts, ≈ 10 GB for Ollama models, rest headroom)
- **Network:** first-time download of Rust toolchain + crates + Ollama + models (~5-10 GB first-time total)
- **Shell familiarity:** able to run commands in Terminal (macOS/Linux) or PowerShell (Windows)

### 1.5 Architecture at a glance

```
┌──────────────────────────────────────────────────────────────┐
│  Your machine                                                │
│                                                              │
│  ┌────────────────────────┐  ┌────────────────────────┐      │
│  │ pai_governance_daemon  │  │ ollama serve           │      │
│  │ axum HTTP              │  │ local LLM runtime      │      │
│  │ :9100 localhost        │  │ :11434 localhost       │      │
│  │                        │  │                        │      │
│  │ witness chain          │  │ llama3.2 / qwen2.5     │      │
│  │ consent gates          │  │ / mixtral / ...        │      │
│  │ drift monitor          │  │                        │      │
│  │ export bundle          │  │                        │      │
│  └────────────────────────┘  └────────────────────────┘      │
│              ▲                           ▲                   │
│              │ JSON API                  │ chat              │
│              │                           │                   │
│  ┌────────────────────────┐  ┌────────────────────────┐      │
│  │ curl / browser         │  │ ollama run             │      │
│  │ pai-console UI         │  │ (your prompts)         │      │
│  └────────────────────────┘  └────────────────────────┘      │
│                                                              │
│  Note: no wire between the two boxes in v2.2.1.              │
│  SDK + model integration is future roadmap.                  │
└──────────────────────────────────────────────────────────────┘
```

---

## § 2 · Prerequisites per OS

### 2.1 macOS (12 Monterey or later)

Install Xcode Command Line Tools if not already present:

```bash
xcode-select --install
```

Confirm the install:

```bash
xcode-select -p
# Expected: /Applications/Xcode.app/Contents/Developer   (or similar path)
```

Install Rust via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Accept defaults. Follow instructions to source the cargo environment.

source "$HOME/.cargo/env"
rustc --version
# Expected: rustc 1.86.0 (...) or later
```

(Optional) Homebrew for Ollama install later:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

---

### 2.2 Windows (10 or 11 · PowerShell)

Install Visual Studio Build Tools 2022 (the C++ workload is required):

1. Download from <https://visualstudio.microsoft.com/downloads/> → "Build Tools for Visual Studio 2022"
2. Run the installer
3. Select the workload: **"Desktop development with C++"**
4. Install (download size ~5 GB, install size ~8 GB)

Install Rust via `rustup-init.exe`:

1. Download <https://win.rustup.rs/x86_64> (or `aarch64` on ARM64 Windows)
2. Run `rustup-init.exe`
3. Accept defaults (stable toolchain, MSVC)
4. Open a **new** PowerShell window after install

```powershell
rustc --version
# Expected: rustc 1.86.0 (...) or later
```

(Recommended) PowerShell 7+:

```powershell
winget install Microsoft.PowerShell
# Or download from https://github.com/PowerShell/PowerShell/releases
```

---

### 2.3 Linux

**Debian / Ubuntu (22.04+):**

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev git curl
```

**Fedora / RHEL / Rocky:**

```bash
sudo dnf install -y gcc gcc-c++ make pkgconfig openssl-devel git curl
```

**Arch / Manjaro:**

```bash
sudo pacman -S --needed base-devel openssl git curl
```

Install Rust via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version
# Expected: rustc 1.86.0 (...) or later
```

---

### 2.4 Verify Rust toolchain version

PAI-Kernel pins Rust 1.86.0 via `rust-toolchain.toml` in the repository. `rustup` should auto-install this version when you enter the repo directory. If you see older version warnings during build:

```bash
rustup update stable
rustup show
```

---

## § 3 · Step 1 · Clone the repository

```bash
git clone https://github.com/PAI-Kernel/pai-kernel.git
cd pai-kernel
git checkout v2.2.1
```

Verify you're on the right tag:

```bash
git describe --tags
# Expected: v2.2.1

git log -1 --oneline
# Expected: some commit SHA (this is the v2.2.1 tagged commit)
```

---

## § 4 · Step 2 · Build the SDK

From the repository root:

```bash
cargo build --workspace --release
```

**Expected time:**

| Platform | First build | Incremental |
|---|---|---|
| macOS (M1/M2/M3) | 5-8 min | 10-30 sec |
| macOS Intel | 8-12 min | 30-60 sec |
| Windows | 10-15 min | 1-2 min |
| Linux (8+ cores) | 5-10 min | 10-30 sec |

**Expected final output lines (abbreviated):**

```
   Compiling pai_api v1.3.0
   Compiling pai_kernel v1.3.0 (/path/to/pai-kernel/runtime/pai_kernel)
    Finished `release` profile [optimized] target(s) in 5m 34s
```

**Verification:**

```bash
./target/release/pai_governance_daemon --version
# Expected:
# PAI-Kernel Governance Sidecar v1.3.0
# PAI-CD: v3.1
# Rust: 1.86.0
```

(On Windows use `.\target\release\pai_governance_daemon.exe --version`.)

**Optional: run the test suite:**

```bash
cargo test --workspace --release
# Expected: test result: ok. 262 passed; 0 failed
```

If all tests pass, your build environment is healthy.

---

## § 5 · Step 3 · Configure and run the governance daemon

### 5.1 Default config file

The repository root contains `pai-kernel.toml` with default settings:

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

This default is safe for local testing. Adjust later if needed.

### 5.2 Run the daemon

**Terminal 1** — keep this running during the test:

```bash
./target/release/pai_governance_daemon --config ./pai-kernel.toml
```

**Expected log output** (JSON-formatted):

```
{"timestamp":"2026-04-23T14:00:12.345Z","level":"INFO","message":"Loading configuration from ./pai-kernel.toml"}
{"timestamp":"2026-04-23T14:00:12.350Z","level":"INFO","message":"Storage backend: sqlite path=./pai-kernel.db"}
{"timestamp":"2026-04-23T14:00:12.355Z","level":"INFO","message":"Policy engine: loaded N rego modules from ./policies/"}
{"timestamp":"2026-04-23T14:00:12.360Z","level":"INFO","message":"PAI-Kernel Governance Sidecar v1.3.0 listening on 127.0.0.1:9100"}
```

The daemon blocks the terminal. To stop it use `Ctrl+C`.

---

## § 6 · Step 4 · Verify the daemon is working

**Terminal 2** — new window:

```bash
curl http://127.0.0.1:9100/api/v1/health
# Expected response (JSON):
# {"status":"ok"}
```

```bash
curl http://127.0.0.1:9100/api/v1/version
# Expected response:
# {"version":"1.3.0","pai_cd_version":"3.1","build_profile":"release"}
#
# Note on version fields: this release is tagged v2.2.1 (corpus snapshot), SDK
# binary version is 1.3.0, and the SDK-enforced invariant set is a superset of
# the published v2.2 corpus (SDK implements additional invariants ahead of
# their publication in a future corpus freeze). This is expected and
# documented in KNOWN_LIMITATIONS.md § 1.3.
```

```bash
curl -s http://127.0.0.1:9100/api/v1/state | jq .
# Expected: multi-field JSON showing current governance state
# (consent grants, objectives, delegations, drift score, etc.)
```

(If you don't have `jq`, pipe to a file and open it: `curl -s ... > state.json && cat state.json`.)

**Browser check:**

Open in browser: `http://127.0.0.1:9100/api/v1/state`

You should see a JSON document rendered by the browser. If Safari or Chrome complain that it's "not valid JSON" but the text looks right, it's fine — some browsers are just pedantic about `Content-Type` on JSON.

---

## § 7 · Step 5 · Install Ollama

### 7.1 macOS

**Option A — Homebrew:**

```bash
brew install ollama
```

**Option B — DMG download:**

Visit <https://ollama.com/download/mac>, download, drag to Applications, open once to allow.

### 7.2 Windows

**Option A — winget:**

```powershell
winget install Ollama.Ollama
```

**Option B — installer:**

Visit <https://ollama.com/download/windows>, download `.exe`, run.

### 7.3 Linux

```bash
curl -fsSL https://ollama.com/install.sh | sh
```

This script detects your distribution and installs appropriately (systemd service on Ubuntu/Debian/Fedora; binary-only on Arch).

### 7.4 Verify Ollama install

```bash
ollama --version
# Expected: ollama version is X.Y.Z (any recent version)

ollama list
# Expected: NAME  ID  SIZE  MODIFIED
# (empty list first time; we'll add a model next)
```

---

## § 8 · Step 6 · Pull a test model

Recommended models for first test:

| Model | Parameters | Download | RAM used | Notes |
|---|---|---|---|---|
| **`llama3.2`** ← recommended first | 3B | ≈ 2 GB | 8 GB | Smallest, fastest, English |
| `qwen2.5:3b` | 3B | ≈ 2 GB | 8 GB | Multilingual (strong Russian) |
| `llama3.1:8b` | 8B | ≈ 5 GB | 16 GB | Quality/size balance |
| `mixtral:8x7b` | 47B | ≈ 26 GB | 48 GB | High-quality multilingual; heavy |

Pull the default:

```bash
ollama pull llama3.2
# Expected: pulling manifest → pulling layers → ... → success
# Size ≈ 2 GB; time 2-10 min depending on your connection
```

Verify:

```bash
ollama list
# Expected:
# NAME              ID              SIZE      MODIFIED
# llama3.2:latest   abc123...       2.0 GB    2 minutes ago
```

---

## § 9 · Step 7 · Run the side-by-side test

### 9.1 Terminal 1 — keep daemon running

From § 5 above — already running.

### 9.2 Terminal 2 — Ollama interactive chat

```bash
ollama run llama3.2
```

Expected prompt:

```
>>> Send a message (/? for help)
```

Try a prompt:

```
>>> What does PAI-CD stand for?
```

Ollama replies based on the model's pre-training (which does not include PAI-CD since PAI-Kernel is not public-web-indexed; expect a plausible-but-made-up answer — this is expected behavior and is itself a good example of why governance substrates matter for AI systems).

Exit chat: type `/bye` or press `Ctrl+D`.

### 9.3 Terminal 3 — inspect SDK state

While Ollama chat is active, open a third terminal:

```bash
curl -s http://127.0.0.1:9100/api/v1/state | jq '.witness_chain.length'
# Expected: 0 (or some small number if you made SDK calls earlier)
```

```bash
curl -s http://127.0.0.1:9100/api/v1/log | jq '.entries | length'
# Expected: 0 (witness log is empty; populated by direct SDK calls, not by Ollama use)
```

**This is the L1 Demo Mode honest truth:** the governance daemon does not observe Ollama, and Ollama does not call the governance daemon. They run in parallel. No witness-chain entries from chat.

---

## § 10 · What you're seeing (Level 1 Demo Mode)

### What works in v2.2.1

- ✅ Governance daemon builds, runs, binds to localhost
- ✅ All 17 `/api/v1/*` endpoints respond with valid JSON
- ✅ Witness chain cryptographically verifies (`GET /api/v1/log/verify`)
- ✅ Direct SDK calls populate witness log (try `POST /api/v1/consent/grant`)
- ✅ Drift engine tracks objective / classification changes
- ✅ SQLite persistence survives daemon restart
- ✅ Export bundle generates portable JSON (`GET /api/v1/export`)
- ✅ OPA/Rego policy modules load and evaluate
- ✅ Ollama runs any supported model locally
- ✅ PAI-Console React UI (optional, § 11) renders governance state

### What does NOT work in v2.2.1 (by design)

- ❌ Ollama response filtering through SDK gates
- ❌ Conservative Mode blocking AI output mid-stream
- ❌ Witness-chain entries auto-populated from Ollama chat
- ❌ SDK-enforced consent gate on model loading
- ❌ TCB attestation backend wiring
- ❌ Multi-principal (multi-Author) coordination (single dyadic deploy only)

### Why this matters

v2.2.1 SDK provides the **governance substrate** — the invariants, witness chain, consent semantics, export primitives, drift monitoring. It is the layer on which AI-mediation will be built.

The AI-mediation wiring itself — routing Ollama's responses through the governance layer, binding Conservative Mode to actual model output, populating witness entries from chat turns — is scheduled for a future release.

**Adopters should read the roadmap framing in `KNOWN_LIMITATIONS.md`** before building on top of this release.

---

## § 11 · Step 8 · (Optional) Run the PAI-Console UI

The `console/` directory contains a React + Vite application that visualizes governance state via the daemon's JSON API. Useful for adopters who prefer a UI over `curl`.

### 10.1 Build the console

Prerequisites: **Node.js 20+** and npm.

Install Node.js:
- macOS: `brew install node` or <https://nodejs.org/>
- Windows: `winget install OpenJS.NodeJS` or <https://nodejs.org/>
- Linux: distro package or <https://github.com/nodesource/distributions>

Build:

```bash
cd console
npm install
npm run build
# Expected: built files in console/dist/ (≈ 300 KB total)
```

### 10.2 Run the console

Development mode (hot reload, proxies `/api` to daemon):

```bash
npm run dev
# Opens http://127.0.0.1:3000 with hot reload
```

Production mode (static file serve):

```bash
# macOS / Linux
npx serve dist/ --port 3000

# Windows (PowerShell)
npx serve .\dist\ --port 3000
```

Open `http://127.0.0.1:3000` in your browser. Navigate through:

- **Governance Status** — health + witness count + Conservative Mode indicator
- **Witness Log Viewer** — table of witness entries + chain-validity badge
- **Drift Dashboard** — composite score + 7-dimension bar chart
- **Consent Manager** — grant / revoke / enter-Conservative / exit-Conservative
- **Export** — download full governance bundle as JSON

---

## § 12 · Troubleshooting

### 11.1 Port 9100 is already in use

Check what's using it:

```bash
# macOS / Linux
lsof -i :9100

# Windows (PowerShell)
Get-NetTCPConnection -LocalPort 9100
```

Options:

- Stop the other process, OR
- Change port: edit `pai-kernel.toml` → `[server] port = 9101` and restart daemon

### 11.2 Rust compilation errors

**macOS:** `error: linking with 'cc' failed` → reinstall Xcode CLT:

```bash
sudo rm -rf /Library/Developer/CommandLineTools
xcode-select --install
```

**Windows:** `error: Microsoft Visual C++ 14.0 or greater is required` → install Visual Studio Build Tools 2022 with the C++ workload (§ 2.2 above).

**Linux:** `error: cannot find -lssl` or `-lcrypto` → install the OpenSSL development headers per § 2.3.

**Any platform:** `rustc version X.Y.Z < 1.86.0` → `rustup update stable && rustup default stable`.

### 11.3 Ollama can't pull a model

**Firewall / proxy:** set environment variables before pull:

```bash
export HTTPS_PROXY=http://your-proxy:port
ollama pull llama3.2
```

**DNS issues:** `ollama pull` uses `registry.ollama.ai`. Verify resolution:

```bash
curl -v https://registry.ollama.ai
```

**Disk space:** Ollama stores models under `~/.ollama/` (macOS/Linux) or `%USERPROFILE%\.ollama\` (Windows). Ensure adequate free space.

### 11.4 Port 11434 (Ollama) conflict

```bash
# Tell Ollama to use a different port
OLLAMA_HOST=127.0.0.1:11435 ollama serve
# Then in another terminal:
OLLAMA_HOST=127.0.0.1:11435 ollama run llama3.2
```

### 11.5 macOS Gatekeeper blocking binary

If macOS warns "cannot verify developer":

```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine ./target/release/pai_governance_daemon

# Or (system-wide preference) System Settings → Privacy & Security → "Open Anyway"
```

(Adopters build from source, so this is normally not an issue — only happens on binaries distributed pre-built.)

### 11.6 Windows Defender quarantine

Freshly compiled binaries may trigger heuristic alerts. Add an exclusion:

```
Settings → Privacy & Security → Windows Security → Virus & threat protection
  → Manage settings → Add or remove exclusions → Add exclusion → Folder
  → Select: C:\path\to\pai-kernel\target\
```

### 11.7 Linux SELinux / AppArmor denying SQLite write

If the daemon logs `Permission denied` on `pai-kernel.db`:

**Check SELinux:**

```bash
sestatus
# If "SELinux status: enabled" and "Current mode: enforcing"
sudo ausearch -m avc -ts recent   # check for denials
```

Workaround: run daemon from a directory with appropriate SELinux context, or temporarily permissive mode (`sudo setenforce 0` — **not recommended for permanent use**).

**Check AppArmor:**

```bash
sudo aa-status | head
```

Similar process.

### 11.8 SQLite permission errors

```
Error: unable to open database file
```

→ You're probably running from a read-only directory. Change to a writable location:

```bash
cp ./target/release/pai_governance_daemon ~/pai-kernel/
cp ./pai-kernel.toml ~/pai-kernel/
cp -r ./policies ~/pai-kernel/
cd ~/pai-kernel
./pai_governance_daemon
```

### 11.9 Console dev server can't reach daemon

Error in browser console: `Failed to fetch /api/v1/...`

- Verify daemon is running (§ 6)
- Verify port: Console dev proxy is at `http://127.0.0.1:9100` by default (see `console/vite.config.ts`)
- If you changed the daemon port, update `console/vite.config.ts` → `server.proxy.'/api'.target` → your port

---

## § 13 · Known limitations (before you build on this)

Full scope: see `KNOWN_LIMITATIONS.md`. Key items:

- **v2.2 is a citationally-stable freeze.** The published corpus is a frozen snapshot; adopters integrating now bind to v2.2 semantics. Future corpus freezes may introduce additional normative content.
- **SDK v1.3.0 exceeds v2.2 corpus scope** — the runtime implements additional invariants ahead of their publication in a future corpus freeze. Adopters using SDK bind to this superset.
- **Formal verification is specification-level**, not runtime-SDK-conformance. Current compliance status requires independent audit.
- **Multi-instance coordination** (multiple PAI Authors cooperating) is not in v2.2.1 scope; dyadic deployments only.
- **Regulatory zone governance and provider-disposition disclosure** are scheduled for a later release; not in v2.2.1.

---

## § 14 · Feedback and next steps

We want feedback from early adopters. What to do:

- **Try the install.** Did it work first time? Which step broke? What was confusing?
- **Explore the corpus.** Start with `Constitutional Core`, then `Bill of Authorial Rights`, then `Glossary`. Browsable at <https://corpus.paikernel.org>.
- **Read the paper.** DOI: [10.2139/ssrn.6512218](https://doi.org/10.2139/ssrn.6512218).
- **Make notes on architectural gaps.** Where does the framework feel incomplete for your use case?

### Feedback channels

- **GitHub Issues** at <https://github.com/PAI-Kernel/pai-kernel/issues> — bugs, install failures, feature requests
- **Direct contact:** `contact@paikernel.org` — strategic feedback, invited-audience discussions
- **Security-sensitive findings:** GitHub security advisory (private) at <https://github.com/PAI-Kernel/pai-kernel/security/advisories>

### What we are NOT asking

- Not asking for production deployments
- Not asking for public promotion or advocacy
- Not asking for reproductions in other frameworks (though permitted under CC BY 4.0 / MIT-Apache — please flag via Issue if you do)

---

## § 15 · Appendix · Commands cheatsheet

### Full install (one block, per OS)

**macOS:**

```bash
# Prerequisites
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
brew install ollama  # or download DMG

# Get & build
git clone https://github.com/PAI-Kernel/pai-kernel.git
cd pai-kernel && git checkout v2.2.1
cargo build --workspace --release

# Run
./target/release/pai_governance_daemon &
ollama pull llama3.2
ollama run llama3.2  # interactive
```

**Windows (PowerShell):**

```powershell
# Prerequisites — install VS Build Tools 2022 first (GUI)
winget install Rustlang.Rustup
# Close/reopen PowerShell
winget install Ollama.Ollama

# Get & build
git clone https://github.com/PAI-Kernel/pai-kernel.git
Set-Location pai-kernel
git checkout v2.2.1
cargo build --workspace --release

# Run
Start-Process .\target\release\pai_governance_daemon.exe
ollama pull llama3.2
ollama run llama3.2
```

**Linux (Debian/Ubuntu):**

```bash
# Prerequisites
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev git curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
curl -fsSL https://ollama.com/install.sh | sh

# Get & build
git clone https://github.com/PAI-Kernel/pai-kernel.git
cd pai-kernel && git checkout v2.2.1
cargo build --workspace --release

# Run
./target/release/pai_governance_daemon &
ollama pull llama3.2
ollama run llama3.2
```

### Alternative — install SDK only (no source tree)

```bash
cargo install pai-kernel
# Creates the `pai_governance_daemon` binary in ~/.cargo/bin/

# Get a config file
curl -o pai-kernel.toml https://raw.githubusercontent.com/PAI-Kernel/pai-kernel/v2.2.1/pai-kernel.toml
mkdir -p policies
# (optional: copy policy modules if you want them)

# Run
pai_governance_daemon --config ./pai-kernel.toml
```

### Useful daemon API calls (once running)

```bash
# Health
curl http://127.0.0.1:9100/api/v1/health

# Version
curl http://127.0.0.1:9100/api/v1/version

# State snapshot
curl -s http://127.0.0.1:9100/api/v1/state | jq .

# Witness log (cryptographic chain of all governance events)
curl -s http://127.0.0.1:9100/api/v1/log | jq '.entries | length'

# Verify witness chain integrity
curl http://127.0.0.1:9100/api/v1/log/verify

# Export portable governance bundle
curl -s http://127.0.0.1:9100/api/v1/export > bundle.json

# Grant consent (example)
curl -X POST http://127.0.0.1:9100/api/v1/consent/grant \
  -H "Content-Type: application/json" \
  -d '{"scope":"example","capability":"test_capability","duration_days":30}'

# Enter Conservative Mode
curl -X POST http://127.0.0.1:9100/api/v1/conservative/enter \
  -H "Content-Type: application/json" \
  -d '{"reason":"manual testing"}'

# Exit Conservative Mode
curl -X POST http://127.0.0.1:9100/api/v1/conservative/exit \
  -H "Content-Type: application/json" \
  -d '{"rationale":"test complete"}'
```

### Uninstall

**Source-tree install:**

```bash
# Stop daemon (Ctrl+C in Terminal 1)
# Remove source tree
rm -rf ~/pai-kernel
# Remove local state
rm -f ~/pai-kernel.db
```

**cargo-install path:**

```bash
cargo uninstall pai-kernel
rm -f ~/pai-kernel.db
```

**Ollama (optional):**

- macOS: `brew uninstall ollama` + `rm -rf ~/.ollama`
- Windows: Settings → Apps → Ollama → Uninstall + delete `%USERPROFILE%\.ollama\`
- Linux: `sudo systemctl stop ollama && sudo rm /usr/local/bin/ollama && rm -rf ~/.ollama`

---

## § 16 · Changelog

| Version | Date | Changes |
|---|---|---|
| **v0.1** | **2026-04-23** | Initial INSTALL.md for v2.2.1 release. Cross-platform (macOS / Windows / Linux). L1 Demo Mode framing. Ollama side-by-side walkthrough. Troubleshooting 9 subsections. Commands cheatsheet + uninstall path. |

---

*Install Guide · PAI-CD v2.2.1 · 2026-04-23*
*Source: `INSTALL.md` · [View on GitHub](https://github.com/PAI-Kernel/pai-kernel/blob/v2.2.1/INSTALL.md)*
