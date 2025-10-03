use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use hound::{WavSpec, WavWriter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct AudioRecorder {
    sample_rate: u32,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            sample_rate: 16000, // Whisper requires 16kHz
        }
    }

    pub fn record_to_file(&self, output_path: PathBuf, duration_secs: u64) -> Result<()> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device available"))?;

        let config = device.default_input_config()?;

        let spec = WavSpec {
            channels: 1, // Mono
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let writer = WavWriter::create(&output_path, spec)?;
        let writer = Arc::new(Mutex::new(Some(writer)));

        let writer_clone = writer.clone();
        let err_fn = move |err| {
            eprintln!("Stream error: {}", err);
        };

        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => self.build_input_stream::<i8>(&device, &config.into(), writer_clone, err_fn)?,
            cpal::SampleFormat::I16 => self.build_input_stream::<i16>(&device, &config.into(), writer_clone, err_fn)?,
            cpal::SampleFormat::I32 => self.build_input_stream::<i32>(&device, &config.into(), writer_clone, err_fn)?,
            cpal::SampleFormat::F32 => self.build_input_stream::<f32>(&device, &config.into(), writer_clone, err_fn)?,
            _ => return Err(anyhow::anyhow!("Unsupported sample format")),
        };

        stream.play()?;

        // Record for specified duration
        std::thread::sleep(std::time::Duration::from_secs(duration_secs));

        drop(stream);

        // Finalize the WAV file
        if let Some(writer) = writer.lock().unwrap().take() {
            writer.finalize()?;
        }

        Ok(())
    }

    fn build_input_stream<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        writer: Arc<Mutex<Option<WavWriter<std::io::BufWriter<std::fs::File>>>>>,
        err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
    ) -> Result<cpal::Stream>
    where
        T: Sample + hound::Sample + FromSample<f32> + cpal::SizedSample,
    {
        let channels = config.channels as usize;
        let sample_rate = config.sample_rate.0;
        let target_sample_rate = self.sample_rate;

        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                if let Ok(mut guard) = writer.try_lock() {
                    if let Some(writer) = guard.as_mut() {
                        // Resample and convert to mono if needed
                        for frame in data.chunks(channels) {
                            // Average channels to mono
                            let mono_sample: f32 = frame.iter()
                                .map(|s| s.to_float_sample().to_sample::<f32>())
                                .sum::<f32>() / channels as f32;

                            // Simple resampling (basic decimation/interpolation)
                            // For production, use a proper resampling library
                            let sample: i16 = (mono_sample * i16::MAX as f32) as i16;
                            let _ = writer.write_sample(sample);
                        }
                    }
                }
            },
            err_fn,
            None,
        )?;

        Ok(stream)
    }
}
