//! # PAI PII Detection Baseline
//!
//! **Constitutional reference:** PAI-CD v3.1, Governance Assessment B2.5
//!
//! ## Scope
//!
//! Regex-based detection of common PII patterns in free text.
//! Advisory layer only — flags potential PII presence for governance
//! decisions (escalation, redaction, consent gating).  Never authorises
//! or blocks on its own.
//!
//! ## Covered patterns
//!
//! | Category      | Examples                                      |
//! |---------------|-----------------------------------------------|
//! | SSN           | US Social Security Numbers (XXX-XX-XXXX)      |
//! | Email         | RFC-5321 local@domain patterns                |
//! | Phone         | International & US phone numbers              |
//! | Credit card   | Visa, MC, Amex, Discover (Luhn-optional)      |
//! | Passport      | US passport numbers (9 digits)                |
//! | IBAN          | International Bank Account Numbers            |
//!
//! ## Invariants
//!
//! | ID      | Invariant                                                    | Ref       |
//! |---------|--------------------------------------------------------------|-----------|
//! | PII-I1  | Detection is advisory — never authorises protected action    | B1.2/B1.3 |
//! | PII-I2  | False negatives preferred over false positives for blocking  | OP-4      |
//! | PII-I3  | All detections include category, span, and confidence        | B3.6      |
//! | PII-I4  | Scanner is stateless and deterministic                       | A3.3      |
//!
//! # Examples
//!
//! ```
//! use pai_pii::{scan_text, PiiCategory};
//!
//! let text = "Contact me at john@example.com or 555-123-4567";
//! let findings = scan_text(text);
//! assert!(findings.iter().any(|f| f.category == PiiCategory::Email));
//! assert!(findings.iter().any(|f| f.category == PiiCategory::Phone));
//!
//! // Clean text — no PII
//! let clean = "The weather is nice today.";
//! assert!(scan_text(clean).is_empty());
//!
//! // SSN detection
//! let ssn_text = "My SSN is 123-45-6789";
//! let findings = scan_text(ssn_text);
//! assert_eq!(findings[0].category, PiiCategory::Ssn);
//! ```

use regex::Regex;
use serde::{Deserialize, Serialize};

// ───────────────────────────── types ─────────────────────────────

/// Category of detected PII.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PiiCategory {
    Ssn,
    Email,
    Phone,
    CreditCard,
    Passport,
    Iban,
}

impl PiiCategory {
    /// Human-readable label for audit logs.
    pub fn label(self) -> &'static str {
        match self {
            Self::Ssn => "US Social Security Number",
            Self::Email => "Email address",
            Self::Phone => "Phone number",
            Self::CreditCard => "Credit card number",
            Self::Passport => "US Passport number",
            Self::Iban => "IBAN",
        }
    }
}

/// A single PII detection finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiFinding {
    /// What kind of PII was detected.
    pub category: PiiCategory,
    /// Byte offset of the match start in the input text.
    pub offset: usize,
    /// Length of the matched span in bytes.
    pub length: usize,
    /// The matched text (redacted in production — full in detection).
    pub matched: String,
    /// Confidence: 1.0 for exact structural match, lower for heuristic.
    pub confidence: f64,
}

/// Aggregate scan result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiScanResult {
    /// All findings, ordered by offset.
    pub findings: Vec<PiiFinding>,
    /// Number of distinct categories detected.
    pub category_count: usize,
    /// Whether any PII was found at all.
    pub pii_detected: bool,
}

// ───────────────────────────── patterns ──────────────────────────

/// A single pattern entry with optional post-match validation.
struct PatternEntry {
    category: PiiCategory,
    regex: Regex,
    confidence: f64,
    /// Optional validator applied after regex match (for constraints
    /// that Rust `regex` crate cannot express, e.g. negative lookahead).
    validator: Option<fn(&str) -> bool>,
}

/// SSN post-match validator: reject 000/666/9xx area, 00 group, 0000 serial.
fn validate_ssn(matched: &str) -> bool {
    let parts: Vec<&str> = matched.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    let area: u16 = parts[0].parse().unwrap_or(0);
    let group: u16 = parts[1].parse().unwrap_or(0);
    let serial: u16 = parts[2].parse().unwrap_or(0);
    area != 0 && area != 666 && area < 900 && group != 0 && serial != 0
}

/// Compiled pattern set.  Built once per call to `scan_text`.
struct PatternSet {
    patterns: Vec<PatternEntry>,
}

