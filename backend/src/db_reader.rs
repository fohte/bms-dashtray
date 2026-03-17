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
    /// ISO 8601 formatted date string converted from UNIX time (milliseconds).
    pub played_at: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DBError {
    #[error("database file not found: {0}")]
    FileNotFound(String),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

fn open_readonly(path: &Path) -> Result<Connection, DBError> {
    if !path.exists() {
        return Err(DBError::FileNotFound(path.display().to_string()));
    }
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    conn.busy_timeout(BUSY_TIMEOUT)?;
    Ok(conn)
}

fn unix_millis_to_iso8601(millis: i64) -> String {
    let dt: DateTime<Utc> = DateTime::from_timestamp_millis(millis).unwrap_or_default();
    dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

pub fn read_all_score_data_logs(path: &Path) -> Result<Vec<ScoreDataLog>, DBError> {
    let conn = open_readonly(path)?;
    let mut stmt = conn.prepare(
        "SELECT sha256, mode, clear, epg, egr, lpg, lgr, minbp, notes, combo, date \
         FROM scoredatalog",
    )?;

    let rows = stmt.query_map([], |row| {
        let epg: i32 = row.get(3)?;
        let egr: i32 = row.get(4)?;
        let lpg: i32 = row.get(5)?;
        let lgr: i32 = row.get(6)?;
        let date_millis: i64 = row.get(10)?;

        Ok(ScoreDataLog {
            sha256: row.get(0)?,
            mode: row.get(1)?,
            clear: row.get(2)?,
            ex_score: epg * 2 + egr + lpg * 2 + lgr,
            min_bp: row.get(7)?,
            notes: row.get(8)?,
            combo: row.get(9)?,
            played_at: unix_millis_to_iso8601(date_millis),
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn validate_db_paths(player_dir: &Path, song_db_path: &Path) -> Result<(), DBError> {
    let scoredatalog_path = player_dir.join("scoredatalog.db");
    if !scoredatalog_path.exists() {
        return Err(DBError::FileNotFound(
            scoredatalog_path.display().to_string(),
        ));
    }
    if !song_db_path.exists() {
        return Err(DBError::FileNotFound(song_db_path.display().to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use rstest::{fixture, rstest};
    use rusqlite::{Connection, OpenFlags};

    const SCOREDATALOG_SCHEMA: &str = "\
        CREATE TABLE scoredatalog (\
            sha256 TEXT NOT NULL, \
            mode INTEGER NOT NULL, \
            clear INTEGER NOT NULL, \
            epg INTEGER NOT NULL, \
            egr INTEGER NOT NULL, \
            egd INTEGER NOT NULL, \
            epr INTEGER NOT NULL, \
            emr INTEGER NOT NULL, \
            ems INTEGER NOT NULL, \
            lpg INTEGER NOT NULL, \
            lgr INTEGER NOT NULL, \
            lgd INTEGER NOT NULL, \
            lpr INTEGER NOT NULL, \
            lmr INTEGER NOT NULL, \
            lms INTEGER NOT NULL, \
            minbp INTEGER NOT NULL, \
            notes INTEGER NOT NULL, \
            combo INTEGER NOT NULL, \
            date INTEGER NOT NULL, \
            PRIMARY KEY (sha256, mode)\
        )";

    struct TestDb {
        dir: tempfile::TempDir,
    }

    impl TestDb {
        fn scoredatalog_path(&self) -> std::path::PathBuf {
            self.dir.path().join("scoredatalog.db")
        }

        fn conn(&self) -> Connection {
            Connection::open_with_flags(self.scoredatalog_path(), OpenFlags::SQLITE_OPEN_READ_WRITE)
                .unwrap()
        }
    }

    #[fixture]
    fn test_db() -> TestDb {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("scoredatalog.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(SCOREDATALOG_SCHEMA).unwrap();
        TestDb { dir }
    }

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
             (sha256, mode, clear, epg, egr, egd, epr, emr, ems, lpg, lgr, lgd, lpr, lmr, lms, minbp, notes, combo, date) \
             VALUES (?1, ?2, ?3, ?4, ?5, 0, 0, 0, 0, ?6, ?7, 0, 0, 0, 0, ?8, ?9, ?10, ?11)",
            rusqlite::params![sha256, mode, clear, epg, egr, lpg, lgr, minbp, notes, combo, date],
        )
        .unwrap();
    }

    #[rstest]
    fn test_read_all_score_data_logs_empty(test_db: TestDb) {
        let results = read_all_score_data_logs(&test_db.scoredatalog_path()).unwrap();
        assert!(results.is_empty());
    }

    #[rstest]
    fn test_read_all_score_data_logs_single_record(test_db: TestDb) {
        let conn = test_db.conn();
        // epg=100, egr=50, lpg=80, lgr=30 → ex_score = 100*2 + 50 + 80*2 + 30 = 440
        // date = 1710400000000 (2024-03-14T07:06:40.000Z)
        insert_record(
            &conn,
            "abc123",
            0,
            6,
            100,
            50,
            80,
            30,
            15,
            800,
            500,
            1710400000000,
        );
        drop(conn);

        let results = read_all_score_data_logs(&test_db.scoredatalog_path()).unwrap();
        assert_eq!(results.len(), 1);

        let record = &results[0];
        assert_eq!(record.sha256, "abc123");
        assert_eq!(record.mode, 0);
        assert_eq!(record.clear, 6);
        assert_eq!(record.ex_score, 440);
        assert_eq!(record.min_bp, 15);
        assert_eq!(record.notes, 800);
        assert_eq!(record.combo, 500);
        assert_eq!(record.played_at, "2024-03-14T07:06:40.000Z");
    }

    #[rstest]
    fn test_read_all_score_data_logs_multiple_records(test_db: TestDb) {
        let conn = test_db.conn();
        insert_record(
            &conn,
            "hash_a",
            0,
            5,
            200,
            100,
            150,
            80,
            10,
            1000,
            900,
            1710400000000,
        );
        insert_record(
            &conn,
            "hash_b",
            1,
            7,
            300,
            50,
            250,
            40,
            5,
            1200,
            1100,
            1710500000000,
        );
        drop(conn);

        let results = read_all_score_data_logs(&test_db.scoredatalog_path()).unwrap();
        assert_eq!(results.len(), 2);

        // hash_a: ex_score = 200*2 + 100 + 150*2 + 80 = 880
        let a = results.iter().find(|r| r.sha256 == "hash_a").unwrap();
        assert_eq!(a.ex_score, 880);
        assert_eq!(a.mode, 0);

        // hash_b: ex_score = 300*2 + 50 + 250*2 + 40 = 1190
        let b = results.iter().find(|r| r.sha256 == "hash_b").unwrap();
        assert_eq!(b.ex_score, 1190);
        assert_eq!(b.mode, 1);
    }

    #[rstest]
    fn test_read_all_score_data_logs_file_not_found() {
        let path = Path::new("/tmp/nonexistent_scoredatalog.db");
        let result = read_all_score_data_logs(path);
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), DBError::FileNotFound(p) if p.contains("nonexistent"))
        );
    }

    #[rstest]
    fn test_busy_timeout_is_set(test_db: TestDb) {
        let conn = open_readonly(&test_db.scoredatalog_path()).unwrap();
        let timeout: i64 = conn
            .pragma_query_value(None, "busy_timeout", |row| row.get(0))
            .unwrap();
        assert_eq!(timeout, 5000);
    }

    #[rstest]
    fn test_database_opened_readonly(test_db: TestDb) {
        let conn = open_readonly(&test_db.scoredatalog_path()).unwrap();
        let result = conn.execute(
            "INSERT INTO scoredatalog \
             (sha256, mode, clear, epg, egr, egd, epr, emr, ems, lpg, lgr, lgd, lpr, lmr, lms, minbp, notes, combo, date) \
             VALUES ('x', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)",
            [],
        );
        assert!(result.is_err());
    }

    #[rstest]
    fn test_validate_db_paths_success(test_db: TestDb) {
        let player_dir = test_db.dir.path();
        let song_db_path = test_db.dir.path().join("songdata.db");
        fs::write(&song_db_path, "").unwrap();

        let result = validate_db_paths(player_dir, &song_db_path);
        assert!(result.is_ok());
    }

    #[rstest]
    fn test_validate_db_paths_missing_scoredatalog() {
        let dir = tempfile::tempdir().unwrap();
        let song_db_path = dir.path().join("songdata.db");
        fs::write(&song_db_path, "").unwrap();

        let result = validate_db_paths(dir.path(), &song_db_path);
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), DBError::FileNotFound(p) if p.contains("scoredatalog"))
        );
    }

    #[rstest]
    fn test_validate_db_paths_missing_songdata() {
        let dir = tempfile::tempdir().unwrap();
        let scoredatalog_path = dir.path().join("scoredatalog.db");
        fs::write(&scoredatalog_path, "").unwrap();
        let missing_song_path = dir.path().join("songdata.db");

        let result = validate_db_paths(dir.path(), &missing_song_path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DBError::FileNotFound(p) if p.contains("songdata")));
    }

    #[rstest]
    #[case::epoch(0, "1970-01-01T00:00:00.000Z")]
    #[case::with_millis(1710400000123, "2024-03-14T07:06:40.123Z")]
    #[case::year_2026(1773849600000, "2026-03-18T16:00:00.000Z")]
    fn test_unix_millis_to_iso8601(#[case] millis: i64, #[case] expected: &str) {
        assert_eq!(unix_millis_to_iso8601(millis), expected);
    }
}
