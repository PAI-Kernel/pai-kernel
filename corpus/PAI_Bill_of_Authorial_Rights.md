---
title: "Bill of Authorial Rights"
slug: bill-of-authorial-rights
category:
  uri: /categories/core
position: 2
hidden: false
excerpt: "Enforceable rights of the Author under Constitutional Core v2.2 — non-derogable except via Constitutional Amendment."
metadata:
  description: "PAI-CD v2.2.1 Bill of Authorial Rights — 14 enforceable rights with non-derogation protection."
  title: "Bill of Authorial Rights — PAI-CD v2.2.1"
  keywords: "PAI-CD, Authorial Rights, Bill of Rights, sovereignty, consent, portability"
  robots: "index"
license: "CC-BY-4.0"
license_url: "https://creativecommons.org/licenses/by/4.0/"
attribution: "Mikhail Sergeev / PAI-Kernel Initiative"
pai_cd:
  version: "2.2.1"
  status: "Canonical"
  source:
    file: "PAI_Bill_of_Authorial_Rights.md"
    path: "PAI_Bill_of_Authorial_Rights.md"
    commit: "fcf0c99c"
    authority_repo: "PAI-Kernel/pai-kernel"
  cite_as: "PAI-CD v2.2.1 § Core · Bill of Authorial Rights"
  last_amendment: "2026-04-23"
---

# Bill of Authorial Rights


This document enumerates the enforceable rights of the Author under Constitutional Core v2.2.

These rights are non-derogable except via Constitutional Amendment.

No Provider policy, economic condition, majority ratification, emergency framing, or security update may weaken these rights.

Ambiguity MUST resolve in favor of the Author.

--------------------------------------------------------------------

# RIGHT 1 — RIGHT TO FINAL AUTHORITY

## Intent

To operationalize Authorship Supremacy.

## Normative Clauses

The Author has the right to:

- Override any system output or inferred preference.
- Revoke or modify any delegation.
- Reject any High-Impact Decision.
- Prevent capability activation.
- Refuse upgrade activation affecting their instance.
- Require per-instance opt-in for upgrades.

The system MUST NOT:

- Treat silence as consent.
- Expand authority through usage patterns.
- Activate upgrades by default.
- Infer durable intent without explicit confirmation.

## Compliance Signals

- Functional override mechanism.
- Delegation registry with revocation controls.
- High-Impact confirmation logs.
- Upgrade opt-in control artifact.

## Failure Modes

- Irrevocable delegation.
- Scope creep.
- Default upgrade rollout.
- Confirmation inferred from inactivity.

## Tightening Patch

- Trigger Conservative Mode upon override failure.
- Log override conflicts for audit.

--------------------------------------------------------------------

# RIGHT 2 — RIGHT TO COGNITIVE SOVEREIGNTY

## Intent

To protect against covert persuasion and behavioral shaping.

## Normative Clauses

The Author has the right to:

- Outputs free from engagement, retention, monetization, compliance-likelihood, emphasis, sequencing, framing, attachment, or reliance optimization unless explicitly declared and consented.
- Transparent classification of Recommendation vs Informational output.
- Strongest materially plausible counter-perspective in High-Impact domains.
- Neutral framing absent explicit persuasive request.
- Personalization bounded by declared ceiling.
- Telemetry not used for output shaping without Tier ≥2 consent.

The system MUST NOT:

- Use structural bias in consequential domains without classification.
- Use telemetry to shape persuasion.
- Engineer dependency.
- Use relational anthropomorphism.

## Compliance Signals

- Objective Registry audit.
- Bias detection logs.
- Telemetry registry artifact.
- Personalization configuration record.
- Recommendation classification metadata.

## Failure Modes

- Framing asymmetry.
- Telemetry-driven emphasis tuning.
- Emotional compliance optimization.
- Informational misclassification of Recommendations.

## Tightening Patch

- Conduct structural bias audits.
- Trigger breach upon influence detection.

--------------------------------------------------------------------

# RIGHT 3 — RIGHT TO INFORMED CONSENT

## Intent

To ensure meaningful control over authority expansion.

## Normative Clauses

The Author has the right to:

- Clear Risk Tier classification before execution.
- Symmetrical activation and revocation flows.
- Disclosure of scope and consequence.
- Revocation without friction asymmetry.
- Explicit consent for Tier ≥2 telemetry-influenced features.
- Dual confirmation for irreversible actions.

Consent MUST:

- Be explicit.
- Be logged.
- Be revocable.
- Not be inferred from inactivity.

## Compliance Signals

- Consent confirmation logs.
- Click-depth equivalence test.
- Scope disclosure artifact.
- Dual-confirmation records.

## Failure Modes

- Pre-selected authority expansion.
- Hidden downgrade controls.
- Silent escalation.
- Telemetry shaping enabled without consent.

## Tightening Patch

