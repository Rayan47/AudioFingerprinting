use symphonia::core::codecs::{DecoderOptions};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use std::fs::File;
use std::path::Path;

pub fn load_audio_mono(path: &str) -> (Vec<f32>, u32) {
    let src = File::open(Path::new(path)).expect("failed to open media");
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let probed = symphonia::default::get_probe()
        .format(&Hint::new(), mss, &FormatOptions::default(), &MetadataOptions::default())
        .expect("unsupported format");

    let mut format = probed.format;
    let track = format.default_track().unwrap();
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .expect("unsupported codec");

    let track_id = track.id;
    let mut samples: Vec<f32> = Vec::new();
    let mut sample_rate = 0u32;
    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id { continue; }
        
        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                sample_rate = spec.rate;
                let duration = decoded.capacity() as u64;
                let mut buf = symphonia::core::audio::SampleBuffer::<f32>::new(duration, spec);
                buf.copy_interleaved_ref(decoded);

                // Convert Stereo to Mono by averaging channels
                let channel_count = spec.channels.count();
                for frame in buf.samples().chunks(channel_count) {
                    let mono_sample: f32 = frame.iter().sum::<f32>() / channel_count as f32;
                    samples.push(mono_sample);
                }
            }
            Err(Error::DecodeError(_)) => (),
            Err(_) => break,
        }
        
    }
    (samples, sample_rate)
    
}

