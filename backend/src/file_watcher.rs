#![allow(
    dead_code,
    reason = "public API for DB monitoring pipeline, not yet consumed"
)]

use std::path::PathBuf;
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

/// Start watching a file for modifications. Calls `callback` (debounced to 1 s)
/// whenever the file is written to.
pub fn start_watching(
    path: PathBuf,
    callback: Box<dyn Fn() + Send>,
) -> Result<WatchHandle, WatchError> {
    let (tx, rx) = mpsc::channel::<()>();

    // Canonicalize the target path so we can reliably compare against event paths
    // which the OS may return in canonical form (e.g. /private/var → /var on macOS).
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            let matches_path = event
                .paths
                .iter()
                .any(|p| p.canonicalize().unwrap_or_else(|_| p.clone()) == canonical_path);
            let is_modify = matches!(
                event.kind,
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
            );
            if matches_path && is_modify {
                // Ignore send errors — the debounce thread may have exited.
                let _ = tx.send(());
            }
        }
    })?;

    // Watch the parent directory so we catch file replacements (atomic writes).
    let watch_dir = path.parent().unwrap_or(&path);
    watcher
        .watch(watch_dir, RecursiveMode::NonRecursive)
        .map_err(|e| WatchError::Watch {
            path: watch_dir.to_path_buf(),
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

    use rstest::rstest;
    use tempfile::TempDir;

    #[rstest]
    #[case::single_write(1)]
    fn test_callback_fires_on_file_write(#[case] _variant: u32) {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.db");
        fs::write(&file_path, b"initial").unwrap();

        let count = Arc::new(AtomicU32::new(0));
        let count_clone = Arc::clone(&count);

        let _handle = start_watching(
            file_path.clone(),
            Box::new(move || {
                count_clone.fetch_add(1, Ordering::SeqCst);
            }),
        )
        .unwrap();

        // Give the watcher time to register.
        thread::sleep(Duration::from_millis(200));

        // Write to the file.
        fs::write(&file_path, b"updated").unwrap();

        // Wait for debounce + processing.
        thread::sleep(Duration::from_millis(2000));

        assert!(
            count.load(Ordering::SeqCst) >= 1,
            "callback should fire at least once"
        );
    }

    #[rstest]
    #[case::burst_writes(5)]
    fn test_debounce_coalesces_rapid_writes(#[case] write_count: u32) {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.db");
        fs::write(&file_path, b"initial").unwrap();

        let count = Arc::new(AtomicU32::new(0));
        let count_clone = Arc::clone(&count);

        let _handle = start_watching(
            file_path.clone(),
            Box::new(move || {
                count_clone.fetch_add(1, Ordering::SeqCst);
            }),
        )
        .unwrap();

        thread::sleep(Duration::from_millis(200));

        // Rapidly write multiple times within the debounce window.
        for i in 0..write_count {
            fs::write(&file_path, format!("update {i}")).unwrap();
            thread::sleep(Duration::from_millis(100));
        }

        // Wait for debounce to settle.
        thread::sleep(Duration::from_millis(2000));

        let fires = count.load(Ordering::SeqCst);
        // Debounce should coalesce the burst into fewer callbacks than writes.
        assert!(
            fires < write_count,
            "expected fewer callbacks ({fires}) than writes ({write_count})"
        );
        assert!(fires >= 1, "callback should fire at least once");
    }
}
