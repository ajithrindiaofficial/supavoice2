use anyhow::{Context, Result};
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

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

        // Create transcription state
        let mut state = self.ctx.create_state()
            .context("Failed to create Whisper state")?;

        // Setup transcription parameters - greedy decoding for speed
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Speed optimizations
        params.set_n_threads(8);
        params.set_translate(false);
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_token_timestamps(false);  // Disable token timestamps
        params.set_max_len(0);               // No length limit
        params.set_suppress_blank(true);     // Skip silent sections
        params.set_suppress_non_speech_tokens(true);

        // Run transcription
        state
            .full(params, &audio_data[..])
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
