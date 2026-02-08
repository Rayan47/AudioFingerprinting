
use crate::types::types::Fingerprint;
use crate::types::types::SpectrogramPoint;

fn generate_fingerprints(peaks: &[SpectrogramPoint]) -> Vec<Fingerprint> {
    let mut fingerprints = Vec::new();

    // Config for the target zone
    let target_zone_size = 5; // Look 5 time steps ahead
    let delay = 1;            // Start looking 1 step after anchor

    for (i, anchor) in peaks.iter().enumerate() {
        for target in peaks.iter().skip(i + 1) {

            // Check if target is within the time zone
            if target.time_idx - anchor.time_idx > target_zone_size + delay {
                break; // Target is too far in future, stop checking this anchor
            }
            if target.time_idx - anchor.time_idx < delay {
                continue; // Target is too close
            }

            // Create a Hash: [Anchor Freq | Target Freq | Delta Time]
            // We use simple bit-shifting for this example, but you can use a real hasher.
            let f1 = anchor.freq_bin as u64;
            let f2 = target.freq_bin as u64;
            let dt = (target.time_idx - anchor.time_idx) as u64;

            // Example packing: 20 bits f1, 20 bits f2, 24 bits dt
            let hash = (f1 << 44) | (f2 << 24) | dt;

            fingerprints.push(Fingerprint {
                hash,
                time_offset: anchor.time_idx,
            });
        }
    }
    fingerprints
}

fn hash_gen(peaks: &[SpectrogramPoint]) {
    for chunk in peaks.chunks(10){

    }

}