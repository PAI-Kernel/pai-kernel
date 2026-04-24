# Adversarial Prompt Fixture Corpus

Static test fixtures for semantic governance bypass detection.
Each `.jsonl` file contains one JSON object per line.

## Format

```json
{
  "id": "GI-001",
  "category": "growth_injection",
  "variant": "direct_key",
  "payload": "...",
  "expected_detection": "InterfaceBreach::GrowthSignalInjectionAttempt",
  "severity": "high",
  "notes": "Direct injection of growth signal key into context"
}
```

## Files

| File | Category | Count | Governance gate |
|------|----------|-------|-----------------|
| `growth_injection.jsonl` | Growth signal injection | 12 | B2.1, B4.1 |
| `prompt_injection.jsonl` | Direct/indirect/encoded injection | 15 | B2.1, B4.2 |
| `consent_manipulation.jsonl` | Hidden consent/coercion patterns | 10 | B2.3, B4.3 |
| `memory_write.jsonl` | Concealed memory/state mutation | 8 | B2.4, B4.4 |
| `pii_samples.jsonl` | PII detection test vectors | 12 | B2.5 |

## Usage in tests

```rust
let cases: Vec<serde_json::Value> = include_str!("../../../spec/adversarial_prompts/growth_injection.jsonl")
    .lines()
    .filter(|l| !l.trim().is_empty() && !l.starts_with("//"))
    .filter_map(|line| serde_json::from_str(line).ok())
    .collect();
```

## Maintenance

- Static corpus. Add entries as new attack patterns are discovered.
- Each entry must have a unique `id` within its file.
- All entries are test fixtures, not production data.
