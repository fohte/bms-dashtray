mod best_score;
mod score_log;
mod song_metadata;

pub use best_score::{BestScore, read_all_best_scores};
pub use score_log::read_score_log;
pub use song_metadata::{build_md5_to_sha256_map, read_song_metadata};

use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScoreDataLog {
    pub sha256: String,
    pub mode: i32,
    pub clear: i32,
    pub ex_score: i32,
    pub min_bp: i32,
    pub notes: i32,
    pub combo: i32,
    /// Number of notes consumed by judgements (used to detect mid-play retirement).
    pub consumed_notes: i32,
    /// ISO 8601 formatted date string converted from UNIX time (seconds).
    pub played_at: String,
    /// Raw UNIX timestamp in seconds from the database.
    pub date_secs: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum DBError {
    #[error("database file not found: {0}")]
    FileNotFound(String),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

fn open_readonly(path: &Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    conn.busy_timeout(BUSY_TIMEOUT)?;
    Ok(conn)
}

fn open_readonly_checked(path: &Path) -> Result<Connection, DBError> {
    if !path.exists() {
        return Err(DBError::FileNotFound(path.display().to_string()));
    }
    Ok(open_readonly(path)?)
}

fn unix_secs_to_iso8601(secs: i64) -> Result<String, rusqlite::Error> {
    let dt: DateTime<Utc> = DateTime::from_timestamp(secs, 0).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            10,
            rusqlite::types::Type::Integer,
            format!("invalid timestamp secs: {secs}").into(),
        )
    })?;
    Ok(dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
}

/// beatoraja mode value for PMS (9-key / popn).
/// In PMS, BAD (ebd/lbd) does not consume notes (`judgeVanish=false`),
/// unlike 5key/7key where BAD does consume notes.
const MODE_PMS: i32 = 2;

