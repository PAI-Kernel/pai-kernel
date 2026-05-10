---
title: "HTTP API Reference — PAI-Kernel SDK"
slug: api
position: 6
hidden: false
excerpt: "Five REST endpoints exposed by pai_governance_daemon · request/response schemas · examples."
pai_cd:
  version: "2.2.3"
  status: "Canonical"
  source:
    file: "api.md"
    path: "docs/api.md"
    commit: "main"
    authority_repo: "PAI-Kernel/pai-kernel"
  cite_as: "PAI-Kernel SDK v2.2.3 · API Reference"
  last_amendment: "2026-05-09"
---

# PAI-Kernel SDK · HTTP API Reference

`pai_governance_daemon` exposes five REST endpoints under `/api/v1/`. All responses are JSON. Default bind address: `127.0.0.1:9100`.

> **Prerequisite:** daemon must be running. Start via `pai_governance_daemon --demo` (testing) OR with required environment variables (production · see [`INSTALL.md`](INSTALL.md) § Configuration).

---

## Authentication

In v2.2.3 all five endpoints are **read-only** and **unauthenticated** at the HTTP layer. Authentication considerations:

- Default bind `127.0.0.1` (loopback only) prevents network exposure
- For network-exposed deployments, place behind reverse proxy (nginx · Caddy · Traefik) with TLS + auth
- The `PAI_AUTHOR_API_KEY` environment variable is used for internal cryptographic operations (witness chain signing) · NOT for HTTP authentication

---

## Endpoints

### `GET /api/v1/version`

Returns SDK version, PAI-CD revision, and Rust toolchain.

**Request:**

```sh
curl -s http://127.0.0.1:9100/api/v1/version
```

**Response (200 OK):**

```json
{
  "version": "1.3.2",
  "pai_cd_version": "3.1",
  "rust_toolchain": "1.88"
}
```

**Field semantics:**

- `version` · workspace SDK version (semver · maps to release tag)
- `pai_cd_version` · constitutional document revision
- `rust_toolchain` · MSRV used to build the binary

**Use case:** version compatibility check · adopter scripts validating runtime version before integration.

---

### `GET /api/v1/health`

Returns daemon health status, witness chain entry count, and conservative mode flag.

**Request:**

```sh
curl -s http://127.0.0.1:9100/api/v1/health
```

**Response (200 OK):**

```json
{
  "status": "ok",
  "witness_entries": 0,
  "conservative_mode": false
}
```

**Field semantics:**

- `status` · `"ok"` (healthy) OR `"degraded"` (partial) OR error
- `witness_entries` · count of witness chain log entries (0 for fresh daemon · grows over time)
- `conservative_mode` · boolean · whether daemon is in conservative mode (Tier ≥2 actions blocked)

**Use case:** monitoring · liveness probe for systemd/Docker/Kubernetes · adopter health checks.

---

### `GET /api/v1/log`

Returns witness log entries (chronological, JSON array).

**Request:**

```sh
curl -s http://127.0.0.1:9100/api/v1/log
```

**Response (200 OK · empty log):**

```json
[]
```

**Response (200 OK · with entries):**

```json
[
  {
    "timestamp": "2026-05-09T10:00:00Z",
    "kind": "ConsentGranted",
    "actor": "AUTHOR",
    "details": {...},
    "prev_hash": "0000...",
    "hash": "abcd..."
  }
]
```

**Use case:** audit log review · adopter auditing daemon-observed events · forensic analysis.

---

### `GET /api/v1/log/verify`

Verifies witness log integrity via hash chain validation.

**Request:**

```sh
curl -s http://127.0.0.1:9100/api/v1/log/verify
```

**Response (200 OK · verified):**

```json
{
  "verified": true,
  "entries_checked": 0,
  "violations": []
}
```

**Response (200 OK · violation detected):**

```json
{
  "verified": false,
  "entries_checked": 100,
  "violations": [
    {
      "entry_index": 42,
      "violation_type": "hash_chain_break",
      "expected_prev_hash": "abcd...",
      "actual_prev_hash": "ef01..."
    }
  ]
}
```

