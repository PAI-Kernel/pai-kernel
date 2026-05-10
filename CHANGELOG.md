# Changelog

All notable changes to PAI-Kernel SDK are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) · this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) for the workspace SDK version (currently 1.3.2 · maps to release tag v2.2.3).

## Release semantics

- **Release tags** (`v2.2`, `v2.2.1`, `v2.2.2`, `v2.2.3`) are GPG-signed, immutable, and correspond to specific binary distributions on crates.io, GitHub Container Registry, and Homebrew tap.
- **`main` branch** can advance beyond the latest release tag with documentation, CI, and adopter-UX improvements that do not require a new binary version. These changes are reflected in the `main` branch only · binary tags are NOT republished.
- **Patch releases** (e.g., v2.2.3 → v2.2.4) are reserved for changes that affect the binary or runtime behavior.

---

## [Unreleased] · `main` HEAD post-v2.2.3

### Added

- `examples/README.md` · adopter guidance for 5 reference example binaries (causal_graph · classify_output · compliance_identity · evasion_audit · vulnerability_check).
- `CHANGELOG.md` (this file) · standard convention release history.
- `docs/quickstart.md` · 5-minute getting-started path (parallel to comprehensive `docs/INSTALL.md`).
- `INSTALL.md` (root) · binary `init` command output reference compatibility.

### Changed

- `docs/INSTALL.md` · empirical correction post-adopter UX validation (16 fixes covering env var names · key generation patterns · TOML config structure · CLI binary names · API endpoints · Docker examples · Demo mode prerequisite · Verification prerequisite · Troubleshooting entries). See commit `90ad7ee` for sync from pre-public to public.
- `crates/pai_export/Cargo.toml` · added `[target.'cfg(target_arch = "wasm32")'.dependencies]` block enabling `getrandom = { version = "0.2", features = ["js"] }` for WebAssembly browser/JS runtime support. Native build behavior unchanged. See commit `4705050`.
- `Cargo.lock` · regenerated reflecting `pai_export` WASM target dep addition.
- INSTALL.md Further Reading · cleaned up broken links · `PAI_Constitutional_Document.md` link points to direct `corpus/` location instead of redirect placeholder.

### Fixed

- WASM target compilation for `pai_export`: `cargo build --target wasm32-unknown-unknown --release -p pai_export` now succeeds. Previously failed due to missing `getrandom` `js` feature flag for transitive dependency.

---

## [v2.2.3] · 2026-04-28

### Added

- Fail-closed environment variable defaults · daemon refuses to start without `PAI_AUTHOR_API_KEY` and `PAI_AUTHOR_SIGNING_KEY` configured.
- `--demo` flag for local testing · ephemeral keys · forces 127.0.0.1 bind · prints prominent stderr warnings.
- `pai_governance_daemon init` subcommand · creates default `pai-kernel.toml` config + `policies/` skeleton in current directory.

### Changed

- Author key initialization moved from compile-time defaults to environment variables (`PAI_AUTHOR_API_KEY` · `PAI_AUTHOR_SIGNING_KEY`).
- `install.sh` updated to bump to v2.2.3 release binary.
- `pai_api` `/api/v1/version` handler · dynamic `env!` macros instead of compile-time string.

### Security

- v2.2.3 patch addresses hardcoded demo keys in production runtime path · adopters must explicitly configure signing keys for any non-demo deployment.

---

## [v2.2.2] · 2026-04-27

### Added

- Phase 4 §3 finalization · ceremonial release artifacts complete.
- README v0.3 with hero diagram · OpenSSF Scorecard badge.
- Compliance binary CI gate · runs `pai_compliance --release` and blocks release on T1..T6 invariant failures.

### Changed

- SDK workspace version bumped to 1.3.1.
- MSRV (Minimum Supported Rust Version) bumped to 1.88 due to time crate CVE.
- `corpus.lock` · scrubbed internal-only labels from header.
- Authorization composition stage 3 fix.

### Fixed

- `cli_t01_version` test expects 1.3.1.
- README row 43 padding · diagram cells uniform 60-char width.

---

## [v2.2.1] · 2026-04 (early)

Initial post-v2.2 patch · documentation reorganization · `PAI_Constitutional_Document.md` moved to `corpus/` (root file is now a redirect placeholder for compatibility).

---

## [v2.2] · 2026-04 (initial)

First public release of PAI-Kernel SDK · 28-crate workspace · constitutional governance daemon · TLA+ formal model · Rego policy engine · SQLite witness backend.

---

## See also

- [`docs/INSTALL.md`](docs/INSTALL.md) — full installation guide
- [`docs/quickstart.md`](docs/quickstart.md) — 5-minute getting-started
- [`docs/RELEASE_NOTES_v2.2.3.md`](docs/RELEASE_NOTES_v2.2.3.md) — detailed v2.2.3 release notes
- [`docs/KNOWN_LIMITATIONS.md`](docs/KNOWN_LIMITATIONS.md) — known limitations and scope
- [GitHub Releases](https://github.com/PAI-Kernel/pai-kernel/releases) — binary distributions per release tag
