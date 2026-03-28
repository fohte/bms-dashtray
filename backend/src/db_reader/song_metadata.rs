use std::collections::HashMap;
use std::path::Path;

use super::open_readonly;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SongMetadata {
    pub title: String,
    pub subtitle: String,
    pub artist: String,
    pub level: i32,
    pub difficulty: i32,
    pub notes: i32,
    pub mode: i32,
}

/// Reads song metadata from songdata.db by sha256 hash.
///
/// Returns `None` if no matching song is found.
pub fn read_song_metadata(
    path: &Path,
    sha256: &str,
) -> Result<Option<SongMetadata>, rusqlite::Error> {
    let conn = open_readonly(path)?;

    let mut stmt = conn.prepare(
        "SELECT title, subtitle, artist, level, difficulty, notes, mode FROM song WHERE sha256 = ?1 LIMIT 1",
    )?;

    let result = stmt.query_row([sha256], |row| {
        Ok(SongMetadata {
            title: row.get(0)?,
            subtitle: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            artist: row.get(2)?,
            level: row.get(3)?,
            difficulty: row.get(4)?,
            notes: row.get(5)?,
            mode: row.get(6)?,
        })
    });

    match result {
        Ok(metadata) => Ok(Some(metadata)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Builds a lookup map from md5 to sha256 using songdata.db.
///
/// Only entries where both md5 and sha256 are non-empty are included.
/// When multiple rows share the same md5, the last one wins.
pub fn build_md5_to_sha256_map(path: &Path) -> Result<HashMap<String, String>, rusqlite::Error> {
    let conn = open_readonly(path)?;

    let mut stmt = conn.prepare("SELECT md5, sha256 FROM song WHERE md5 != '' AND sha256 != ''")?;

    stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect()
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::{fixture, rstest};
    use tempfile::NamedTempFile;

    use super::*;

    #[fixture]
    fn songdata_db() -> NamedTempFile {
        let file = NamedTempFile::new().unwrap();
        let conn = rusqlite::Connection::open(file.path()).unwrap();
        conn.execute_batch(indoc! {"
            CREATE TABLE song (
                md5 TEXT NOT NULL,
                sha256 TEXT NOT NULL,
                title TEXT,
                subtitle TEXT,
                genre TEXT,
                artist TEXT,
                subartist TEXT,
                tag TEXT,
                path TEXT PRIMARY KEY,
                folder TEXT,
                stagefile TEXT,
                banner TEXT,
                backbmp TEXT,
                preview TEXT,
                parent TEXT,
                level INTEGER,
                difficulty INTEGER,
                maxbpm INTEGER,
                minbpm INTEGER,
                length INTEGER,
                mode INTEGER,
                judge INTEGER,
                feature INTEGER,
                content INTEGER,
                date INTEGER,
                favorite INTEGER,
                adddate INTEGER,
                notes INTEGER,
                charthash TEXT
            );
        "})
            .unwrap();
        conn.execute(
            "INSERT INTO song (md5, sha256, title, artist, level, difficulty, notes, mode, path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                "md5hash",
                "abc123def456",
                "Test Song",
                "Test Artist",
                12,
                1,
                1500,
                0,
                "/path/to/song.bms"
            ],
        )
        .unwrap();
        file
    }

    #[rstest]
    #[case::found(
        "abc123def456",
        Some(SongMetadata {
            title: "Test Song".to_string(),
            subtitle: String::new(),
            artist: "Test Artist".to_string(),
            level: 12,
            difficulty: 1,
            notes: 1500,
            mode: 0,
        })
    )]
    #[case::not_found("nonexistent_sha256", None)]
    fn test_read_song_metadata(
        songdata_db: NamedTempFile,
        #[case] sha256: &str,
        #[case] expected: Option<SongMetadata>,
    ) {
        let result = read_song_metadata(songdata_db.path(), sha256).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    fn test_build_md5_to_sha256_map(songdata_db: NamedTempFile) {
        // The fixture already has one row: md5="md5hash", sha256="abc123def456"
        let conn = rusqlite::Connection::open(songdata_db.path()).unwrap();
        conn.execute(
            "INSERT INTO song (md5, sha256, title, artist, level, difficulty, notes, mode, path)
             VALUES ('md5_second', 'sha256_second', 'Song 2', 'Artist 2', 10, 2, 1000, 0, '/path/to/song2.bms')",
            [],
        ).unwrap();
        drop(conn);

        let map = build_md5_to_sha256_map(songdata_db.path()).unwrap();
        assert_eq!(map.get("md5hash").unwrap(), "abc123def456");
        assert_eq!(map.get("md5_second").unwrap(), "sha256_second");
        assert_eq!(map.len(), 2);
    }
}