**Use case:** integrity check · run periodically OR after suspected tampering · CI gate for adopter audit pipelines.

**Note:** verification is fail-closed at daemon startup · this endpoint provides on-demand re-verification.

---

### `GET /api/v1/export`

Exports the full governance bundle as JSON (witness log + governance state + capability registry + drift state).

**Request:**

```sh
curl -s http://127.0.0.1:9100/api/v1/export > governance_bundle.json
```

**Response (200 OK · partial structure):**

```json
{
  "witness_log": [...],
  "governance_state": {
    "authority_context": {
      "active_delegate": null,
      "actor_type": "Author",
      "current_actor": "AUTHOR"
    },
    "breach_flag": null,
    "capability_registry": [...],
    "objectives": [...],
    "drift_state": {...}
  },
  "metadata": {
    "exported_at": "2026-05-09T10:00:00Z",
    "version": "1.3.2"
  }
}
```

**Field semantics:**

- `witness_log` · same content as `/api/v1/log`
- `governance_state` · current authority context · breach flag · capability registry · objectives · drift state
- `metadata` · export timestamp + version

**Use case:** state portability per Authorial Sovereignty principle (PAI-CD §P1) · adopter migration · backup snapshots · audit exports.

---

## Error responses

| HTTP code | Meaning | Body |
|---|---|---|
| `200` | Success | JSON response per endpoint |
| `404` | Endpoint not found | `{"error": "not_found", "path": "/some/path"}` |
| `500` | Internal error | `{"error": "internal", "message": "..."}` |
| `503` | Daemon not ready | `{"error": "not_ready", "message": "..."}` |

---

## Endpoints NOT exposed in v2.2.3

For transparency, these endpoints are sometimes assumed but do not exist in v2.2.3:

- `POST /api/v1/...` · no write endpoints (all current endpoints read-only)
- `/api/v1/policy` · policy state via export only · no dedicated endpoint
- `/api/v1/verify` · use `/api/v1/log/verify` (witness log scope) instead
- `/api/v1/witness/append` · witness chain append is internal to daemon · not exposed via HTTP in v2.2.3
- `/api/v1/admin/*` · no admin endpoints

Future versions may expose write endpoints with explicit authentication. See [`docs/RELEASE_NOTES_v2.2.3.1.md`](RELEASE_NOTES_v2.2.3.1.md) for current direction.

---

## Programmatic clients

### Rust (auto-generated docs)

```sh
# View pai_api crate documentation
cargo doc -p pai_api --open
```

OR online at <https://docs.rs/pai_api>.

### Python (manual integration)

```python
import requests

BASE = "http://127.0.0.1:9100/api/v1"

def get_version():
    return requests.get(f"{BASE}/version").json()

def get_health():
    return requests.get(f"{BASE}/health").json()

def get_log():
    return requests.get(f"{BASE}/log").json()

def verify_log():
    return requests.get(f"{BASE}/log/verify").json()

def export_bundle():
    return requests.get(f"{BASE}/export").json()

if __name__ == "__main__":
    print(get_version())
    print(get_health())
```

### Shell (curl + jq)

```sh
curl -s http://127.0.0.1:9100/api/v1/health | jq '.status'
curl -s http://127.0.0.1:9100/api/v1/version | jq '.version'
```

---

## See also

- [`docs/INSTALL.md`](INSTALL.md) — installation + configuration
- [`docs/quickstart.md`](quickstart.md) — 5-minute getting-started
- [`docs/architecture.md`](architecture.md) — high-level architecture
- [`docs/audit_checklist.md`](audit_checklist.md) — verification procedures
- `cargo doc -p pai_api` — Rust API documentation (offline)

---

*PAI-Kernel SDK · v2.2.3 · Constitutional Governance Framework for AI · MIT OR Apache-2.0 · Mikhail Sergeev / PAI-Kernel Initiative*
