use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[cfg(test)]
thread_local! {
    pub static TEST_CONFIG_PATH: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum TuiTheme {
    #[default]
    Cyan,
    Purple,
    Emerald,
    Amber,
    Monochrome,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct AgentQuotaSettings {
    pub limit: u32,
    #[serde(default)]
    pub custom: bool,
}

impl Default for AgentQuotaSettings {
    fn default() -> Self {
        Self {
            limit: 100,
            custom: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub refresh_rate_ms: u64,
    pub soft_limit_percent: f64,
    pub hard_limit_percent: f64,
    pub theme: TuiTheme,
    pub codex_quota: AgentQuotaSettings,
    pub opencode_quota: AgentQuotaSettings,
    pub agy_quota: AgentQuotaSettings,
    pub zed_quota: AgentQuotaSettings,
    #[serde(default)]
    pub aider_quota: AgentQuotaSettings,
    #[serde(default)]
    pub ollama_quota: AgentQuotaSettings,
    #[serde(default)]
    pub continue_quota: AgentQuotaSettings,
    #[serde(default)]
    pub cody_quota: AgentQuotaSettings,
    #[serde(default)]
    pub supermaven_quota: AgentQuotaSettings,
    #[serde(default)]
    pub model_limits: HashMap<String, u32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut model_limits = HashMap::new();
        model_limits.insert("gpt-5".to_string(), 50);
        model_limits.insert("gpt-4.1".to_string(), 100);
        model_limits.insert("gpt-4o".to_string(), 50);
        model_limits.insert("gpt-4o-mini".to_string(), 200);
        model_limits.insert("o3".to_string(), 30);
        model_limits.insert("o4-mini".to_string(), 100);
        model_limits.insert("deepseek-chat".to_string(), 150);
        model_limits.insert("deepseek-reasoner".to_string(), 50);
        model_limits.insert("gpt-oss".to_string(), 100);
        model_limits.insert("gemini-3.5-flash".to_string(), 1500);
        model_limits.insert("gemini-3.1-pro".to_string(), 50);
        model_limits.insert("Gemini 3.5 Flash".to_string(), 1500);
        model_limits.insert("Gemini 3.1 Pro".to_string(), 50);
        model_limits.insert("claude-4.7".to_string(), 150);

        Self {
            refresh_rate_ms: 2000,
            soft_limit_percent: 80.0,
            hard_limit_percent: 100.0,
            theme: TuiTheme::Cyan,
            codex_quota: AgentQuotaSettings {
                limit: 200,
                custom: false,
            },
            opencode_quota: AgentQuotaSettings {
                limit: 1000,
                custom: false,
            },
            agy_quota: AgentQuotaSettings {
                limit: 500,
                custom: false,
            },
            zed_quota: AgentQuotaSettings {
                limit: 300,
                custom: false,
            },
            aider_quota: AgentQuotaSettings {
                limit: 200,
                custom: false,
            },
            ollama_quota: AgentQuotaSettings {
                limit: 1000,
                custom: false,
            },
            continue_quota: AgentQuotaSettings {
                limit: 500,
                custom: false,
            },
            cody_quota: AgentQuotaSettings {
                limit: 400,
                custom: false,
            },
            supermaven_quota: AgentQuotaSettings {
                limit: 2000,
                custom: false,
            },
            model_limits,
        }
    }
}

impl AppConfig {
    #[cfg(not(test))]
    pub fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "quotachecker-tui", "dashboard")
            .map(|proj_dirs| proj_dirs.config_dir().join("config.json"))
    }

    #[cfg(test)]
    pub fn config_path() -> Option<PathBuf> {
        TEST_CONFIG_PATH.with(|p| p.borrow().clone()).or_else(|| {
            ProjectDirs::from("com", "quotachecker-tui", "dashboard")
                .map(|proj_dirs| proj_dirs.config_dir().join("config.json"))
        })
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(mut config) = serde_json::from_str::<AppConfig>(&content) {
                        // Ensure newly added fields are initialized if they were absent in legacy JSON
                        if config.model_limits.is_empty() {
                            config.model_limits = Self::default().model_limits;
                            let _ = config.save();
                        }
                        return config;
                    }
                }
            }
        }

        let default_config = Self::default();
        let _ = default_config.save();
        default_config
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(path, content)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn set_test_config_path(path: Option<PathBuf>) {
        TEST_CONFIG_PATH.with(|p| *p.borrow_mut() = path);
    }

    #[test]
    fn test_load_non_existent_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        set_test_config_path(Some(config_path.clone()));

        assert!(!config_path.exists());
        let config = AppConfig::load();

        assert!(config_path.exists());
        assert_eq!(config.refresh_rate_ms, 2000);
        assert_eq!(config.theme, TuiTheme::Cyan);
    }

    #[test]
    fn test_load_existing_valid_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        set_test_config_path(Some(config_path.clone()));

        let config = AppConfig {
            refresh_rate_ms: 5000,
            theme: TuiTheme::Amber,
            ..Default::default()
        };

        let content = serde_json::to_string(&config).unwrap();
        fs::write(&config_path, content).unwrap();

        let loaded_config = AppConfig::load();
        assert_eq!(loaded_config.refresh_rate_ms, 5000);
        assert_eq!(loaded_config.theme, TuiTheme::Amber);
    }

    #[test]
    fn test_load_legacy_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        set_test_config_path(Some(config_path.clone()));

        let legacy_json = r#"{
            "refresh_rate_ms": 3000,
            "soft_limit_percent": 75.0,
            "hard_limit_percent": 90.0,
            "theme": "Purple",
            "codex_quota": { "limit": 100, "custom": false },
            "opencode_quota": { "limit": 100, "custom": false },
            "agy_quota": { "limit": 100, "custom": false },
            "zed_quota": { "limit": 100, "custom": false }
        }"#;

        fs::write(&config_path, legacy_json).unwrap();

        let loaded_config = AppConfig::load();

        assert_eq!(loaded_config.refresh_rate_ms, 3000);
        assert!(!loaded_config.model_limits.is_empty());
        assert_eq!(loaded_config.model_limits.get("gpt-5"), Some(&50));
    }

    #[test]
    fn test_load_invalid_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        set_test_config_path(Some(config_path.clone()));

        fs::write(&config_path, "invalid json format").unwrap();

        let loaded_config = AppConfig::load();

        assert_eq!(loaded_config.refresh_rate_ms, 2000);

        let new_content = fs::read_to_string(&config_path).unwrap();
        assert!(serde_json::from_str::<AppConfig>(&new_content).is_ok());
    }

    #[test]
    fn test_save_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        set_test_config_path(Some(config_path.clone()));

        let config = AppConfig {
            refresh_rate_ms: 9999,
            theme: TuiTheme::Purple,
            ..Default::default()
        };

        assert!(config.save().is_ok());
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        let loaded: AppConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded.refresh_rate_ms, 9999);
        assert_eq!(loaded.theme, TuiTheme::Purple);
    }
}
