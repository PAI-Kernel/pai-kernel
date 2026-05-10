---
title: "Upgrade Guide — PAI-Kernel SDK v2.2.x"
slug: upgrade
position: 3
hidden: false
excerpt: "Upgrade procedures between PAI-Kernel SDK release tags. Currently covers v2.2.2 → v2.2.3 (breaking environment variable rename)."
pai_cd:
  version: "2.2.3"
  status: "Canonical"
  source:
    file: "upgrade.md"
    path: "docs/upgrade.md"
    commit: "main"
    authority_repo: "PAI-Kernel/pai-kernel"
  cite_as: "PAI-Kernel SDK v2.2.3 · Upgrade Guide"
  last_amendment: "2026-05-09"
---

# PAI-Kernel SDK · Upgrade Guide

This guide covers upgrade procedures between PAI-Kernel SDK release tags.

> **Quick check:** are you upgrading?
>
> ```sh
> pai_governance_daemon --version
> # pai-kernel <current-version>
> ```
>
> Compare к [latest release](https://github.com/PAI-Kernel/pai-kernel/releases/latest).

---

## Upgrade matrix

| From | To | Breaking changes | Required action |
|---|---|---|---|
| v2.2.2 | v2.2.3 | YES · environment variable rename | See § v2.2.2 → v2.2.3 below |
| v2.2.1 | v2.2.3 | Combined (v2.2.1 → v2.2.2 → v2.2.3) | Follow both upgrade sections sequentially |
| v2.2 | v2.2.3 | Combined release sequence | Follow each upgrade section sequentially |

---

## v2.2.2 → v2.2.3 (BREAKING · environment variable rename)

**Released:** 2026-04-28

### Summary of breaking change

In v2.2.3, author key initialization moved from compile-time defaults к **mandatory environment variables** (fail-closed default). The daemon refuses к start without these set.

If you ran v2.2.2 successfully without environment configuration, this is the change that affects you.

### Required environment variables (NEW в v2.2.3)

```sh
export PAI_AUTHOR_API_KEY="$(uuidgen)"          # any sufficiently random string
export PAI_AUTHOR_SIGNING_KEY="$(openssl rand -hex 32)"  # 32-byte hex
```

### Step-by-step upgrade

#### Step 1 · Stop the running v2.2.2 daemon

```sh
# If running via systemd/launchd:
sudo systemctl stop pai-kernel  # OR: launchctl unload /path/to/plist

# If running interactively:
# Find PID: ps aux | grep pai_governance_daemon
# Kill: kill <PID>
```

#### Step 2 · Backup current state

```sh
# Backup working directory containing pai-kernel.toml + policies/ + pai-kernel.db
tar -czf pai-kernel-backup-$(date +%Y%m%d).tar.gz \
  pai-kernel.toml policies/ pai-kernel.db
```

State persistence works across versions · Cargo.lock and TLA+ model unchanged · witness chain integrity preserved.

#### Step 3 · Upgrade binary

Pick the matching channel:

```sh
# Homebrew
brew upgrade pai-kernel

# Cargo
cargo install pai_kernel --version 1.3.2 --force

# Docker
docker pull ghcr.io/pai-kernel/pai-kernel:v2.2.3

# Manual binary download
# https://github.com/PAI-Kernel/pai-kernel/releases/tag/v2.2.3
```

#### Step 4 · Set new environment variables

For shell session (testing):

```sh
export PAI_AUTHOR_API_KEY="$(uuidgen)"
export PAI_AUTHOR_SIGNING_KEY="$(openssl rand -hex 32)"
```

For persistent setup (production):

```sh
echo "export PAI_AUTHOR_API_KEY='$(uuidgen)'" >> ~/.zshrc
echo "export PAI_AUTHOR_SIGNING_KEY='$(openssl rand -hex 32)'" >> ~/.zshrc
source ~/.zshrc
```

For systemd service:

```sh
# Add к /etc/systemd/system/pai-kernel.service [Service] block:
Environment="PAI_AUTHOR_API_KEY=<your-api-key>"
Environment="PAI_AUTHOR_SIGNING_KEY=<your-32-byte-hex>"

# Reload + restart:
sudo systemctl daemon-reload
sudo systemctl restart pai-kernel
```

#### Step 5 · Restart daemon

```sh
pai_governance_daemon --config ./pai-kernel.toml
# Should see structured JSON log "Starting PAI-Kernel" + "listening on 127.0.0.1:9100"
```

If daemon fails к start with `Error: missing PAI_AUTHOR_API_KEY environment variable`, env vars are not set in daemon's environment. Check shell vs service configuration.

#### Step 6 · Verify

```sh
curl -s http://127.0.0.1:9100/api/v1/health
# {"status":"ok","witness_entries":<count>,"conservative_mode":false}

curl -s http://127.0.0.1:9100/api/v1/version
# {"version":"1.3.2","pai_cd_version":"3.1","rust_toolchain":"1.88"}
```

### State preservation guaranteed

- Witness chain integrity: validated at startup via hash chain check · tampering or corruption fails-closed
- Policy files (`./policies/*.rego`) unchanged · loaded as before
- Configuration (`pai-kernel.toml`) format unchanged · v2.2.2 config files work unchanged in v2.2.3 (no schema migration)

### Rollback procedure

If v2.2.3 deployment unsuccessful:

```sh
# Restore backup
tar -xzf pai-kernel-backup-YYYYMMDD.tar.gz

# Reinstall v2.2.2 binary
brew install PAI-Kernel/tap/pai-kernel@2.2.2  # if formula versioned
# OR: cargo install pai_kernel --version 1.3.1 --force
# OR: docker pull ghcr.io/pai-kernel/pai-kernel:v2.2.2

# Unset v2.2.3 env vars
unset PAI_AUTHOR_API_KEY PAI_AUTHOR_SIGNING_KEY

# Restart v2.2.2 daemon (no env vars needed)
pai_governance_daemon
```

### Demo mode bypass (for upgrade testing)

If you want к verify v2.2.3 binary works before committing к environment variable setup:

```sh
pai_governance_daemon --demo
# Forces 127.0.0.1:9100 bind · ephemeral keys · prints WARNINGs · NOT for production
```

This works without env vars · helpful for «is the new binary even installed correctly?» checks.

---

## v2.2.1 → v2.2.2 (non-breaking · documentation reorganization)

**Released:** 2026-04-27

### Summary

- Workspace SDK version bump 1.3.1 → 1.3.1 (unchanged at workspace level · this is the v2.2.2 ceremonial release)
- Hero diagram + OpenSSF Scorecard badge added к README
- Compliance binary CI gate introduced
- MSRV bumped к 1.88 (Rust toolchain · time crate CVE)

### Required action

Upgrade Rust toolchain if currently below 1.88:

```sh
rustup update stable
rustc --version  # should be >= 1.88.0
```

Then upgrade binary via standard channels:

```sh
brew upgrade pai-kernel
# OR cargo install pai_kernel --version 1.3.1 --force
```

No environment variable changes · no configuration changes · no breaking API changes.

---

## v2.2 → v2.2.1 (non-breaking · documentation reorganization)

**Released:** 2026-04 (early)

### Summary

- `PAI_Constitutional_Document.md` moved from root к `corpus/`. Root file is now a redirect placeholder for compatibility.

### Required action

If you have external links к `PAI_Constitutional_Document.md`, update them к `corpus/PAI_Constitutional_Document.md`. The redirect placeholder at root preserves working URLs but adds one click.

No binary changes · no environment variable changes · no API changes.

---

## See also

- [`docs/INSTALL.md`](INSTALL.md) — full installation guide
- [`docs/quickstart.md`](quickstart.md) — 5-minute getting-started
- [`docs/RELEASE_NOTES_v2.2.3.md`](RELEASE_NOTES_v2.2.3.md) — v2.2.3 detailed release notes
- [`docs/RELEASE_NOTES_v2.2.3.1.md`](RELEASE_NOTES_v2.2.3.1.md) — v2.2.3 main HEAD documentation improvements
- [`CHANGELOG.md`](../CHANGELOG.md) — release history
- [`docs/KNOWN_LIMITATIONS.md`](KNOWN_LIMITATIONS.md) — known limitations and scope

---

*PAI-Kernel SDK · v2.2.3 · Constitutional Governance Framework for AI · MIT OR Apache-2.0 · Mikhail Sergeev / PAI-Kernel Initiative*
