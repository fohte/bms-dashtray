use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::db_reader::{
    BestScore, ScoreDataLog, read_best_score, read_score_data_logs, read_score_log,
    read_song_metadata,
};
use crate::history_store::PlayRecord;
use crate::table_reader::TableLevel;

#[derive(Debug, thiserror::Error)]
pub enum DiffError {
    #[error("database error: {0}")]
    DB(#[from] crate::db_reader::DBError),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

type ChartKey = (String, i32);

pub struct DiffDetector {
    /// Previous snapshot: (sha256, mode) -> ScoreDataLog
    snapshot: HashMap<ChartKey, ScoreDataLog>,
    /// Best score cache: (sha256, mode) -> BestScore (lazily populated)
    best_cache: HashMap<ChartKey, BestScore>,
    /// Whether this is the first read (no previous snapshot exists)
    is_first_read: bool,
    /// Difficulty table levels: sha256 -> Vec<TableLevel>
    table_levels: HashMap<String, Vec<TableLevel>>,
}

/// DB paths required by DiffDetector.
pub struct DbPaths<'a> {
    pub scoredatalog: &'a Path,
    pub score: &'a Path,
    pub scorelog: &'a Path,
    pub songdata: &'a Path,
}

impl Default for DiffDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffDetector {
    pub fn new() -> Self {
        Self {
            snapshot: HashMap::new(),
            best_cache: HashMap::new(),
            is_first_read: true,
            table_levels: HashMap::new(),
        }
    }

    /// Sets the difficulty table level lookup map.
    pub fn set_table_levels(&mut self, table_levels: HashMap<String, Vec<TableLevel>>) {
        self.table_levels = table_levels;
    }

    /// Returns a map of sha256 -> Vec<label string> for use in updating store records.
    pub fn table_level_labels(&self) -> HashMap<String, Vec<String>> {
        self.table_levels
            .iter()
            .map(|(sha256, levels)| {
                (
                    sha256.clone(),
                    levels.iter().map(|l| l.label.clone()).collect(),
                )
            })
            .collect()
    }

    /// Called when the DB file changes. Detects new plays by comparing against the previous snapshot.
    ///
    /// On the first read, `restored_keys` should contain (sha256, mode, played_at) tuples
    /// from the HistoryStore so that already-restored records are skipped.
    ///
    /// `min_date_secs` limits the scoredatalog query to rows with `date >= min_date_secs`,
    /// avoiding a full table scan for users with large play histories.
    pub fn on_db_changed(
        &mut self,
        db_paths: &DbPaths<'_>,
        restored_keys: &HashSet<(String, i32, String)>,
        min_date_secs: Option<i64>,
    ) -> Result<Vec<PlayRecord>, DiffError> {
        let current_logs = read_score_data_logs(db_paths.scoredatalog, min_date_secs)?;

        let current_map: HashMap<ChartKey, ScoreDataLog> = current_logs
            .into_iter()
            .map(|log| ((log.sha256.clone(), log.mode), log))
            .collect();

        let new_plays = if self.is_first_read {
            self.detect_first_read(&current_map, restored_keys)
        } else {
            self.detect_changes(&current_map)
        };

        let records = self.enrich_plays(&new_plays, db_paths)?;

        self.snapshot = current_map;
        self.is_first_read = false;

        Ok(records)
    }

    /// On first read, every record whose (sha256, mode, played_at) is NOT in restored_keys
    /// is considered a new play.
    fn detect_first_read(
        &self,
        current: &HashMap<ChartKey, ScoreDataLog>,
        restored_keys: &HashSet<(String, i32, String)>,
    ) -> Vec<ScoreDataLog> {
        current
            .values()
            .filter(|log| {
                !restored_keys.contains(&(log.sha256.clone(), log.mode, log.played_at.clone()))
            })
            .cloned()
            .collect()
    }

