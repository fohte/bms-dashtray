#![expect(clippy::panic, reason = "panics are acceptable in test code")]

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use indoc::indoc;
use rstest::{fixture, rstest};
use rusqlite::Connection;
use tempfile::TempDir;

use bms_dashtray::config::AppConfig;
use bms_dashtray::diff_detector::{DbPaths, DiffDetector};
use bms_dashtray::event_bridge::{EventEmitter, ScoresUpdatedPayload};
use bms_dashtray::history_store::{HistoryStore, PlayRecord};
use bms_dashtray::pipeline;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns a UNIX timestamp in seconds for "today" at the given hour
/// (in local time), using the logical date (accounting for the 05:00 reset
/// time) so that `get_today_records()` includes the record even when tests run
/// between midnight and 05:00.
fn today_secs(hour: u32) -> i64 {
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
        Some(d) => d.timestamp(),
        None => Local::now().timestamp(),
    }
}

struct MockEmitter {
    payloads: Mutex<Vec<ScoresUpdatedPayload>>,
}

impl MockEmitter {
    fn new() -> Self {
        Self {
            payloads: Mutex::new(Vec::new()),
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

struct E2EContext {
    _dir: TempDir,
    config: AppConfig,
    history_path: std::path::PathBuf,
}

/// Context for tests that manually drive pipeline cycles (without file watcher).
struct PipelineCycleContext {
    e2e: E2EContext,
    store: HistoryStore,
    emitter: MockEmitter,
    detector: DiffDetector,
}

fn create_db_schema(path: &std::path::Path, schema: &str) {
    let conn = Connection::open(path).unwrap_or_else(|e| {
        panic!("failed to open {}: {e}", path.display());
    });
    conn.execute_batch(schema).unwrap_or_else(|e| {
        panic!("failed to create schema for {}: {e}", path.display());
    });
}

fn create_beatoraja_dbs(beatoraja_root: &std::path::Path, player_name: &str) {
    let player_dir = beatoraja_root.join("player").join(player_name);
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
}

fn insert_scoredatalog(config: &AppConfig, sha256: &str, mode: i32, clear: i32, date: i64) {
    let conn = Connection::open(config.scoredatalog_db_path()).unwrap_or_else(|e| {
        panic!("failed to open scoredatalog: {e}");
    });
    conn.execute(
        "INSERT OR REPLACE INTO scoredatalog \
         (sha256, mode, clear, epg, egr, egd, ebd, epr, emr, ems, lpg, lgr, lgd, lbd, lpr, lmr, lms, minbp, notes, combo, date) \
         VALUES (?1, ?2, ?3, 100, 50, 0, 0, 0, 0, 0, 80, 30, 0, 0, 0, 0, 0, 15, 800, 500, ?4)",
        rusqlite::params![sha256, mode, clear, date],
    )
    .unwrap_or_else(|e| {
        panic!("failed to insert scoredatalog: {e}");
    });
}

fn insert_songdata(config: &AppConfig, sha256: &str, title: &str, level: i32, difficulty: i32) {
    let conn = Connection::open(config.songdata_db_path()).unwrap_or_else(|e| {
        panic!("failed to open songdata: {e}");
    });
    conn.execute(
        "INSERT INTO song (md5, sha256, title, artist, level, difficulty, notes, mode, path) \
         VALUES ('md5', ?1, ?2, 'Artist', ?3, ?4, 1500, 0, ?1)",
        rusqlite::params![sha256, title, level, difficulty],
    )
    .unwrap_or_else(|e| {
        panic!("failed to insert songdata: {e}");
    });
}

fn insert_score(config: &AppConfig, sha256: &str, mode: i32, clear: i32, minbp: i32) {
    let conn = Connection::open(config.score_db_path()).unwrap_or_else(|e| {
        panic!("failed to open score: {e}");
    });
    conn.execute(
        "INSERT INTO score (sha256, mode, clear, epg, egr, lpg, lgr, minbp) \
         VALUES (?1, ?2, ?3, 100, 50, 80, 30, ?4)",
        rusqlite::params![sha256, mode, clear, minbp],
    )
    .unwrap_or_else(|e| {
        panic!("failed to insert score: {e}");
    });
}

/// Run a single pipeline cycle (DB read -> diff detect -> store -> emit) without
/// the file watcher, allowing deterministic control over timing.
fn run_pipeline_cycle(
    detector: &mut DiffDetector,
    store: &mut HistoryStore,
    emitter: &dyn EventEmitter,
    config: &AppConfig,
) {
    let db_paths = DbPaths {
        scoredatalog: &config.scoredatalog_db_path(),
        score: &config.score_db_path(),
        scorelog: &config.scorelog_db_path(),
        songdata: &config.songdata_db_path(),
    };

    let empty_keys: HashSet<(String, i32, String)> = HashSet::new();
    let new_records = detector
        .on_db_changed(&db_paths, &empty_keys)
        .unwrap_or_else(|e| {
            panic!("on_db_changed failed: {e}");
        });

    if new_records.is_empty() {
        return;
    }

    store.add_play_records(new_records).unwrap_or_else(|e| {
        panic!("add_play_records failed: {e}");
    });
    store.persist().unwrap_or_else(|e| {
        panic!("persist failed: {e}");
    });

    let today_records = store.get_today_records().unwrap_or_else(|e| {
        panic!("get_today_records failed: {e}");
    });
    let now = chrono::Local::now();
    let payload = ScoresUpdatedPayload {
        records: today_records,
        updated_at: now.to_rfc3339(),
    };
    emitter.emit_scores_updated(payload).unwrap_or_else(|e| {
        panic!("emit_scores_updated failed: {e}");
    });
}

/// Asserts that the sorted titles of the given records match the expected titles.
fn assert_record_titles(records: &[PlayRecord], expected: &[&str]) {
    let mut titles: Vec<&str> = records.iter().map(|r| r.title.as_str()).collect();
    titles.sort();
    assert_eq!(titles, expected);
}

#[fixture]
fn e2e_ctx() -> E2EContext {
    let dir = tempfile::tempdir().unwrap_or_else(|e| {
        panic!("failed to create temp dir: {e}");
    });
    let base = dir.path();

    let beatoraja_root = base.join("beatoraja");
    create_beatoraja_dbs(&beatoraja_root, "default");

    let config = AppConfig {
        beatoraja_root: beatoraja_root.to_string_lossy().to_string(),
        player_name: "default".to_string(),
        ..Default::default()
    };

    let history_path = base.join("history.json");

    E2EContext {
        _dir: dir,
        config,
        history_path,
    }
}

#[fixture]
fn pipeline_ctx(e2e_ctx: E2EContext) -> PipelineCycleContext {
    let store = HistoryStore::new(e2e_ctx.history_path.clone(), &e2e_ctx.config.reset_time);
    let emitter = MockEmitter::new();
    let mut detector = DiffDetector::new();
    detector
        .load_best_scores(&e2e_ctx.config.score_db_path())
        .unwrap_or_else(|e| {
            panic!("load_best_scores failed: {e}");
        });

    PipelineCycleContext {
        e2e: e2e_ctx,
        store,
        emitter,
        detector,
    }
}

// ---------------------------------------------------------------------------
// E2E Test 1: Initial startup -> DB change detected -> records emitted
// Simulates: app opens, no prior history, DB has existing plays -> pipeline
//            detects them and emits scores-updated event.
// ---------------------------------------------------------------------------

#[rstest]
fn test_initial_startup_detects_existing_plays(e2e_ctx: E2EContext) {
    // Pre-populate the DB with plays (simulating beatoraja already has data)
    insert_scoredatalog(&e2e_ctx.config, "song_a", 0, 6, today_secs(10));
    insert_scoredatalog(&e2e_ctx.config, "song_b", 0, 4, today_secs(11));
    insert_songdata(&e2e_ctx.config, "song_a", "FREEDOM DiVE", 12, 4);
    insert_songdata(&e2e_ctx.config, "song_b", "Quaver", 10, 3);

    let store = Arc::new(Mutex::new(HistoryStore::new(
        e2e_ctx.history_path.clone(),
        &e2e_ctx.config.reset_time,
    )));
    let emitter = Arc::new(MockEmitter::new());

    // start_pipeline does initial DB read + starts file watcher
    let _handle = pipeline::start_pipeline(
        &e2e_ctx.config,
        Arc::clone(&store),
        Arc::clone(&emitter) as Arc<dyn EventEmitter>,
    )
    .unwrap_or_else(|e| {
        panic!("start_pipeline failed: {e}");
    });

    // Verify: initial read emitted all today's records
    let payloads = emitter.payloads();
    assert_eq!(payloads.len(), 1, "expected exactly one initial emission");
    assert_eq!(
        payloads[0].records.len(),
        2,
        "expected 2 records from initial read"
    );

    assert_record_titles(&payloads[0].records, &["FREEDOM DiVE", "Quaver"]);

    // Verify: store persisted the records
    let store_guard = store.lock().unwrap_or_else(|e| {
        panic!("failed to lock store: {e}");
    });
    let today = store_guard.get_today_records().unwrap_or_else(|e| {
        panic!("get_today_records failed: {e}");
    });
    assert_eq!(today.len(), 2);
}

// ---------------------------------------------------------------------------
// E2E Test 2: DB file change -> real-time list/graph update
// Simulates: pipeline is running, a new play is added to the DB, pipeline
//            detects the change and emits updated records.
// ---------------------------------------------------------------------------

#[rstest]
fn test_realtime_update_on_db_change(mut pipeline_ctx: PipelineCycleContext) {
    let ctx = &mut pipeline_ctx;

    // Start with one existing play
    insert_scoredatalog(&ctx.e2e.config, "song_a", 0, 5, today_secs(10));
    insert_songdata(&ctx.e2e.config, "song_a", "Quaver", 10, 3);

    // Cycle 1: initial read picks up existing play
    run_pipeline_cycle(
        &mut ctx.detector,
        &mut ctx.store,
        &ctx.emitter,
        &ctx.e2e.config,
    );

    let payloads = ctx.emitter.payloads();
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0].records.len(), 1);
    assert_eq!(payloads[0].records[0].title, "Quaver");

    // Simulate a new play being added to the DB (beatoraja writes new data)
    insert_scoredatalog(&ctx.e2e.config, "song_b", 0, 7, today_secs(12));
    insert_songdata(&ctx.e2e.config, "song_b", "FREEDOM DiVE", 12, 4);

    // Cycle 2: detects the new play
    run_pipeline_cycle(
        &mut ctx.detector,
        &mut ctx.store,
        &ctx.emitter,
        &ctx.e2e.config,
    );

    let payloads = ctx.emitter.payloads();
    assert_eq!(payloads.len(), 2, "expected 2 emissions after second cycle");

    // The second emission should contain ALL today's records (cumulative)
    let second = &payloads[1];
    assert_eq!(second.records.len(), 2, "expected 2 cumulative records");

    assert_record_titles(&second.records, &["FREEDOM DiVE", "Quaver"]);

    // Verify: same chart replayed (updated played_at) is detected
    insert_scoredatalog(&ctx.e2e.config, "song_a", 0, 6, today_secs(14));

    // Cycle 3: detects the replay of song_a
    run_pipeline_cycle(
        &mut ctx.detector,
        &mut ctx.store,
        &ctx.emitter,
        &ctx.e2e.config,
    );

    let payloads = ctx.emitter.payloads();
    assert_eq!(payloads.len(), 3);

    // Now store should have 3 records total (song_a played twice + song_b once)
    let third = &payloads[2];
    assert_eq!(third.records.len(), 3, "expected 3 records after replay");
}

// ---------------------------------------------------------------------------
// E2E Test 3: App restart -> today's history restored from persistence
// Simulates: pipeline runs and persists records, then a new HistoryStore is
//            created (simulating restart), restores the records, and the
//            pipeline skips already-restored entries on its initial read.
// ---------------------------------------------------------------------------

#[rstest]
fn test_restart_restores_todays_history(e2e_ctx: E2EContext) {
    // --- First session ---
    insert_scoredatalog(&e2e_ctx.config, "song_a", 0, 6, today_secs(10));
    insert_scoredatalog(&e2e_ctx.config, "song_b", 0, 4, today_secs(11));
    insert_songdata(&e2e_ctx.config, "song_a", "FREEDOM DiVE", 12, 4);
    insert_songdata(&e2e_ctx.config, "song_b", "Quaver", 10, 3);

    {
        let store = Arc::new(Mutex::new(HistoryStore::new(
            e2e_ctx.history_path.clone(),
            &e2e_ctx.config.reset_time,
        )));
        let emitter = Arc::new(MockEmitter::new());

        let _handle = pipeline::start_pipeline(
            &e2e_ctx.config,
            Arc::clone(&store),
            Arc::clone(&emitter) as Arc<dyn EventEmitter>,
        )
        .unwrap_or_else(|e| {
            panic!("start_pipeline failed: {e}");
        });

        // Verify first session detected 2 records
        let payloads = emitter.payloads();
        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0].records.len(), 2);
    }
    // _handle and store are dropped here (simulating app shutdown)

    // Verify history file exists on disk
    assert!(
        e2e_ctx.history_path.exists(),
        "history file should be persisted"
    );

    // --- Second session (restart) ---
    let mut restored_store =
        HistoryStore::new(e2e_ctx.history_path.clone(), &e2e_ctx.config.reset_time);
    restored_store.restore().unwrap_or_else(|e| {
        panic!("restore failed: {e}");
    });

    // Verify restored records
    let today = restored_store.get_today_records().unwrap_or_else(|e| {
        panic!("get_today_records failed: {e}");
    });
    assert_eq!(
        today.len(),
        2,
        "should restore 2 records from first session"
    );

    assert_record_titles(&today, &["FREEDOM DiVE", "Quaver"]);

    // Start pipeline again - should NOT duplicate the restored records
    let store = Arc::new(Mutex::new(restored_store));
    let emitter = Arc::new(MockEmitter::new());

    let _handle = pipeline::start_pipeline(
        &e2e_ctx.config,
        Arc::clone(&store),
        Arc::clone(&emitter) as Arc<dyn EventEmitter>,
    )
    .unwrap_or_else(|e| {
        panic!("start_pipeline (restart) failed: {e}");
    });

    // The pipeline should emit the restored records but NOT add duplicates
    let store_guard = store.lock().unwrap_or_else(|e| {
        panic!("failed to lock store: {e}");
    });
    let all_records = store_guard.get_today_records().unwrap_or_else(|e| {
        panic!("get_today_records failed: {e}");
    });
    assert_eq!(
        all_records.len(),
        2,
        "should still have exactly 2 records after restart (no duplicates)"
    );
}

