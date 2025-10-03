use anyhow::{Context, Result};
use byteorder::ByteOrder;
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::whisper::{self as m, Config};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokenizers::Tokenizer;

const SAMPLE_RATE: usize = 16000;
const N_FFT: usize = 400;
const HOP_LENGTH: usize = 160;
const N_MELS: usize = 80;

// Whisper special tokens
const SOT_TOKEN: u32 = 50258;  // Start of transcript
const EOT_TOKEN: u32 = 50257;  // End of transcript
const NO_TIMESTAMPS_TOKEN: u32 = 50363;
const TRANSCRIBE_TOKEN: u32 = 50359;  // Task: transcribe (vs translate)

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TranscriptionSegment {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TranscriptionResult {
    pub text: String,
    pub segments: Vec<TranscriptionSegment>,
}

pub struct WhisperTranscriber {
    model_path: PathBuf,
}

impl WhisperTranscriber {
    pub fn new(model_path: PathBuf) -> Self {
        Self { model_path }
    }

    pub fn transcribe(&self, audio_path: &PathBuf) -> Result<TranscriptionResult> {
        // Load audio samples
        let audio_data = self.load_audio(audio_path)?;

        // Initialize device (Metal on macOS, CUDA on NVIDIA, CPU fallback)
        let device = Self::get_device()?;

        // Load model configuration
        let config = self.load_config()?;

        // Load model weights
        let mut model = self.load_model(&config, &device)?;

        // Load tokenizer
        let tokenizer = self.load_tokenizer()?;

        // Convert audio to mel spectrogram
        let mel = self.audio_to_mel(&audio_data, &config, &device)?;

        // Run inference with full decoder
        let text = self.decode(&mut model, &mel, &config, &device, &tokenizer)?;

        // For MVP, return single segment with full text
        // TODO: Add proper segmentation with timestamps in future
        let duration = audio_data.len() as f64 / SAMPLE_RATE as f64;

        Ok(TranscriptionResult {
            text: text.clone(),
            segments: vec![TranscriptionSegment {
                start: 0.0,
                end: duration,
                text,
            }],
        })
    }

    fn get_device() -> Result<Device> {
        #[cfg(target_os = "macos")]
        {
            // Try Metal first on macOS
            if let Ok(device) = Device::new_metal(0) {
                println!("Using Metal GPU acceleration");
                return Ok(device);
            }
        }

        #[cfg(feature = "cuda")]
        {
            // Try CUDA if available
            if let Ok(device) = Device::new_cuda(0) {
                println!("Using CUDA GPU acceleration");
                return Ok(device);
            }
        }

        // Fallback to CPU
        println!("Using CPU");
        Ok(Device::Cpu)
    }

    fn load_config(&self) -> Result<Config> {
        // Load config.json if it exists, otherwise create default
        let config_path = self.model_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid model path"))?
            .join("config.json");

        if config_path.exists() {
            let config_str = std::fs::read_to_string(&config_path)
                .context("Failed to read config.json")?;
            serde_json::from_str(&config_str)
                .context("Failed to parse config.json")
        } else {
            // Create default config for small model (assumes English)
            Ok(Config {
                num_mel_bins: 80,
                max_source_positions: 1500,
                max_target_positions: 448,
                d_model: 768,
                encoder_attention_heads: 12,
                encoder_layers: 12,
                decoder_attention_heads: 12,
                decoder_layers: 12,
                vocab_size: 51865,
                suppress_tokens: vec![],
            })
        }
    }

    fn load_tokenizer(&self) -> Result<Tokenizer> {
        let tokenizer_path = self.model_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid model path"))?
            .join("tokenizer.json");

        Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))
    }

    fn load_model(&self, config: &Config, device: &Device) -> Result<m::model::Whisper> {
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[&self.model_path], m::DTYPE, device)?
        };

        m::model::Whisper::load(&vb, config.clone())
            .context("Failed to load Whisper model")
    }

    fn audio_to_mel(&self, audio: &[f32], config: &Config, device: &Device) -> Result<Tensor> {
        // Whisper expects exactly 30 seconds of audio (480,000 samples at 16kHz)
        const MAX_SAMPLES: usize = 480000; // 30 seconds * 16000 Hz

        // Pad or trim audio to exactly 30 seconds
        let mut padded_audio = audio.to_vec();
        if padded_audio.len() < MAX_SAMPLES {
            // Pad with zeros
            padded_audio.resize(MAX_SAMPLES, 0.0);
        } else if padded_audio.len() > MAX_SAMPLES {
            // Trim to 30 seconds
            padded_audio.truncate(MAX_SAMPLES);
        }

        // Load mel filterbank based on config
        let mel_bytes = match config.num_mel_bins {
            80 => include_bytes!("melfilters.bytes").as_slice(),
            128 => include_bytes!("melfilters128.bytes").as_slice(),
            n => anyhow::bail!("Unexpected num_mel_bins: {}", n),
        };

        let mut mel_filters = vec![0f32; mel_bytes.len() / 4];
        byteorder::LittleEndian::read_f32_into(mel_bytes, &mut mel_filters);

        // Convert PCM to mel spectrogram
        let mel = m::audio::pcm_to_mel(config, &padded_audio, &mel_filters);
        let mel_len = mel.len();
        let mel = Tensor::from_vec(
            mel,
            (1, config.num_mel_bins, mel_len / config.num_mel_bins),
            device,
        )?;

        Ok(mel)
    }

    fn decode(
        &self,
        model: &mut m::model::Whisper,
        mel: &Tensor,
        config: &Config,
        device: &Device,
        tokenizer: &Tokenizer,
    ) -> Result<String> {
        // Run encoder to get audio features
        let audio_features = model.encoder.forward(mel, true)?;

        println!("Audio features shape: {:?}", audio_features.shape());

        // Initialize token sequence with special tokens
        // Format: [SOT, language (English), task (transcribe), no_timestamps, ...]
        let mut tokens = vec![
            SOT_TOKEN,
            50259,              // English language token
            TRANSCRIBE_TOKEN,   // Transcribe task
            NO_TIMESTAMPS_TOKEN, // No timestamp tokens
        ];

        // Maximum sequence length
        let sample_len = config.max_target_positions / 2;

        // Autoregressive decoding loop
        for i in 0..sample_len {
            // Convert tokens to tensor
            let tokens_t = Tensor::new(tokens.as_slice(), device)?;
            let tokens_t = tokens_t.unsqueeze(0)?;

            // Run decoder
            let ys = model.decoder.forward(&tokens_t, &audio_features, i == 0)?;

            // Get logits for the last token position
            let seq_len = tokens.len();
            let logits = model
                .decoder
                .final_linear(&ys.narrow(1, seq_len - 1, 1)?)?
                .squeeze(0)?
                .squeeze(0)?;

            // Greedy decoding: select token with highest probability
            let logits_v: Vec<f32> = logits.to_vec1()?;
            let next_token = logits_v
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(idx, _)| idx as u32)
                .unwrap();

            // Stop if we hit end-of-transcript token
            if next_token == EOT_TOKEN {
                break;
            }

            tokens.push(next_token);
        }

        // Decode tokens to text
        let text = tokenizer
            .decode(&tokens, true)
            .map_err(|e| anyhow::anyhow!("Failed to decode tokens: {}", e))?;

        Ok(text)
    }

    fn load_audio(&self, audio_path: &PathBuf) -> Result<Vec<f32>> {
        // Read WAV file
        let mut reader = hound::WavReader::open(audio_path)
            .context("Failed to open audio file")?;
        let spec = reader.spec();

        // Ensure 16kHz sample rate
        if spec.sample_rate != SAMPLE_RATE as u32 {
            anyhow::bail!(
                "Audio must be 16kHz (got {}Hz). Please resample.",
                spec.sample_rate
            );
        }

        // Convert to mono f32 samples normalized to [-1.0, 1.0]
        let mut samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Int => reader
                .samples::<i16>()
                .map(|s| s.unwrap() as f32 / i16::MAX as f32)
                .collect(),
            hound::SampleFormat::Float => {
                reader.samples::<f32>().map(|s| s.unwrap()).collect()
            }
        };

        // Convert stereo to mono if needed
        if spec.channels == 2 {
            samples = samples
                .chunks(2)
                .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
                .collect();
        }

        Ok(samples)
    }
}
