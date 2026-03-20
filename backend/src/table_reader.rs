use std::collections::HashMap;
use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use serde::Deserialize;

/// Header portion of a .bmt difficulty table file.
#[derive(Debug, Deserialize)]
struct TableHeader {
    #[serde(default)]
    tag: String,
    #[serde(default)]
    symbol: String,
}

/// Single entry in the difficulty table data array.
#[derive(Debug, Deserialize)]
struct TableEntry {
    #[serde(default)]
    sha256: String,
    #[serde(default)]
    level: String,
}

/// Parsed difficulty table: header + entries.
#[derive(Debug, Deserialize)]
struct BmTable {
    header: TableHeader,
    body: Vec<TableEntry>,
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

        // Use tag first (as beatoraja does), fall back to symbol
        let prefix = if !table.header.tag.is_empty() {
            &table.header.tag
        } else {
            &table.header.symbol
        };

        for entry in &table.body {
            if entry.sha256.is_empty() {
                continue;
            }

            let label = format!("{prefix}{}", entry.level);
            map.entry(entry.sha256.clone())
                .or_default()
                .push(TableLevel { label });
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
        let json = r#"{
            "header": {"name": "Satellite", "tag": "st", "symbol": ""},
            "body": [
                {"md5": "aaa", "sha256": "sha_aaa", "level": "3"},
                {"md5": "bbb", "sha256": "sha_bbb", "level": "5"}
            ]
        }"#;
        write_bmt(table_dir.path(), "test.bmt", json);

        let table = read_bmt_file(&table_dir.path().join("test.bmt")).unwrap();
        assert_eq!(table.header.tag, "st");
        assert_eq!(table.body.len(), 2);
        assert_eq!(table.body[0].sha256, "sha_aaa");
        assert_eq!(table.body[0].level, "3");
    }

    #[rstest]
    fn test_build_table_level_map_basic(table_dir: TempDir) {
        let json = r#"{
            "header": {"name": "Satellite", "tag": "st", "symbol": ""},
            "body": [
                {"md5": "aaa", "sha256": "sha_aaa", "level": "3"},
                {"md5": "bbb", "sha256": "sha_bbb", "level": "5"}
            ]
        }"#;
        write_bmt(table_dir.path(), "table1.bmt", json);

        let map = build_table_level_map(table_dir.path()).unwrap();
        assert_eq!(map.get("sha_aaa").unwrap().len(), 1);
        assert_eq!(map.get("sha_aaa").unwrap()[0].label, "st3");
        assert_eq!(map.get("sha_bbb").unwrap()[0].label, "st5");
    }

    #[rstest]
    fn test_build_table_level_map_multiple_tables(table_dir: TempDir) {
        let json1 = r#"{
            "header": {"name": "Satellite", "tag": "st", "symbol": ""},
            "body": [{"md5": "", "sha256": "sha_x", "level": "3"}]
        }"#;
        let json2 = r#"{
            "header": {"name": "Insane", "tag": "", "symbol": "★"},
            "body": [{"md5": "", "sha256": "sha_x", "level": "24"}]
        }"#;
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
    fn test_build_table_level_map_uses_symbol_as_fallback(table_dir: TempDir) {
        let json = r#"{
            "header": {"name": "Test", "tag": "", "symbol": "◆"},
            "body": [{"md5": "", "sha256": "sha_y", "level": "10"}]
        }"#;
        write_bmt(table_dir.path(), "test.bmt", json);

        let map = build_table_level_map(table_dir.path()).unwrap();
        assert_eq!(map.get("sha_y").unwrap()[0].label, "◆10");
    }

    #[rstest]
    fn test_build_table_level_map_skips_entries_without_sha256(table_dir: TempDir) {
        let json = r#"{
            "header": {"name": "Test", "tag": "t", "symbol": ""},
            "body": [
                {"md5": "only_md5", "sha256": "", "level": "1"},
                {"md5": "", "sha256": "has_sha", "level": "2"}
            ]
        }"#;
        write_bmt(table_dir.path(), "test.bmt", json);

        let map = build_table_level_map(table_dir.path()).unwrap();
        assert!(!map.contains_key(""));
        assert_eq!(map.get("has_sha").unwrap()[0].label, "t2");
    }

    #[rstest]
    fn test_build_table_level_map_empty_dir(table_dir: TempDir) {
        let map = build_table_level_map(table_dir.path()).unwrap();
        assert!(map.is_empty());
    }

    #[rstest]
    fn test_build_table_level_map_nonexistent_dir() {
        let map = build_table_level_map(Path::new("/nonexistent/table/dir")).unwrap();
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
        let valid = r#"{
            "header": {"name": "Good", "tag": "g", "symbol": ""},
            "body": [{"md5": "", "sha256": "sha_good", "level": "1"}]
        }"#;
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
