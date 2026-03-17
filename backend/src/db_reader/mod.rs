mod best_score;
mod score_log;
mod song_metadata;

pub use best_score::{BestScore, read_best_score};
pub use score_log::{ScoreLog, read_score_log};
pub use song_metadata::{SongMetadata, read_song_metadata};

use std::path::Path;
use std::time::Duration;

use rusqlite::{Connection, OpenFlags};

const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

fn open_readonly(path: &Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    conn.busy_timeout(BUSY_TIMEOUT)?;
    Ok(conn)
}
