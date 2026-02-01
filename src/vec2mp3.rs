use mp3lame_encoder::{Builder, FlushNoGap, MonoPcm, Quality};
use std::fs::File;
use std::io::Write;

/// Saves mono f32 samples to an MP3 file.
///
/// * `path`: Output filename (e.g., "output.mp3")
/// * `samples`: The Vec<f32> from your loader
/// * `sample_rate`: The sample rate (Must get this from the decoder!)
pub fn save_mono_mp3(path: &str, samples: Vec<f32>, sample_rate: u32) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configure the Encoder
    let mut builder = Builder::new().expect("Failed to create builder");
    builder.set_num_channels(1).expect("set channels");
    builder.set_sample_rate(sample_rate).expect("set sample_rate");
    builder.set_brate(mp3lame_encoder::Bitrate::Kbps192).expect("set quality");
    builder.set_quality(mp3lame_encoder::Quality::Best).expect("set quality");

    // Build the encoder
    let mut mp3_encoder = builder.build().expect("To initialize LAME encoder");

    // 2. Convert f32 samples to i16
    // LAME requires Integer PCM. We scale -1.0..1.0 to -32767..32767
    let pcm_data: Vec<i16> = samples.iter().map(|&x| {
        // Clamp values to prevent distortion/wrapping
        let clamped = x.max(-1.0).min(1.0);
        (clamped * 32767.0) as i16
    }).collect();
    let input = MonoPcm(pcm_data.as_slice());

    // 3. Prepare Buffer and Encode
    // Estimate size: PCM size / compression ratio (approx 1/10th) + headroom
    let mut mp3_out_buffer = Vec::new();
    mp3_out_buffer.reserve(mp3lame_encoder::max_required_buffer_size(pcm_data.len()));
    let encoded_size = mp3_encoder.encode(input, mp3_out_buffer.spare_capacity_mut()).expect("To encode");
    unsafe {
        mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
    }

    let encoded_size = mp3_encoder.flush::<FlushNoGap>(mp3_out_buffer.spare_capacity_mut()).expect("to flush");
    unsafe {
        mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
    }
    // 5. Write to Disk
    let mut file = File::create(path)?;
    file.write_all(&mp3_out_buffer)?;

    println!("Saved MP3 to: {}", path);
    Ok(())
}