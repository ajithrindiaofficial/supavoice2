use anyhow::{Context, Result};
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use rayon::prelude::*;

pub struct WhisperTranscriber {
    ctx: WhisperContext,
}

impl WhisperTranscriber {
    pub fn new<P: AsRef<Path>>(model_path: P) -> Result<Self> {
        println!("Loading Whisper model from: {:?}", model_path.as_ref());

        let ctx = WhisperContext::new_with_params(
            model_path.as_ref().to_str().context("Invalid model path")?,
            WhisperContextParameters::default(),
        )
        .context("Failed to load Whisper model")?;

        Ok(Self { ctx })
    }

    pub fn transcribe(&self, audio_path: &str) -> Result<String> {
        // Load and convert audio
        let audio_data = self.load_audio(audio_path)?;

        // For short audio (<30s), use single-pass transcription
        let sample_rate = 16000;
        let duration_secs = audio_data.len() as f32 / sample_rate as f32;

        if duration_secs < 30.0 {
            return self.transcribe_single(&audio_data);
        }

        // For long audio, split into chunks and process in parallel
        self.transcribe_chunked(&audio_data)
    }

    fn transcribe_single(&self, audio_data: &[f32]) -> Result<String> {
        // Create transcription state
        let mut state = self.ctx.create_state()
            .context("Failed to create Whisper state")?;

        let params = self.create_params();

        // Run transcription
        state
            .full(params, audio_data)
            .context("Failed to run Whisper transcription")?;

        // Extract text from all segments
        let num_segments = state
            .full_n_segments()
            .context("Failed to get number of segments")?;

        let mut full_text = String::new();
        for i in 0..num_segments {
            let segment = state
                .full_get_segment_text(i)
                .context(format!("Failed to get segment {}", i))?;
            full_text.push_str(&segment);
            full_text.push(' ');
        }

        Ok(full_text.trim().to_string())
    }

    fn transcribe_chunked(&self, audio_data: &[f32]) -> Result<String> {
        // Split audio into 30-second chunks with 1s overlap for context
        let sample_rate = 16000;
        let chunk_size = 30 * sample_rate; // 30 seconds
        let overlap = sample_rate; // 1 second overlap

        let chunks: Vec<Vec<f32>> = audio_data
            .chunks(chunk_size - overlap)
            .enumerate()
            .map(|(i, chunk)| {
                if i > 0 && audio_data.len() > chunk_size {
                    // Add overlap from previous chunk
                    let start = (i * (chunk_size - overlap)).saturating_sub(overlap);
                    audio_data[start..std::cmp::min(start + chunk_size, audio_data.len())].to_vec()
                } else {
                    chunk.to_vec()
                }
            })
            .collect();

        println!("🔪 Split audio into {} chunks for parallel processing", chunks.len());

        // Process chunks in parallel (whisper_rs context is Send + Sync)
        let transcripts: Result<Vec<String>> = chunks
            .par_iter()
            .enumerate()
            .map(|(i, chunk)| {
                println!("🧵 Processing chunk {}/{}", i + 1, chunks.len());
                self.transcribe_single(chunk)
            })
            .collect();

        let transcripts = transcripts?;

        // Stitch transcripts together
        Ok(transcripts.join(" "))
    }

    fn create_params(&self) -> FullParams {
        // Setup transcription parameters - greedy decoding for speed
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Speed optimizations
        params.set_n_threads(2); // Lower per-chunk since we're running in parallel
        params.set_translate(false);
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_token_timestamps(false);
        params.set_max_len(0);
        params.set_suppress_blank(true);
        params.set_suppress_non_speech_tokens(true);

        params
    }

    fn load_audio(&self, audio_path: &str) -> Result<Vec<f32>> {
        let mut reader = hound::WavReader::open(audio_path)
            .context("Failed to open audio file")?;

        let spec = reader.spec();

        // Read samples and convert to f32
        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => {
                reader.samples::<f32>()
                    .collect::<Result<Vec<f32>, _>>()
                    .context("Failed to read float samples")?
            }
            hound::SampleFormat::Int => {
                let bits = spec.bits_per_sample;
                reader.samples::<i32>()
                    .map(|s| {
                        let sample = s.context("Failed to read sample")?;
                        let max_val = 2_i32.pow(bits as u32 - 1) as f32;
                        Ok(sample as f32 / max_val)
                    })
                    .collect::<Result<Vec<f32>, anyhow::Error>>()?
            }
        };

        // Convert to mono if stereo
        let mono_samples = if spec.channels == 2 {
            samples
                .chunks(2)
                .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
                .collect()
        } else {
            samples
        };

        Ok(mono_samples)
    }
}
