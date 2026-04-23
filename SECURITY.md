# Security Policy

## Scope

This repository is a constitutional governance framework for personal AI systems.
Security reports should focus on vulnerabilities that can enable:

- Governance bypass (GOV.BYPASS)
- Unauthorized state mutation
- Objective injection (OBJ.INJECTION)
- Decision log tampering (LOG.TAMPER)
- Conservative Mode disablement or avoidance (CONS.MODE.VIOLATION)
- Drift threshold manipulation (DRIFT.OVERTHRESHOLD)
- Consent gate bypass
- Witness chain integrity compromise

## Supported Versions

| Version | Supported |
|---------|-----------|
| Latest tagged release | Yes |
| `main` branch (HEAD) | Best-effort |
| Older releases | No |

## Reporting a Vulnerability

**Use GitHub Security Advisories (private disclosure):**

1. Go to **Security** tab → **Advisories** → **Report a vulnerability**
2. Provide:
   - Expected behavior vs observed behavior
   - Breach classification (if applicable)
   - Affected files/paths
   - Minimal reproduction steps
   - Any suggested fix

**Response timeline:**
- Acknowledgment: within 72 hours
- Triage: within 7 days
- Resolution target: within 30 days for confirmed vulnerabilities

**If private reporting is unavailable:** Email `contact@paikernel.org` with `[SECURITY]` in subject line. Do not include exploit details in public issues.

## Disclosure Policy

- We follow coordinated disclosure.
- Reporters will be credited unless they request anonymity.
- We will not pursue legal action against good-faith security researchers.

## Security Controls

This repository employs:
- Pinned GitHub Actions with SHA hashes
- Dependabot for Actions ecosystem monitoring
- Markdown linting and link verification in CI
- Cryptographic integrity verification (SHA-256 manifests for release assets)

Runtime security controls (cargo audit, CodeQL, SPARK formal verification, TLA+ model checking, OPA/Rego policy enforcement) are maintained in the internal development repository and verified before public releases.
