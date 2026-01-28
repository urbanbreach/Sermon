use rubato::{
    FastFixedIn, PolynomialDegree, Resampler as RubatoResampler, SincFixedIn,
    SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use tracing::{debug, trace};

pub enum ResamplerQuality {
    Fast,
    HighQuality,
}

pub struct Resampler {
    inner: ResamplerInner,
    source_rate: u32,
    target_rate: u32,
    channels: usize,
    input_buffer: Vec<Vec<f32>>,
    output_buffer: Vec<Vec<f32>>,
}

enum ResamplerInner {
    Fast(FastFixedIn<f32>),
    Sinc(SincFixedIn<f32>),
}

impl Resampler {
    pub fn new(
        source_rate: u32,
        target_rate: u32,
        channels: usize,
        quality: ResamplerQuality,
    ) -> Result<Self, String> {
        let ratio = target_rate as f64 / source_rate as f64;
        let chunk_size = 1024;

        debug!(
            source_rate,
            target_rate, ratio, channels, "Creating resampler"
        );

        let inner = match quality {
            ResamplerQuality::Fast => {
                let resampler = FastFixedIn::<f32>::new(
                    ratio,
                    1.0,
                    PolynomialDegree::Septic,
                    chunk_size,
                    channels,
                )
                .map_err(|e| format!("Failed to create fast resampler: {}", e))?;
                ResamplerInner::Fast(resampler)
            }
            ResamplerQuality::HighQuality => {
                let params = SincInterpolationParameters {
                    sinc_len: 256,
                    f_cutoff: 0.95,
                    interpolation: SincInterpolationType::Linear,
                    oversampling_factor: 256,
                    window: WindowFunction::BlackmanHarris2,
                };
                let resampler = SincFixedIn::<f32>::new(ratio, 1.0, params, chunk_size, channels)
                    .map_err(|e| format!("Failed to create sinc resampler: {}", e))?;
                ResamplerInner::Sinc(resampler)
            }
        };

        let input_frames = match &inner {
            ResamplerInner::Fast(r) => r.input_frames_max(),
            ResamplerInner::Sinc(r) => r.input_frames_max(),
        };
        let output_frames = match &inner {
            ResamplerInner::Fast(r) => r.output_frames_max(),
            ResamplerInner::Sinc(r) => r.output_frames_max(),
        };

        Ok(Self {
            inner,
            source_rate,
            target_rate,
            channels,
            input_buffer: vec![vec![0.0; input_frames]; channels],
            output_buffer: vec![vec![0.0; output_frames]; channels],
        })
    }

    pub fn input_frames_next(&self) -> usize {
        match &self.inner {
            ResamplerInner::Fast(r) => r.input_frames_next(),
            ResamplerInner::Sinc(r) => r.input_frames_next(),
        }
    }

    pub fn process_interleaved(&mut self, input: &[f32]) -> Result<Vec<f32>, String> {
        let input_frames = input.len() / self.channels;
        if input_frames == 0 {
            return Ok(Vec::new());
        }

        for ch in 0..self.channels {
            self.input_buffer[ch].clear();
            for frame in 0..input_frames {
                self.input_buffer[ch].push(input[frame * self.channels + ch]);
            }
        }

        let input_slices: Vec<&[f32]> = self.input_buffer.iter().map(|v| v.as_slice()).collect();

        let (_, output_frames) = match &mut self.inner {
            ResamplerInner::Fast(r) => r
                .process_into_buffer(&input_slices, &mut self.output_buffer, None)
                .map_err(|e| format!("Fast resampler error: {}", e))?,
            ResamplerInner::Sinc(r) => r
                .process_into_buffer(&input_slices, &mut self.output_buffer, None)
                .map_err(|e| format!("Sinc resampler error: {}", e))?,
        };

        trace!(input_frames, output_frames, "Resampled audio");

        let mut output = Vec::with_capacity(output_frames * self.channels);
        for frame in 0..output_frames {
            for ch in 0..self.channels {
                output.push(self.output_buffer[ch][frame]);
            }
        }

        Ok(output)
    }

    pub fn reset(&mut self) {
        match &mut self.inner {
            ResamplerInner::Fast(r) => r.reset(),
            ResamplerInner::Sinc(r) => r.reset(),
        }
    }

    pub fn source_rate(&self) -> u32 {
        self.source_rate
    }

    pub fn target_rate(&self) -> u32 {
        self.target_rate
    }
}
