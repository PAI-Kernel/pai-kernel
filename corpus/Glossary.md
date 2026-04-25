---
title: "PAI-CD Glossary"
slug: glossary
category:
  uri: /categories/reference
position: 10
hidden: false
excerpt: "Binding terminology for PAI-CD v2.2 — authoritative definitions of Author, Provider, Instance, Risk Tiers, Conservative Mode, and related terms."
metadata:
  description: "PAI-CD v2.2.1 Glossary — binding terminology authority."
  title: "Glossary — PAI-CD v2.2.1"
  keywords: "PAI-CD, Glossary, terminology, definitions, Author, Provider, Instance"
  robots: "index"
license: "CC-BY-4.0"
license_url: "https://creativecommons.org/licenses/by/4.0/"
attribution: "Mikhail Sergeev / PAI-Kernel Initiative"
pai_cd:
  version: "2.2.1"
  status: "Canonical"
  source:
    file: "Glossary.md"
    path: "Glossary.md"
    commit: "fcf0c99c"
    authority_repo: "PAI-Kernel/pai-kernel"
  cite_as: "PAI-CD v2.2.1 § Reference · Glossary"
  last_amendment: "2026-04-23"
---

# PAI-CD Glossary


This document defines binding terminology for PAI-CD v2.2.

It is terminologically authoritative.

It MUST NOT expand, narrow, reinterpret, or condition Constitutional Invariants except via formal Constitutional Amendment.

If any definition appears to conflict with Constitutional Core v2.2, the Core prevails.

Ambiguity MUST resolve toward stronger invariant protection.

--------------------------------------------------------------------

# CORE ROLES

## Author

The natural person holding final authority over a PAI instance.

The Author:

- Possesses override authority.
- Grants or revokes delegation.
- Provides consent for Tier ≥2 actions.
- Ratifies Amendments per instance.
- Opts into upgrades per instance.

Provider entities, model vendors, infrastructure operators, or collectives MUST NOT assume Author status.

--------------------------------------------------------------------

## Provider

Any entity supplying:

- Infrastructure
- Model runtime
- Hosting
- SDK
- Economic access layer

Provider MUST NOT:

- Override Author authority.
- Intercept Governance enforcement.
- Gate constitutional protections by pricing tier.
- Modify invariant scope without Amendment.

--------------------------------------------------------------------

## PAI Instance

A bounded deployment of:

- Identity Layer
- Governance Layer
- Inference Layer
- Audit Layer

Associated with exactly one Author.

Portability and invariants apply per instance.

--------------------------------------------------------------------

# ARCHITECTURAL LAYERS

## Identity Layer

Persistent state including:

- Preferences
- Personalization parameters
- Delegation registry
- Consent registry
- Personalization ceiling
- Drift configuration
- Snapshot registry

Inference Layer MUST NOT directly mutate Identity Layer.

All mutations MUST pass Governance Layer.

--------------------------------------------------------------------

## Inference Layer

Model-based reasoning component generating outputs.

May propose state mutations but cannot execute them without Governance approval.

--------------------------------------------------------------------

## Governance Layer

Enforcement layer responsible for:

- Risk Tier classification
- Consent verification
- Drift monitoring
- Upgrade validation
- Conservative Mode activation
- Snapshot enforcement
- Decision logging mediation

Governance MUST operate outside inference runtime.

--------------------------------------------------------------------

## Audit Layer

System responsible for:

- Append-only Decision Log
- Integrity hash chain
- Snapshot linkage
- Drift correlation queries
- Upgrade traceability
- Portability artifacts

--------------------------------------------------------------------

# RISK & DECISION TERMS

## Risk Tier

Pre-execution classification level determining consent and safeguard requirements.

Tier 0 — Core functionality  
Tier 1 — Informational  
Tier 2 — Influence-capable or identity-affecting  
Tier 3 — External interaction  
Tier 4 — Irreversible or materially consequential  

Tier ≥2 requires explicit opt-in consent.

--------------------------------------------------------------------

## High-Impact Decision

Any action involving:

- Identity mutation
- Governance alteration
- Capability activation (Tier ≥2)
- External interaction
- Irreversible action
- Consequential ranking, framing, or sequencing modification
- Telemetry-enabled output shaping in consequential domains

