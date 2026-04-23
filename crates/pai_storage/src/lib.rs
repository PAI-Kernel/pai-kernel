//! # PAI Storage Layer (Component #3)
//!
//! Provides durable, append-only persistence for the PAI governance daemon.
//!
//! ## Architecture
//!
//! - **`GovernanceStore` trait** — abstract interface for governance state persistence.
//! - **`SqliteStore`** — SQLite-backed concrete implementation with:
//!   - WAL journal mode for concurrent-read safety.
//!   - Append-only triggers on `decision_log` (UPDATE / DELETE raise ABORT).
//!   - Separate `conservative_mode` table for fast crash-recovery reads.
//!
//! ## Invariants
//!
//! - STO-I1: Decision log rows are immutable once written (enforced by triggers).
//! - STO-I2: Conservative Mode flag survives process restart.
//! - STO-I3: Hash chain ordering is preserved (entries stored and retrieved by seq).
//! - STO-I4: Full `GovernanceState` round-trips through JSON without data loss.

#![forbid(unsafe_code)]

use pai_governance_daemon::{BreachClass, DecisionEntry, GovernanceState};
use thiserror::Error;

// ── Error ──────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("append-only violation: {0}")]
    AppendOnlyViolation(String),

    #[error("integrity error: {0}")]
    Integrity(String),
}

// ── Trait ───────────────────────────────────────────────────────────────

/// Abstract governance persistence layer.
///
/// Implementations MUST enforce append-only semantics on the decision log:
/// once a `DecisionEntry` is written, it cannot be modified or deleted through
/// the store interface.
pub trait GovernanceStore {
    /// Persist the full governance state snapshot (upsert).
    fn save_state(&self, state: &GovernanceState) -> Result<(), StorageError>;

    /// Load the most recently persisted governance state.
    /// Returns `None` on a fresh / empty store.
    fn load_state(&self) -> Result<Option<GovernanceState>, StorageError>;

    /// Append a single decision entry to the immutable log.
    fn append_decision(&self, entry: &DecisionEntry) -> Result<(), StorageError>;

    /// Append a batch of decision entries in a single transaction.
    fn append_decisions(&self, entries: &[DecisionEntry]) -> Result<(), StorageError>;

    /// Retrieve all decision entries ordered by sequence number.
    fn decisions(&self) -> Result<Vec<DecisionEntry>, StorageError>;

    /// Number of entries currently in the decision log.
    fn decision_count(&self) -> Result<u64, StorageError>;

    /// Persist the conservative-mode flag and optional breach class
    /// for fast crash-recovery without replaying the full log.
    fn save_conservative_mode(
        &self,
        active: bool,
        breach: Option<BreachClass>,
    ) -> Result<(), StorageError>;

    /// Load the persisted conservative-mode flag.
    /// Returns `(false, None)` on a fresh store.
    fn load_conservative_mode(&self) -> Result<(bool, Option<BreachClass>), StorageError>;
}

// ── SQLite backend ─────────────────────────────────────────────────────

/// Production-ready SQLite-backed governance store.
///
/// Initialises the schema and append-only triggers on first open.
/// Uses WAL journal mode for concurrent-read safety.
pub struct SqliteStore {
    conn: rusqlite::Connection,
}

