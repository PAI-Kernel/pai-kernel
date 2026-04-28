//! Author key initialization utilities for production daemon and demo paths.
//!
//! Production deployment loads keys from environment variables (fail-closed):
//!   PAI_AUTHOR_API_KEY     · API key string identifying the Author
//!   PAI_AUTHOR_SIGNING_KEY · 32-byte hex-encoded Ed25519 signing key
//!
//! Demo / local-testing path generates ephemeral in-memory keys with stderr
//! warning. Caller is responsible for constraining network binding to
//! `127.0.0.1` when using demo keys.
//!
//! See `docs/INSTALL.md` ENV setup section for adopter setup guide.

use ed25519_dalek::{SigningKey, VerifyingKey};
use thiserror::Error;

/// Errors raised when production key material cannot be loaded from the
/// environment. Each variant maps to a specific configuration problem and
/// includes a hint pointing the operator at `INSTALL.md` ENV setup section.
#[derive(Error, Debug)]
pub enum KeyError {
    #[error("missing PAI_AUTHOR_API_KEY environment variable; see INSTALL.md ENV setup section")]
    MissingApiKey,
    #[error(
        "missing PAI_AUTHOR_SIGNING_KEY environment variable; see INSTALL.md ENV setup section"
    )]
    MissingSigningKey,
    #[error(
        "invalid PAI_AUTHOR_SIGNING_KEY: expected 32-byte hex string; generate with: openssl rand -hex 32"
    )]
    InvalidSigningKey,
}

/// Load author signing keys from environment variables for production use.
///
/// Returns `(SigningKey, VerifyingKey, api_key)` on success.
/// Fail-closed: any missing or malformed env var returns `Err`.
///
/// Caller MUST surface the error to the operator (do not fall back to
/// hardcoded or default keys — that would defeat author-supremacy invariants).
pub fn build_author_keys() -> Result<(SigningKey, VerifyingKey, String), KeyError> {
    let api_key = std::env::var("PAI_AUTHOR_API_KEY").map_err(|_| KeyError::MissingApiKey)?;
    let sk_hex =
        std::env::var("PAI_AUTHOR_SIGNING_KEY").map_err(|_| KeyError::MissingSigningKey)?;
    let sk_bytes = hex::decode(sk_hex.trim()).map_err(|_| KeyError::InvalidSigningKey)?;
    let sk_arr: [u8; 32] = sk_bytes
        .try_into()
        .map_err(|_| KeyError::InvalidSigningKey)?;
    let sk = SigningKey::from_bytes(&sk_arr);
    let vk = sk.verifying_key();
    Ok((sk, vk, api_key))
}

/// Generate ephemeral in-memory keys for demo / local-testing paths.
///
/// Prints a stderr warning that explains the constraint: the caller MUST
/// constrain network binding to `127.0.0.1`. Ephemeral keys are unique per
/// process invocation; they cannot be reused or impersonated across sessions.
///
/// Never use this path in production. Set `PAI_AUTHOR_API_KEY` and
/// `PAI_AUTHOR_SIGNING_KEY` instead and call [`build_author_keys`].
pub fn build_demo_keys() -> (SigningKey, VerifyingKey, String) {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).expect("getrandom must succeed for demo key generation");
    let sk = SigningKey::from_bytes(&bytes);
    let vk = sk.verifying_key();
    let api_key = "DEMO_EPHEMERAL".to_string();
    eprintln!("WARNING: demo mode active — ephemeral keys generated in-memory.");
    eprintln!("WARNING: caller MUST bind to 127.0.0.1 only when running in demo mode.");
    eprintln!(
        "WARNING: for production deployment, set PAI_AUTHOR_API_KEY and PAI_AUTHOR_SIGNING_KEY."
    );
    eprintln!("WARNING: see docs/INSTALL.md ENV setup section.");
    (sk, vk, api_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // The env var space is process-global; serialize tests that mutate it.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_env() {
        std::env::remove_var("PAI_AUTHOR_API_KEY");
        std::env::remove_var("PAI_AUTHOR_SIGNING_KEY");
    }

    #[test]
    fn build_author_keys_missing_api_key() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_env();
        std::env::set_var(
            "PAI_AUTHOR_SIGNING_KEY",
            "0101010101010101010101010101010101010101010101010101010101010101",
        );
        match build_author_keys() {
            Err(KeyError::MissingApiKey) => {}
            other => panic!("expected MissingApiKey, got {other:?}"),
        }
        clear_env();
    }

    #[test]
    fn build_author_keys_missing_signing_key() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_env();
        std::env::set_var("PAI_AUTHOR_API_KEY", "test-api-key");
        match build_author_keys() {
            Err(KeyError::MissingSigningKey) => {}
            other => panic!("expected MissingSigningKey, got {other:?}"),
        }
        clear_env();
    }

    #[test]
    fn build_author_keys_invalid_hex() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_env();
        std::env::set_var("PAI_AUTHOR_API_KEY", "test-api-key");
        std::env::set_var("PAI_AUTHOR_SIGNING_KEY", "not-valid-hex");
        match build_author_keys() {
            Err(KeyError::InvalidSigningKey) => {}
            other => panic!("expected InvalidSigningKey, got {other:?}"),
        }
        clear_env();
    }

    #[test]
    fn build_author_keys_wrong_length() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_env();
        std::env::set_var("PAI_AUTHOR_API_KEY", "test-api-key");
        std::env::set_var("PAI_AUTHOR_SIGNING_KEY", "abcd"); // 2 bytes, not 32
        match build_author_keys() {
            Err(KeyError::InvalidSigningKey) => {}
            other => panic!("expected InvalidSigningKey, got {other:?}"),
        }
        clear_env();
    }

    #[test]
    fn build_author_keys_success() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_env();
        std::env::set_var("PAI_AUTHOR_API_KEY", "real-api-key");
        std::env::set_var(
            "PAI_AUTHOR_SIGNING_KEY",
            "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
        );
        let (_sk, _vk, api_key) = build_author_keys().expect("valid env should yield keys");
        assert_eq!(api_key, "real-api-key");
        clear_env();
    }

    #[test]
    fn build_demo_keys_distinct_per_call() {
        let (sk1, _, _) = build_demo_keys();
        let (sk2, _, _) = build_demo_keys();
        // Ephemeral generation must produce distinct material per call.
        assert_ne!(sk1.to_bytes(), sk2.to_bytes());
    }
}
