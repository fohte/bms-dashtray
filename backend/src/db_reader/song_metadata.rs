use std::path::Path;

use super::open_readonly;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SongMetadata {
    pub title: String,
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
        "SELECT title, artist, level, difficulty, notes, mode FROM song WHERE sha256 = ?1 LIMIT 1",
    )?;

    let result = stmt.query_row([sha256], |row| {
        Ok(SongMetadata {
            title: row.get(0)?,
            artist: row.get(1)?,
            level: row.get(2)?,
            difficulty: row.get(3)?,
            notes: row.get(4)?,
            mode: row.get(5)?,
        })
    });

    match result {
        Ok(metadata) => Ok(Some(metadata)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use tempfile::NamedTempFile;

    use super::*;

    fn create_songdata_db() -> NamedTempFile {
        let file = NamedTempFile::new().unwrap();
        let conn = rusqlite::Connection::open(file.path()).unwrap();
        conn.execute_batch(
            "CREATE TABLE song (
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
            );",
        )
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
            artist: "Test Artist".to_string(),
            level: 12,
            difficulty: 1,
            notes: 1500,
            mode: 0,
        })
    )]
    #[case::not_found("nonexistent_sha256", None)]
    fn test_read_song_metadata(
        #[case] sha256: &str,
        #[case] expected: Option<SongMetadata>,
    ) {
        let db_file = create_songdata_db();
        let result = read_song_metadata(db_file.path(), sha256).unwrap();
        assert_eq!(result, expected);
    }
}