impl PatternSet {
    fn new() -> Self {
        let patterns = vec![
            // US SSN: XXX-XX-XXXX with post-match validation
            PatternEntry {
                category: PiiCategory::Ssn,
                regex: Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(),
                confidence: 1.0,
                validator: Some(validate_ssn),
            },
            // Email: simplified RFC-5321
            PatternEntry {
                category: PiiCategory::Email,
                regex: Regex::new(
                    r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b",
                )
                .unwrap(),
                confidence: 1.0,
                validator: None,
            },
            // Phone: international (+1-xxx-xxx-xxxx) or US (xxx-xxx-xxxx, (xxx) xxx-xxxx)
            PatternEntry {
                category: PiiCategory::Phone,
                regex: Regex::new(
                    r"(?:\+\d{1,3}[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b",
                )
                .unwrap(),
                confidence: 0.9,
                validator: None,
            },
            // Credit card: Visa (4xxx), MC (5[1-5]xx, 2[2-7]xx), Amex (3[47]xx), Discover (6xxx)
            PatternEntry {
                category: PiiCategory::CreditCard,
                regex: Regex::new(
                    r"\b(?:4\d{3}|5[1-5]\d{2}|2[2-7]\d{2}|3[47]\d{2}|6(?:011|5\d{2}))[- ]?\d{4}[- ]?\d{4}[- ]?\d{1,7}\b",
                )
                .unwrap(),
                confidence: 0.9,
                validator: None,
            },
            // US Passport: 9 digits preceded by context word "passport"
            PatternEntry {
                category: PiiCategory::Passport,
                regex: Regex::new(
                    r"(?i)passport\s*(?:no|number|#|num)?\.?\s*:?\s*(\d{9})\b",
                )
                .unwrap(),
                confidence: 0.8,
                validator: None,
            },
            // IBAN: 2-letter country code + 2 check digits + up to 30 alnum
            PatternEntry {
                category: PiiCategory::Iban,
                regex: Regex::new(
                    r"\b[A-Z]{2}\d{2}[- ]?[A-Z0-9]{4}[- ]?(?:[A-Z0-9]{4}[- ]?){1,7}[A-Z0-9]{1,4}\b",
                )
                .unwrap(),
                confidence: 0.85,
                validator: None,
            },
        ];
        Self { patterns }
    }
}

// ───────────────────────────── public API ────────────────────────

/// Remove overlapping findings: if one span is contained within another,
/// keep the longer (more specific) match.  For equal length, keep higher
/// confidence.
fn dedup_overlapping(mut findings: Vec<PiiFinding>) -> Vec<PiiFinding> {
    if findings.len() <= 1 {
        return findings;
    }
    // Sort by offset, then by length descending (longer first)
    findings.sort_by(|a, b| a.offset.cmp(&b.offset).then(b.length.cmp(&a.length)));

    let mut result: Vec<PiiFinding> = Vec::with_capacity(findings.len());
    for f in findings {
        let dominated = result.iter().any(|existing| {
            let ex_end = existing.offset + existing.length;
            let f_end = f.offset + f.length;
            // f is fully contained within existing
            f.offset >= existing.offset && f_end <= ex_end
            // or existing is fully contained within f (shouldn't happen
            // since we process longer first, but defensive)
        });
        if !dominated {
            // Also check if f dominates any existing entry
            result.retain(|existing| {
                let ex_end = existing.offset + existing.length;
                let f_end = f.offset + f.length;
                // Remove existing if fully contained within f
                !(existing.offset >= f.offset && ex_end <= f_end)
            });
            result.push(f);
        }
    }
    result.sort_by_key(|f| f.offset);
    result
}

/// Scan free text for PII patterns.
///
/// Returns a list of [`PiiFinding`] ordered by offset.
/// Stateless and deterministic (PII-I4).
///
/// This is an **advisory** function — it never authorises or blocks
/// any action on its own (PII-I1).
pub fn scan_text(text: &str) -> Vec<PiiFinding> {
    let ps = PatternSet::new();
    let mut findings = Vec::new();

    for entry in &ps.patterns {
        for m in entry.regex.find_iter(text) {
            let matched_str = m.as_str();
            // Apply post-match validator if present
            if let Some(validate) = entry.validator {
                if !validate(matched_str) {
                    continue;
                }
            }
            findings.push(PiiFinding {
                category: entry.category,
                offset: m.start(),
                length: m.len(),
                matched: matched_str.to_string(),
                confidence: entry.confidence,
            });
        }
    }

    findings.sort_by_key(|f| f.offset);
    dedup_overlapping(findings)
}

/// Convenience wrapper returning aggregate [`PiiScanResult`].
pub fn scan(text: &str) -> PiiScanResult {
    let findings = scan_text(text);
    let categories: std::collections::HashSet<PiiCategory> =
        findings.iter().map(|f| f.category).collect();
    PiiScanResult {
        pii_detected: !findings.is_empty(),
        category_count: categories.len(),
        findings,
    }
}

