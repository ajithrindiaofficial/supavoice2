use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ModelKind {
    Whisper,
    LLM,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ModelStatus {
    NotInstalled,
    Downloading { progress: f32, bytes: u64, total: u64 },
    Installed,
    Failed { error: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelRecord {
    pub id: String,
    pub name: String,
    pub kind: ModelKind,
    pub size_mb: u32,
    pub download_url: String,
    pub checksum: String, // SHA-256
    pub status: ModelStatus,
    pub path: Option<PathBuf>,
}
