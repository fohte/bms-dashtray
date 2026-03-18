use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default)]
    pub beatoraja_root: String,
    #[serde(default)]
    pub player_name: String,
    #[serde(default = "default_reset_time")]
    pub reset_time: String,
    #[serde(default)]
    pub background_transparent: bool,
    #[serde(default = "default_font_size")]
    pub font_size: i32,
}

fn default_reset_time() -> String {
    "05:00".to_string()
}

fn default_font_size() -> i32 {
    13
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            beatoraja_root: String::new(),
            player_name: String::new(),
            reset_time: default_reset_time(),
            background_transparent: false,
            font_size: default_font_size(),
        }
    }
}

impl AppConfig {
    /// Derives the path to songdata.db from beatoraja_root.
    pub fn songdata_db_path(&self) -> PathBuf {
        Path::new(&self.beatoraja_root).join("songdata.db")
    }

    /// Derives the path to scoredatalog.db from beatoraja_root and player_name.
    pub fn scoredatalog_db_path(&self) -> PathBuf {
        Path::new(&self.beatoraja_root)
            .join("player")
            .join(&self.player_name)
            .join("scoredatalog.db")
    }

    /// Derives the path to score.db from beatoraja_root and player_name.
    pub fn score_db_path(&self) -> PathBuf {
        Path::new(&self.beatoraja_root)
            .join("player")
            .join(&self.player_name)
            .join("score.db")
    }

    /// Derives the path to scorelog.db from beatoraja_root and player_name.
    pub fn scorelog_db_path(&self) -> PathBuf {
        Path::new(&self.beatoraja_root)
            .join("player")
            .join(&self.player_name)
            .join("scorelog.db")
    }