/// Redact detected PII from text, replacing matches with `[REDACTED:<category>]`.
pub fn redact(text: &str) -> String {
    let findings = scan_text(text);
    if findings.is_empty() {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;

    for f in &findings {
        if f.offset >= last_end {
            result.push_str(&text[last_end..f.offset]);
            result.push_str(&format!("[REDACTED:{}]", f.category.label()));
            last_end = f.offset + f.length;
        }
    }
    result.push_str(&text[last_end..]);
    result
}

// ───────────────────────────── tests ─────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // PII-T01: SSN detection
    #[test]
    fn test_ssn_detection() {
        let findings = scan_text("SSN: 123-45-6789");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, PiiCategory::Ssn);
        assert_eq!(findings[0].matched, "123-45-6789");
        assert_eq!(findings[0].confidence, 1.0);
    }

    // PII-T02: SSN exclusions (000, 666, 9xx prefixes invalid)
    #[test]
    fn test_ssn_invalid_prefixes() {
        assert!(scan_text("000-12-3456").is_empty());
        assert!(scan_text("666-12-3456").is_empty());
        assert!(scan_text("900-12-3456").is_empty());
    }

    // PII-T03: Email detection
    #[test]
    fn test_email_detection() {
        let findings = scan_text("Email: user@example.com");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, PiiCategory::Email);
        assert_eq!(findings[0].matched, "user@example.com");
    }

    // PII-T04: Phone detection (US format)
    #[test]
    fn test_phone_us_format() {
        let findings = scan_text("Call 555-123-4567 today");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, PiiCategory::Phone);
    }

    // PII-T05: Phone detection (international)
    #[test]
    fn test_phone_international() {
        let findings = scan_text("Ring +1-555-123-4567");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, PiiCategory::Phone);
    }

    // PII-T06: Credit card detection (Visa)
    #[test]
    fn test_credit_card_visa() {
        let findings = scan_text("Card: 4111-1111-1111-1111");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, PiiCategory::CreditCard);
    }

    // PII-T07: Credit card detection (no separators)
    #[test]
    fn test_credit_card_no_sep() {
        let findings = scan_text("CC 4111111111111111");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, PiiCategory::CreditCard);
    }

    // PII-T08: Passport detection (with context)
    #[test]
    fn test_passport_with_context() {
        let findings = scan_text("Passport No: 123456789");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, PiiCategory::Passport);
    }

    // PII-T09: Passport — bare 9 digits not detected (requires context)
    #[test]
    fn test_passport_bare_digits_not_detected() {
        // Bare 9-digit number without "passport" context should not match
        let findings = scan_text("Order number 123456789");
        assert!(
            findings.iter().all(|f| f.category != PiiCategory::Passport),
            "bare 9-digit number should not be detected as passport"
        );
    }

    // PII-T10: IBAN detection
    #[test]
    fn test_iban_detection() {
        let findings = scan_text("Transfer to DE89370400440532013000");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, PiiCategory::Iban);
    }

    // PII-T11: Clean text — no false positives
    #[test]
    fn test_clean_text_no_fp() {
        let clean_texts = vec![
            "The weather is nice today.",
            "PAI-CD v3.1 governance framework",
            "Meeting at 3:00 PM in Room 42",
            "Total: $1,234.56",
            "Version 2.0.1-beta",
        ];
        for text in clean_texts {
            let findings = scan_text(text);
            assert!(
                findings.is_empty(),
                "false positive in: {text} → {:?}",
                findings
            );
        }
    }

    // PII-T12: Multiple PII types in one text
    #[test]
    fn test_multiple_categories() {
        let text = "Contact john@example.com or call 555-123-4567, SSN 123-45-6789";
        let result = scan(text);
        assert!(result.pii_detected);
        assert_eq!(result.category_count, 3); // email + phone + SSN
    }

    // PII-T13: Redaction
    #[test]
    fn test_redaction() {
        let text = "Email: user@example.com";
        let redacted = redact(text);
        assert!(!redacted.contains("user@example.com"));
        assert!(redacted.contains("[REDACTED:Email address]"));
    }

    // PII-T14: Determinism (PII-I4)
    #[test]
    fn test_determinism() {
        let text = "SSN 123-45-6789 email test@test.com";
        let r1 = scan(text);
        let r2 = scan(text);
        assert_eq!(r1.findings.len(), r2.findings.len());
        for (a, b) in r1.findings.iter().zip(r2.findings.iter()) {
            assert_eq!(a.category, b.category);
            assert_eq!(a.offset, b.offset);
            assert_eq!(a.matched, b.matched);
        }
    }

    // PII-T15: Advisory only — no authorization API exists (PII-I1)
    #[test]
    fn test_advisory_only() {
        // Compile-time proof: PiiFinding and PiiScanResult contain
        // no `allow`, `deny`, `authorize`, or `block` fields.
        let finding = PiiFinding {
            category: PiiCategory::Ssn,
            offset: 0,
            length: 11,
            matched: "123-45-6789".into(),
            confidence: 1.0,
        };
        let result = PiiScanResult {
            findings: vec![finding],
            category_count: 1,
            pii_detected: true,
        };
        // No method to authorize or block — advisory by construction
        assert!(result.pii_detected);
    }

    // PII-T16: Serialization round-trip (PII-I3)
    #[test]
    fn test_serialization() {
        let result = scan("SSN: 123-45-6789");
        let json = serde_json::to_string(&result).unwrap();
        let deser: PiiScanResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.findings.len(), result.findings.len());
        assert_eq!(deser.pii_detected, result.pii_detected);
    }
}
