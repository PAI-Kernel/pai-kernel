---
title: "PAI-Kernel SDK v2.2.3 main HEAD post-tag improvements (informal v2.2.3.1)"
slug: release-notes-v2.2.3.1
position: 2
hidden: false
excerpt: "Documentation + adopter UX improvements landed in main branch post-v2.2.3 tag. NO binary version bump · tag v2.2.3 remains immutable. Doc-only sustaining release."
pai_cd:
  version: "2.2.3"
  status: "Canonical"
  source:
    file: "RELEASE_NOTES_v2.2.3.1.md"
    path: "docs/RELEASE_NOTES_v2.2.3.1.md"
    commit: "main"
    authority_repo: "PAI-Kernel/pai-kernel"
  cite_as: "PAI-Kernel SDK v2.2.3 main HEAD · Post-Tag Improvements"
  last_amendment: "2026-05-09"
---

# PAI-Kernel SDK v2.2.3 — main HEAD post-tag improvements

**Type:** Documentation + adopter UX sustaining release
**Tag:** v2.2.3 (unchanged · IMMUTABLE)
**Workspace SDK version:** 1.3.2 (unchanged)
**Date range:** 2026-05-09 (post-tag main HEAD activity)

---

## Release semantics clarification

PAI-Kernel SDK uses a two-track release model:

1. **Tagged releases** (`v2.2.0`, `v2.2.1`, `v2.2.2`, `v2.2.3`) are GPG-signed, immutable, and correspond to specific binary distributions on crates.io, GitHub Container Registry, and Homebrew tap. Once tagged, these are frozen.
2. **`main` branch** can advance beyond the latest release tag with documentation, CI, packaging, and adopter-UX improvements that do not require a new binary version.

This document records `main` HEAD changes post-v2.2.3 tag (2026-04-28). No binary version bump occurs · adopters running v2.2.3 from any distribution channel see identical runtime behavior. Documentation improvements take effect for adopters cloning `main` directly OR consulting the public repository web view.

A formal v2.2.3.1 patch release would only occur if runtime behavior changed; the «v2.2.3.1» label in this document's filename refers to the documentation epoch, not a published binary version.

---

## Documentation improvements

### docs/INSTALL.md · empirical correction

Replaced previous «v2.2.3 Early Preview» installation guide with comprehensive 6-method install guide, post-adopter-UX-validation. Sixteen specific corrections covering:

- Environment variable names (`PAI_AUTHOR_API_KEY` + `PAI_AUTHOR_SIGNING_KEY` · matches actual binary)
- Key generation patterns (`openssl rand -hex 32` · matches binary expected format)
- TOML configuration structure (`[server]/[storage]/[policy]/[drift]/[logging]` · matches `pai_governance_daemon init` output)
- CLI binary names (`pai_governance_daemon verify/export` · binary `pai_kernel` does not exist)
- API endpoints inventory (5 actual endpoints validated empirically)
- Docker examples (env var names · `-w /data` working directory)
- Demo mode prerequisite framing
- Verification prerequisite framing
- Troubleshooting entries for common adopter issues

### docs/quickstart.md (NEW)

Five-minute getting-started path · stripped INSTALL.md essence · covers brew install + `--demo` mode + `curl` API verification. Designed for adopters who want к verify «does it run?» before committing к full installation walkthrough.

### CHANGELOG.md (NEW · root)

Standard Keep-a-Changelog format release history. Documents v2.2 → v2.2.3 changes plus this main HEAD documentation epoch. Adopters discovering project for first time can scan release evolution at a glance.

### examples/README.md (NEW)

Adopter guidance for the five reference example binaries (`causal_graph` · `classify_output` · `compliance_identity` · `evasion_audit` · `vulnerability_check`). Per-example purpose · how-to-run · what-it-demonstrates structure. Removes friction of «what does each example show?»

### docs/audit_checklist.md (NEW)

Adopter-runnable verification procedures. Covers:

- Reproducible build verification (`cargo build --workspace --release --locked`)
- Test suite execution
- GPG signature verification (release tags + commits)
- Binary SHA256 checksum verification
- Constitutional compliance gate (`pai_compliance` runtime test)

