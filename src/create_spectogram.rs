use rustfft::{FftPlanner, num_complex::Complex};

const WINDOW_SIZE: usize = 1024;
const OVERLAP: usize = WINDOW_SIZE/2;


pub fn create_spectrogram(samples: &[f32]) -> Vec<Vec<f32>> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(WINDOW_SIZE);

    // Hann Window function to reduce spectral leakage
    let window: Vec<f32> = (0..WINDOW_SIZE)
        .map(|i| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (WINDOW_SIZE as f32 - 1.0)).cos()))
        .collect();

    let mut spectrogram = Vec::new();

    // Sliding window
    for chunk in samples.windows(WINDOW_SIZE).step_by(WINDOW_SIZE - OVERLAP) {
        let mut buffer: Vec<Complex<f32>> = chunk.iter()
            .zip(&window)
            .map(|(&s, &w)| Complex::new(s * w, 0.0))
            .collect();

        fft.process(&mut buffer);

        // Calculate magnitude for the first half (Nyquist limit)
        let magnitudes: Vec<f32> = buffer.iter()
            .take(WINDOW_SIZE/2)
            .map(|c| c.norm())
            .collect();

        spectrogram.push(magnitudes);
    }
    spectrogram
}

