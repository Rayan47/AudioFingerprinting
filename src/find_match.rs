use std::collections::HashMap;
use crate::types::types::Fingerprint;

fn find_match(
    database: &HashMap<u64, Vec<(String, usize)>>,
    sample_fingerprints: &[Fingerprint]
) -> Option<String> {

    // Map of SongID -> List of Offset Deltas
    let mut matches: HashMap<String, Vec<i64>> = HashMap::new();

    for fp in sample_fingerprints {
        if let Some(db_entries) = database.get(&fp.hash) {
            for (song_id, db_time) in db_entries {
                // The relative time in the song should be constant
                // Real Time = DB Time - Sample Time
                let relative_time = *db_time as i64 - fp.time_offset as i64;
                matches.entry(song_id.clone()).or_default().push(relative_time);
            }
        }
    }

    // Find the song with the most occurrences of a specific relative_time
    // (A real implementation would use a histogram here)
    let mut best_song = None;
    let mut max_count = 0;

    for (song_id, deltas) in matches {
        // Find the mode (most common delta) for this song
        let mut delta_counts = HashMap::new();
        for d in deltas {
            *delta_counts.entry(d).or_insert(0) += 1;
        }

        let (best_delta, count) = delta_counts.iter().max_by_key(|entry| entry.1).unwrap();

        // Simple threshold: if we have more than 10 aligned hashes, it's a match
        if *count > 10 && *count > max_count {
            max_count = *count;
            best_song = Some(song_id);
        }
    }

    best_song
}