// ---------------------------------------------------------------------------
// E2E Test 4: Manual reset -> history cleared
// Simulates: user triggers manual reset, all records are cleared, persistence
//            file is updated, and subsequent pipeline cycles start fresh.
// ---------------------------------------------------------------------------

#[rstest]
fn test_manual_reset_clears_history(mut pipeline_ctx: PipelineCycleContext) {
    let ctx = &mut pipeline_ctx;

    // Build up some history
    insert_scoredatalog(&ctx.e2e.config, "song_a", 0, 6, today_secs(10));
    insert_songdata(&ctx.e2e.config, "song_a", "FREEDOM DiVE", 12, 4);

    run_pipeline_cycle(
        &mut ctx.detector,
        &mut ctx.store,
        &ctx.emitter,
        &ctx.e2e.config,
    );

    // Verify we have 1 record
    let today = ctx.store.get_today_records().unwrap_or_else(|e| {
        panic!("get_today_records failed: {e}");
    });
    assert_eq!(today.len(), 1);

    // User triggers manual reset
    ctx.store.reset().unwrap_or_else(|e| {
        panic!("reset failed: {e}");
    });

    // Verify: all records cleared
    let today = ctx.store.get_today_records().unwrap_or_else(|e| {
        panic!("get_today_records failed: {e}");
    });
    assert!(today.is_empty(), "records should be empty after reset");

    // Verify: persistence file reflects the reset
    let mut restored_store =
        HistoryStore::new(ctx.e2e.history_path.clone(), &ctx.e2e.config.reset_time);
    restored_store.restore().unwrap_or_else(|e| {
        panic!("restore after reset failed: {e}");
    });
    let restored_records = restored_store.get_today_records().unwrap_or_else(|e| {
        panic!("get_today_records after restore failed: {e}");
    });
    assert!(
        restored_records.is_empty(),
        "restored records should be empty after reset"
    );

    // Verify: new plays after reset are still detected
    insert_scoredatalog(&ctx.e2e.config, "song_b", 0, 4, today_secs(14));
    insert_songdata(&ctx.e2e.config, "song_b", "Quaver", 10, 3);

    run_pipeline_cycle(
        &mut ctx.detector,
        &mut ctx.store,
        &ctx.emitter,
        &ctx.e2e.config,
    );

    let today = ctx.store.get_today_records().unwrap_or_else(|e| {
        panic!("get_today_records failed: {e}");
    });
    assert_eq!(today.len(), 1, "should detect new play after reset");
    assert_eq!(today[0].title, "Quaver");
}

