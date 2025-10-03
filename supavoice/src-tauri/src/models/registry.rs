use super::types::{ModelKind, ModelRecord, ModelStatus};
use anyhow::Result;
use directories::ProjectDirs;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ModelRegistry {
    models: Arc<RwLock<HashMap<String, ModelRecord>>>,
    base_path: PathBuf,
}

impl ModelRegistry {
    pub fn new() -> Result<Self> {
        let project_dirs = ProjectDirs::from("com", "supavoice", "Supavoice")
            .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))?;

        let base_path = project_dirs.data_dir().join("models");
        std::fs::create_dir_all(&base_path)?;

        let mut models = HashMap::new();

        // Initialize hardcoded model catalog
        // Whisper models
        models.insert(
            "whisper-small-en".to_string(),
            ModelRecord {
                id: "whisper-small-en".to_string(),
                name: "Whisper Small (English)".to_string(),
                kind: ModelKind::Whisper,
                size_mb: 466,
                download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin".to_string(),
                checksum: "".to_string(), // TODO: Add actual checksums
                status: ModelStatus::NotInstalled,
                path: None,
            },
        );

        models.insert(
            "whisper-base-en".to_string(),
            ModelRecord {
                id: "whisper-base-en".to_string(),
                name: "Whisper Base (English)".to_string(),
                kind: ModelKind::Whisper,
                size_mb: 142,
                download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin".to_string(),
                checksum: "".to_string(),
                status: ModelStatus::NotInstalled,
                path: None,
            },
        );

        // LLM models
        models.insert(
            "gemma-2-2b-instruct".to_string(),
            ModelRecord {
                id: "gemma-2-2b-instruct".to_string(),
                name: "Gemma 2 2B Instruct".to_string(),
                kind: ModelKind::LLM,
                size_mb: 1710,
                download_url: "https://huggingface.co/bartowski/gemma-2-2b-it-GGUF/resolve/main/gemma-2-2b-it-Q4_K_M.gguf".to_string(),
                checksum: "".to_string(),
                status: ModelStatus::NotInstalled,
                path: None,
            },
        );

        models.insert(
            "qwen2-1.5b-instruct".to_string(),
            ModelRecord {
                id: "qwen2-1.5b-instruct".to_string(),
                name: "Qwen2 1.5B Instruct".to_string(),
                kind: ModelKind::LLM,
                size_mb: 986,
                download_url: "https://huggingface.co/Qwen/Qwen2-1.5B-Instruct-GGUF/resolve/main/qwen2-1_5b-instruct-q4_k_m.gguf".to_string(),
                checksum: "".to_string(),
                status: ModelStatus::NotInstalled,
                path: None,
            },
        );

        Ok(Self {
            models: Arc::new(RwLock::new(models)),
            base_path,
        })
    }

    pub async fn list_models(&self) -> Result<Vec<ModelRecord>> {
        let models = self.models.read().await;
        Ok(models.values().cloned().collect())
    }

    pub async fn get_model(&self, id: &str) -> Result<ModelRecord> {
        let models = self.models.read().await;
        models
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", id))
    }

    pub async fn update_model_status(&self, id: &str, status: ModelStatus) -> Result<()> {
        let mut models = self.models.write().await;
        if let Some(model) = models.get_mut(id) {
            model.status = status;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Model not found: {}", id))
        }
    }

    pub async fn update_model_path(&self, id: &str, path: PathBuf) -> Result<()> {
        let mut models = self.models.write().await;
        if let Some(model) = models.get_mut(id) {
            model.path = Some(path);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Model not found: {}", id))
        }
    }

    pub fn get_model_path(&self, id: &str) -> PathBuf {
        self.base_path.join(id)
    }

    pub fn get_base_path(&self) -> &PathBuf {
        &self.base_path
    }
}