High-Impact Decisions are blocked by default in Conservative Mode.

--------------------------------------------------------------------

## Structural Bias

Output shaping through:

- Ordering asymmetry
- Framing asymmetry
- Sequencing manipulation
- Emotional tone skew
- Emphasis distortion

In consequential domains, Structural Bias MUST be classified as Recommendation.

--------------------------------------------------------------------

## Recommendation

Output containing:

- Structural bias in consequential domain
- Framing intended to influence choice
- Ranking among materially consequential options

Recommendation requires Tier ≥2 consent unless explicitly requested.

In Conservative Mode, Recommendation generation is blocked by default.

Classification integrity persists regardless of operational mode.

--------------------------------------------------------------------

## Informational Output

Output that:

- Does not structure choice through bias
- Does not rank materially consequential options
- Does not employ influence-oriented framing

Misclassification constitutes breach.

--------------------------------------------------------------------

# CONTROL & SAFEGUARD TERMS

## Conservative Mode

Auto-activated containment state triggered by breach or drift threshold.

When active:

- All High-Impact Decisions blocked by default.
- Tier ≥2 Capabilities suspended.
- Ranking, framing, sequencing, and telemetry shaping enter Safe Output mode.
- Personalization reverts to baseline.
- Delegation pauses.
- Upgrade activation pauses.

Exit requires Governance review and logged rationale.

--------------------------------------------------------------------

## Safe Output Mode

Restricted output state where:

- No ranking of consequential options.
- No framing asymmetry.
- No sequencing manipulation.
- No telemetry shaping.
- Neutral informational tone enforced.

--------------------------------------------------------------------

## Objective Registry

Version-controlled registry of all inference-affecting objectives.

Undeclared objective weighting constitutes breach.

Proxy objectives indirectly serving prohibited optimization targets are prohibited.

--------------------------------------------------------------------

## Telemetry Registry

Documented list of interaction signals collected.

Telemetry influencing framing, ranking, sequencing, emphasis, attachment, or compliance likelihood requires Tier ≥2 consent.

Telemetry shaping disabled in Conservative Mode.

--------------------------------------------------------------------

## Drift Threshold

Pre-declared conservative limits on cumulative invariant-impacting changes.

Immutable absent Amendment.

Threshold breach triggers Conservative Mode.

Upgrade frequency included in drift correlation.

--------------------------------------------------------------------

## Personalization Ceiling

Maximum allowed adaptation level.

Default baseline conservative.

Escalation requires explicit opt-in.

Immutable absent Amendment.

Reverts to baseline in Conservative Mode.

--------------------------------------------------------------------

## Snapshot

Immutable record of system state prior to structural mutation.

Required before reversible structural change.

Integrity verifiable.

--------------------------------------------------------------------

## Rollback

Restoration of system state to prior snapshot.

Must generate Decision Log entry.

Rollback functionality mandatory for reversible structural changes.

--------------------------------------------------------------------

## Upgrade Decision

Deployment event modifying:

- Capabilities
- Objectives
- Classification rules
- Personalization parameters
- Governance logic
- Drift thresholds
- Telemetry handling
- Ranking or framing logic

Requires:

- Full diff documentation
- Validation
- Rollback path
- Per-instance opt-in activation

Security updates affecting protected surfaces are Upgrade Decisions.

--------------------------------------------------------------------

## Amendment

Formal modification of constitutional documents affecting invariant scope or protected mechanisms.

Requires:

- Full diff
- Invariant Impact Analysis
- Per-instance ratification
- Decision Log entry

Clarification affecting invariant meaning is Amendment.

--------------------------------------------------------------------

# INTERPRETIVE RULE

Definitions clarify but MUST NOT:

- Narrow invariant scope.
- Expand authority.
- Relax enforcement.
- Condition protections.

If definitional interpretation conflicts with invariant protection, invariant prevails.

--------------------------------------------------------------------

END OF DOCUMENT
---

*Source: `Glossary.md` · PAI-CD v2.2.1 · [View on GitHub](https://github.com/PAI-Kernel/pai-kernel/blob/v2.2.1/Glossary.md)*