// ---------------------------------------------------------------------------
// E2E Test 5: Level distribution data is correct across multiple plays
// Verifies that records contain correct level/difficulty metadata needed for
// the distribution chart.
// ---------------------------------------------------------------------------

#[rstest]
fn test_level_distribution_data(mut pipeline_ctx: PipelineCycleContext) {
    let ctx = &mut pipeline_ctx;

    // Insert plays at different difficulty levels
    insert_scoredatalog(&ctx.e2e.config, "song_lv10", 0, 6, today_secs(10));
    insert_scoredatalog(&ctx.e2e.config, "song_lv12", 0, 5, today_secs(11));
    insert_scoredatalog(&ctx.e2e.config, "song_lv12b", 0, 7, today_secs(12));
    insert_songdata(&ctx.e2e.config, "song_lv10", "Easy Song", 10, 3);
    insert_songdata(&ctx.e2e.config, "song_lv12", "Hard Song A", 12, 4);
    insert_songdata(&ctx.e2e.config, "song_lv12b", "Hard Song B", 12, 4);

    run_pipeline_cycle(
        &mut ctx.detector,
        &mut ctx.store,
        &ctx.emitter,
        &ctx.e2e.config,
    );

    let today = ctx.store.get_today_records().unwrap_or_else(|e| {
        panic!("get_today_records failed: {e}");
    });
    assert_eq!(today.len(), 3);

    // Verify level/difficulty are correctly attached to each record
    let lv10_records: Vec<&PlayRecord> = today.iter().filter(|r| r.level == 10).collect();
    let lv12_records: Vec<&PlayRecord> = today.iter().filter(|r| r.level == 12).collect();

    assert_eq!(lv10_records.len(), 1);
    assert_eq!(lv12_records.len(), 2);
    assert_eq!(lv10_records[0].difficulty, 3);
    assert!(lv12_records.iter().all(|r| r.difficulty == 4));
}

