use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct LlmFormatter {
    llama_binary_path: PathBuf,
}

impl LlmFormatter {
    pub fn new() -> Result<Self> {
        // Get the path to llama-cli binary in resources
        let resources_dir = std::env::current_exe()?
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Failed to get parent directory"))?
            .join("../Resources");

        let llama_binary_path = resources_dir.join("llama-cli");

        if !llama_binary_path.exists() {
            return Err(anyhow::anyhow!(
                "llama-cli binary not found at {:?}",
                llama_binary_path
            ));
        }

        println!("✅ Found llama-cli at: {:?}", llama_binary_path);

        Ok(Self { llama_binary_path })
    }

    pub fn format_as_email(&self, model_path: &PathBuf, transcript: &str) -> Result<String> {
        let prompt = format!(
            "<|im_start|>system\nYou are a helpful assistant that rewrites voice transcripts as professional emails.<|im_end|>\n\
            <|im_start|>user\nRewrite the following voice transcript as a professional email. \
            Make it clear, concise, and well-structured with proper greeting and closing.\n\n\
            Transcript: {}<|im_end|>\n\
            <|im_start|>assistant\n",
            transcript
        );

        self.generate(model_path, &prompt)
    }

    pub fn format_as_notes(&self, model_path: &PathBuf, transcript: &str) -> Result<String> {
        let prompt = format!(
            "<|im_start|>system\nYou are a helpful assistant that converts voice transcripts into organized notes.<|im_end|>\n\
            <|im_start|>user\nConvert the following voice transcript into clear, organized notes. \
            Use bullet points and organize by topic where appropriate.\n\n\
            Transcript: {}<|im_end|>\n\
            <|im_start|>assistant\n",
            transcript
        );

        self.generate(model_path, &prompt)
    }

    fn generate(&self, model_path: &PathBuf, prompt: &str) -> Result<String> {
        println!("🔄 Running llama.cpp with model: {:?}", model_path);

        // Run llama-cli as subprocess
        let output = Command::new(&self.llama_binary_path)
            .arg("-m")
            .arg(model_path)
            .arg("-p")
            .arg(prompt)
            .arg("-n")
            .arg("512") // max tokens to generate
            .arg("--temp")
            .arg("0.7")
            .arg("-ngl")
            .arg("99") // GPU layers (use all for Metal)
            .arg("--no-display-prompt") // Don't echo the prompt
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("Failed to execute llama-cli")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("llama-cli failed: {}", stderr));
        }

        let result = String::from_utf8_lossy(&output.stdout);

        // Clean up the output - remove any system messages and extra whitespace
        let cleaned = result
            .trim()
            .lines()
            .filter(|line| !line.starts_with("llama") && !line.starts_with("ggml")) // Filter out llama.cpp debug messages
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();

        println!("✅ Generated {} characters", cleaned.len());

        Ok(cleaned)
    }
}
