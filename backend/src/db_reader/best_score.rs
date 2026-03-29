use std::path::Path;

use rusqlite::OptionalExtension as _;

use super::open_readonly;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BestScore {
    pub clear: i32,
    pub ex_score: i32,
    pub min_bp: i32,
}

/// Reads the best score for a single (sha256, mode) pair from score.db.
///
/// Returns `None` if no matching entry exists.
pub fn read_best_score(
    path: &Path,
    sha256: &str,
    mode: i32,
) -> Result<Option<BestScore>, rusqlite::Error> {
    let conn = open_readonly(path)?;

    let mut stmt = conn.prepare(
        "SELECT clear, epg, egr, lpg, lgr, minbp FROM score WHERE sha256 = ?1 AND mode = ?2 LIMIT 1",
    )?;

    stmt.query_row(rusqlite::params![sha256, mode], |row| {
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
    })
    .optional()
}