impl SqliteStore {
    /// Open (or create) a store backed by a file at `path`.
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, StorageError> {
        let conn = rusqlite::Connection::open(path)?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    /// Open an ephemeral in-memory store (useful for tests).
    pub fn open_in_memory() -> Result<Self, StorageError> {
        let conn = rusqlite::Connection::open_in_memory()?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<(), StorageError> {
        self.conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            -- Full governance state (single-row, upsert pattern)
            CREATE TABLE IF NOT EXISTS governance_state (
                id          INTEGER PRIMARY KEY CHECK (id = 1),
                state_json  TEXT    NOT NULL,
                updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
            );

            -- Append-only decision log
            CREATE TABLE IF NOT EXISTS decision_log (
                seq         INTEGER PRIMARY KEY,
                entry_json  TEXT    NOT NULL,
                hash        TEXT    NOT NULL,
                prev_hash   TEXT    NOT NULL,
                created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
            );

            -- Conservative Mode quick-access (single-row, upsert)
            CREATE TABLE IF NOT EXISTS conservative_mode (
                id            INTEGER PRIMARY KEY CHECK (id = 1),
                active        INTEGER NOT NULL DEFAULT 0,
                breach_class  TEXT,
                updated_at    TEXT    NOT NULL DEFAULT (datetime('now'))
            );

            -- ── Append-only triggers (STO-I1) ──────────────────────
            CREATE TRIGGER IF NOT EXISTS decision_log_no_update
            BEFORE UPDATE ON decision_log
            BEGIN
                SELECT RAISE(ABORT, 'decision_log is append-only: UPDATE forbidden');
            END;

            CREATE TRIGGER IF NOT EXISTS decision_log_no_delete
            BEFORE DELETE ON decision_log
            BEGIN
                SELECT RAISE(ABORT, 'decision_log is append-only: DELETE forbidden');
            END;
            ",
        )?;
        Ok(())
    }

    /// Raw connection reference — exposed for testing trigger enforcement.
    #[cfg(test)]
    fn conn(&self) -> &rusqlite::Connection {
        &self.conn
    }
}

impl GovernanceStore for SqliteStore {
    fn save_state(&self, state: &GovernanceState) -> Result<(), StorageError> {
        let json = serde_json::to_string(state)?;
        self.conn.execute(
            "INSERT INTO governance_state (id, state_json, updated_at)
             VALUES (1, ?1, datetime('now'))
             ON CONFLICT(id) DO UPDATE SET
                state_json = excluded.state_json,
                updated_at = excluded.updated_at",
            [&json],
        )?;
        Ok(())
    }