    /// On subsequent reads, a record is new if its `played_at` differs from the snapshot.
    fn detect_changes(&self, current: &HashMap<ChartKey, ScoreDataLog>) -> Vec<ScoreDataLog> {
        let mut new_plays = Vec::new();

        for (key, log) in current {
            match self.snapshot.get(key) {
                Some(prev) if prev.played_at != log.played_at => {
                    new_plays.push(log.clone());
                }
                None => {
                    // Completely new chart entry
                    new_plays.push(log.clone());
                }
                _ => {}
            }
        }

        new_plays
    }

    /// Enriches raw score data logs with song metadata and best score information.
    fn enrich_plays(
        &mut self,
        plays: &[ScoreDataLog],
        db_paths: &DbPaths<'_>,
    ) -> Result<Vec<PlayRecord>, DiffError> {
        let mut records = Vec::with_capacity(plays.len());

        for play in plays {
            let key = (play.sha256.clone(), play.mode);

            // Determine previous best: check scorelog for best update, otherwise lazily query score.db
            let previous = self.resolve_previous_best(&key, play, db_paths)?;

            // Update best cache if the current play is better
            self.update_best_cache(&key, play);

            let metadata = read_song_metadata(db_paths.songdata, &play.sha256)?;

            let (title, subtitle, artist, level, difficulty) = match metadata {
                Some(m) => (m.title, m.subtitle, m.artist, m.level, m.difficulty),
                None => (String::new(), String::new(), String::new(), 0, 0),
            };

            let table_levels = self
                .table_levels
                .get(&play.sha256)
                .map(|levels| levels.iter().map(|l| l.label.clone()).collect())
                .unwrap_or_default();

            let is_retired = play.clear == 1 && play.consumed_notes < play.notes;

            records.push(PlayRecord {
                id: uuid::Uuid::new_v4().to_string(),
                sha256: play.sha256.clone(),
                mode: play.mode,
                clear: play.clear,
                ex_score: play.ex_score,
                min_bp: play.min_bp,
                notes: play.notes,
                combo: play.combo,
                played_at: play.played_at.clone(),
                title,
                subtitle,
                artist,
                level,
                difficulty,
                table_levels,
                previous_clear: previous.as_ref().map(|p| p.clear),
                previous_ex_score: previous.as_ref().map(|p| p.ex_score),
                previous_min_bp: previous.as_ref().map(|p| p.min_bp),
                is_retired,
            });
        }

        Ok(records)
    }

    /// Resolves the previous best score for a play.
    /// If scorelog has an entry for this timestamp (best update happened),
    /// use the old values from scorelog. Otherwise, look up score.db (with lazy caching).
    fn resolve_previous_best(
        &mut self,
        key: &ChartKey,
        play: &ScoreDataLog,
        db_paths: &DbPaths<'_>,
    ) -> Result<Option<BestScore>, DiffError> {
        // Both scorelog.date and scoredatalog.date store UNIX seconds
        let score_log = read_score_log(db_paths.scorelog, &play.sha256, play.mode, play.date_secs)?;

        if let Some(log) = score_log {
            // Best was updated: the old values represent the previous best.
            // beatoraja uses sentinel values for "no previous play":
            // old_clear=0, old_score=0, old_min_bp=i32::MAX.
            // Treat this as no previous best.
            if log.old_clear == 0 && log.old_score == 0 && log.old_min_bp == i32::MAX {
                return Ok(None);
            }
            let previous = BestScore {
                clear: log.old_clear,
                ex_score: log.old_score,
                min_bp: log.old_min_bp,
            };
            // Seed best_cache so that update_best_cache merges against the
            // score.db baseline rather than initializing from the current play alone.
            if !self.best_cache.contains_key(key) {
                self.best_cache.insert(key.clone(), previous.clone());
            }
            Ok(Some(previous))
        } else {
            // No best update: use cached best score, or lazily fetch from score.db
            if let Some(cached) = self.best_cache.get(key) {
                return Ok(Some(cached.clone()));
            }
            let best = read_best_score(db_paths.score, &play.sha256, play.mode)?;
            if let Some(ref b) = best {
                self.best_cache.insert(key.clone(), b.clone());
            }
            Ok(best)
        }
    }