pub fn read_all_score_data_logs(path: &Path) -> Result<Vec<ScoreDataLog>, DBError> {
    let conn = open_readonly_checked(path)?;
    let mut stmt = conn.prepare(
        "SELECT sha256, mode, clear, epg, egr, egd, ebd, epr, lpg, lgr, lgd, lbd, lpr, \
         minbp, notes, combo, date \
         FROM scoredatalog",
    )?;

    // Column indices matching the SELECT clause above.
    const COL_SHA256: usize = 0;
    const COL_MODE: usize = 1;
    const COL_CLEAR: usize = 2;
    const COL_EPG: usize = 3;
    const COL_EGR: usize = 4;
    const COL_EGD: usize = 5;
    const COL_EBD: usize = 6;
    const COL_EPR: usize = 7;
    const COL_LPG: usize = 8;
    const COL_LGR: usize = 9;
    const COL_LGD: usize = 10;
    const COL_LBD: usize = 11;
    const COL_LPR: usize = 12;
    const COL_MINBP: usize = 13;
    const COL_NOTES: usize = 14;
    const COL_COMBO: usize = 15;
    const COL_DATE: usize = 16;

    let rows = stmt.query_map([], |row| {
        let mode: i32 = row.get(COL_MODE)?;
        let epg: i32 = row.get(COL_EPG)?;
        let egr: i32 = row.get(COL_EGR)?;
        let egd: i32 = row.get(COL_EGD)?;
        let ebd: i32 = row.get(COL_EBD)?;
        let epr: i32 = row.get(COL_EPR)?;
        let lpg: i32 = row.get(COL_LPG)?;
        let lgr: i32 = row.get(COL_LGR)?;
        let lgd: i32 = row.get(COL_LGD)?;
        let lbd: i32 = row.get(COL_LBD)?;
        let lpr: i32 = row.get(COL_LPR)?;
        let notes: i32 = row.get(COL_NOTES)?;
        let date_secs: i64 = row.get(COL_DATE)?;

        // In PMS mode, BAD does not consume notes (judgeVanish=false for BAD).
        let bad_count = if mode == MODE_PMS { 0 } else { ebd + lbd };
        let consumed_notes = epg + lpg + egr + lgr + egd + lgd + bad_count + epr + lpr;

        Ok(ScoreDataLog {
            sha256: row.get(COL_SHA256)?,
            mode,
            clear: row.get(COL_CLEAR)?,
            ex_score: epg * 2 + egr + lpg * 2 + lgr,
            min_bp: row.get(COL_MINBP)?,
            notes,
            combo: row.get(COL_COMBO)?,
            consumed_notes,
            played_at: unix_secs_to_iso8601(date_secs)?,
            date_secs,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(DBError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;
    use rstest::{fixture, rstest};
    use rusqlite::{Connection, OpenFlags};

    const SCOREDATALOG_SCHEMA: &str = indoc! {"
        CREATE TABLE scoredatalog (
            sha256 TEXT NOT NULL,
            mode INTEGER NOT NULL,
            clear INTEGER NOT NULL,
            epg INTEGER NOT NULL,
            egr INTEGER NOT NULL,
            egd INTEGER NOT NULL,
            ebd INTEGER NOT NULL,
            epr INTEGER NOT NULL,
            emr INTEGER NOT NULL,
            ems INTEGER NOT NULL,
            lpg INTEGER NOT NULL,
            lgr INTEGER NOT NULL,
            lgd INTEGER NOT NULL,
            lbd INTEGER NOT NULL,
            lpr INTEGER NOT NULL,
            lmr INTEGER NOT NULL,
            lms INTEGER NOT NULL,
            minbp INTEGER NOT NULL,
            notes INTEGER NOT NULL,
            combo INTEGER NOT NULL,
            date INTEGER NOT NULL,
            PRIMARY KEY (sha256, mode)
        )
    "};

    struct TestDb {
        dir: tempfile::TempDir,
    }

    impl TestDb {
        fn scoredatalog_path(&self) -> std::path::PathBuf {
            self.dir.path().join("scoredatalog.db")
        }

        fn conn(&self) -> Connection {
            Connection::open_with_flags(self.scoredatalog_path(), OpenFlags::SQLITE_OPEN_READ_WRITE)
                .expect("failed to open test db")
        }
    }

    #[fixture]
    fn test_db() -> TestDb {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("scoredatalog.db");
        let conn = Connection::open(&db_path).expect("failed to open db");
        conn.execute_batch(SCOREDATALOG_SCHEMA)
            .expect("failed to create schema");
        TestDb { dir }
    }

    #[expect(clippy::too_many_arguments, reason = "test helper mirrors DB columns")]
    fn insert_record(
        conn: &Connection,
        sha256: &str,
        mode: i32,
        clear: i32,
        epg: i32,
        egr: i32,
        lpg: i32,
        lgr: i32,
        minbp: i32,
        notes: i32,
        combo: i32,
        date: i64,
    ) {
        conn.execute(
            "INSERT INTO scoredatalog \
             (sha256, mode, clear, epg, egr, egd, ebd, epr, emr, ems, lpg, lgr, lgd, lbd, lpr, lmr, lms, minbp, notes, combo, date) \
             VALUES (?1, ?2, ?3, ?4, ?5, 0, 0, 0, 0, 0, ?6, ?7, 0, 0, 0, 0, 0, ?8, ?9, ?10, ?11)",
            rusqlite::params![sha256, mode, clear, epg, egr, lpg, lgr, minbp, notes, combo, date],
        )
        .expect("failed to insert record");
    }

    #[rstest]
    fn test_read_all_score_data_logs_empty(test_db: TestDb) {
        let results = read_all_score_data_logs(&test_db.scoredatalog_path())
            .expect("failed to read score data logs");
        assert!(results.is_empty());
    }

    #[rstest]
    fn test_read_all_score_data_logs_single_record(test_db: TestDb) {
        let conn = test_db.conn();
        // epg=100, egr=50, lpg=80, lgr=30 → ex_score = 100*2 + 50 + 80*2 + 30 = 440
        // date = 1710400000 (2024-03-14T07:06:40Z)
        insert_record(
            &conn, "abc123", 0, 6, 100, 50, 80, 30, 15, 800, 500, 1710400000,
        );
        drop(conn);

        let results = read_all_score_data_logs(&test_db.scoredatalog_path())
            .expect("failed to read score data logs");
        assert_eq!(results.len(), 1);

        let record = &results[0];
        assert_eq!(record.sha256, "abc123");
        assert_eq!(record.mode, 0);
        assert_eq!(record.clear, 6);
        assert_eq!(record.ex_score, 440);
        assert_eq!(record.min_bp, 15);
        assert_eq!(record.notes, 800);
        assert_eq!(record.combo, 500);
        // consumed_notes = epg + lpg + egr + lgr + 0 (egd,lgd,ebd,lbd,epr,lpr all 0)
        assert_eq!(record.consumed_notes, 100 + 80 + 50 + 30);
        assert_eq!(record.played_at, "2024-03-14T07:06:40Z");
        assert_eq!(record.date_secs, 1710400000);
    }

    #[rstest]
    fn test_read_all_score_data_logs_multiple_records(test_db: TestDb) {
        let conn = test_db.conn();
        insert_record(
            &conn, "hash_a", 0, 5, 200, 100, 150, 80, 10, 1000, 900, 1710400000,
        );
        insert_record(
            &conn, "hash_b", 1, 7, 300, 50, 250, 40, 5, 1200, 1100, 1710500000,
        );
        drop(conn);

        let results = read_all_score_data_logs(&test_db.scoredatalog_path())
            .expect("failed to read score data logs");
        assert_eq!(results.len(), 2);

        // hash_a: ex_score = 200*2 + 100 + 150*2 + 80 = 880
        let a = results
            .iter()
            .find(|r| r.sha256 == "hash_a")
            .expect("hash_a not found");
        assert_eq!(a.ex_score, 880);
        assert_eq!(a.mode, 0);

        // hash_b: ex_score = 300*2 + 50 + 250*2 + 40 = 1190
        let b = results
            .iter()
            .find(|r| r.sha256 == "hash_b")
            .expect("hash_b not found");
        assert_eq!(b.ex_score, 1190);
        assert_eq!(b.mode, 1);
    }

    #[rstest]
    fn test_read_all_score_data_logs_file_not_found() {
        let path = Path::new("/tmp/nonexistent_scoredatalog.db");
        let result = read_all_score_data_logs(path);
        assert!(matches!(&result, Err(DBError::FileNotFound(p)) if p.contains("nonexistent")));
    }

    #[rstest]
    fn test_busy_timeout_is_set(test_db: TestDb) {
        let conn = open_readonly_checked(&test_db.scoredatalog_path()).expect("failed to open db");
        let timeout: i64 = conn
            .pragma_query_value(None, "busy_timeout", |row| row.get(0))
            .expect("failed to query busy_timeout");
        assert_eq!(timeout, 5000);
    }

    #[rstest]
    fn test_database_opened_readonly(test_db: TestDb) {
        let conn = open_readonly_checked(&test_db.scoredatalog_path()).expect("failed to open db");
        let result = conn.execute(
            "INSERT INTO scoredatalog \
             (sha256, mode, clear, epg, egr, egd, epr, emr, ems, lpg, lgr, lgd, lpr, lmr, lms, minbp, notes, combo, date) \
             VALUES ('x', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)",
            [],
        );
        assert!(result.is_err());
    }

    struct JudgeCounts {
        epg: i32,
        egr: i32,
        egd: i32,
        ebd: i32,
        epr: i32,
        lpg: i32,
        lgr: i32,
        lgd: i32,
        lbd: i32,
        lpr: i32,
    }

    fn insert_record_full(
        conn: &Connection,
        sha256: &str,
        mode: i32,
        clear: i32,
        j: &JudgeCounts,
        notes: i32,
        date: i64,
    ) {
        conn.execute(
            "INSERT INTO scoredatalog \
             (sha256, mode, clear, epg, egr, egd, ebd, epr, emr, ems, lpg, lgr, lgd, lbd, lpr, lmr, lms, minbp, notes, combo, date) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 0, ?9, ?10, ?11, ?12, ?13, 0, 0, 0, ?14, 0, ?15)",
            rusqlite::params![sha256, mode, clear, j.epg, j.egr, j.egd, j.ebd, j.epr, j.lpg, j.lgr, j.lgd, j.lbd, j.lpr, notes, date],
        )
        .expect("failed to insert record");
    }

    #[rstest]
    #[case::seven_key_all_consumed(
        0,  // mode: 7key
        JudgeCounts { epg: 100, egr: 50, egd: 10, ebd: 5, epr: 3, lpg: 80, lgr: 30, lgd: 8, lbd: 4, lpr: 2 },
        // consumed = 100+80+50+30+10+8+5+4+3+2 = 292
        292,
    )]
    #[case::pms_excludes_bad(
        2,  // mode: PMS (9key)
        JudgeCounts { epg: 100, egr: 50, egd: 10, ebd: 5, epr: 3, lpg: 80, lgr: 30, lgd: 8, lbd: 4, lpr: 2 },
        // consumed = 100+80+50+30+10+8+3+2 = 283 (ebd=5 + lbd=4 excluded)
        283,
    )]
    fn test_consumed_notes(
        test_db: TestDb,
        #[case] mode: i32,
        #[case] judge: JudgeCounts,
        #[case] expected_consumed: i32,
    ) {
        let conn = test_db.conn();
        insert_record_full(&conn, "abc123", mode, 1, &judge, 500, 1710400000);
        drop(conn);

        let results = read_all_score_data_logs(&test_db.scoredatalog_path())
            .expect("failed to read score data logs");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].consumed_notes, expected_consumed);
    }

    #[rstest]
    #[case::epoch(0, "1970-01-01T00:00:00Z")]
    #[case::specific(1710400000, "2024-03-14T07:06:40Z")]
    #[case::year_2026(1773849600, "2026-03-18T16:00:00Z")]
    fn test_unix_secs_to_iso8601(#[case] secs: i64, #[case] expected: &str) {
        assert_eq!(
            unix_secs_to_iso8601(secs).expect("failed to convert timestamp"),
            expected
        );
    }
}
