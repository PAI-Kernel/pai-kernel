#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "Running compliance harness (pai_compliance)..."
cargo run -p pai_compliance --locked > compliance_report.json
echo "Wrote compliance_report.json"
