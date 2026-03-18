#![allow(
    dead_code,
    reason = "public API for DB monitoring pipeline, not yet consumed"
)]

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WatchError {
    #[error("failed to create file watcher: {0}")]
    Create(#[from] notify::Error),

    #[error("failed to watch path: {path}")]
    Watch {
        path: PathBuf,
        #[source]
        source: notify::Error,
    },
}

/// Handle returned by `start_watching`. Dropping it stops the watcher.
pub struct WatchHandle {
    _watcher: RecommendedWatcher,
    _debounce_thread: std::thread::JoinHandle<()>,
}

const DEBOUNCE_DURATION: Duration = Duration::from_secs(1);

/// Resolve the canonical form of `path`, falling back to the original if
/// canonicalization fails (e.g. the file does not yet exist).
fn canonical_or_raw(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

/// Start watching a file for modifications. Calls `callback` (debounced to 1 s)
/// whenever the file is written to.
pub fn start_watching(
    path: PathBuf,
    callback: Box<dyn Fn() + Send>,
) -> Result<WatchHandle, WatchError> {
    let (tx, rx) = mpsc::channel::<()>();

    // Resolve canonical path once so the event handler only does cheap comparisons.
    // macOS FSEvents returns canonical paths (e.g. /private/var/... instead of /var/...),
    // so we must compare in canonical form.
    let canonical_path = canonical_or_raw(&path);
    let file_name = path.file_name().map(|n| n.to_os_string());

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            let is_modify = matches!(
                event.kind,
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
            );
            if !is_modify {
                return;
            }

            // Fast path: compare file names before attempting canonicalization.
            let matches_path = event.paths.iter().any(|p| {
                if let (Some(event_name), Some(target_name)) = (p.file_name(), &file_name)
                    && event_name != target_name.as_os_str()
                {
                    return false;
                }
                canonical_or_raw(p) == canonical_path
            });

            if matches_path {
                // Ignore send errors — the debounce thread may have exited.
                let _ = tx.send(());
            }
        }
    })?;

    // Watch the parent directory so we catch file replacements (atomic writes).
    // Default to "." if the path has no parent component (bare filename).
    let watch_dir = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    };
    watcher
        .watch(&watch_dir, RecursiveMode::NonRecursive)
        .map_err(|e| WatchError::Watch {
            path: watch_dir,
            source: e,
        })?;

    let debounce_thread = std::thread::spawn(move || {
        debounce_loop(&rx, &callback, DEBOUNCE_DURATION);
    });

    Ok(WatchHandle {
        _watcher: watcher,
        _debounce_thread: debounce_thread,
    })
}

/// Consume events from `rx`, coalescing bursts within `duration` into a single
/// callback invocation.
fn debounce_loop(rx: &mpsc::Receiver<()>, callback: &dyn Fn(), duration: Duration) {
    loop {
        // Block until the first event (or channel closes).
        if rx.recv().is_err() {
            return;
        }

        // Drain any events that arrive within the debounce window.
        loop {
            match rx.recv_timeout(duration) {
                Ok(()) => continue,
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    // Channel closed during debounce — fire one last time.
                    callback();
                    return;
                }
            }
        }

        callback();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::thread;

    use rstest::{fixture, rstest};
    use tempfile::TempDir;

    struct WatchFixture {
        file_path: PathBuf,
        count: Arc<AtomicU32>,
        _handle: WatchHandle,
        _dir: TempDir,
    }

    #[fixture]
    fn watched_file() -> WatchFixture {
        let dir = TempDir::new().expect("failed to create temp dir");
        let file_path = dir.path().join("test.db");
        fs::write(&file_path, b"initial").expect("failed to write initial file");

        let count = Arc::new(AtomicU32::new(0));
        let count_clone = Arc::clone(&count);

        let handle = start_watching(
            file_path.clone(),
            Box::new(move || {
                count_clone.fetch_add(1, Ordering::SeqCst);
            }),
        )
        .expect("failed to start watching");

        // Give the watcher time to register.
        thread::sleep(Duration::from_millis(200));

        WatchFixture {
            file_path,
            count,
            _handle: handle,
            _dir: dir,
        }
    }

    #[rstest]
    #[case::single_write(1)]
    fn test_callback_fires_on_file_write(watched_file: WatchFixture, #[case] _variant: u32) {
        fs::write(&watched_file.file_path, b"updated").expect("failed to write file");

        // Wait for debounce + processing.
        thread::sleep(Duration::from_millis(2000));

        assert!(
            watched_file.count.load(Ordering::SeqCst) >= 1,
            "callback should fire at least once"
        );
    }

    #[rstest]
    #[case::burst_writes(5)]
    fn test_debounce_coalesces_rapid_writes(watched_file: WatchFixture, #[case] write_count: u32) {
        // Rapidly write multiple times within the debounce window.
        for i in 0..write_count {
            fs::write(&watched_file.file_path, format!("update {i}"))
                .expect("failed to write file");
            thread::sleep(Duration::from_millis(100));
        }

        // Wait for debounce to settle.
        thread::sleep(Duration::from_millis(2000));

        let fires = watched_file.count.load(Ordering::SeqCst);
        // Debounce should coalesce the burst into fewer callbacks than writes.
        assert!(
            fires < write_count,
            "expected fewer callbacks ({fires}) than writes ({write_count})"
        );
        assert!(fires >= 1, "callback should fire at least once");
    }
}
