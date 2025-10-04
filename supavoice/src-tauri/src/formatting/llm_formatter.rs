use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

pub struct LlmFormatter {
    llama_server_path: PathBuf,
    server_process: Arc<Mutex<Option<Child>>>,
    server_port: u16,
}

impl LlmFormatter {
    pub fn new() -> Result<Self> {
        // Try multiple locations for llama-server binary
        let exe_dir = std::env::current_exe()?
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Failed to get parent directory"))?
            .to_path_buf();

        // Possible locations (dev vs production)
        let possible_paths = vec![
            // Production: macOS app bundle
            exe_dir.join("../Resources/llama-server"),
            // Dev: src-tauri/resources
            exe_dir.join("../../resources/llama-server"),
            // Dev: alternative
            exe_dir.join("../../../src-tauri/resources/llama-server"),
        ];

        let llama_server_path = possible_paths
            .iter()
            .find(|path| path.exists())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "llama-server binary not found. Tried:\n{}",
                    possible_paths
                        .iter()
                        .map(|p| format!("  - {:?}", p))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })?
            .clone();

        println!("✅ Found llama-server at: {:?}", llama_server_path);

        Ok(Self {
            llama_server_path,
            server_process: Arc::new(Mutex::new(None)),
            server_port: 8765, // Use a fixed port for local server
        })
    }

    pub fn start_server_if_needed(&self, model_path: &PathBuf) -> Result<()> {
        let mut process_guard = self.server_process.lock().unwrap();

        // Check if server is already running
        if process_guard.is_some() {
            println!("⚡ Server already running");
            return Ok(());
        }

        println!("🚀 Starting llama-server with model: {:?}", model_path);

        // Start llama-server with the model loaded
        let child = Command::new(&self.llama_server_path)
            .arg("-m")
            .arg(model_path)
            .arg("--port")
            .arg(self.server_port.to_string())
            .arg("-ngl")
            .arg("99") // GPU layers
            .arg("-c")
            .arg("2048") // context size
            .arg("--log-disable") // Disable logging for cleaner output
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start llama-server")?;

        *process_guard = Some(child);

        // Give server time to start
        std::thread::sleep(std::time::Duration::from_millis(500));

        println!("✅ Server started on port {}", self.server_port);

        Ok(())
    }

    pub async fn format_as_email(&self, model_path: &PathBuf, transcript: &str) -> Result<String> {
        let prompt = format!(
            "<|im_start|>system\nYou are a helpful assistant that rewrites voice transcripts as professional emails.<|im_end|>\n\
            <|im_start|>user\nRewrite the following voice transcript as a professional email. \
            Make it clear, concise, and well-structured with proper greeting and closing.\n\n\
            Transcript: {}<|im_end|>\n\
            <|im_start|>assistant\n",
            transcript
        );

        self.generate(model_path, &prompt).await
    }

    pub async fn format_as_notes(&self, model_path: &PathBuf, transcript: &str) -> Result<String> {
        let prompt = format!(
            "<|im_start|>system\nYou are a helpful assistant that converts voice transcripts into organized notes.<|im_end|>\n\
            <|im_start|>user\nConvert the following voice transcript into clear, organized notes. \
            Use bullet points and organize by topic where appropriate.\n\n\
            Transcript: {}<|im_end|>\n\
            <|im_start|>assistant\n",
            transcript
        );

        self.generate(model_path, &prompt).await
    }

    async fn generate(&self, model_path: &PathBuf, prompt: &str) -> Result<String> {
        // Start server if not running (only happens once)
        self.start_server_if_needed(model_path)?;

        println!("🔄 Sending completion request to llama-server...");

        // Make HTTP request to llama-server (async)
        let client = reqwest::Client::new();
        let response = client
            .post(format!("http://localhost:{}/completion", self.server_port))
            .json(&serde_json::json!({
                "prompt": prompt,
                "n_predict": 512,
                "temperature": 0.7,
                "stop": ["<|im_end|>", "</s>"],
                "cache_prompt": true, // Cache the prompt for faster subsequent requests
            }))
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .context("Failed to send request to llama-server")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Server returned error: {}",
                response.status()
            ));
        }

        let json: serde_json::Value = response.json().await?;
        let content = json["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No content in response"))?;

        println!("✅ Generated {} characters", content.len());

        Ok(content.trim().to_string())
    }
}
