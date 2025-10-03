use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use hound::{WavSpec, WavWriter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

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
        let stop_flag = Arc::new(AtomicBool::new(false));
        self.record_to_file_cancellable(output_path, Some(duration_secs), stop_flag)
    }

    pub fn record_to_file_cancellable(&self, output_path: PathBuf, max_duration_secs: Option<u64>, stop_flag: Arc<AtomicBool>) -> Result<()> {
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

        // Record until stop flag is set or max duration is reached
        let start = std::time::Instant::now();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));

            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            if let Some(max_duration) = max_duration_secs {
                if start.elapsed().as_secs() >= max_duration {
                    break;
                }
            }
        }

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

        // Calculate resampling ratio
        let resample_ratio = sample_rate as f64 / target_sample_rate as f64;
        let sample_index = Arc::new(Mutex::new(0.0f64));

        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                if let Ok(mut guard) = writer.try_lock() {
                    if let Some(writer) = guard.as_mut() {
                        let mut index = sample_index.lock().unwrap();

                        // Process frames and resample
                        for frame in data.chunks(channels) {
                            // Average channels to mono
                            let mono_sample: f32 = frame.iter()
                                .map(|s| s.to_float_sample().to_sample::<f32>())
                                .sum::<f32>() / channels as f32;

                            // Only write sample when we've accumulated enough input samples
                            if *index >= 1.0 {
                                *index -= 1.0;

                                // Convert to i16 and write
                                let sample: i16 = (mono_sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                                let _ = writer.write_sample(sample);
                            }

                            *index += 1.0 / resample_ratio;
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
