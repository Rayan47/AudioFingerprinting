
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
    let target_zone_size = 10; // Look 5 time steps ahead
    let delay = 5;            // Start looking 1 step after anchor

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
pub fn generate_fingerprints_quad(peaks: &[SpectrogramPoint]) -> Vec<Fingerprint> {
    let mut fingerprints = Vec::new();

    // Config for the target zone
    let target_zone_size = 5; // Look 5 peaks ahead
    let delay = 3;            // Start looking 3 peaks after the 3rd anchor

    // We iterate up to the point where we can safely fit 3 anchors + delay + target zone
    for i in 0..peaks.len() {
        if i + 2 + delay + target_zone_size >= peaks.len() {
            break;
        }

        // Define our 3 anchor points
        let a1 = &peaks[i];
        let a2 = &peaks[i + 1];
        let a3 = &peaks[i + 2];

        // Target zone starts after the 3rd anchor + delay
        let target_start = i + 2 + delay;
        let target_end = target_start + target_zone_size;

        for target in &peaks[target_start..target_end] {
            // 1. Mask frequencies to 9 bits (max value 511)
            let f1 = (a1.freq_bin as u64) & 0x1FF;
            let f2 = (a2.freq_bin as u64) & 0x1FF;
            let f3 = (a3.freq_bin as u64) & 0x1FF;
            let f4 = (target.freq_bin as u64) & 0x1FF;

            // 2. Calculate time deltas relative to the FIRST anchor (a1)
            // Using saturating_sub ensures we don't underflow if time indices are out of order
            let dt1 = (a2.time_idx.saturating_sub(a1.time_idx) as u64) & 0x1FF;     // 9 bits
            let dt2 = (a3.time_idx.saturating_sub(a1.time_idx) as u64) & 0x1FF;     // 9 bits
            let dt3 = (target.time_idx.saturating_sub(a1.time_idx) as u64) & 0x3FF; // 10 bits

            // 3. Pack into a single 64-bit integer
            // Layout: [f1: 9][f2: 9][f3: 9][f4: 9][dt1: 9][dt2: 9][dt3: 10]
            let hash = (f1 << 55) |
                (f2 << 46) |
                (f3 << 37) |
                (f4 << 28) |
                (dt1 << 19) |
                (dt2 << 10) |
                dt3;

            fingerprints.push(Fingerprint {
                hash,
                time_offset: a1.time_idx, // Use the first anchor as the reference time
            });
        }
    }

    fingerprints
}
// We modify our fingerprint generation function to ONLY expand 
// hashes when we are QUERYING the database (listening to the mic).
// When indexing a song into the DB, we only save the exact hash.

pub fn generate_fuzzy_query_hashes(peaks: &[SpectrogramPoint]) -> Vec<Fingerprint> {
    let mut fingerprints = Vec::new();
    let target_zone_size = 5;
    let delay = 3;

    for i in 0..peaks.len() {
        if i + 2 + delay + target_zone_size >= peaks.len() { break; }

        let a1 = &peaks[i];
        let a2 = &peaks[i + 1];
        let a3 = &peaks[i + 2];
        let target_start = i + 2 + delay;
        let target_end = target_start + target_zone_size;

        for target in &peaks[target_start..target_end] {
            let f1 = (a1.freq_bin as u64) & 0x1FF;
            let f2 = (a2.freq_bin as u64) & 0x1FF;
            let f3 = (a3.freq_bin as u64) & 0x1FF;

            let dt1 = (a2.time_idx.saturating_sub(a1.time_idx) as u64) & 0x1FF;
            let dt2 = (a3.time_idx.saturating_sub(a1.time_idx) as u64) & 0x1FF;

            // The exact values
            let exact_f4 = target.freq_bin as i64;
            let exact_dt3 = target.time_idx.saturating_sub(a1.time_idx) as i64;

            // "Wiggle" the target frequency and time delta by -1, 0, and +1
            // This generates 9 total variations per quad (3x3 grid)
            for f4_fuzz in [-1, 0, 1] {
                for dt3_fuzz in [-1, 0, 1] {

                    let f4_fuzzed = (exact_f4 + f4_fuzz).max(0) as u64 & 0x1FF;
                    let dt3_fuzzed = (exact_dt3 + dt3_fuzz).max(0) as u64 & 0x3FF;

                    let fuzzy_hash = (f1 << 55) |
                        (f2 << 46) |
                        (f3 << 37) |
                        (f4_fuzzed << 28) |
                        (dt1 << 19) |
                        (dt2 << 10) |
                        dt3_fuzzed;

                    fingerprints.push(Fingerprint {
                        hash: fuzzy_hash,
                        time_offset: a1.time_idx,
                    });
                }
            }
        }
    }
    fingerprints
}