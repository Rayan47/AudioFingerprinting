use rubato::{Resampler, SincFixedIn, SincInterpolationType, SincInterpolationParameters, WindowFunction};

pub fn downsample(input: &[f32], src_rate: u32, target_rate: u32) -> Vec<f32> {
    if src_rate == target_rate {
        return input.to_vec();
    }

    // 1. Calculate the resampling ratio
    let ratio = target_rate as f64 / src_rate as f64;

    // 2. Configure the Resampler (High quality Sinc interpolation)
    // SincFixedIn is easier to use for arbitrary ratios than FFT-based ones.
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95, // Cut off at 95% of the Nyquist frequency to avoid aliasing
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 128,
        window: WindowFunction::BlackmanHarris2,
    };

    // 1 Channel (Mono), chunk size (input samples per batch)
    let chunk_size = 1024;
    let mut resampler = SincFixedIn::<f32>::new(
        ratio,
        5f64,
        params,
        chunk_size,
        1, // Channels
    ).unwrap();

    // 3. Prepare buffers
    let mut output_buffer = Vec::with_capacity((input.len() as f64 * ratio) as usize);
    let mut input_frames = vec![vec![0.0; chunk_size]; 1]; // Rubato expects Vec<Vec<f32>> (channels)

    // 4. Process chunks
    for chunk in input.chunks(chunk_size) {
        // Copy chunk into input_frames
        // If the last chunk is smaller than chunk_size, we must pad it with zeros
        let current_len = chunk.len();
        input_frames[0][..current_len].copy_from_slice(chunk);
        if current_len < chunk_size {
            input_frames[0][current_len..].fill(0.0);
        }

        // Process
        // `process` returns the resampled audio for this chunk
        // Note: The last chunk might return slightly more silence due to padding, 
        // but for fingerprinting, this is negligible.
        let output_frames = resampler.process(&input_frames, None).unwrap();

        // Append mono channel to output
        output_buffer.extend_from_slice(&output_frames[0]);
    }

    // Optional: Trim excess capacity or silence if strict length is needed
    output_buffer
}