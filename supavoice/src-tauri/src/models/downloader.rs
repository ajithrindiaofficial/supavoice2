use super::registry::ModelRegistry;
use super::types::ModelStatus;
use anyhow::Result;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tauri::Emitter;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct ModelDownloader {
    client: Client,
    registry: std::sync::Arc<ModelRegistry>,
}

impl ModelDownloader {
    pub fn new(registry: std::sync::Arc<ModelRegistry>) -> Self {
        Self {
            client: Client::new(),
            registry,
        }
    }

    pub async fn download_model(
        &self,
        model_id: String,
        app_handle: tauri::AppHandle,
    ) -> Result<()> {
        use super::types::ModelKind;

        let model = self.registry.get_model(&model_id).await?;
        let download_url = model.download_url.clone();
        let model_path = self.registry.get_model_path(&model_id);

        // Ensure parent directory exists
        if let Some(parent) = model_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Download model file (GGML/GGUF format)
        self.download_file(&download_url, &model_path, &model_id, &app_handle).await?;

        // Update registry
        self.registry
            .update_model_status(&model_id, ModelStatus::Installed)
            .await?;
        self.registry.update_model_path(&model_id, model_path).await?;

        // Emit completion event
        app_handle.emit(
            "download_complete",
            serde_json::json!({
                "model_id": model_id,
            }),
        )?;

        Ok(())
    }

    async fn download_file(
        &self,
        url: &str,
        file_path: &PathBuf,
        model_id: &str,
        app_handle: &tauri::AppHandle,
    ) -> Result<()> {
        // Download to .part file first
        let part_path = file_path.with_extension("part");

        // Start download
        let response = self.client.get(url).send().await?;
        let total_size = response.content_length().unwrap_or(0);

        let mut file = File::create(&part_path).await?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            let progress = if total_size > 0 {
                (downloaded as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };

            // Update status in registry
            self.registry
                .update_model_status(
                    model_id,
                    ModelStatus::Downloading {
                        progress,
                        bytes: downloaded,
                        total: total_size,
                    },
                )
                .await?;

            // Emit progress event
            app_handle.emit(
                "download_progress",
                serde_json::json!({
                    "model_id": model_id,
                    "progress": progress,
                    "bytes": downloaded,
                    "total": total_size,
                }),
            )?;
        }

        file.flush().await?;
        drop(file);

        // Verify checksum if available (skipping for now as checksums are empty)
        // TODO: Implement checksum verification

        // Rename .part to final file
        tokio::fs::rename(&part_path, file_path).await?;

        Ok(())
    }

    pub async fn delete_model(&self, model_id: String) -> Result<()> {
        let model_path = self.registry.get_model_path(&model_id);

        if model_path.exists() {
            tokio::fs::remove_file(&model_path).await?;
        }

        self.registry
            .update_model_status(&model_id, ModelStatus::NotInstalled)
            .await?;
        self.registry
            .update_model_path(&model_id, PathBuf::new())
            .await?;

        Ok(())
    }

    async fn verify_checksum(&self, file_path: &PathBuf, expected: &str) -> Result<bool> {
        if expected.is_empty() {
            return Ok(true); // Skip verification if no checksum provided
        }

        let mut file = File::open(file_path).await?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0; 8192];

        use tokio::io::AsyncReadExt;

        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let hash = format!("{:x}", hasher.finalize());
        Ok(hash == expected)
    }
}
