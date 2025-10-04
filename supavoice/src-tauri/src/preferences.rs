use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppPreferences {
    pub active_whisper_model: Option<String>,
    pub active_llm_model: Option<String>,
    #[serde(default)]
    pub custom_vocabulary: Vec<String>,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            active_whisper_model: None, // None means use auto-selection
            active_llm_model: None,
            custom_vocabulary: Vec::new(),
        }
    }
}

pub struct PreferencesManager {
    preferences: Arc<RwLock<AppPreferences>>,
    config_path: PathBuf,
}

impl PreferencesManager {
    pub fn new() -> Result<Self> {
        let project_dirs = directories::ProjectDirs::from("com", "supavoice", "Supavoice")
            .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))?;

        let config_dir = project_dirs.config_dir();
        std::fs::create_dir_all(config_dir)?;

        let config_path = config_dir.join("preferences.json");

        // Load existing preferences or create default
        let preferences = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AppPreferences::default()
        };

        Ok(Self {
            preferences: Arc::new(RwLock::new(preferences)),
            config_path,
        })
    }

    pub async fn get_preferences(&self) -> AppPreferences {
        self.preferences.read().await.clone()
    }

    pub async fn set_active_whisper_model(&self, model_id: Option<String>) -> Result<()> {
        let mut prefs = self.preferences.write().await;
        prefs.active_whisper_model = model_id;
        self.save(&prefs).await?;
        Ok(())
    }

    pub async fn set_active_llm_model(&self, model_id: Option<String>) -> Result<()> {
        let mut prefs = self.preferences.write().await;
        prefs.active_llm_model = model_id;
        self.save(&prefs).await?;
        Ok(())
    }

    pub async fn add_vocabulary_word(&self, word: String) -> Result<()> {
        let mut prefs = self.preferences.write().await;
        // Avoid duplicates
        if !prefs.custom_vocabulary.contains(&word) {
            prefs.custom_vocabulary.push(word);
            self.save(&prefs).await?;
        }
        Ok(())
    }

    pub async fn remove_vocabulary_word(&self, word: String) -> Result<()> {
        let mut prefs = self.preferences.write().await;
        prefs.custom_vocabulary.retain(|w| w != &word);
        self.save(&prefs).await?;
        Ok(())
    }

    pub async fn get_vocabulary(&self) -> Vec<String> {
        self.preferences.read().await.custom_vocabulary.clone()
    }

    async fn save(&self, prefs: &AppPreferences) -> Result<()> {
        let json = serde_json::to_string_pretty(prefs)?;
        tokio::fs::write(&self.config_path, json).await?;
        Ok(())
    }
}