    /// Returns all derived DB paths.
    pub fn all_db_paths(&self) -> Vec<PathBuf> {
        vec![
            self.songdata_db_path(),
            self.scoredatalog_db_path(),
            self.score_db_path(),
            self.scorelog_db_path(),
        ]
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    ReadFile(#[source] std::io::Error),
    #[error("failed to write config file: {0}")]
    WriteFile(#[source] std::io::Error),
    #[error("failed to create config directory: {0}")]
    CreateDir(#[source] std::io::Error),
    #[error("failed to parse config: {0}")]
    Parse(#[source] serde_json::Error),
    #[error("failed to serialize config: {0}")]
    Serialize(#[source] serde_json::Error),
    #[error("DB file not found: {path}")]
    DbFileNotFound { path: String },
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// Loads config from disk. Returns None if the config file does not exist yet.
    pub fn load(&self) -> Result<Option<AppConfig>, ConfigError> {
        if !self.config_path.exists() {
            return Ok(None);
        }
        let contents = fs::read_to_string(&self.config_path).map_err(ConfigError::ReadFile)?;
        serde_json::from_str(&contents)
            .map(Some)
            .map_err(ConfigError::Parse)
    }

    /// Saves config to disk.
    pub fn save(&self, config: &AppConfig) -> Result<(), ConfigError> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(ConfigError::CreateDir)?;
        }
        let contents = serde_json::to_string_pretty(config).map_err(ConfigError::Serialize)?;
        fs::write(&self.config_path, contents).map_err(ConfigError::WriteFile)
    }

    /// Validates that all required DB files exist for the given beatoraja_root,
    /// then saves the config.
    pub fn validate_and_save(&self, beatoraja_root: &str) -> Result<(), ConfigError> {
        let mut config = self.load()?.unwrap_or_default();
        config.beatoraja_root = beatoraja_root.to_string();

        for path in config.all_db_paths() {
            if !path.exists() {
                return Err(ConfigError::DbFileNotFound {
                    path: path.to_string_lossy().to_string(),
                });
            }
        }

        self.save(&config)
    }

    /// Updates display/data settings (only provided fields are updated).
    pub fn update_settings(
        &self,
        reset_time: Option<&str>,
        background_transparent: Option<bool>,
        font_size: Option<i32>,
    ) -> Result<(), ConfigError> {
        let mut config = self.load()?.unwrap_or_default();

        if let Some(rt) = reset_time {
            config.reset_time = rt.to_string();
        }
        if let Some(bt) = background_transparent {
            config.background_transparent = bt;
        }
        if let Some(fs) = font_size {
            config.font_size = fs;
        }

        self.save(&config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use rstest::rstest;
    use std::fs;
    use tempfile::TempDir;

    struct TestContext {
        _dir: TempDir,
        manager: ConfigManager,
    }

    impl TestContext {
        fn dir_path(&self) -> &Path {
            self._dir.path()
        }
    }

    #[rstest::fixture]
    fn ctx() -> TestContext {
        let dir = tempfile::tempdir().unwrap();
        let manager = ConfigManager::new(dir.path().join("config.json"));
        TestContext { _dir: dir, manager }
    }

    /// Creates a fake beatoraja directory structure with the required DB files.
    /// The player-scoped DB files are placed under `player/<player_name>/`.
    fn create_fake_beatoraja_dir(base: &Path, player_name: &str) -> PathBuf {
        let root = base.join("beatoraja");
        fs::create_dir_all(root.join("player").join(player_name)).unwrap();
        fs::write(root.join("songdata.db"), "").unwrap();
        for file in &["scoredatalog.db", "score.db", "scorelog.db"] {
            fs::write(root.join("player").join(player_name).join(file), "").unwrap();
        }
        root
    }

    #[rstest]
    fn test_load_returns_none_when_file_missing(ctx: TestContext) {
        assert!(ctx.manager.load().unwrap().is_none());
    }

    #[rstest]
    fn test_save_and_load_roundtrip(ctx: TestContext) {
        let config = AppConfig {
            beatoraja_root: "/path/to/beatoraja".to_string(),
            player_name: "player1".to_string(),
            reset_time: "06:00".to_string(),
            background_transparent: true,
            font_size: 16,
        };
        ctx.manager.save(&config).unwrap();
        let loaded = ctx.manager.load().unwrap().unwrap();
        assert_eq!(loaded, config);
    }

    #[rstest]
    fn test_load_parses_json(ctx: TestContext) {
        let json = indoc! {r#"
            {
              "beatorajaRoot": "C:\\beatoraja",
              "playerName": "default",
              "resetTime": "05:00",
              "backgroundTransparent": false,
              "fontSize": 13
            }
        "#};
        fs::write(ctx.dir_path().join("config.json"), json).unwrap();
        let config = ctx.manager.load().unwrap().unwrap();
        assert_eq!(config.beatoraja_root, "C:\\beatoraja");
        assert_eq!(config.player_name, "default");
    }

    #[rstest]
    fn test_validate_and_save_succeeds_with_valid_paths(ctx: TestContext) {
        let beatoraja_root = create_fake_beatoraja_dir(ctx.dir_path(), "");
        let result = ctx
            .manager
            .validate_and_save(beatoraja_root.to_str().unwrap());
        assert!(result.is_ok());

        let config = ctx.manager.load().unwrap().unwrap();
        assert_eq!(config.beatoraja_root, beatoraja_root.to_str().unwrap());
    }

    #[rstest]
    fn test_validate_and_save_fails_with_missing_db(ctx: TestContext) {
        let result = ctx.manager.validate_and_save("/nonexistent/path");
        assert!(result.is_err());
        match result {
            Err(ConfigError::DbFileNotFound { path }) => {
                assert_eq!(path, "/nonexistent/path/songdata.db");
            }
            _ => panic!("expected DbFileNotFound error"),
        }
    }

    #[rstest]
    fn test_validate_and_save_preserves_existing_settings(ctx: TestContext) {
        let initial = AppConfig {
            beatoraja_root: String::new(),
            player_name: String::new(),
            reset_time: "07:00".to_string(),
            background_transparent: true,
            font_size: 20,
        };
        ctx.manager.save(&initial).unwrap();

        let beatoraja_root = create_fake_beatoraja_dir(ctx.dir_path(), "");
        ctx.manager
            .validate_and_save(beatoraja_root.to_str().unwrap())
            .unwrap();

        let config = ctx.manager.load().unwrap().unwrap();
        assert_eq!(config.reset_time, "07:00");
        assert!(config.background_transparent);
        assert_eq!(config.font_size, 20);
    }

    #[rstest]
    fn test_update_settings_partial(ctx: TestContext) {
        ctx.manager.save(&AppConfig::default()).unwrap();

        ctx.manager
            .update_settings(Some("06:30"), None, Some(18))
            .unwrap();

        let config = ctx.manager.load().unwrap().unwrap();
        assert_eq!(config.reset_time, "06:30");
        assert!(!config.background_transparent); // unchanged
        assert_eq!(config.font_size, 18);
    }

    mod path_derivation {
        use super::*;

        #[rstest]
        #[case::songdata("C:\\beatoraja", "default", "C:\\beatoraja/songdata.db")]
        #[case::unix_path("/home/user/beatoraja", "player1", "/home/user/beatoraja/songdata.db")]
        fn test_songdata_db_path(#[case] root: &str, #[case] player: &str, #[case] expected: &str) {
            let config = AppConfig {
                beatoraja_root: root.to_string(),
                player_name: player.to_string(),
                ..Default::default()
            };
            assert_eq!(config.songdata_db_path(), PathBuf::from(expected));
        }

        #[rstest]
        #[case::scoredatalog_default(
            "/beatoraja",
            "default",
            "/beatoraja/player/default/scoredatalog.db"
        )]
        #[case::scoredatalog_custom(
            "/beatoraja",
            "myplayer",
            "/beatoraja/player/myplayer/scoredatalog.db"
        )]
        #[case::score_db("/beatoraja", "default", "/beatoraja/player/default/score.db")]
        #[case::scorelog_db("/beatoraja", "default", "/beatoraja/player/default/scorelog.db")]
        fn test_player_scoped_db_path(
            #[case] root: &str,
            #[case] player: &str,
            #[case] expected: &str,
        ) {
            let config = AppConfig {
                beatoraja_root: root.to_string(),
                player_name: player.to_string(),
                ..Default::default()
            };
            let expected_path = PathBuf::from(expected);
            let file_name = expected_path.file_name().unwrap();
            let actual = if file_name == "scoredatalog.db" {
                config.scoredatalog_db_path()
            } else if file_name == "score.db" {
                config.score_db_path()
            } else {
                config.scorelog_db_path()
            };
            assert_eq!(actual, expected_path);
        }

        #[rstest]
        fn test_all_db_paths_returns_four_paths() {
            let config = AppConfig {
                beatoraja_root: "/beatoraja".to_string(),
                player_name: "default".to_string(),
                ..Default::default()
            };
            let paths = config.all_db_paths();
            assert_eq!(paths.len(), 4);
            assert_eq!(paths[0], PathBuf::from("/beatoraja/songdata.db"));
            assert_eq!(
                paths[1],
                PathBuf::from("/beatoraja/player/default/scoredatalog.db")
            );
            assert_eq!(
                paths[2],
                PathBuf::from("/beatoraja/player/default/score.db")
            );
            assert_eq!(
                paths[3],
                PathBuf::from("/beatoraja/player/default/scorelog.db")
            );
        }
    }
}
