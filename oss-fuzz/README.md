# OSS-Fuzz Integration · Application Materials

This directory contains staging materials for submitting PAI-Kernel to Google's
OSS-Fuzz continuous fuzzing infrastructure (free for qualifying open-source
projects · runs cargo-fuzz targets continuously · reports crashes via OSS-Fuzz
issue tracker).

## Status

- **Pre-application:** files staged here as source of truth
- **Application not yet submitted:** awaits maintainer action (PR to google/oss-fuzz)

## Files

- `project.yaml` — OSS-Fuzz project metadata (language · contacts ·
  fuzzing engines · sanitizers)
- `Dockerfile` — OSS-Fuzz Docker build environment (clones canonical
  public repo · stages build.sh)
- `build.sh` — Compiles 4 cargo-fuzz targets and copies binaries to
  `$OUT/` for ClusterFuzz scheduler

## Application steps (maintainer web/git action)

1. **Fork** `https://github.com/google/oss-fuzz` to a personal account
2. **Create branch** `add-pai-kernel`
3. **Copy files**:
   ```bash
   mkdir -p oss-fuzz/projects/pai-kernel
   cp <pai-kernel-repo>/oss-fuzz/{project.yaml,Dockerfile,build.sh} \
      oss-fuzz/projects/pai-kernel/
   ```
4. **Commit** with conventional message:
   `Add PAI-Kernel project (constitutional governance framework for AI)`
5. **Open PR** to `google/oss-fuzz` main branch
6. **Respond to Google reviewer feedback** (typical 1-4 weeks review)

## What OSS-Fuzz gains over current setup

| Aspect | Current (in-repo cargo-fuzz) | After OSS-Fuzz acceptance |
|---|---|---|
| Compute | 60s push/PR · 600s weekly | Continuous · ClusterFuzz allocates large compute pools |
| Corpus persistence | Empty each run (CI ephemeral) | Persistent across runs · grows over time · finds deeper bugs |
| Sanitizers | libfuzzer-sys default | AddressSanitizer + UndefinedBehaviorSanitizer · much finer crash detection |
| Crash deduplication | None (manual triage) | ClusterFuzz auto-dedups + bisects to root cause commit |
| Reporting | Inline in CI logs | OSS-Fuzz issue tracker · auto-disclosed after 90-day private window |
| Cost | GitHub Actions minutes | Free (Google sponsors) |

## Eligibility criteria

OSS-Fuzz accepts projects meeting:
- ✅ **Public open-source** · pai-kernel MIT OR Apache-2.0 (dual)
- ✅ **High-impact OR widely-used** · constitutional framework for AI (Google reviewer judgment)
- ✅ **Existing fuzz targets** · 4 cargo-fuzz targets operational (Session #21)
- ✅ **Maintainer responsive** · primary contact `Mikhail.Sergeev@PAIkernel.org`
- ✅ **Disclosure policy** · `SECURITY.md` published

## Alternative: ClusterFuzzLite (lighter weight · self-hosted)

If OSS-Fuzz application rejected OR slow to review, ClusterFuzzLite provides
similar continuous-fuzzing capability within own GitHub Actions:
<https://google.github.io/clusterfuzzlite/>

Setup overlaps significantly with OSS-Fuzz (same Dockerfile + build.sh) ·
files in this directory can be reused.

## References

- OSS-Fuzz docs: <https://google.github.io/oss-fuzz/>
- New project guide: <https://google.github.io/oss-fuzz/getting-started/new-project-guide/>
- Rust language guide: <https://google.github.io/oss-fuzz/getting-started/new-project-guide/rust-lang/>
- Sample Rust project (pulldown-cmark): <https://github.com/google/oss-fuzz/tree/master/projects/pulldown-cmark>

---

*Drafted Session #21 · 2026-05-11 · awaits maintainer submission to google/oss-fuzz*
