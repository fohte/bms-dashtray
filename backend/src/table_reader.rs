use std::collections::HashMap;
use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use serde::Deserialize;

/// A song entry within a difficulty table folder.
#[derive(Debug, Deserialize)]
struct TableSong {
    sha256: String,
}

/// A folder (level group) in the difficulty table.
/// The `name` field contains the level label (e.g. "★1", "st3").
#[derive(Debug, Deserialize)]
struct TableFolder {
    name: String,
    #[serde(default)]
    songs: Vec<TableSong>,
}

/// Parsed difficulty table in beatoraja's .bmt cache format.
///
/// beatoraja serializes its `TableData` class via libGDX's Json serializer.
/// The structure has top-level `name`/`tag` fields and a `folder` array,
/// where each folder contains a `name` (the level label) and `songs` array.
#[derive(Debug, Deserialize)]
struct BmTable {
    #[serde(default)]
    folder: Vec<TableFolder>,
}

/// A difficulty table level label for a song (e.g. "★24", "sl5").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableLevel {
    pub label: String,
}

#[derive(Debug, thiserror::Error)]
pub enum TableReaderError {
    #[error("failed to read .bmt file: {path}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to decompress .bmt file: {path}")]
    Decompress {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse .bmt file: {path}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to read table directory: {path}")]
    ReadDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Reads and decompresses a single .bmt file (GZIP-compressed JSON).
fn read_bmt_file(path: &Path) -> Result<BmTable, TableReaderError> {
    let data = fs::read(path).map_err(|e| TableReaderError::ReadFile {
        path: path.to_path_buf(),
        source: e,
    })?;

    let mut decoder = GzDecoder::new(&data[..]);
    let mut json_str = String::new();
    decoder
        .read_to_string(&mut json_str)
        .map_err(|e| TableReaderError::Decompress {
            path: path.to_path_buf(),
            source: e,
        })?;

    serde_json::from_str(&json_str).map_err(|e| TableReaderError::Parse {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Builds a lookup map from sha256 hash to list of table level labels
/// by reading all .bmt files in the given directory.
///
/// Songs that appear in multiple tables will have multiple entries.
/// md5-only entries are ignored since PlayRecord uses sha256 for identification.
pub fn build_table_level_map(
    table_dir: &Path,
) -> Result<HashMap<String, Vec<TableLevel>>, TableReaderError> {
    let mut map: HashMap<String, Vec<TableLevel>> = HashMap::new();

    if !table_dir.exists() {
        return Ok(map);
    }

    let entries = fs::read_dir(table_dir).map_err(|e| TableReaderError::ReadDir {
        path: table_dir.to_path_buf(),
        source: e,
    })?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("bmt") {
            continue;
        }

        let table = match read_bmt_file(&path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("skipping malformed .bmt file: {e}");
                continue;
            }
        };

        for folder in &table.folder {
            let label = &folder.name;
            for song in &folder.songs {
                if song.sha256.is_empty() {
                    continue;
                }
                map.entry(song.sha256.clone())
                    .or_default()
                    .push(TableLevel {
                        label: label.clone(),
                    });
            }
        }
    }

    Ok(map)
}

/// Returns the table directory path from beatoraja root.
/// beatoraja stores difficulty tables in `{root}/table/`.
pub fn table_dir_path(beatoraja_root: &str) -> PathBuf {
    Path::new(beatoraja_root).join("table")
}

#[cfg(test)]
mod tests {
    use super::*;

    use flate2::Compression;
    use flate2::write::GzEncoder;
    use indoc::indoc;
    use rstest::{fixture, rstest};
    use std::io::Write as _;
    use tempfile::TempDir;

    fn write_bmt(dir: &Path, filename: &str, json: &str) {
        let path = dir.join(filename);
        let file = fs::File::create(path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(json.as_bytes()).unwrap();
        encoder.finish().unwrap();
    }

    #[fixture]
    fn table_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    #[rstest]
    fn test_read_bmt_file_parses_correctly(table_dir: TempDir) {
        let json = indoc! {r#"
            {
                "name": "Satellite",
                "tag": "st",
                "folder": [
                    {
                        "name": "st3",
                        "songs": [
                            {"sha256": "sha_aaa", "title": "Song A"},
                            {"sha256": "sha_bbb", "title": "Song B"}
                        ]
                    }
                ]
            }
        "#};
        write_bmt(table_dir.path(), "test.bmt", json);

        let table = read_bmt_file(&table_dir.path().join("test.bmt")).unwrap();
        assert_eq!(table.folder.len(), 1);
        assert_eq!(table.folder[0].name, "st3");
        assert_eq!(table.folder[0].songs.len(), 2);
        assert_eq!(table.folder[0].songs[0].sha256, "sha_aaa");
    }

    #[rstest]
    #[case::satellite(
        r#"{"name":"Satellite","tag":"st","folder":[{"name":"st3","songs":[{"sha256":"sha_a"}]}]}"#,
        "sha_a",
        "st3"
    )]
    #[case::insane(
        r#"{"name":"Insane","tag":"★","folder":[{"name":"★10","songs":[{"sha256":"sha_b"}]}]}"#,
        "sha_b",
        "★10"
    )]
    fn test_build_table_level_map_label(
        table_dir: TempDir,
        #[case] json: &str,
        #[case] sha256: &str,
        #[case] expected_label: &str,
    ) {
        write_bmt(table_dir.path(), "table.bmt", json);

        let map = build_table_level_map(table_dir.path()).unwrap();
        assert_eq!(map.get(sha256).unwrap()[0].label, expected_label);
    }

    #[rstest]
    fn test_build_table_level_map_multiple_tables(table_dir: TempDir) {
        let json1 = indoc! {r#"
            {
                "name": "Satellite",
                "tag": "st",
                "folder": [
                    {"name": "st3", "songs": [{"sha256": "sha_x"}]}
                ]
            }
        "#};
        let json2 = indoc! {r#"
            {
                "name": "Insane",
                "tag": "★",
                "folder": [
                    {"name": "★24", "songs": [{"sha256": "sha_x"}]}
                ]
            }
        "#};
        write_bmt(table_dir.path(), "satellite.bmt", json1);
        write_bmt(table_dir.path(), "insane.bmt", json2);

        let map = build_table_level_map(table_dir.path()).unwrap();
        let levels = map.get("sha_x").unwrap();
        assert_eq!(levels.len(), 2);

        let mut labels: Vec<&str> = levels.iter().map(|l| l.label.as_str()).collect();
        labels.sort();
        assert_eq!(labels, vec!["st3", "★24"]);
    }

    #[rstest]
    fn test_build_table_level_map_skips_songs_without_sha256(table_dir: TempDir) {
        let json = indoc! {r#"
            {
                "name": "Test",
                "tag": "t",
                "folder": [
                    {
                        "name": "t1",
                        "songs": [
                            {"sha256": ""},
                            {"sha256": "has_sha"}
                        ]
                    }
                ]
            }
        "#};
        write_bmt(table_dir.path(), "test.bmt", json);

        let map = build_table_level_map(table_dir.path()).unwrap();
        assert!(!map.contains_key(""));
        assert_eq!(map.get("has_sha").unwrap()[0].label, "t1");
    }

    #[rstest]
    #[case::empty_dir(true)]
    #[case::nonexistent_dir(false)]
    fn test_build_table_level_map_returns_empty(#[case] create_dir: bool) {
        let (_guard, dir) = if create_dir {
            let d = tempfile::tempdir().unwrap();
            let p = d.path().to_path_buf();
            (Some(d), p)
        } else {
            (None, PathBuf::from("/nonexistent/table/dir"))
        };
        let map = build_table_level_map(&dir).unwrap();
        assert!(map.is_empty());
    }

    #[rstest]
    fn test_build_table_level_map_skips_non_bmt_files(table_dir: TempDir) {
        fs::write(table_dir.path().join("readme.txt"), "not a table").unwrap();
        let map = build_table_level_map(table_dir.path()).unwrap();
        assert!(map.is_empty());
    }

    #[rstest]
    fn test_build_table_level_map_skips_malformed_bmt(table_dir: TempDir) {
        // Write valid table
        let valid = indoc! {r#"
            {
                "name": "Good",
                "tag": "g",
                "folder": [
                    {"name": "g1", "songs": [{"sha256": "sha_good"}]}
                ]
            }
        "#};
        write_bmt(table_dir.path(), "good.bmt", valid);

        // Write malformed file (not valid gzip)
        fs::write(table_dir.path().join("bad.bmt"), "not gzip data").unwrap();

        let map = build_table_level_map(table_dir.path()).unwrap();
        assert_eq!(map.get("sha_good").unwrap()[0].label, "g1");
    }

    #[rstest]
    fn test_table_dir_path() {
        assert_eq!(
            table_dir_path("/home/user/beatoraja"),
            PathBuf::from("/home/user/beatoraja/table")
        );
    }
}
