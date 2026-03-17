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

impl From<ConfigError> for String {
    fn from(e: ConfigError) -> Self {
        e.to_string()
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// Loads config from disk. Returns default config if file doesn't exist.
    pub fn load(&self) -> Result<AppConfig, ConfigError> {
        if !self.config_path.exists() {
            return Ok(AppConfig::default());
        }
        let contents = fs::read_to_string(&self.config_path).map_err(ConfigError::ReadFile)?;
        serde_json::from_str(&contents).map_err(ConfigError::Parse)
    }

    /// Saves config to disk.
    pub fn save(&self, config: &AppConfig) -> Result<(), ConfigError> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(ConfigError::CreateDir)?;
        }
        let contents = serde_json::to_string_pretty(config).map_err(ConfigError::Serialize)?;
        fs::write(&self.config_path, contents).map_err(ConfigError::WriteFile)
    }

    /// Validates that all required DB files exist for the given beatoraja_root and player_name,
    /// then saves the config.
    pub fn validate_and_save(
        &self,
        beatoraja_root: &str,
        player_name: &str,
    ) -> Result<(), ConfigError> {
        let mut config = self.load()?;
        config.beatoraja_root = beatoraja_root.to_string();
        config.player_name = player_name.to_string();

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
        let mut config = self.load()?;

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

    #[rstest::fixture]
    fn config_dir() -> TempDir {
        tempfile::tempdir().unwrap_or_else(|_| {
            std::process::exit(1);
        })
    }

    fn make_manager(dir: &TempDir) -> ConfigManager {
        ConfigManager::new(dir.path().join("config.json"))
    }

    /// Creates a fake beatoraja directory structure with the required DB files.
    fn create_fake_beatoraja_dir(dir: &TempDir) -> PathBuf {
        let root = dir.path().join("beatoraja");
        fs::create_dir_all(root.join("player").join("default"))
            .unwrap_or_else(|_| std::process::exit(1));
        for file in &[
            "songdata.db",
            "player/default/scoredatalog.db",
            "player/default/score.db",
            "player/default/scorelog.db",
        ] {
            fs::write(root.join(file), "").unwrap_or_else(|_| std::process::exit(1));
        }
        root
    }

    #[rstest]
    fn test_load_returns_default_when_file_missing(config_dir: TempDir) {
        let manager = make_manager(&config_dir);
        let config = manager.load().unwrap_or_else(|_| std::process::exit(1));
        assert_eq!(config, AppConfig::default());
    }

    #[rstest]
    fn test_save_and_load_roundtrip(config_dir: TempDir) {
        let manager = make_manager(&config_dir);
        let config = AppConfig {
            beatoraja_root: "/path/to/beatoraja".to_string(),
            player_name: "player1".to_string(),
            reset_time: "06:00".to_string(),
            background_transparent: true,
            font_size: 16,
        };
        manager
            .save(&config)
            .unwrap_or_else(|_| std::process::exit(1));
        let loaded = manager.load().unwrap_or_else(|_| std::process::exit(1));
        assert_eq!(loaded, config);
    }

    #[rstest]
    fn test_load_parses_json(config_dir: TempDir) {
        let manager = make_manager(&config_dir);
        let json = indoc! {r#"
            {
              "beatorajaRoot": "C:\\beatoraja",
              "playerName": "default",
              "resetTime": "05:00",
              "backgroundTransparent": false,
              "fontSize": 13
            }
        "#};
        fs::write(config_dir.path().join("config.json"), json)
            .unwrap_or_else(|_| std::process::exit(1));
        let config = manager.load().unwrap_or_else(|_| std::process::exit(1));
        assert_eq!(config.beatoraja_root, "C:\\beatoraja");
        assert_eq!(config.player_name, "default");
    }

    #[rstest]
    fn test_validate_and_save_succeeds_with_valid_paths(config_dir: TempDir) {
        let manager = make_manager(&config_dir);
        let beatoraja_root = create_fake_beatoraja_dir(&config_dir);
        let result = manager.validate_and_save(
            beatoraja_root
                .to_str()
                .unwrap_or_else(|| std::process::exit(1)),
            "default",
        );
        assert!(result.is_ok());

        let config = manager.load().unwrap_or_else(|_| std::process::exit(1));
        assert_eq!(
            config.beatoraja_root,
            beatoraja_root
                .to_str()
                .unwrap_or_else(|| std::process::exit(1))
        );
        assert_eq!(config.player_name, "default");
    }

    #[rstest]
    fn test_validate_and_save_fails_with_missing_db(config_dir: TempDir) {
        let manager = make_manager(&config_dir);
        let result = manager.validate_and_save("/nonexistent/path", "player1");
        assert!(result.is_err());
        match result {
            Err(ConfigError::DbFileNotFound { path }) => {
                assert_eq!(path, "/nonexistent/path/songdata.db");
            }
            _ => std::process::exit(1),
        }
    }

    #[rstest]
    fn test_validate_and_save_preserves_existing_settings(config_dir: TempDir) {
        let manager = make_manager(&config_dir);
        let initial = AppConfig {
            beatoraja_root: String::new(),
            player_name: String::new(),
            reset_time: "07:00".to_string(),
            background_transparent: true,
            font_size: 20,
        };
        manager
            .save(&initial)
            .unwrap_or_else(|_| std::process::exit(1));

        let beatoraja_root = create_fake_beatoraja_dir(&config_dir);
        manager
            .validate_and_save(
                beatoraja_root
                    .to_str()
                    .unwrap_or_else(|| std::process::exit(1)),
                "default",
            )
            .unwrap_or_else(|_| std::process::exit(1));

        let config = manager.load().unwrap_or_else(|_| std::process::exit(1));
        assert_eq!(config.reset_time, "07:00");
        assert!(config.background_transparent);
        assert_eq!(config.font_size, 20);
    }

    #[rstest]
    fn test_update_settings_partial(config_dir: TempDir) {
        let manager = make_manager(&config_dir);
        manager
            .save(&AppConfig::default())
            .unwrap_or_else(|_| std::process::exit(1));

        manager
            .update_settings(Some("06:30"), None, Some(18))
            .unwrap_or_else(|_| std::process::exit(1));

        let config = manager.load().unwrap_or_else(|_| std::process::exit(1));
        assert_eq!(config.reset_time, "06:30");
        assert!(!config.background_transparent); // unchanged
        assert_eq!(config.font_size, 18);
    }

    mod path_derivation {
        use super::*;

        #[rstest]
        #[case::songdata("C:\\beatoraja", "default", "C:\\beatoraja/songdata.db")]
        #[case::unix_path("/home/user/beatoraja", "player1", "/home/user/beatoraja/songdata.db")]
        fn test_songdata_db_path(
            #[case] root: &str,
            #[case] _player: &str,
            #[case] expected: &str,
        ) {
            let config = AppConfig {
                beatoraja_root: root.to_string(),
                player_name: _player.to_string(),
                ..Default::default()
            };
            assert_eq!(config.songdata_db_path(), PathBuf::from(expected));
        }

        #[rstest]
        #[case::default_player(
            "/beatoraja",
            "default",
            "/beatoraja/player/default/scoredatalog.db"
        )]
        #[case::custom_player(
            "/beatoraja",
            "myplayer",
            "/beatoraja/player/myplayer/scoredatalog.db"
        )]
        fn test_scoredatalog_db_path(
            #[case] root: &str,
            #[case] player: &str,
            #[case] expected: &str,
        ) {
            let config = AppConfig {
                beatoraja_root: root.to_string(),
                player_name: player.to_string(),
                ..Default::default()
            };
            assert_eq!(config.scoredatalog_db_path(), PathBuf::from(expected));
        }

        #[rstest]
        #[case::score_db("/beatoraja", "default", "/beatoraja/player/default/score.db")]
        #[case::scorelog_db("/beatoraja", "default", "/beatoraja/player/default/scorelog.db")]
        fn test_score_and_scorelog_db_paths(
            #[case] root: &str,
            #[case] player: &str,
            #[case] _expected: &str,
        ) {
            let config = AppConfig {
                beatoraja_root: root.to_string(),
                player_name: player.to_string(),
                ..Default::default()
            };
            // Verify all player-scoped DB paths are under the correct directory
            let player_dir = PathBuf::from(root).join("player").join(player);
            assert!(config.score_db_path().starts_with(&player_dir));
            assert!(config.scorelog_db_path().starts_with(&player_dir));
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
