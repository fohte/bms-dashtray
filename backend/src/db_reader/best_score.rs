use std::collections::HashMap;
use std::path::Path;

use super::open_readonly;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BestScore {
    pub clear: i32,
    pub ex_score: i32,
    pub min_bp: i32,
}

/// Reads best score from score.db by sha256 hash and mode.
///
/// EX score is calculated as: epg*2 + egr + lpg*2 + lgr
///
/// Returns `None` if no matching score is found.
pub fn read_best_score(
    path: &Path,
    sha256: &str,
    mode: i32,
) -> Result<Option<BestScore>, rusqlite::Error> {
    let conn = open_readonly(path)?;

    let mut stmt = conn.prepare(
        "SELECT clear, epg, egr, lpg, lgr, minbp FROM score WHERE sha256 = ?1 AND mode = ?2",
    )?;

    let result = stmt.query_row(rusqlite::params![sha256, mode], |row| {
        let clear: i32 = row.get(0)?;
        let epg: i32 = row.get(1)?;
        let egr: i32 = row.get(2)?;
        let lpg: i32 = row.get(3)?;
        let lgr: i32 = row.get(4)?;
        let min_bp: i32 = row.get(5)?;

        let ex_score = epg * 2 + egr + lpg * 2 + lgr;

        Ok(BestScore {
            clear,
            ex_score,
            min_bp,
        })
    });

    match result {
        Ok(score) => Ok(Some(score)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Reads all best scores from score.db, returning a map keyed by (sha256, mode).
pub fn read_all_best_scores(
    path: &Path,
) -> Result<HashMap<(String, i32), BestScore>, rusqlite::Error> {
    let conn = open_readonly(path)?;

    let mut stmt =
        conn.prepare("SELECT sha256, mode, clear, epg, egr, lpg, lgr, minbp FROM score")?;

    stmt.query_map([], |row| {
        let sha256: String = row.get(0)?;
        let mode: i32 = row.get(1)?;
        let clear: i32 = row.get(2)?;
        let epg: i32 = row.get(3)?;
        let egr: i32 = row.get(4)?;
        let lpg: i32 = row.get(5)?;
        let lgr: i32 = row.get(6)?;
        let min_bp: i32 = row.get(7)?;

        let ex_score = epg * 2 + egr + lpg * 2 + lgr;

        Ok((
            (sha256, mode),
            BestScore {
                clear,
                ex_score,
                min_bp,
            },
        ))
    })?
    .collect()
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::{fixture, rstest};
    use tempfile::NamedTempFile;

    use super::*;

    #[fixture]
    fn score_db() -> NamedTempFile {
        let file = NamedTempFile::new().unwrap();
        let conn = rusqlite::Connection::open(file.path()).unwrap();
        conn.execute_batch(indoc! {"
            CREATE TABLE score (
                sha256 TEXT NOT NULL,
                mode INTEGER,
                clear INTEGER,
                epg INTEGER,
                lpg INTEGER,
                egr INTEGER,
                lgr INTEGER,
                egd INTEGER,
                lgd INTEGER,
                ebd INTEGER,
                lbd INTEGER,
                epr INTEGER,
                lpr INTEGER,
                ems INTEGER,
                lms INTEGER,
                notes INTEGER,
                combo INTEGER,
                minbp INTEGER,
                avgjudge INTEGER NOT NULL DEFAULT 2147483647,
                playcount INTEGER,
                clearcount INTEGER,
                trophy TEXT,
                ghost TEXT,
                option INTEGER,
                seed INTEGER,
                random INTEGER,
                date INTEGER,
                state INTEGER,
                scorehash TEXT,
                PRIMARY KEY (sha256, mode)
            );
        "})
            .unwrap();
        // epg=100, egr=50, lpg=80, lgr=30 → EX score = 100*2 + 50 + 80*2 + 30 = 440
        conn.execute(
            "INSERT INTO score (sha256, mode, clear, epg, egr, lpg, lgr, minbp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params!["abc123def456", 0, 6, 100, 50, 80, 30, 15],
        )
        .unwrap();
        file
    }

    #[rstest]
    #[case::found("abc123def456", 0, Some(BestScore { clear: 6, ex_score: 440, min_bp: 15 }))]
    #[case::not_found("nonexistent_sha256", 0, None)]
    #[case::wrong_mode("abc123def456", 1, None)]
    fn test_read_best_score(
        score_db: NamedTempFile,
        #[case] sha256: &str,
        #[case] mode: i32,
        #[case] expected: Option<BestScore>,
    ) {
        let result = read_best_score(score_db.path(), sha256, mode).unwrap();
        assert_eq!(result, expected);
    }
}
