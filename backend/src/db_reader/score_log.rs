use std::path::Path;

use super::open_readonly;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreLog {
    pub old_clear: i32,
    pub old_score: i32,
    pub old_min_bp: i32,
}

/// Reads the score log entry from scorelog.db that matches sha256, mode, and date.
///
/// This represents a "best update" event: when a player achieves a new best,
/// beatoraja logs the old values (before the update) in the scorelog table.
///
/// Returns `None` if no matching log entry is found (no best update occurred).
pub fn read_score_log(
    path: &Path,
    sha256: &str,
    mode: i32,
    date: i64,
) -> Result<Option<ScoreLog>, rusqlite::Error> {
    let conn = open_readonly(path)?;

    let mut stmt = conn.prepare(
        "SELECT oldclear, oldscore, oldminbp FROM scorelog WHERE sha256 = ?1 AND mode = ?2 AND date = ?3 LIMIT 1",
    )?;

    let result = stmt.query_row(rusqlite::params![sha256, mode, date], |row| {
        Ok(ScoreLog {
            old_clear: row.get(0)?,
            old_score: row.get(1)?,
            old_min_bp: row.get(2)?,
        })
    });

    match result {
        Ok(log) => Ok(Some(log)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::{fixture, rstest};
    use tempfile::NamedTempFile;

    use super::*;

    #[fixture]
    fn scorelog_db() -> NamedTempFile {
        let file = NamedTempFile::new().unwrap();
        let conn = rusqlite::Connection::open(file.path()).unwrap();
        conn.execute_batch(indoc! {"
            CREATE TABLE scorelog (
                sha256 TEXT NOT NULL,
                mode INTEGER,
                clear INTEGER,
                oldclear INTEGER,
                score INTEGER,
                oldscore INTEGER,
                combo INTEGER,
                oldcombo INTEGER,
                minbp INTEGER,
                oldminbp INTEGER,
                date INTEGER
            );
        "})
            .unwrap();
        conn.execute(
            "INSERT INTO scorelog (sha256, mode, clear, oldclear, score, oldscore, combo, oldcombo, minbp, oldminbp, date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params!["abc123def456", 0, 6, 5, 440, 400, 500, 480, 15, 20, 1710400000],
        )
        .unwrap();
        file
    }

    #[rstest]
    #[case::found(
        "abc123def456",
        0,
        1710400000,
        Some(ScoreLog { old_clear: 5, old_score: 400, old_min_bp: 20 })
    )]
    #[case::not_found("nonexistent_sha256", 0, 1710400000, None)]
    #[case::wrong_mode("abc123def456", 1, 1710400000, None)]
    #[case::wrong_date("abc123def456", 0, 9999999999, None)]
    fn test_read_score_log(
        scorelog_db: NamedTempFile,
        #[case] sha256: &str,
        #[case] mode: i32,
        #[case] date: i64,
        #[case] expected: Option<ScoreLog>,
    ) {
        let result = read_score_log(scorelog_db.path(), sha256, mode, date).unwrap();
        assert_eq!(result, expected);
    }
}
