---
title: "PAI-Kernel SDK v2.2.3 — Patch Release Notes"
slug: release-notes-v2.2.3
position: 1
hidden: false
excerpt: "Runtime configuration improvements: author key initialization moved from compile-time defaults to environment variables. Patch release supersedes v2.2.2."
pai_cd:
  version: "2.2.3"
  status: "Canonical"
  source:
    file: "RELEASE_NOTES_v2.2.3.md"
    path: "docs/RELEASE_NOTES_v2.2.3.md"
    commit: "v2.2.3"
    authority_repo: "PAI-Kernel/pai-kernel"
  cite_as: "PAI-CD v2.2.3 · Release Notes"
  last_amendment: "2026-04-28"
---

# PAI-Kernel SDK v2.2.3 — Patch Release

**Release date:** 2026-04-28
**Supersedes:** v2.2.2 (released 2026-04-27)
**Workspace SDK version:** 1.3.1 → 1.3.2 (PATCH)
**Corpus version:** 2.2 (frozen, unchanged)

> **Adopters: upgrade via standard cargo or docker. See "Adopter upgrade" below.**

---

## Headline change · runtime configuration improvements

The v1.3.2 SDK moves author key initialization from compile-time defaults to environment variables across three production paths: the daemon binary entry, the `pai_compliance` test-suite binary, and the `pai_api` `AppState` constructor. Production deployments now load the author signing identity at startup from `PAI_AUTHOR_API_KEY` and `PAI_AUTHOR_SIGNING_KEY` (32-byte hex), and refuse to start if either is missing or malformed.

This protects author-supremacy invariants in adopter deployments: the operator generates the signing identity, never the bundled binary.

A new optional `--demo` flag generates ephemeral in-memory keys for short-lived local testing, prints stderr warnings, and forces the bind address to `127.0.0.1`. Demo keys are unique per process invocation and cannot be reused across sessions.

### Architectural principles applied

- **Fail-closed:** missing env vars cause the daemon to exit with a clear setup-guide message pointing at INSTALL.md, never falling back to defaults.
- **No hardcoded signing keys** in any production code path. Test paths (`#[cfg(test)] mod tests`) continue to use deterministic test fixtures and are unaffected.
- **Demo path discipline:** ephemeral keys + 127.0.0.1 binding constraint + stderr warnings + INSTALL.md pointer.

### Adopter upgrade

```sh
cargo update -p pai_governance_daemon
# or pull the new binaries directly:
curl -fsSL https://paikernel.org/install.sh | sh
# or
docker pull ghcr.io/pai-kernel/pai-kernel:v2.2.3
```

Then set the two production env vars before running the daemon (one-time setup):

```sh
export PAI_AUTHOR_API_KEY="your-author-api-key"
export PAI_AUTHOR_SIGNING_KEY="$(openssl rand -hex 32)"
```

Persist these in your shell rc file or secrets manager. See INSTALL.md § 5.2 (ENV setup) for the full guide and the `--demo` flag details.

---

## What's new

### Runtime

- `pai_governance_daemon` daemon entry: env-var key loading at startup (fail-closed).
- `pai_compliance` binary: ephemeral demo keys per session (the binary verifies daemon behavior within its own process; per-session ephemeral keys are sufficient and safer than hardcoded fixtures).
- `pai_api` `AppState`: `new_in_memory()` now returns `Result<Self, KeyError>` and loads from env vars. New companion `AppState::new_in_memory_demo()` generates ephemeral keys for testing. New `AppState::with_explicit_keys(sk, vk, api_key)` for advanced callers.
- `pai_governance_daemon::keyloader` module published: `build_author_keys()` (env-var loader) and `build_demo_keys()` (ephemeral generator with stderr warnings).

### CLI

- New `--demo` flag on the daemon binary: generates ephemeral in-memory keys, forces bind to `127.0.0.1`, prints stderr warnings. For local testing only.
- `pai_governance_daemon version` subcommand now reads the Rust toolchain version dynamically from `Cargo.toml` `rust-version` at compile time (the previous v1.3.1 binary printed a hardcoded `1.86.0` literal that did not reflect the actual repository pin of `1.88.0`).

### Documentation

- INSTALL.md § 5.2: new ENV setup section with key generation guide and `--demo` flag explanation.
- INSTALL.md CHANGELOG v0.4 entry covers the v2.2.3 changes.

### Distribution

- 18 crates re-published to crates.io as v1.3.2.
- Pre-compiled binaries via Homebrew tap and `install.sh` fast path.
- GHCR Docker image tag `v2.2.3` published alongside `latest`.

---

## Migration from v2.2.2

For adopters who deployed v2.2.2 in any production-shaped path:

```sh
# 1. Update dependency / re-pull binary
cargo update -p pai_governance_daemon       # cargo path
docker pull ghcr.io/pai-kernel/pai-kernel:v2.2.3   # docker path

# 2. Set production env vars (one-time)
export PAI_AUTHOR_API_KEY="your-author-api-key"
export PAI_AUTHOR_SIGNING_KEY="$(openssl rand -hex 32)"

# 3. Run as before
./pai_governance_daemon --config ./pai-kernel.toml
```

**Library callers of `pai_api::AppState::new_in_memory()`:** the return type changes from `Self` to `Result<Self, keyloader::KeyError>`. For local-test call sites, switch to `AppState::new_in_memory_demo()`. For production call sites, propagate the `Result` and ensure env vars are set.

**MSRV:** Rust 1.88.0 (unchanged from v2.2.2).
**Corpus:** v2.2 (unchanged — the v2.2 corpus snapshot remains frozen at March 2026; v2.2.x patches refine SDK and tooling without modifying the constitutional framework).
**SDK API:** one breaking change in `pai_api::AppState::new_in_memory()` (return type now `Result`). All other public APIs unchanged.

---

## Looking ahead

- **v2.2.3.x patches** (post-§5f cleanup window): adopter feedback integration, KNOWN_LIMITATIONS.md remediation backlog, OpenSSF Scorecard remediation.
- **v2.2.4** (~early-mid May 2026): `docs/EXAMPLES.md` canonical 6-invariant walkthrough grounded in real adopter scenarios.
- **v2.3** (~late May / early June 2026): Russian translation of the canonical 10 documents, bilingual mdBook documentation portal, subdomain consolidation, and pre-publication audit framework expanded to a 6th pillar (adversarial own-corpus scan).

---

## Citation

```bibtex
@techreport{paikernel2026paicd,
  author       = {Sergeev, Mikhail Anatolievich},
  title        = {{PAI-CD}: A Constitutional Framework for Authorial Sovereignty in Deployed {AI} Systems},
  institution  = {PAI-Kernel Initiative},
  year         = {2026},
  doi          = {10.5281/zenodo.19151899}
}
```

Companion paper: [SSRN DOI 10.2139/ssrn.6512218](https://doi.org/10.2139/ssrn.6512218).

---

## Feedback

- GitHub Issues: <https://github.com/PAI-Kernel/pai-kernel/issues>
- Direct contact: `contact@paikernel.org`
- Security-sensitive findings: GitHub security advisory (private)

For the benefit of all living beings.

---

*Release Notes · PAI-CD v2.2.3 · 2026-04-28*
*Source: `docs/RELEASE_NOTES_v2.2.3.md` · [View on GitHub](https://github.com/PAI-Kernel/pai-kernel/blob/v2.2.3/docs/RELEASE_NOTES_v2.2.3.md)*