    fn load_state(&self) -> Result<Option<GovernanceState>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT state_json FROM governance_state WHERE id = 1")?;
        let result = stmt.query_row([], |row| row.get::<_, String>(0));
        match result {
            Ok(json) => Ok(Some(serde_json::from_str(&json)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn append_decision(&self, entry: &DecisionEntry) -> Result<(), StorageError> {
        let json = serde_json::to_string(entry)?;
        self.conn.execute(
            "INSERT INTO decision_log (seq, entry_json, hash, prev_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))",
            rusqlite::params![entry.seq(), json, entry.hash(), entry.prev_hash(),],
        )?;
        Ok(())
    }

    fn append_decisions(&self, entries: &[DecisionEntry]) -> Result<(), StorageError> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO decision_log (seq, entry_json, hash, prev_hash, created_at)
                 VALUES (?1, ?2, ?3, ?4, datetime('now'))",
            )?;
            for entry in entries {
                let json = serde_json::to_string(entry)?;
                stmt.execute(rusqlite::params![
                    entry.seq(),
                    json,
                    entry.hash(),
                    entry.prev_hash(),
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    fn decisions(&self) -> Result<Vec<DecisionEntry>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT entry_json FROM decision_log ORDER BY seq ASC")?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<String>, _>>()?;
        rows.into_iter()
            .map(|json| serde_json::from_str(&json).map_err(StorageError::from))
            .collect()
    }

    fn decision_count(&self) -> Result<u64, StorageError> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM decision_log", [], |row| row.get(0))?;
        Ok(count as u64)
    }

    fn save_conservative_mode(
        &self,
        active: bool,
        breach: Option<BreachClass>,
    ) -> Result<(), StorageError> {
        let breach_json = match breach {
            Some(b) => Some(serde_json::to_string(&b)?),
            None => None,
        };
        self.conn.execute(
            "INSERT INTO conservative_mode (id, active, breach_class, updated_at)
             VALUES (1, ?1, ?2, datetime('now'))
             ON CONFLICT(id) DO UPDATE SET
                active       = excluded.active,
                breach_class = excluded.breach_class,
                updated_at   = excluded.updated_at",
            rusqlite::params![active as i32, breach_json],
        )?;
        Ok(())
    }

    fn load_conservative_mode(&self) -> Result<(bool, Option<BreachClass>), StorageError> {
        let result = self.conn.query_row(
            "SELECT active, breach_class FROM conservative_mode WHERE id = 1",
            [],
            |row| {
                let active: i32 = row.get(0)?;
                let breach_json: Option<String> = row.get(1)?;
                Ok((active, breach_json))
            },
        );
        match result {
            Ok((active, breach_json)) => {
                let breach = breach_json.and_then(|j| serde_json::from_str(&j).ok());
                Ok((active != 0, breach))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok((false, None)),
            Err(e) => Err(e.into()),
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use pai_governance_daemon::GovernanceDaemon;

    /// Helper: create a daemon, run some operations, return (daemon, store).
    fn daemon_with_ops() -> GovernanceDaemon {
        let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
        let vk = sk.verifying_key();
        let mut gd =
            GovernanceDaemon::new(10).with_author_keys("TEST_KEY", vk, Some(sk));
        gd.open_gate_for_testing();
        gd.snapshot();
        let _ = gd.ratify_add_objective("OBJ.ALPHA");
        let _ = gd.ratify_add_objective("OBJ.BETA");
        gd.accumulate_drift(3);
        gd
    }

    // ── STO-T01: Save and load governance state round-trip ─────────
    #[test]
    fn sto_t01_state_round_trip() {
        let gd = daemon_with_ops();
        let store = SqliteStore::open_in_memory().unwrap();

        store.save_state(gd.state()).unwrap();
        let loaded = store.load_state().unwrap().expect("state should exist");

        assert_eq!(loaded.drift(), gd.state().drift());
        assert_eq!(loaded.conservative(), gd.state().conservative());
        assert_eq!(loaded.objectives(), gd.state().objectives());
        assert_eq!(loaded.objective_registry(), gd.state().objective_registry());
        assert_eq!(loaded.drift_threshold(), gd.state().drift_threshold());
        assert_eq!(loaded.breach_flag(), gd.state().breach_flag());
        assert_eq!(
            loaded.capability_registry().len(),
            gd.state().capability_registry().len()
        );
        assert_eq!(
            loaded.consent_ledger().len(),
            gd.state().consent_ledger().len()
        );
        assert_eq!(
            loaded.delegations().len(),
            gd.state().delegations().len()
        );
    }

    // ── STO-T02: Append-only triggers block UPDATE and DELETE ──────
    #[test]
    fn sto_t02_append_only_triggers() {
        let gd = daemon_with_ops();
        let store = SqliteStore::open_in_memory().unwrap();

        // Populate decision log
        for entry in gd.log() {
            store.append_decision(entry).unwrap();
        }
        assert!(store.decision_count().unwrap() > 0);

        // Get the actual first seq in the log
        let first_seq = gd.log()[0].seq();

        // Attempt UPDATE — must be rejected by trigger
        let update_err = store.conn().execute(
            "UPDATE decision_log SET hash = 'tampered' WHERE seq = ?1",
            rusqlite::params![first_seq],
        );
        assert!(
            update_err.is_err(),
            "UPDATE on decision_log must be blocked by append-only trigger"
        );

        // Attempt DELETE — must be rejected by trigger
        let delete_err = store.conn().execute(
            "DELETE FROM decision_log WHERE seq = ?1",
            rusqlite::params![first_seq],
        );
        assert!(
            delete_err.is_err(),
            "DELETE on decision_log must be blocked by append-only trigger"
        );

        // Verify original data is untouched
        let entries = store.decisions().unwrap();
        assert_eq!(entries.len(), gd.log().len());
        assert_eq!(entries[0].hash(), gd.log()[0].hash());
    }

    // ── STO-T03: Conservative Mode persists across store instances ─
    #[test]
    fn sto_t03_conservative_mode_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("gov.db");

        // Instance 1: write conservative mode
        {
            let store = SqliteStore::open(&db_path).unwrap();
            store
                .save_conservative_mode(true, Some(BreachClass::DriftOverthreshold))
                .unwrap();
        }

        // Instance 2: reopen and verify
        {
            let store = SqliteStore::open(&db_path).unwrap();
            let (active, breach) = store.load_conservative_mode().unwrap();
            assert!(active, "conservative mode must survive reopen");
            assert_eq!(
                breach,
                Some(BreachClass::DriftOverthreshold),
                "breach class must survive reopen"
            );
        }

        // Instance 3: clear and verify
        {
            let store = SqliteStore::open(&db_path).unwrap();
            store.save_conservative_mode(false, None).unwrap();
            let (active, breach) = store.load_conservative_mode().unwrap();
            assert!(!active);
            assert_eq!(breach, None);
        }
    }

    // ── STO-T04: Decision log append and retrieval preserves order ─
    #[test]
    fn sto_t04_decision_log_ordering() {
        let gd = daemon_with_ops();
        let store = SqliteStore::open_in_memory().unwrap();

        // Append all entries
        store.append_decisions(gd.log()).unwrap();
        let count = store.decision_count().unwrap();
        assert_eq!(count, gd.log().len() as u64);

        // Retrieve and verify ordering
        let loaded = store.decisions().unwrap();
        assert_eq!(loaded.len(), gd.log().len());

        for (i, (stored, original)) in loaded.iter().zip(gd.log().iter()).enumerate() {
            assert_eq!(
                stored.seq(),
                original.seq(),
                "sequence mismatch at index {}",
                i
            );
            assert_eq!(
                stored.hash(),
                original.hash(),
                "hash mismatch at index {}",
                i
            );
            assert_eq!(
                stored.prev_hash(),
                original.prev_hash(),
                "prev_hash mismatch at index {}",
                i
            );
        }
    }

    // ── STO-T05: Hash chain integrity survives storage round-trip ──
    #[test]
    fn sto_t05_hash_chain_integrity() {
        let gd = daemon_with_ops();
        let store = SqliteStore::open_in_memory().unwrap();

        store.append_decisions(gd.log()).unwrap();
        let loaded = store.decisions().unwrap();

        // Verify chain linkage: each entry's prev_hash == previous entry's hash
        for i in 1..loaded.len() {
            assert_eq!(
                loaded[i].prev_hash(),
                loaded[i - 1].hash(),
                "hash chain broken at seq {} -> {}",
                loaded[i - 1].seq(),
                loaded[i].seq()
            );
        }

        // First entry should link to GENESIS or the daemon's genesis hash
        if !loaded.is_empty() {
            // Verify first entry matches original
            assert_eq!(loaded[0].prev_hash(), gd.log()[0].prev_hash());
        }
    }

    // ── STO-T06: Empty store returns correct defaults ──────────────
    #[test]
    fn sto_t06_empty_store_defaults() {
        let store = SqliteStore::open_in_memory().unwrap();

        // No state saved yet
        assert!(
            store.load_state().unwrap().is_none(),
            "fresh store must return None for state"
        );

        // No decisions yet
        assert_eq!(store.decision_count().unwrap(), 0);
        assert!(store.decisions().unwrap().is_empty());

        // Conservative mode defaults to (false, None)
        let (active, breach) = store.load_conservative_mode().unwrap();
        assert!(!active);
        assert_eq!(breach, None);
    }

    // ── STO-T07: State upsert overwrites previous snapshot ─────────
    #[test]
    fn sto_t07_state_upsert() {
        let store = SqliteStore::open_in_memory().unwrap();

        // Save initial state
        let gd1 = GovernanceDaemon::new(10);
        store.save_state(gd1.state()).unwrap();
        let s1 = store.load_state().unwrap().unwrap();
        assert_eq!(s1.drift(), 0);
        assert!(s1.objectives().is_empty());

        // Modify daemon and save again
        let mut gd2 = GovernanceDaemon::new(10);
        gd2.accumulate_drift(5);
        gd2.snapshot();
        store.save_state(gd2.state()).unwrap();

        // Reload — should reflect updated state
        let s2 = store.load_state().unwrap().unwrap();
        assert_eq!(s2.drift(), 5);
        assert_eq!(s2.drift_threshold(), 10);
    }

    // ── STO-T08: Full lifecycle — store, close, reopen, verify ─────
    #[test]
    fn sto_t08_full_lifecycle() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("lifecycle.db");

        let gd = daemon_with_ops();

        // Phase 1: persist everything
        {
            let store = SqliteStore::open(&db_path).unwrap();
            store.save_state(gd.state()).unwrap();
            store.append_decisions(gd.log()).unwrap();
            store
                .save_conservative_mode(
                    gd.state().conservative(),
                    gd.state().breach_flag(),
                )
                .unwrap();
        }

        // Phase 2: reopen from scratch and verify all data
        {
            let store = SqliteStore::open(&db_path).unwrap();

            // State
            let state = store.load_state().unwrap().expect("state must persist");
            assert_eq!(state.drift(), gd.state().drift());
            assert_eq!(state.conservative(), gd.state().conservative());
            assert_eq!(state.objectives(), gd.state().objectives());
            assert_eq!(state.drift_threshold(), gd.state().drift_threshold());

            // Decision log
            let decisions = store.decisions().unwrap();
            assert_eq!(decisions.len(), gd.log().len());
            for (stored, original) in decisions.iter().zip(gd.log().iter()) {
                assert_eq!(stored.seq(), original.seq());
                assert_eq!(stored.hash(), original.hash());
                assert_eq!(stored.prev_hash(), original.prev_hash());
                assert_eq!(stored.action(), original.action());
            }

            // Hash chain
            for i in 1..decisions.len() {
                assert_eq!(
                    decisions[i].prev_hash(),
                    decisions[i - 1].hash(),
                    "hash chain broken after reopen at index {}",
                    i
                );
            }

            // Conservative mode
            let (active, breach) = store.load_conservative_mode().unwrap();
            assert_eq!(active, gd.state().conservative());
            assert_eq!(breach, gd.state().breach_flag());
        }
    }
}