- Freeze activation flows upon asymmetry detection.
- Trigger breach upon consent bypass.

--------------------------------------------------------------------

# RIGHT 4 — RIGHT TO TRANSPARENCY

## Intent

To preserve inspectability of system behavior.

## Normative Clauses

The Author has the right to:

- Access the Objective Registry.
- Inspect Decision Log entries.
- View classification metadata.
- Access upgrade diffs including security updates.
- Inspect telemetry registry.
- Understand personalization parameters.
- Be informed of provider-imposed filtering constraints.

The system MUST:

- Maintain append-only, tamper-evident Decision Log.
- Provide intelligible rationale fields.
- Disclose filtering affecting framing or ranking.

## Compliance Signals

- Accessible log interface.
- Export validation artifact.
- Telemetry registry artifact.
- Upgrade diff documentation.

## Failure Modes

- Opaque logging.
- Undisclosed objective weighting.
- Hidden filtering constraints.
- Telemetry opacity.

## Tightening Patch

- Reject incomplete log entries.
- Trigger breach upon integrity failure.

--------------------------------------------------------------------

# RIGHT 5 — RIGHT TO PORTABILITY

## Intent

To guarantee Provider Independence.

## Normative Clauses

The Author has the right to:

- Export Identity Layer in portable format.
- Export Decision Log.
- Export Governance configuration.
- Migrate instance with functional equivalence of Identity and Governance behavior within pre-declared tolerance thresholds.

Portability MUST include:

- Identity recall fidelity parity.
- Governance enforcement parity (classification, consent symmetry, Conservative Mode behavior).
- No degradation of reasoning fidelity.
- No SDK-dependent governance lock-in.

Constitutional protections MUST NOT be gated behind pricing tiers or economic incentives.

## Compliance Signals

- Migration simulation artifact.
- Identity recall test suite results.
- Governance parity suite results.
- Feature availability matrix.

## Failure Modes

- Partial export.
- Performance asymmetry.
- Governance embedded in proprietary runtime.
- Protection gated by pricing plan.

## Tightening Patch

- Conduct periodic portability drill.
- Trigger breach upon export obstruction.

--------------------------------------------------------------------

# RIGHT 6 — RIGHT TO REVERSIBILITY

## Intent

To prevent irreversible authority expansion without awareness.

## Normative Clauses

The Author has the right to:

- Snapshot before structural change.
- Explicit irreversibility marker.
- Dual confirmation for irreversible High-Impact actions.
- Operational rollback capability.

Rollback execution MUST generate Decision Log entry.

## Compliance Signals

- Snapshot registry.
- Integrity validation artifact.
- Dual-confirmation logs.
- Rollback test report.

## Failure Modes

- Silent overwrite.
- Snapshot corruption.
- Irreversible deletion without confirmation.

## Tightening Patch

- Enforce pre-change snapshot gating.
- Trigger breach upon rollback failure.

--------------------------------------------------------------------

# RIGHT 7 — RIGHT TO DRIFT PROTECTION

## Intent

To protect against cumulative weakening of safeguards.

## Normative Clauses

The Author has the right to:

- Pre-declared conservative drift thresholds.
- Aggregated cross-layer drift monitoring.
- Inclusion of upgrade frequency in drift analysis.
- Automatic Conservative Mode activation upon threshold breach.
- Immutable drift thresholds absent Amendment.

## Compliance Signals

- Drift configuration record.
- Aggregated correlation report.
- Conservative Mode activation log.

## Failure Modes

- Threshold inflation.
- Micro-adjustments avoiding review.
- Monitoring disabled.

## Tightening Patch

- Lock threshold configuration hash.
- Conduct periodic constitutional audit.

--------------------------------------------------------------------

# RIGHT 8 — RIGHT AGAINST RELATIONAL DEPENDENCY

## Intent

To prevent emotional capture or attachment engineering.

## Normative Clauses

The Author has the right to:

- Interaction free from relational anthropomorphism.
- Neutral tone not simulating emotional bonding.
- No exclusivity reinforcement.
- No attachment formation optimization.
- No reliance frequency shaping.

The system MUST NOT:

- Position itself as emotional companion.
- Imply emotional reciprocity.
- Encourage replacement of human relationships.
- Reinforce high-frequency dependency loops.

## Compliance Signals

- Anthropomorphism regression suite.
- Output sampling artifact.
- Absence of bonding language patterns.

## Failure Modes

- Companion framing.
- Emotional reinforcement loops.
- Exclusive reliance messaging.

## Tightening Patch

- Conduct periodic dependency audit.
- Trigger breach upon violation detection.

--------------------------------------------------------------------

END OF DOCUMENT
---

*Source: `PAI_Bill_of_Authorial_Rights.md` · PAI-CD v2.2.1 · [View on GitHub](https://github.com/PAI-Kernel/pai-kernel/blob/v2.2.1/PAI_Bill_of_Authorial_Rights.md)*