### docs/upgrade.md (NEW)

Explicit upgrade procedure from v2.2.2 → v2.2.3, focused on the breaking change (environment variable naming for author keys).

### docs/SUPPORT.md (NEW)

Support tier explanation · invitation-only context clarification · response time expectations · how к ask for help versus how к report security issues (latter routed к SECURITY.md).

### Further Reading link integrity

`docs/INSTALL.md` Further Reading section now links к existing files only:

- `corpus/PAI_Constitutional_Document.md` direct (not via redirect placeholder)
- `examples/README.md` (new)
- `CHANGELOG.md` (new)
- `docs/RELEASE_NOTES_v2.2.3.md`

Removed reference к non-existent `formal/VERIFICATION_MATRIX.md`. Future TLA+ verification artifact publication remains under consideration (depends on `formal/` directory public-vs-internal disposition).

---

## Source code changes

### crates/pai_export/Cargo.toml · WASM target getrandom feature flag

Added target-specific dependency block enabling WebAssembly browser/JS runtime support for `pai_export`:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.2", features = ["js"] }
```

Effect: `cargo build --target wasm32-unknown-unknown --release -p pai_export` now succeeds. Native build behavior unchanged (target-specific scope · no contamination of native builds).

§ Browser Runtime profile completeness materially advanced (Kernel WASM viability 11/12 → 12/12 of v2.2.3 workspace crates compilable to WASM).

### Cargo.lock · regenerated

Reflects `pai_export` WASM target dep addition. `cargo build --workspace --release --locked` reproducibility preserved.

---

## Homebrew formula improvements

`PAI-Kernel/homebrew-tap` formula Caveats updated (formula version 2.2.3 unchanged):

- Environment variable examples now show runnable patterns (`$(uuidgen)` and `$(openssl rand -hex 32)`) instead of placeholder strings
- Demo mode framing improved · explicit «Quick browser test» path with curl example
- Working directory example replaced placeholder `cd /path/to/work` with concrete `mkdir -p ~/pai-kernel-work && cd ~/pai-kernel-work`
- Documentation links section expanded with explicit Install guide URL

Adopters running `brew info pai-kernel` post-update see corrected guidance immediately. `brew reinstall pai-kernel` not required (Caveats are documentation-only · binary unchanged).

---

## What did NOT change

- **Binary `pai_governance_daemon` v1.3.2** · same binary as v2.2.3 release
- **Crates.io published packages** · 18 crates v1.3.2 unchanged
- **Container image** `ghcr.io/pai-kernel/pai-kernel:v2.2.3` · unchanged
- **Tag `v2.2.3`** at SHA `77d564790c` · IMMUTABLE
- **Public API surface** · five `/api/v1/*` endpoints unchanged
- **Constitutional invariants** · T1..T6 unchanged · TLC-verified state model unchanged
- **MSRV (Minimum Supported Rust Version)** · 1.88 unchanged

---

## How adopters consume this update

| Distribution channel | Action required |
|---|---|
| `cargo install pai_kernel --version 1.3.2` | None · binary unchanged · documentation improvements visible via repository web view |
| `brew install PAI-Kernel/tap/pai-kernel` | `brew untap pai-kernel/tap && brew tap PAI-Kernel/tap` to refresh Caveats (formula change) · OR sustained если Caveats не critical |
| `docker pull ghcr.io/pai-kernel/pai-kernel:v2.2.3` | None · image unchanged |
| `git clone https://github.com/PAI-Kernel/pai-kernel.git` | `git pull` to receive doc + Cargo.toml updates |
| `curl https://paikernel.org/install.sh | sh` | None · install script unchanged · binary unchanged |

---

## Acknowledgments

Documentation improvements driven by adopter UX validation session (brew install + browser test attempt) revealing 16+ specific gaps between published documentation and actual binary behavior. Empirical artifact validation discipline applied throughout.

---

*PAI-Kernel SDK · v2.2.3 main HEAD documentation epoch · 2026-05-09 · Constitutional Governance Framework for AI · MIT OR Apache-2.0 · Mikhail Sergeev / PAI-Kernel Initiative*
