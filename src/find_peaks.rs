use crate::types::types::SpectrogramPoint;
pub fn find_peaks(spectrogram: &[Vec<f32>]) -> Vec<SpectrogramPoint> {
    let mut peaks = Vec::new();
    let threshold = 50.0; // Minimum amplitude to be considered a peak

    for (t, spectrum) in spectrogram.iter().enumerate() {
        for (f, &mag) in spectrum.iter().enumerate() {
            // Optimization: Only keep peaks within a specific frequency range (e.g., 300Hz - 5kHz)
            // and check if `mag` is a local maximum compared to neighbors.
            // For brevity, we just check a simple threshold here.
            if mag > threshold {
                peaks.push(SpectrogramPoint {
                    freq_bin: f,
                    magnitude: mag,
                    time_idx: t,
                });
            }
        }
    }
    peaks
}