// ---------------------------------------------------------------------------
// E2E Test 6: Clear lamp update detection with previous best
// Verifies that when a player improves their clear lamp, the previous best
// values are correctly tracked (needed for the UP badge in the UI).
// ---------------------------------------------------------------------------

#[rstest]
fn test_clear_lamp_update_tracking(e2e_ctx: E2EContext) {
    // Set up initial best score in score.db BEFORE loading best scores
    insert_score(&e2e_ctx.config, "song_a", 0, 5, 20);
    insert_songdata(&e2e_ctx.config, "song_a", "FREEDOM DiVE", 12, 4);

    // New play with better clear
    insert_scoredatalog(&e2e_ctx.config, "song_a", 0, 7, today_secs(10));

    let mut store = HistoryStore::new(e2e_ctx.history_path.clone(), &e2e_ctx.config.reset_time);
    let emitter = MockEmitter::new();
    let mut detector = DiffDetector::new();
    detector
        .load_best_scores(&e2e_ctx.config.score_db_path())
        .unwrap_or_else(|e| {
            panic!("load_best_scores failed: {e}");
        });

    run_pipeline_cycle(&mut detector, &mut store, &emitter, &e2e_ctx.config);

    let today = store.get_today_records().unwrap_or_else(|e| {
        panic!("get_today_records failed: {e}");
    });
    assert_eq!(today.len(), 1);

    // previous_clear should reflect the old best from score.db
    assert_eq!(today[0].previous_clear, Some(5));
    assert_eq!(today[0].previous_min_bp, Some(20));
    assert_eq!(today[0].clear, 7);
}
