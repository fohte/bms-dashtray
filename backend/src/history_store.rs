use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, Offset as _};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlayRecord {
    pub id: String,
    pub sha256: String,
    pub mode: i32,
    pub clear: i32,
    pub ex_score: i32,
    pub min_bp: i32,
    pub notes: i32,
    pub combo: i32,
    pub played_at: String,
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    pub artist: String,
    pub level: i32,
    pub difficulty: i32,
    #[serde(default)]
    pub table_levels: Vec<String>,
    pub previous_clear: Option<i32>,
    pub previous_ex_score: Option<i32>,
    pub previous_min_bp: Option<i32>,
    /// Whether the player retired mid-play (gauge reached 0 before finishing).
    /// Only meaningful when `clear == 1` (Failed).
    #[serde(default)]
    pub is_retired: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HistoryFile {
    date: String,
    records: Vec<PlayRecord>,
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("failed to read history file: {0}")]
    ReadFile(#[source] std::io::Error),
    #[error("failed to write history file: {0}")]
    WriteFile(#[source] std::io::Error),
    #[error("failed to create directory: {0}")]
    CreateDir(#[source] std::io::Error),
    #[error("failed to parse history file: {0}")]
    Parse(#[source] serde_json::Error),
    #[error("failed to serialize history: {0}")]
    Serialize(#[source] serde_json::Error),
    #[error("invalid reset time format: {0}")]
    InvalidResetTime(String),
    #[error("invalid playedAt timestamp: {0}")]
    InvalidTimestamp(String),
}

/// Returns the logical "today" date given a reset time and a fixed-offset datetime.
///
/// If the wall-clock time in the given offset is before `reset_time`, the
/// logical date is yesterday; otherwise it is the calendar date.
fn logical_today(now: DateTime<FixedOffset>, reset_time: NaiveTime) -> NaiveDate {
    if now.time() < reset_time {
        now.date_naive() - chrono::Duration::days(1)
    } else {
        now.date_naive()
    }
}

fn parse_reset_time(reset_time: &str) -> Result<NaiveTime, StoreError> {
    NaiveTime::parse_from_str(reset_time, "%H:%M")
        .map_err(|_| StoreError::InvalidResetTime(reset_time.to_string()))
}

/// Extracts the logical date of a play record given a reset time.
///
/// The `played_at` field is an ISO 8601 timestamp (possibly in UTC). It is
/// converted to the provided `local_offset` before comparing against the reset
/// time so that records stored in UTC are evaluated in the user's timezone.
fn record_logical_date(
    played_at: &str,
    reset_time: NaiveTime,
    local_offset: FixedOffset,
) -> Result<NaiveDate, StoreError> {
    let dt = DateTime::parse_from_rfc3339(played_at)
        .map_err(|_| StoreError::InvalidTimestamp(played_at.to_string()))?;
    let local = dt.with_timezone(&local_offset);
    if local.time() < reset_time {
        Ok(local.date_naive() - chrono::Duration::days(1))
    } else {
        Ok(local.date_naive())
    }
}

/// Returns "now" as a `DateTime<FixedOffset>` in the local timezone.
fn now_local_fixed() -> DateTime<FixedOffset> {
    let local_now = chrono::Local::now();
    local_now.with_timezone(&local_now.offset().fix())
}

pub struct HistoryStore {
    history_path: PathBuf,
    records: Vec<PlayRecord>,
    reset_time: String,
}

impl HistoryStore {
    pub fn new(history_path: PathBuf, reset_time: &str) -> Self {
        Self {
            history_path,
            records: Vec::new(),
            reset_time: reset_time.to_string(),
        }
    }

    pub fn history_path(&self) -> &std::path::Path {
        &self.history_path
    }

    pub fn set_reset_time(&mut self, reset_time: &str) {
        self.reset_time = reset_time.to_string();
    }

    /// Updates table_levels for all records based on the provided lookup map.
    pub fn update_table_levels(
        &mut self,
        table_map: &std::collections::HashMap<String, Vec<String>>,
    ) {
        for record in &mut self.records {
            record.table_levels = table_map.get(&record.sha256).cloned().unwrap_or_default();
        }
    }

    pub fn add_play_records(&mut self, records: Vec<PlayRecord>) -> Result<(), StoreError> {
        self.records.extend(records);
        // Sort by playedAt descending (most recent first)
        self.records.sort_by(|a, b| {
            let a_dt = DateTime::parse_from_rfc3339(&a.played_at);
            let b_dt = DateTime::parse_from_rfc3339(&b.played_at);
            match (a_dt, b_dt) {
                (Ok(a), Ok(b)) => b.cmp(&a),
                _ => b.played_at.cmp(&a.played_at),
            }
        });
        Ok(())
    }

    pub fn get_today_records(&self) -> Result<Vec<PlayRecord>, StoreError> {
        self.get_today_records_impl(now_local_fixed())
    }

    fn get_today_records_impl(
        &self,
        now: DateTime<FixedOffset>,
    ) -> Result<Vec<PlayRecord>, StoreError> {
        let reset_time = parse_reset_time(&self.reset_time)?;
        let today = logical_today(now, reset_time);
        let offset = *now.offset();

        let filtered: Vec<PlayRecord> = self
            .records
            .iter()
            .filter(|r| {
                matches!(
                    record_logical_date(&r.played_at, reset_time, offset),
                    Ok(d) if d == today
                )
            })
            .cloned()
            .collect();
        Ok(filtered)
    }

    pub fn reset(&mut self) -> Result<(), StoreError> {
        let previous = std::mem::take(&mut self.records);
        match self.persist() {
            Ok(()) => Ok(()),
            Err(e) => {
                self.records = previous;
                Err(e)
            }
        }
    }

    pub fn persist(&self) -> Result<(), StoreError> {
        self.persist_impl(now_local_fixed())
    }

    fn persist_impl(&self, now: DateTime<FixedOffset>) -> Result<(), StoreError> {
        if let Some(parent) = self.history_path.parent() {
            fs::create_dir_all(parent).map_err(StoreError::CreateDir)?;
        }

        let reset_time = parse_reset_time(&self.reset_time)?;
        let today = logical_today(now, reset_time);

        let file = HistoryFile {
            date: today.format("%Y-%m-%d").to_string(),
            records: self.records.clone(),
        };
        let contents = serde_json::to_string_pretty(&file).map_err(StoreError::Serialize)?;
        fs::write(&self.history_path, contents).map_err(StoreError::WriteFile)
    }

    pub fn restore(&mut self) -> Result<(), StoreError> {
        self.restore_impl(now_local_fixed())
    }

    fn restore_impl(&mut self, now: DateTime<FixedOffset>) -> Result<(), StoreError> {
        if !self.history_path.exists() {
            return Ok(());
        }

        let contents = fs::read_to_string(&self.history_path).map_err(StoreError::ReadFile)?;
        let file: HistoryFile = serde_json::from_str(&contents).map_err(StoreError::Parse)?;

        let reset_time = parse_reset_time(&self.reset_time)?;
        let today = logical_today(now, reset_time);

        if file.date == today.format("%Y-%m-%d").to_string() {
            self.records = file.records;
            // Ensure sorted by playedAt descending
            self.records.sort_by(|a, b| {
                let a_dt = DateTime::parse_from_rfc3339(&a.played_at);
                let b_dt = DateTime::parse_from_rfc3339(&b.played_at);
                match (a_dt, b_dt) {
                    (Ok(a), Ok(b)) => b.cmp(&a),
                    _ => b.played_at.cmp(&a.played_at),
                }
            });
        } else {
            // Date has changed; discard stale records
            self.records.clear();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone as _;
    use indoc::indoc;
    use rstest::{fixture, rstest};
    use tempfile::TempDir;

    const JST: i32 = 9 * 3600;

    struct TestContext {
        _dir: TempDir,
        store: HistoryStore,
    }

    impl TestContext {
        fn history_path(&self) -> PathBuf {
            self._dir.path().join("history.json")
        }
    }

    #[fixture]
    fn ctx() -> TestContext {
        let dir = tempfile::tempdir().unwrap();
        let history_path = dir.path().join("history.json");
        let store = HistoryStore::new(history_path, "05:00");
        TestContext { _dir: dir, store }
    }

    fn jst_datetime(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> DateTime<FixedOffset> {
        FixedOffset::east_opt(JST)
            .unwrap()
            .with_ymd_and_hms(y, m, d, h, min, s)
            .unwrap()
    }

    fn make_record(id: &str, played_at: &str) -> PlayRecord {
        PlayRecord {
            id: id.to_string(),
            sha256: "abc123".to_string(),
            mode: 0,
            clear: 6,
            ex_score: 1234,
            min_bp: 15,
            notes: 800,
            combo: 500,
            played_at: played_at.to_string(),
            title: "Test Song".to_string(),
            subtitle: String::new(),
            artist: "Test Artist".to_string(),
            level: 12,
            difficulty: 1,
            table_levels: Vec::new(),
            previous_clear: Some(5),
            previous_ex_score: Some(1100),
            previous_min_bp: Some(20),
            is_retired: false,
        }
    }

    #[rstest]
    fn test_add_records_sorts_by_played_at_descending(mut ctx: TestContext) {
        let r1 = make_record("r1", "2026-03-18T10:00:00+09:00");
        let r2 = make_record("r2", "2026-03-18T12:00:00+09:00");
        let r3 = make_record("r3", "2026-03-18T11:00:00+09:00");

        ctx.store.add_play_records(vec![r1, r2, r3]).unwrap();

        let records = ctx.store.records.clone();
        assert_eq!(records[0].id, "r2");
        assert_eq!(records[1].id, "r3");
        assert_eq!(records[2].id, "r1");
    }

    #[rstest]
    #[case::filters_by_date(
        // now=2026-03-18 14:00 JST, today record + yesterday record
        jst_datetime(2026, 3, 18, 14, 0, 0),
        vec![
            ("today", "2026-03-18T10:00:00+09:00"),
            ("yesterday", "2026-03-17T10:00:00+09:00"),
        ],
        vec!["today"],
    )]
    #[case::reset_time_boundary(
        // At 04:59 JST, logical today = 2026-03-17 (before reset time 05:00)
        jst_datetime(2026, 3, 18, 4, 59, 0),
        vec![
            ("late", "2026-03-17T23:00:00+09:00"),
            ("next", "2026-03-18T06:00:00+09:00"),
        ],
        vec!["late"],
    )]
    #[case::early_morning_belongs_to_previous_day(
        // At 03:30 JST, play at 03:00 belongs to logical date 2026-03-17
        jst_datetime(2026, 3, 18, 3, 30, 0),
        vec![("early", "2026-03-18T03:00:00+09:00")],
        vec!["early"],
    )]
    #[case::utc_timestamp_matches_local_today(
        // User in JST at 13:59 (= 04:59 UTC), record stored as UTC
        jst_datetime(2026, 3, 18, 13, 59, 0),
        vec![("utc", "2026-03-18T04:59:00.000Z")],
        vec!["utc"],
    )]
    fn test_get_today_records(
        mut ctx: TestContext,
        #[case] now: DateTime<FixedOffset>,
        #[case] input_records: Vec<(&str, &str)>,
        #[case] expected_ids: Vec<&str>,
    ) {
        let records: Vec<PlayRecord> = input_records
            .into_iter()
            .map(|(id, played_at)| make_record(id, played_at))
            .collect();
        ctx.store.add_play_records(records).unwrap();

        let today_records = ctx.store.get_today_records_impl(now).unwrap();
        let ids: Vec<&str> = today_records.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, expected_ids);
    }

    #[rstest]
    fn test_reset_clears_all_records(mut ctx: TestContext) {
        let r1 = make_record("r1", "2026-03-18T10:00:00+09:00");
        ctx.store.add_play_records(vec![r1]).unwrap();

        ctx.store.reset().unwrap();

        assert!(ctx.store.records.is_empty());
    }

    #[rstest]
    fn test_persist_and_restore_roundtrip(mut ctx: TestContext) {
        let now = jst_datetime(2026, 3, 18, 14, 0, 0);

        let r1 = make_record("r1", "2026-03-18T10:00:00+09:00");
        let r2 = make_record("r2", "2026-03-18T12:00:00+09:00");

        ctx.store.add_play_records(vec![r1, r2]).unwrap();
        ctx.store.persist_impl(now).unwrap();

        // Create a new store and restore
        let mut new_store = HistoryStore::new(ctx.history_path(), "05:00");
        new_store.restore_impl(now).unwrap();

        assert_eq!(new_store.records.len(), 2);
        assert_eq!(new_store.records[0].id, "r2");
        assert_eq!(new_store.records[1].id, "r1");
    }

    #[rstest]
    fn test_restore_discards_stale_records_on_date_change(mut ctx: TestContext) {
        let yesterday_file = indoc! {r#"
            {
              "date": "2026-03-17",
              "records": [
                {
                  "id": "old",
                  "sha256": "abc",
                  "mode": 0,
                  "clear": 6,
                  "exScore": 1234,
                  "minBp": 15,
                  "notes": 800,
                  "combo": 500,
                  "playedAt": "2026-03-17T10:00:00+09:00",
                  "title": "Test",
                  "subtitle": "",
                  "artist": "Test",
                  "level": 12,
                  "difficulty": 1,
                  "previousClear": null,
                  "previousExScore": null,
                  "previousMinBp": null
                }
              ]
            }
        "#};
        fs::write(ctx.history_path(), yesterday_file).unwrap();

        let now = jst_datetime(2026, 3, 18, 14, 0, 0);
        ctx.store.restore_impl(now).unwrap();

        assert!(ctx.store.records.is_empty());
    }

    #[rstest]
    fn test_restore_with_no_file_is_noop(mut ctx: TestContext) {
        ctx.store.restore().unwrap();
        assert!(ctx.store.records.is_empty());
    }

    #[rstest]
    fn test_reset_persists_empty_file(mut ctx: TestContext) {
        let r1 = make_record("r1", "2026-03-18T10:00:00+09:00");
        ctx.store.add_play_records(vec![r1]).unwrap();

        ctx.store.reset().unwrap();

        assert!(ctx.history_path().exists());
        let contents = fs::read_to_string(ctx.history_path()).unwrap();
        let file: HistoryFile = serde_json::from_str(&contents).unwrap();
        assert!(file.records.is_empty());
    }
}
