//! Narrow native smoke tests for prospective dependencies, not browser evidence.

use std::io::Cursor;

use rubato::audioadapter::Adapter;
use rubato::audioadapter_buffers::direct::SequentialSliceOfVecs;
use rubato::{
    Async, FixedAsync, Resampler, SincInterpolationParameters, SincInterpolationType,
    WindowFunction,
};

#[test]
fn hound_round_trips_pcm_wav() {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut output, spec).unwrap();
        for sample in [0_i16, 8192, -8192] {
            writer.write_sample(sample).unwrap();
        }
        writer.finalize().unwrap();
    }
    let mut reader = hound::WavReader::new(Cursor::new(output.into_inner())).unwrap();
    assert_eq!(reader.spec(), spec);
    assert_eq!(
        reader
            .samples::<i16>()
            .collect::<Result<Vec<_>, _>>()
            .unwrap(),
        [0, 8192, -8192]
    );
}

#[test]
fn rubato_resamples_a_mono_buffer() {
    let params = SincInterpolationParameters {
        sinc_len: 64,
        f_cutoff: Some(0.95),
        interpolation: SincInterpolationType::Cubic,
        oversampling_factor: 16,
        window: WindowFunction::BlackmanHarris2,
    };
    let mut resampler =
        Async::<f32>::new_sinc(1.0 / 3.0, 1.1, &params, 1024, 1, FixedAsync::Input).unwrap();
    let samples = vec![vec![0.0_f32; 4800]];
    let input = SequentialSliceOfVecs::new(&samples, 1, 4800).unwrap();
    let output = resampler.process_all(&input, 4800, None).unwrap();
    assert_eq!(output.channels(), 1);
    assert_eq!(output.frames(), 1600);
    assert!((0..output.frames()).all(|frame| output.read_sample(0, frame).unwrap() == 0.0));
}
