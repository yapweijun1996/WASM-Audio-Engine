//! Low-level WebAssembly bindings for block-oriented PCM processing.

use wasm_bindgen::prelude::*;

use crate::resample::StreamingResampler;
use crate::silence::{SilenceDetector, SilenceInterval};
use crate::waveform::{WaveformBuilder, WaveformPoint};

/// Return alternating frequency and amplitude values for a mono PCM window.
#[wasm_bindgen]
pub fn analyze_spectrum(samples: &[f32], sample_rate: u32) -> Result<Vec<f32>, JsValue> {
    let bins = crate::spectrum::spectrum(samples, sample_rate).map_err(js_error)?;
    Ok(bins
        .into_iter()
        .flat_map(|bin| [bin.frequency_hz, bin.amplitude])
        .collect())
}

fn js_error(error: impl std::fmt::Debug) -> JsValue {
    JsValue::from_str(&format!("{error:?}"))
}

#[wasm_bindgen]
pub struct WebResampler {
    inner: StreamingResampler,
}

#[wasm_bindgen]
impl WebResampler {
    #[wasm_bindgen(constructor)]
    pub fn new(
        input_rate: u32,
        output_rate: u32,
        channels: u32,
        chunk_frames: u32,
    ) -> Result<WebResampler, JsValue> {
        let inner = StreamingResampler::new(
            input_rate,
            output_rate,
            channels as usize,
            chunk_frames as usize,
        )
        .map_err(js_error)?;
        Ok(Self { inner })
    }
    pub fn process(&mut self, samples: &[f32]) -> Result<Vec<f32>, JsValue> {
        self.inner.process(samples).map_err(js_error)
    }
    pub fn finish(&mut self) -> Result<Vec<f32>, JsValue> {
        self.inner.finish().map_err(js_error)
    }
}

#[wasm_bindgen]
pub struct WebSilenceDetector {
    inner: SilenceDetector,
}

fn intervals_to_vec(intervals: impl IntoIterator<Item = SilenceInterval>) -> Vec<u64> {
    intervals
        .into_iter()
        .flat_map(|interval| [interval.start_frame, interval.end_frame])
        .collect()
}

#[wasm_bindgen]
impl WebSilenceDetector {
    #[wasm_bindgen(constructor)]
    pub fn new(
        channels: u32,
        threshold: f32,
        min_frames: u64,
    ) -> Result<WebSilenceDetector, JsValue> {
        Ok(Self {
            inner: SilenceDetector::new(channels as usize, threshold, min_frames)
                .map_err(js_error)?,
        })
    }
    pub fn process(&mut self, samples: &[f32]) -> Result<Vec<u64>, JsValue> {
        Ok(intervals_to_vec(
            self.inner.process(samples).map_err(js_error)?,
        ))
    }
    pub fn finish(&mut self) -> Result<Vec<u64>, JsValue> {
        Ok(intervals_to_vec(self.inner.finish().map_err(js_error)?))
    }
}

#[wasm_bindgen]
pub struct WebWaveformBuilder {
    inner: WaveformBuilder,
}

fn points_to_vec(points: impl IntoIterator<Item = WaveformPoint>) -> Result<Vec<f64>, JsValue> {
    let mut result = Vec::new();
    for point in points {
        if point.end_frame > 9_007_199_254_740_991 {
            return Err(JsValue::from_str(
                "waveform frame exceeds JavaScript's safe integer range",
            ));
        }
        result.extend_from_slice(&[
            point.start_frame as f64,
            point.end_frame as f64,
            point.min as f64,
            point.max as f64,
        ]);
    }
    Ok(result)
}

#[wasm_bindgen]
impl WebWaveformBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(channels: u32, window_frames: u64) -> Result<WebWaveformBuilder, JsValue> {
        Ok(Self {
            inner: WaveformBuilder::new(channels as usize, window_frames).map_err(js_error)?,
        })
    }
    pub fn process(&mut self, samples: &[f32]) -> Result<Vec<f64>, JsValue> {
        points_to_vec(self.inner.process(samples).map_err(js_error)?)
    }
    pub fn finish(&mut self) -> Result<Vec<f64>, JsValue> {
        points_to_vec(self.inner.finish().map_err(js_error)?)
    }
}

/// Execute Hound reader/writer validation in a dedicated validation build only.
#[cfg(feature = "validation")]
#[wasm_bindgen]
pub fn validate_hound_pcm16() -> Result<bool, JsValue> {
    use std::io::Cursor;
    let mut bytes = Vec::new();
    let input = [0.0, 0.5, -0.5, 0.25, -0.25, 0.0];
    {
        let mut writer =
            crate::wav::Pcm16Writer::new(Cursor::new(&mut bytes), 16_000, 2).map_err(js_error)?;
        writer.write_chunk(&input).map_err(js_error)?;
        writer.finish().map_err(js_error)?;
    }
    let mut reader = crate::wav::Pcm16Reader::new(Cursor::new(bytes)).map_err(js_error)?;
    let actual = reader.read_chunk(3).map_err(js_error)?;
    Ok(actual == input
        && reader.spec().sample_rate == 16_000
        && reader.spec().channels == 2
        && reader.read_chunk(1).map_err(js_error)?.is_empty())
}
