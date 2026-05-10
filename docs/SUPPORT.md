---
title: "Support — PAI-Kernel SDK"
slug: support
position: 4
hidden: false
excerpt: "Support tiers · invitation-only context · response time expectations · how to ask for help versus how to report security issues."
pai_cd:
  version: "2.2.3"
  status: "Canonical"
  source:
    file: "SUPPORT.md"
    path: "docs/SUPPORT.md"
    commit: "main"
    authority_repo: "PAI-Kernel/pai-kernel"
  cite_as: "PAI-Kernel SDK v2.2.3 · Support"
  last_amendment: "2026-05-09"
---

# PAI-Kernel SDK · Support

This document describes how to get help with PAI-Kernel SDK · what to expect · and what is out of scope.

> **Security issues:** see [`SECURITY.md`](../SECURITY.md). NOT this document. Security reports route through GitHub Security Advisory (private) OR PGP-encrypted email.

---

## Project context

PAI-Kernel SDK is an **early-preview · invitation-only distribution** maintained by **Mikhail Sergeev** as Independent Researcher and PAI-Kernel Initiative founder. The project is in active development pre-v2.3 cycle.

This means:

- **No commercial support contract** · this is open-source software (MIT OR Apache-2.0 · documentation CC BY 4.0)
- **No SLA-backed response times** · best effort by single maintainer
- **No on-call rotation** · responses depend on maintainer availability
- **No paid support tier** at present · changes to support model documented separately when introduced

If your use case requires guaranteed response times OR commercial support, please open a discussion (see Channels below) to explore arrangements.

---

## Support channels

### 1. GitHub Discussions · general questions

<https://github.com/PAI-Kernel/pai-kernel/discussions>

Use for:

- «How do I configure X?»
- «Is Y a supported use case?»
- «Has anyone tried Z deployment scenario?»
- Architecture / design questions
- Roadmap / future direction questions

Response expectation: best effort · typically days to weeks. Community contributions welcome (responses from non-maintainers also valuable).

### 2. GitHub Issues · bug reports + feature requests

<https://github.com/PAI-Kernel/pai-kernel/issues>

Use for:

- Reproducible bugs (include `pai_governance_daemon --version` · OS · steps to reproduce · expected vs actual behavior)
- Feature requests (include use case rationale · alternative solutions considered)
- Documentation gaps (include specific page · section · what's missing OR confusing)

Response expectation: triage best-effort. Critical bugs (security · data integrity · daemon crash) prioritized highest. Cosmetic issues OR nice-to-have features may sustain unaddressed for cycles.

Please **search existing issues first** to avoid duplicates.

### 3. GitHub Security Advisory (private) · security issues

<https://github.com/PAI-Kernel/pai-kernel/security/advisories/new>

For vulnerability reports · NOT general bugs. See [`SECURITY.md`](../SECURITY.md) for full responsible disclosure protocol · PGP key · scope.

### 4. Direct email · invitation-only adopter coordination

`<contact@paikernel.org>`

Reserved for:

- Adopters who received explicit invitation to the early-preview distribution
- Standards body engagement coordination
- Strategic partnership discussions
- Press / academic citation inquiries

**Not** for general bug reports OR support questions (use GitHub Issues / Discussions instead).

---

## What to include in a support request

### For bugs

- PAI-Kernel version: `pai_governance_daemon --version`
- OS + version: `uname -a`
- Rust toolchain (if building from source): `rustc --version`
- Installation method: brew · cargo · docker · install.sh · manual binary · source
- Reproducible steps (numbered list)
- Expected behavior vs actual behavior
- Daemon log output (`pai_governance_daemon --demo` for testing scenarios)
- Configuration: `pai-kernel.toml` content (redact secrets if any)

### For configuration questions

- What you're trying to accomplish (use case)
- What you've tried (configuration · commands)
- What's not working (error message · unexpected behavior)
- Reference to [`docs/INSTALL.md`](INSTALL.md) section if you've already consulted

### For feature requests

- Use case rationale (what problem does this solve?)
- Alternative solutions considered (why none of them work?)
- Constitutional alignment context (does this fit framework principles?)
- Implementation sketch (optional · how might this work?)

---

## Response time expectations

| Channel | Typical response time | Notes |
|---|---|---|
| GitHub Security Advisory | 7 days | Per [`SECURITY.md`](../SECURITY.md) protocol |
| GitHub Issues · critical bug | 7-14 days | Best effort · single maintainer |
| GitHub Issues · normal bug | 14-30 days | May sustain longer if blocked on architectural decision |
| GitHub Issues · feature request | 30+ days OR sustained | Triaged into roadmap · response may be «sustained until vN+1 cycle» |
| GitHub Discussions | 7-30 days | Community responses possible |
| Email · invitation-only | Per individual arrangement | Reserved scope |

These are expectations · not guarantees. Maintainer availability fluctuates with project cycle phases (release ceremony · audit cycle · etc.).

---

## What is out of scope

PAI-Kernel SDK is a constitutional governance substrate for AI systems · NOT:

- An AI model itself (no language model · no inference engine bundled)
- A general policy framework (focused on PAI-CD invariants T1..T6)
- A production-ready SaaS product (you self-host the daemon)
- A multi-tenant SaaS (single-tenant by design)
- A replacement for application-level governance frameworks (PAI-Kernel SDK is a primitive · application built on top supplies the rest)

Questions outside scope receive «out of scope · see X for related concerns» style responses · not full support engagement.

---

## Contributing

If you encountered an issue and want to contribute a fix:

1. Open an issue first to discuss the problem (avoid wasted effort on unwanted changes)
2. Read [`CONTRIBUTING.md`](../CONTRIBUTING.md) for contribution guidelines
3. Fork · branch · PR · sign commits with GPG (release discipline preserves signed-commit chain)
4. Be patient with review (single maintainer · may take 14-30+ days)

---

## See also

- [`SECURITY.md`](../SECURITY.md) — security vulnerability reporting (REQUIRED for security issues)
- [`CONTRIBUTING.md`](../CONTRIBUTING.md) — contribution guidelines
- [`docs/INSTALL.md`](INSTALL.md) — installation guide (read first for install issues)
- [`docs/quickstart.md`](quickstart.md) — 5-minute getting-started
- [`docs/KNOWN_LIMITATIONS.md`](KNOWN_LIMITATIONS.md) — known limitations and scope statement

---

*PAI-Kernel SDK · v2.2.3 · Constitutional Governance Framework for AI · MIT OR Apache-2.0 · Mikhail Sergeev / PAI-Kernel Initiative*
