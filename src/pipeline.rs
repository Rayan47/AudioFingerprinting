use std::path::Path;
use crate::load_audio_mono::load_audio_from_path;
use crate::downsampler::downsample;
use crate::create_spectogram::create_spectrogram;
use crate::find_peaks::save_spectrogram_peaks;
use crate::generate_fingerprints::generate_fingerprints;
use crate::generate_fingerprints::generate_fingerprints_quad;
use crate::generate_fingerprints::generate_fuzzy_query_hashes;




use crate::types::types::{Fingerprint, SpectrogramPoint};

pub fn extract_peaks(song: &Path, modifier: f32) -> Vec<SpectrogramPoint> {
    let (samples, sample_rate) = load_audio_from_path(song);
    let target_rate = 11025;
    let dsample = downsample(&samples, sample_rate, target_rate);

    let spectrum = create_spectrogram(&dsample);
    let mut peaks = save_spectrogram_peaks(&spectrum, modifier);
    peaks.sort_by(|a, b| a.time_idx.cmp(&b.time_idx));
    peaks
}
pub fn extract_features(song: &Path, modifier: f32) -> Vec<Fingerprint>{
    generate_fingerprints(extract_peaks(song, modifier).as_slice())
}

pub fn extract_features_quad(song: &Path, modifier: f32) -> Vec<Fingerprint>{
    generate_fingerprints_quad(extract_peaks(song, modifier).as_slice())
}

pub fn extract_features_client_fuzzy(song: &Path, modifier:f32) -> Vec<Fingerprint>{
    generate_fuzzy_query_hashes(extract_peaks(song, modifier).as_slice())
}