    /// Updates the best score cache per-metric: each metric is tracked independently.
    fn update_best_cache(&mut self, key: &ChartKey, play: &ScoreDataLog) {
        let entry = self.best_cache.entry(key.clone()).or_insert(BestScore {
            clear: play.clear,
            ex_score: play.ex_score,
            min_bp: play.min_bp,
        });
        entry.clear = entry.clear.max(play.clear);
        entry.ex_score = entry.ex_score.max(play.ex_score);
        entry.min_bp = entry.min_bp.min(play.min_bp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;
    use rstest::{fixture, rstest};
    use rusqlite::Connection;
    use tempfile::TempDir;

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

    const SCORE_SCHEMA: &str = indoc! {"
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
        )
    "};

    const SCORELOG_SCHEMA: &str = indoc! {"
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
        )
    "};

    const SONGDATA_SCHEMA: &str = indoc! {"
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
        )
    "};

    struct TestDbs {
        _dir: TempDir,
        paths: TestDbPaths,
    }

    struct TestDbPaths {
        scoredatalog: std::path::PathBuf,
        score: std::path::PathBuf,
        scorelog: std::path::PathBuf,
        songdata: std::path::PathBuf,
    }

    impl TestDbs {
        fn db_paths(&self) -> DbPaths<'_> {
            DbPaths {
                scoredatalog: &self.paths.scoredatalog,
                score: &self.paths.score,
                scorelog: &self.paths.scorelog,
                songdata: &self.paths.songdata,
            }
        }

        fn scoredatalog_conn(&self) -> Connection {
            Connection::open(&self.paths.scoredatalog).expect("open scoredatalog")
        }

        fn score_conn(&self) -> Connection {
            Connection::open(&self.paths.score).expect("open score")
        }

        fn scorelog_conn(&self) -> Connection {
            Connection::open(&self.paths.scorelog).expect("open scorelog")
        }

        fn songdata_conn(&self) -> Connection {
            Connection::open(&self.paths.songdata).expect("open songdata")
        }
    }

    #[fixture]
    fn test_dbs() -> TestDbs {
        let dir = tempfile::tempdir().expect("create temp dir");
        let base = dir.path();

        let scoredatalog = base.join("scoredatalog.db");
        let score = base.join("score.db");
        let scorelog = base.join("scorelog.db");
        let songdata = base.join("songdata.db");

        Connection::open(&scoredatalog)
            .expect("open")
            .execute_batch(SCOREDATALOG_SCHEMA)
            .expect("schema");
        Connection::open(&score)
            .expect("open")
            .execute_batch(SCORE_SCHEMA)
            .expect("schema");
        Connection::open(&scorelog)
            .expect("open")
            .execute_batch(SCORELOG_SCHEMA)
            .expect("schema");
        Connection::open(&songdata)
            .expect("open")
            .execute_batch(SONGDATA_SCHEMA)
            .expect("schema");

        TestDbs {
            _dir: dir,
            paths: TestDbPaths {
                scoredatalog,
                score,
                scorelog,
                songdata,
            },
        }
    }

    fn insert_scoredatalog(conn: &Connection, sha256: &str, mode: i32, clear: i32, date: i64) {
        insert_scoredatalog_full(conn, sha256, mode, clear, 100, 50, 80, 30, 15, date);
    }

    #[expect(clippy::too_many_arguments, reason = "test helper mirrors DB columns")]
    fn insert_scoredatalog_full(
        conn: &Connection,
        sha256: &str,
        mode: i32,
        clear: i32,
        epg: i32,
        egr: i32,
        lpg: i32,
        lgr: i32,
        minbp: i32,
        date: i64,
    ) {
        conn.execute(
            "INSERT OR REPLACE INTO scoredatalog \
             (sha256, mode, clear, epg, egr, egd, ebd, epr, emr, ems, lpg, lgr, lgd, lbd, lpr, lmr, lms, minbp, notes, combo, date) \
             VALUES (?1, ?2, ?3, ?4, ?5, 0, 0, 0, 0, 0, ?6, ?7, 0, 0, 0, 0, 0, ?8, 800, 500, ?9)",
            rusqlite::params![sha256, mode, clear, epg, egr, lpg, lgr, minbp, date],
        )
        .expect("insert scoredatalog");
    }

    fn insert_score(conn: &Connection, sha256: &str, mode: i32, clear: i32, minbp: i32) {
        conn.execute(
            "INSERT INTO score (sha256, mode, clear, epg, egr, lpg, lgr, minbp) \
             VALUES (?1, ?2, ?3, 100, 50, 80, 30, ?4)",
            rusqlite::params![sha256, mode, clear, minbp],
        )
        .expect("insert score");
    }

    fn insert_scorelog(
        conn: &Connection,
        sha256: &str,
        mode: i32,
        old_clear: i32,
        old_score: i32,
        old_minbp: i32,
        date: i64,
    ) {
        conn.execute(
            "INSERT INTO scorelog (sha256, mode, clear, oldclear, score, oldscore, combo, oldcombo, minbp, oldminbp, date) \
             VALUES (?1, ?2, 6, ?3, 440, ?4, 500, 480, 15, ?5, ?6)",
            rusqlite::params![sha256, mode, old_clear, old_score, old_minbp, date],
        )
        .expect("insert scorelog");
    }

    fn assert_previous_best(
        results: &[PlayRecord],
        expected_clear: Option<i32>,
        expected_ex_score: Option<i32>,
        expected_min_bp: Option<i32>,
    ) {
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].previous_clear, expected_clear);
        assert_eq!(results[0].previous_ex_score, expected_ex_score);
        assert_eq!(results[0].previous_min_bp, expected_min_bp);
    }

