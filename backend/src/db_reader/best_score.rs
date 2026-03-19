use std::collections::HashMap;
use std::path::Path;

use super::open_readonly;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BestScore {
    pub clear: i32,
    pub ex_score: i32,
    pub min_bp: i32,
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
