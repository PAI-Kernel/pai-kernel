# Contributing to PAI-Kernel

## PAI Constitutional Document (PAI-CD)

Thank you for your interest in PAI-Kernel.

PAI-Kernel is a normative framework — the PAI Constitutional Document (PAI-CD) —
accompanied by a reference implementation (the PAI-Kernel SDK). Contributions can
engage with either:

- **Governance** — the constitutional corpus, threat model, invariants,
  amendment procedure
- **Implementation** — the SDK code, Conformance Suite, formal verification

The bar for both is high by design: a weakened framework that is widely adopted
is worse than a rigorous framework adopted narrowly. The same discipline applies
to the implementation.

---

## What we are looking for

### High-value contributions

**Specification gaps.** If you find a requirement in PAI-CD that cannot be implemented
unambiguously by an independent team, that is a genuine finding. Open an Issue.

**Threat model gaps.** If you identify a manipulation or governance capture vector not
addressed by the current threat model, describe it precisely: the attack, the mechanism,
the failure mode, and why existing invariants do not close it.

**Invariant conflicts.** If two invariants produce contradictory requirements in a
realistic deployment scenario, document the scenario and the conflict.

**Formal verification contributions.** The TLA+ specification (StateModel.tla,
CoordinationModel.tla, governance_boundary_model.tla; verified properties documented
in VERIFICATION_MATRIX.md) is currently maintained in the project's internal
repository. Public release of the formal artifacts is scheduled for the framework's
Phase 3 gate. Contributors interested in formal verification work may reach out via
<contact@paikernel.org> for current state and collaboration.

**Reference implementations.** Independent implementations of PAI-CD that pass the
Conformance Suite are the primary metric of the framework's success. We want to know
about them.

### What we are not looking for

- Simplification of normative requirements for adoption convenience.
- Changes that weaken protections without closing a documented threat.
- Philosophical reformulations that do not change enforceable behavior.

---

## Amendment procedure

Changes to the normative corpus (PAI-CD documents) require a formal Amendment procedure:

1. **Open an Issue** describing the proposed change, the invariant it affects, and the direction (strengthening or clarification).
2. **14-day discussion period.** The community engages with the proposal.
3. **Invariant Impact Analysis.** The change must be assessed for protective
   direction — no amendment may weaken invariant protection without explicit
   justification.
4. **Ratification.** P0 ratifies after CAO non-weakening declaration.
5. **7-day pre-activation delay** before the change takes effect.

Amendments that strengthen protections have a lower procedural burden than amendments
that weaken them. This asymmetry is intentional.

---

## How to open an Issue

Use the provided issue templates:

- **Amendment Proposal** — for proposed changes to normative documents
- **Specification Gap** — for ambiguities or implementation gaps

If your contribution does not fit either template, open a blank issue with a clear title describing the type of contribution.

---

## Style

- Write in the language of the document you are modifying (English for the normative corpus).
- Use MUST / MUST NOT / SHOULD / MAY for normative clauses. Avoid "should" as a
  casual hedge — use SHOULD only when you mean RFC 2119.
- Name the specific clause, principle, or invariant affected by your change.
- Describe failure modes explicitly.

---

## Attribution

Documentation contributions to the constitutional corpus are licensed under
CC BY 4.0. Code contributions to the SDK are licensed under MIT OR Apache-2.0.
By submitting a contribution, you agree that it may be included in the
PAI-Kernel project under these terms.

Significant contributions will be acknowledged in the project's CONTRIBUTORS file and, where appropriate, in CITATION.cff.

---

PAI-Kernel · paikernel.org · <contact@paikernel.org>
