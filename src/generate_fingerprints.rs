
use crate::types::types::{Constellation, Fingerprint};
use crate::types::types::SpectrogramPoint;

impl Constellation {
    fn new() -> Self {
        Self{
            arr: [(0, 0); 5],
            top: 0
        }
    }
    fn push(&mut self, freq:usize, delta:usize) {
        self.arr[self.top] = (freq, delta);
        self.top += 1;
    }
    
    
}

pub fn generate_fingerprints(peaks: &[SpectrogramPoint]) -> Vec<Fingerprint> {
    let mut fingerprints = Vec::new();

    // Config for the target zone
    let target_zone_size = 5; // Look 5 time steps ahead
    let delay = 3;            // Start looking 1 step after anchor

    for (i, anchor) in peaks.iter().enumerate() {
        if i+delay+target_zone_size >= peaks.len() {
            break;
        }
        for target in peaks[(i+delay)..(i+delay+target_zone_size)].iter() {
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