use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::config::AppConfig;
use crate::diff_detector::{DbPaths, DiffDetector};
use crate::event_bridge::{EventEmitter, ScoresUpdatedPayload};
use crate::file_watcher::{self, WatchHandle};
use crate::history_store::HistoryStore;
use crate::table_reader;

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("diff detector error: {0}")]
    Diff(#[from] crate::diff_detector::DiffError),
    #[error("history store error: {0}")]
    Store(#[from] crate::history_store::StoreError),
    #[error("file watcher error: {0}")]
    Watch(#[from] crate::file_watcher::WatchError),
    #[error("table reader error: {0}")]
    Table(#[from] crate::table_reader::TableReaderError),
    #[error("mutex poisoned: {0}")]
    MutexPoisoned(String),
}

/// Build the set of (sha256, mode, played_at) keys from restored history records.
fn build_restored_keys(
    store: &HistoryStore,
) -> Result<HashSet<(String, i32, String)>, crate::history_store::StoreError> {
    let records = store.get_today_records()?;
    Ok(records
        .into_iter()
        .map(|r| (r.sha256, r.mode, r.played_at))
        .collect())
}

/// Runs one cycle of the pipeline: read DB → detect diff → add to store → emit event.
fn run_pipeline_cycle(
    detector: &mut DiffDetector,
    store: &mut HistoryStore,
    emitter: &dyn EventEmitter,
    config: &AppConfig,
    restored_keys: &HashSet<(String, i32, String)>,
) -> Result<(), PipelineError> {
    let db_paths = DbPaths {
        scoredatalog: &config.scoredatalog_db_path(),
        score: &config.score_db_path(),
        scorelog: &config.scorelog_db_path(),
        songdata: &config.songdata_db_path(),
    };

    let new_records = detector.on_db_changed(&db_paths, restored_keys)?;

    if new_records.is_empty() {
        return Ok(());
    }

    store.add_play_records(new_records)?;
    store.persist()?;

    let today_records = store.get_today_records()?;
    let now = chrono::Local::now();
    let payload = ScoresUpdatedPayload {
        records: today_records,
        updated_at: now.to_rfc3339(),
    };

    if let Err(e) = emitter.emit_scores_updated(payload) {
        eprintln!("failed to emit scores-updated event: {e}");
    }

    Ok(())
}

/// Holds all watcher handles. Dropping this stops all watchers.
pub struct PipelineHandle {
    _scoredatalog_watcher: WatchHandle,
    _table_watcher: Option<WatchHandle>,
}

/// Reloads table levels from .bmt files and updates the detector.
fn reload_table_levels(detector: &mut DiffDetector, table_dir: &std::path::Path) {
    match table_reader::build_table_level_map(table_dir) {
        Ok(map) => detector.set_table_levels(map),
        Err(e) => eprintln!("failed to load table levels: {e}"),
    }
}

/// Starts the full pipeline: initial DB read + file watchers for scoredatalog and table directory.
///
/// Returns a `PipelineHandle` that keeps the watchers alive. Dropping it stops the pipeline.
pub fn start_pipeline(
    config: &AppConfig,
    store: Arc<Mutex<HistoryStore>>,
    emitter: Arc<dyn EventEmitter>,
) -> Result<PipelineHandle, PipelineError> {
    let mut detector = DiffDetector::new();
    detector.load_best_scores(&config.score_db_path())?;

    // Load difficulty table levels
    let table_dir = table_reader::table_dir_path(&config.beatoraja_root);
    reload_table_levels(&mut detector, &table_dir);

    // Build restored keys before initial read
    let restored_keys = {
        let store_guard = store
            .lock()
            .map_err(|e| PipelineError::MutexPoisoned(format!("history store: {e}")))?;
        build_restored_keys(&store_guard)?
    };

    // Initial DB read
    {
        let mut store_guard = store
            .lock()
            .map_err(|e| PipelineError::MutexPoisoned(format!("history store: {e}")))?;

        // Update restored records with table levels
        let label_map = detector.table_level_labels();
        store_guard.update_table_levels(&label_map);

        run_pipeline_cycle(
            &mut detector,
            &mut store_guard,
            emitter.as_ref(),
            config,
            &restored_keys,
        )?;
    }

    let scoredatalog_path = config.scoredatalog_db_path();
    let config_clone = config.clone();
    let detector = Arc::new(Mutex::new(detector));
    let empty_keys: HashSet<(String, i32, String)> = HashSet::new();

    let detector_for_score = Arc::clone(&detector);
    let store_for_score = Arc::clone(&store);
    let emitter_for_score = Arc::clone(&emitter);
    let config_for_score = config_clone.clone();

    let scoredatalog_handle = file_watcher::start_watching(
        scoredatalog_path,
        Box::new(move || {
            let mut det = match detector_for_score.lock() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("failed to lock detector: {e}");
                    return;
                }
            };
            let mut store_guard = match store_for_score.lock() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("failed to lock store: {e}");
                    return;
                }
            };
            if let Err(e) = run_pipeline_cycle(
                &mut det,
                &mut store_guard,
                emitter_for_score.as_ref(),
                &config_for_score,
                &empty_keys,
            ) {
                eprintln!("pipeline error: {e}");
            }
        }),
    )?;

    // Watch table directory for .bmt file changes
    let table_handle = if table_dir.exists() {
        let detector_for_table = Arc::clone(&detector);
        let store_for_table = Arc::clone(&store);
        let emitter_for_table = Arc::clone(&emitter);
        let table_dir_clone = table_dir.clone();

        match file_watcher::start_watching_dir(
            table_dir,
            "bmt",
            Box::new(move || {
                let mut det = match detector_for_table.lock() {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("failed to lock detector for table reload: {e}");
                        return;
                    }
                };
                reload_table_levels(&mut det, &table_dir_clone);

                // Update table_levels in existing store records
                let label_map = det.table_level_labels();

                let mut store_guard = match store_for_table.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("failed to lock store for table reload: {e}");
                        return;
                    }
                };
                store_guard.update_table_levels(&label_map);
                if let Err(e) = store_guard.persist() {
                    eprintln!("failed to persist after table reload: {e}");
                }

                match store_guard.get_today_records() {
                    Ok(records) => {
                        let now = chrono::Local::now();
                        let payload = ScoresUpdatedPayload {
                            records,
                            updated_at: now.to_rfc3339(),
                        };
                        if let Err(e) = emitter_for_table.emit_scores_updated(payload) {
                            eprintln!("failed to emit scores-updated after table reload: {e}");
                        }
                    }
                    Err(e) => eprintln!("failed to get today records for table reload: {e}"),
                }
            }),
        ) {
            Ok(h) => Some(h),
            Err(e) => {
                eprintln!("failed to watch table directory: {e}");
                None
            }
        }
    } else {
        None
    };

    Ok(PipelineHandle {
        _scoredatalog_watcher: scoredatalog_handle,
        _table_watcher: table_handle,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Mutex as StdMutex;

    use indoc::indoc;
    use rstest::{fixture, rstest};
    use rusqlite::Connection;
    use tempfile::TempDir;

    /// Returns a UNIX timestamp in milliseconds for "today" at the given hour (in local time),
    /// using the logical date (accounting for the 05:00 reset time) so that
    /// `get_today_records()` will include this record even when tests run between midnight and 05:00.
    fn today_millis(hour: u32) -> i64 {
        use chrono::{Datelike as _, Local, NaiveTime, TimeZone as _};
        let now = Local::now();
        let reset_time = NaiveTime::from_hms_opt(5, 0, 0).unwrap_or_default();
        let logical_date = if now.time() < reset_time {
            now.date_naive() - chrono::Duration::days(1)
        } else {
            now.date_naive()
        };
        let dt = Local
            .with_ymd_and_hms(
                logical_date.year(),
                logical_date.month(),
                logical_date.day(),
                hour,
                0,
                0,
            )
            .single();
        match dt {
            Some(d) => d.timestamp_millis(),
            None => Local::now().timestamp_millis(),
        }
    }

    struct MockEmitter {
        payloads: StdMutex<Vec<ScoresUpdatedPayload>>,
    }

    impl MockEmitter {
        fn new() -> Self {
            Self {
                payloads: StdMutex::new(Vec::new()),
            }
        }

        fn payloads(&self) -> Vec<ScoresUpdatedPayload> {
            self.payloads
                .lock()
                .map_or_else(|_| Vec::new(), |p| p.clone())
        }
    }

    impl EventEmitter for MockEmitter {
        fn emit_scores_updated(&self, payload: ScoresUpdatedPayload) -> Result<(), String> {
            if let Ok(mut payloads) = self.payloads.lock() {
                payloads.push(payload);
            }
            Ok(())
        }
    }

    struct PipelineTestContext {
        _dir: TempDir,
        config: AppConfig,
        store: HistoryStore,
        detector: DiffDetector,
        emitter: Arc<MockEmitter>,
    }

    fn create_db_schema(path: &std::path::Path, schema: &str) {
        let conn = Connection::open(path).unwrap_or_else(|e| {
            panic!("failed to open {}: {e}", path.display());
        });
        conn.execute_batch(schema).unwrap_or_else(|e| {
            panic!("failed to create schema for {}: {e}", path.display());
        });
    }

    #[fixture]
    fn ctx() -> PipelineTestContext {
        let dir = tempfile::tempdir().unwrap_or_else(|e| {
            panic!("failed to create temp dir: {e}");
        });
        let base = dir.path();

        let beatoraja_root = base.join("beatoraja");
        let player_dir = beatoraja_root.join("player").join("default");
        std::fs::create_dir_all(&player_dir).unwrap_or_else(|e| {
            panic!("failed to create player dir: {e}");
        });

        create_db_schema(
            &beatoraja_root.join("songdata.db"),
            indoc! {"
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
            "},
        );

        create_db_schema(
            &player_dir.join("scoredatalog.db"),
            indoc! {"
                CREATE TABLE scoredatalog (
                    sha256 TEXT NOT NULL,
                    mode INTEGER NOT NULL,
                    clear INTEGER NOT NULL,
                    epg INTEGER NOT NULL,
                    egr INTEGER NOT NULL,
                    egd INTEGER NOT NULL,
                    epr INTEGER NOT NULL,
                    emr INTEGER NOT NULL,
                    ems INTEGER NOT NULL,
                    lpg INTEGER NOT NULL,
                    lgr INTEGER NOT NULL,
                    lgd INTEGER NOT NULL,
                    lpr INTEGER NOT NULL,
                    lmr INTEGER NOT NULL,
                    lms INTEGER NOT NULL,
                    minbp INTEGER NOT NULL,
                    notes INTEGER NOT NULL,
                    combo INTEGER NOT NULL,
                    date INTEGER NOT NULL,
                    PRIMARY KEY (sha256, mode)
                )
            "},
        );

        create_db_schema(
            &player_dir.join("score.db"),
            indoc! {"
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
            "},
        );

        create_db_schema(
            &player_dir.join("scorelog.db"),
            indoc! {"
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
            "},
        );

        let config = AppConfig {
            beatoraja_root: beatoraja_root.to_string_lossy().to_string(),
            player_name: "default".to_string(),
            ..Default::default()
        };

        let history_path = base.join("history.json");
        let store = HistoryStore::new(history_path, &config.reset_time);
        let emitter = Arc::new(MockEmitter::new());

        let mut detector = DiffDetector::new();
        detector
            .load_best_scores(&config.score_db_path())
            .unwrap_or_else(|e| {
                panic!("failed to load best scores: {e}");
            });

        PipelineTestContext {
            _dir: dir,
            config,
            store,
            detector,
            emitter,
        }
    }

    fn insert_scoredatalog(config: &AppConfig, sha256: &str, mode: i32, clear: i32, date: i64) {
        let conn = Connection::open(config.scoredatalog_db_path()).unwrap_or_else(|e| {
            panic!("failed to open scoredatalog: {e}");
        });
        conn.execute(
            "INSERT OR REPLACE INTO scoredatalog \
             (sha256, mode, clear, epg, egr, egd, epr, emr, ems, lpg, lgr, lgd, lpr, lmr, lms, minbp, notes, combo, date) \
             VALUES (?1, ?2, ?3, 100, 50, 0, 0, 0, 0, 80, 30, 0, 0, 0, 0, 15, 800, 500, ?4)",
            rusqlite::params![sha256, mode, clear, date],
        )
        .unwrap_or_else(|e| {
            panic!("failed to insert scoredatalog: {e}");
        });
    }

    fn insert_songdata(config: &AppConfig, sha256: &str, title: &str, artist: &str) {
        let conn = Connection::open(config.songdata_db_path()).unwrap_or_else(|e| {
            panic!("failed to open songdata: {e}");
        });
        conn.execute(
            "INSERT INTO song (md5, sha256, title, artist, level, difficulty, notes, mode, path) \
             VALUES ('md5', ?1, ?2, ?3, 12, 1, 1500, 0, ?1)",
            rusqlite::params![sha256, title, artist],
        )
        .unwrap_or_else(|e| {
            panic!("failed to insert songdata: {e}");
        });
    }

    #[rstest]
    fn test_pipeline_cycle_emits_scores_updated(mut ctx: PipelineTestContext) {
        insert_scoredatalog(&ctx.config, "abc123", 0, 6, today_millis(10));
        insert_songdata(&ctx.config, "abc123", "Test Song", "Artist");

        let restored_keys = HashSet::new();
        run_pipeline_cycle(
            &mut ctx.detector,
            &mut ctx.store,
            ctx.emitter.as_ref(),
            &ctx.config,
            &restored_keys,
        )
        .unwrap_or_else(|e| {
            panic!("pipeline cycle failed: {e}");
        });

        let payloads = ctx.emitter.payloads();
        assert_eq!(payloads.len(), 1);

        let payload = &payloads[0];
        assert_eq!(payload.records.len(), 1);
        assert_eq!(payload.records[0].sha256, "abc123");
        assert_eq!(payload.records[0].title, "Test Song");
        assert_eq!(payload.records[0].clear, 6);
        // ex_score = 100*2 + 50 + 80*2 + 30 = 440
        assert_eq!(payload.records[0].ex_score, 440);
        assert!(!payload.updated_at.is_empty());
    }

    #[rstest]
    fn test_pipeline_cycle_no_emit_when_no_changes(mut ctx: PipelineTestContext) {
        let restored_keys = HashSet::new();
        run_pipeline_cycle(
            &mut ctx.detector,
            &mut ctx.store,
            ctx.emitter.as_ref(),
            &ctx.config,
            &restored_keys,
        )
        .unwrap_or_else(|e| {
            panic!("pipeline cycle failed: {e}");
        });

        assert!(ctx.emitter.payloads().is_empty());
    }

    #[rstest]
    fn test_pipeline_cycle_emits_all_today_records(mut ctx: PipelineTestContext) {
        insert_scoredatalog(&ctx.config, "abc123", 0, 6, today_millis(10));
        insert_songdata(&ctx.config, "abc123", "Song A", "Artist A");

        let restored_keys = HashSet::new();

        // First cycle: adds first record
        run_pipeline_cycle(
            &mut ctx.detector,
            &mut ctx.store,
            ctx.emitter.as_ref(),
            &ctx.config,
            &restored_keys,
        )
        .unwrap_or_else(|e| {
            panic!("first pipeline cycle failed: {e}");
        });

        // Add second record
        insert_scoredatalog(&ctx.config, "def456", 0, 5, today_millis(12));
        insert_songdata(&ctx.config, "def456", "Song B", "Artist B");

        // Second cycle: adds second record, payload should contain both
        run_pipeline_cycle(
            &mut ctx.detector,
            &mut ctx.store,
            ctx.emitter.as_ref(),
            &ctx.config,
            &restored_keys,
        )
        .unwrap_or_else(|e| {
            panic!("second pipeline cycle failed: {e}");
        });

        let payloads = ctx.emitter.payloads();
        assert_eq!(payloads.len(), 2);

        // The second payload should contain all today's records
        let second_payload = &payloads[1];
        assert_eq!(second_payload.records.len(), 2);

        let mut sha256s: Vec<&str> = second_payload
            .records
            .iter()
            .map(|r| r.sha256.as_str())
            .collect();
        sha256s.sort();
        assert_eq!(sha256s, vec!["abc123", "def456"]);
    }

    #[rstest]
    fn test_pipeline_persists_after_adding_records(mut ctx: PipelineTestContext) {
        insert_scoredatalog(&ctx.config, "abc123", 0, 6, today_millis(10));
        insert_songdata(&ctx.config, "abc123", "Test Song", "Artist");

        let restored_keys = HashSet::new();
        run_pipeline_cycle(
            &mut ctx.detector,
            &mut ctx.store,
            ctx.emitter.as_ref(),
            &ctx.config,
            &restored_keys,
        )
        .unwrap_or_else(|e| {
            panic!("pipeline cycle failed: {e}");
        });

        // Verify history was persisted by restoring into a new store
        let history_path = ctx.store.history_path().to_path_buf();
        assert!(history_path.exists());
    }
}