    fn insert_songdata(conn: &Connection, sha256: &str, title: &str, artist: &str) {
        conn.execute(
            "INSERT INTO song (md5, sha256, title, artist, level, difficulty, notes, mode, path) \
             VALUES ('md5', ?1, ?2, ?3, 12, 1, 1500, 0, ?1)",
            rusqlite::params![sha256, title, artist],
        )
        .expect("insert songdata");
    }

    #[rstest]
    fn test_detect_new_record(test_dbs: TestDbs) {
        // date = 1710400000 (2024-03-14T07:06:40Z)
        insert_scoredatalog(&test_dbs.scoredatalog_conn(), "abc123", 0, 6, 1710400000);
        insert_songdata(&test_dbs.songdata_conn(), "abc123", "Test Song", "Artist");

        let mut detector = DiffDetector::new();

        let results = detector
            .on_db_changed(&test_dbs.db_paths(), &HashSet::new(), None)
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].sha256, "abc123");
        assert_eq!(results[0].title, "Test Song");
        assert_eq!(results[0].clear, 6);
        // ex_score = 100*2 + 50 + 80*2 + 30 = 440
        assert_eq!(results[0].ex_score, 440);
    }

    #[rstest]
    fn test_detect_updated_record(test_dbs: TestDbs) {
        insert_scoredatalog(&test_dbs.scoredatalog_conn(), "abc123", 0, 5, 1710400000);
        insert_songdata(&test_dbs.songdata_conn(), "abc123", "Test Song", "Artist");

        let mut detector = DiffDetector::new();

        // First read: establishes snapshot
        let _ = detector
            .on_db_changed(&test_dbs.db_paths(), &HashSet::new(), None)
            .unwrap();

        // Update the record (simulate a new play)
        insert_scoredatalog(&test_dbs.scoredatalog_conn(), "abc123", 0, 6, 1710500000);

        let results = detector
            .on_db_changed(&test_dbs.db_paths(), &HashSet::new(), None)
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].sha256, "abc123");
        assert_eq!(results[0].clear, 6);
    }

    #[rstest]
    fn test_no_change_detected(test_dbs: TestDbs) {
        insert_scoredatalog(&test_dbs.scoredatalog_conn(), "abc123", 0, 6, 1710400000);
        insert_songdata(&test_dbs.songdata_conn(), "abc123", "Test Song", "Artist");

        let mut detector = DiffDetector::new();

        // First read
        let _ = detector
            .on_db_changed(&test_dbs.db_paths(), &HashSet::new(), None)
            .unwrap();

        // Second read with same data: no changes
        let results = detector
            .on_db_changed(&test_dbs.db_paths(), &HashSet::new(), None)
            .unwrap();

        assert!(results.is_empty());
    }

    #[rstest]
    fn test_skip_restored_records_on_first_read(test_dbs: TestDbs) {
        // Two records in the DB
        insert_scoredatalog(&test_dbs.scoredatalog_conn(), "abc123", 0, 6, 1710400000);
        insert_scoredatalog(&test_dbs.scoredatalog_conn(), "def456", 0, 5, 1710500000);
        insert_songdata(&test_dbs.songdata_conn(), "abc123", "Song A", "Artist");
        insert_songdata(&test_dbs.songdata_conn(), "def456", "Song B", "Artist");

        let mut detector = DiffDetector::new();

        // abc123 was already restored from history
        let mut restored = HashSet::new();
        restored.insert(("abc123".to_string(), 0, "2024-03-14T07:06:40Z".to_string()));

        let results = detector
            .on_db_changed(&test_dbs.db_paths(), &restored, None)
            .unwrap();

        // Only def456 should be detected as new
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].sha256, "def456");
    }

    #[rstest]
    #[case::best_update_uses_scorelog_old_values(
        Some((5, 20)),    // score.db: clear=5, minbp=20
        Some((5, 400, 20, 1710500000)),  // scorelog: old_clear=5, old_score=400, old_minbp=20
        (Some(5), Some(400), Some(20)),
    )]
    #[case::no_best_update_uses_cached_best(
        Some((6, 15)),    // score.db: clear=6, minbp=15
        None,             // no scorelog entry
        (Some(6), Some(440), Some(15)),   // cache values (ex_score=440 from score.db fixture)
    )]
    #[case::no_previous_best_for_first_play(
        None,             // no score.db entry
        None,             // no scorelog entry
        (None, None, None),
    )]
    #[case::first_play_with_scorelog_sentinel_values(
        None,             // no score.db entry
        Some((0, 0, i32::MAX, 1710500000)),  // scorelog: beatoraja sentinel for "no previous play"
        (None, None, None),
    )]
    fn test_resolve_previous_best(
        test_dbs: TestDbs,
        #[case] score_db_entry: Option<(i32, i32)>,
        #[case] scorelog_entry: Option<(i32, i32, i32, i64)>,
        #[case] expected: (Option<i32>, Option<i32>, Option<i32>),
    ) {
        if let Some((clear, minbp)) = score_db_entry {
            insert_score(&test_dbs.score_conn(), "abc123", 0, clear, minbp);
        }
        insert_scoredatalog(&test_dbs.scoredatalog_conn(), "abc123", 0, 6, 1710500000);
        if let Some((old_clear, old_score, old_minbp, date)) = scorelog_entry {
            insert_scorelog(
                &test_dbs.scorelog_conn(),
                "abc123",
                0,
                old_clear,
                old_score,
                old_minbp,
                date,
            );
        }
        insert_songdata(&test_dbs.songdata_conn(), "abc123", "Test Song", "Artist");

        let mut detector = DiffDetector::new();

        let results = detector
            .on_db_changed(&test_dbs.db_paths(), &HashSet::new(), None)
            .unwrap();

        assert_previous_best(&results, expected.0, expected.1, expected.2);
    }

    #[rstest]
    fn test_best_cache_updated_after_best_update(test_dbs: TestDbs) {
        insert_score(&test_dbs.score_conn(), "abc123", 0, 5, 20);
        insert_songdata(&test_dbs.songdata_conn(), "abc123", "Test Song", "Artist");

        let mut detector = DiffDetector::new();

        // First play: best update happens
        insert_scoredatalog(&test_dbs.scoredatalog_conn(), "abc123", 0, 6, 1710500000);
        insert_scorelog(
            &test_dbs.scorelog_conn(),
            "abc123",
            0,
            5,
            400,
            20,
            1710500000,
        );
        let _ = detector
            .on_db_changed(&test_dbs.db_paths(), &HashSet::new(), None)
            .unwrap();

        // Second play: no best update, should use the updated cache
        insert_scoredatalog(&test_dbs.scoredatalog_conn(), "abc123", 0, 5, 1710600000);
        let results = detector
            .on_db_changed(&test_dbs.db_paths(), &HashSet::new(), None)
            .unwrap();

        // Cache should reflect the updated best (from the first play: clear=6, ex_score=440, min_bp=15)
        assert_previous_best(&results, Some(6), Some(440), Some(15));
    }

    /// Regression test: when the scorelog path is taken (best update happened),
    /// best_cache must be seeded from score.db so that update_best_cache
    /// does not lose the baseline for metrics the current play did not improve.
    #[rstest]
    fn test_best_cache_seeded_on_scorelog_path(test_dbs: TestDbs) {
        // score.db: clear=5, ex_score=440 (from fixture epg/egr/lpg/lgr), min_bp=10
        insert_score(&test_dbs.score_conn(), "abc123", 0, 5, 10);
        insert_songdata(&test_dbs.songdata_conn(), "abc123", "Test Song", "Artist");

        let mut detector = DiffDetector::new();

        // First play: best update happens (clear improved 5→7, but ex_score=200 < 440)
        // epg=50, egr=50, lpg=25, lgr=0 → ex_score = 50*2+50+25*2+0 = 200
        insert_scoredatalog_full(
            &test_dbs.scoredatalog_conn(),
            "abc123",
            0,
            7,
            50,
            50,
            25,
            0,
            25,
            1710500000,
        );
        insert_scorelog(
            &test_dbs.scorelog_conn(),
            "abc123",
            0,
            5,
            440,
            10,
            1710500000,
        );
        let _ = detector
            .on_db_changed(&test_dbs.db_paths(), &HashSet::new(), None)
            .unwrap();

        // Second play: no best update, cache should have merged baselines correctly
        // Expected: clear=max(5,7)=7, ex_score=max(440,200)=440, min_bp=min(10,25)=10
        insert_scoredatalog_full(
            &test_dbs.scoredatalog_conn(),
            "abc123",
            0,
            3,
            10,
            10,
            10,
            10,
            30,
            1710600000,
        );
        let results = detector
            .on_db_changed(&test_dbs.db_paths(), &HashSet::new(), None)
            .unwrap();

        // Without the fix, ex_score would be 200 (from the first play only) instead of 440
        assert_previous_best(&results, Some(7), Some(440), Some(10));
    }

    #[rstest]
    #[case::metadata_found(
        "abc123",
        Some(("FREEDOM DiVE", "xi")),
        ("FREEDOM DiVE", "xi", 12, 1),
    )]
    #[case::metadata_missing(
        "unknown",
        None,
        ("", "", 0, 0),
    )]
    fn test_song_metadata(
        test_dbs: TestDbs,
        #[case] sha256: &str,
        #[case] songdata: Option<(&str, &str)>,
        #[case] expected: (&str, &str, i32, i32),
    ) {
        insert_scoredatalog(&test_dbs.scoredatalog_conn(), sha256, 0, 6, 1710400000);
        if let Some((title, artist)) = songdata {
            insert_songdata(&test_dbs.songdata_conn(), sha256, title, artist);
        }

        let mut detector = DiffDetector::new();

        let results = detector
            .on_db_changed(&test_dbs.db_paths(), &HashSet::new(), None)
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, expected.0);
        assert_eq!(results[0].artist, expected.1);
        assert_eq!(results[0].level, expected.2);
        assert_eq!(results[0].difficulty, expected.3);
    }

    #[rstest]
    fn test_best_cache_merges_per_metric(test_dbs: TestDbs) {
        // Initial best: clear=5, ex_score=500 (epg=200,egr=50,lpg=25,lgr=0), min_bp=10
        insert_score(&test_dbs.score_conn(), "abc123", 0, 5, 10);
        // Override ex_score in score.db to 500 by using custom epg/egr/lpg/lgr
        // (score fixture uses epg=100,egr=50,lpg=80,lgr=30 → 440, so we use insert_score_full)
        // Actually, insert_score uses fixed epg=100,egr=50,lpg=80,lgr=30 → ex_score=440
        // So cache starts at {clear=5, ex_score=440, min_bp=10}
        insert_songdata(&test_dbs.songdata_conn(), "abc123", "Test Song", "Artist");

        let mut detector = DiffDetector::new();

        // Play with better clear (7) but worse ex_score (200) and worse min_bp (25)
        // epg=50, egr=50, lpg=25, lgr=0 → ex_score = 50*2+50+25*2+0 = 200
        insert_scoredatalog_full(
            &test_dbs.scoredatalog_conn(),
            "abc123",
            0,
            7,
            50,
            50,
            25,
            0,
            25,
            1710500000,
        );
        let _ = detector
            .on_db_changed(&test_dbs.db_paths(), &HashSet::new(), None)
            .unwrap();

        // Next play: no best update, cache should have merged per-metric bests
        // Expected cache: clear=max(5,7)=7, ex_score=max(440,200)=440, min_bp=min(10,25)=10
        insert_scoredatalog_full(
            &test_dbs.scoredatalog_conn(),
            "abc123",
            0,
            3,
            10,
            10,
            10,
            10,
            30,
            1710600000,
        );
        let results = detector
            .on_db_changed(&test_dbs.db_paths(), &HashSet::new(), None)
            .unwrap();

        assert_previous_best(&results, Some(7), Some(440), Some(10));
    }

    #[rstest]
    #[case::retired_mid_play(
        1,   // clear: Failed
        260, // consumed_notes (epg+egr+lpg+lgr = 100+50+80+30, all other judge = 0)
        800, // notes
        true,
    )]
    #[case::failed_but_completed(
        1,   // clear: Failed
        260, // consumed_notes == notes (all notes consumed)
        260, // notes == consumed (260)
        false,
    )]
    #[case::cleared_not_retired(
        6,   // clear: Hard
        260, // consumed < notes, but clear >= 2 so not retired
        800,
        false,
    )]
    fn test_is_retired(
        test_dbs: TestDbs,
        #[case] clear: i32,
        #[case] _consumed: i32,
        #[case] notes: i32,
        #[case] expected_retired: bool,
    ) {
        // insert_scoredatalog sets epg=100,egr=50,lpg=80,lgr=30, all other judges=0.
        // consumed_notes = 260, notes parameter controls the total.
        // For the "failed_but_completed" case we set notes=260 so consumed==notes.
        let conn = test_dbs.scoredatalog_conn();
        conn.execute(
            "INSERT OR REPLACE INTO scoredatalog \
             (sha256, mode, clear, epg, egr, egd, ebd, epr, emr, ems, lpg, lgr, lgd, lbd, lpr, lmr, lms, minbp, notes, combo, date) \
             VALUES ('abc123', 0, ?1, 100, 50, 0, 0, 0, 0, 0, 80, 30, 0, 0, 0, 0, 0, 15, ?2, 500, 1710400000)",
            rusqlite::params![clear, notes],
        )
        .expect("insert scoredatalog");
        drop(conn);

        insert_songdata(&test_dbs.songdata_conn(), "abc123", "Test Song", "Artist");

        let mut detector = DiffDetector::new();

        let results = detector
            .on_db_changed(&test_dbs.db_paths(), &HashSet::new(), None)
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].is_retired, expected_retired);
    }
}
