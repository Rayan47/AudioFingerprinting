use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::pipeline::extract_features;
use crate::types::types::Fingerprint;



pub struct AudioDatabase {
    pub songs: HashMap<u32, String>,
    pub hashes: HashMap<u64, Vec<(u32, usize)>>,
    next_song_id: u32,
}

impl AudioDatabase {
    pub fn new() -> Self {
        AudioDatabase {
            songs: HashMap::new(),
            hashes: HashMap::new(),
            next_song_id: 0,
        }
    }

    /// Recursively traverses a directory and indexes all audio files
    pub fn index_directory(&mut self, directory: &str) -> &mut Self {
        let path = Path::new(directory);
        if !path.is_dir() {
            eprintln!("Error: {} is not a directory", directory);
            return self;
        }

        self.visit_dirs(path);
        self
    }

    // Helper to walk directories recursively
    fn visit_dirs(&mut self, dir: &Path) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    self.visit_dirs(&path);
                } else {
                    self.process_file(&path);
                }
            }
        }
    }

    fn process_file(&mut self, path: &Path) {
        // Only process specific audio extensions
        let is_audio = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "mp3" || ext == "wav")
            .unwrap_or(false);

        if !is_audio {
            return;
        }

        let filename = path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        println!("Indexing: {}", filename);

        // 1. Assign an ID and store the song name
        let song_id = self.next_song_id;
        self.songs.insert(song_id, filename);
        self.next_song_id += 1;

        // 2. Run your audio pipeline to get fingerprints
        // (You will need to wrap your load -> downsample -> FFT -> peak pipeline here)
        let fingerprints = extract_features(path, 1f32);

        // 3. Insert all fingerprints into the hash database
        for fp in fingerprints {
            // .entry() allows us to get the existing Vec or insert a new one in O(1) time
            self.hashes
                .entry(fp.hash)
                .or_insert_with(Vec::new)
                .push((song_id, fp.time_offset));
        }
    }
    pub fn lookup_hash(&self, hash: u64) -> Option<Vec<(u32,usize)>> {
        self.hashes.get(&hash).cloned()
    }
    pub fn lookup_song(&self, song_id: u32) -> Option<String> {
        self.songs.get(&song_id).cloned()
    }
    pub fn find_best_match(&self, query_fingerprints: &[Fingerprint]) -> Option<String> {
        // We need to map: SongID -> (TimeDelta -> MatchCount)
        // We use i64 for the delta because the query could technically 
        // start slightly before the indexed song due to prepended silence/noise.
        let mut match_counts: HashMap<u32, HashMap<i64, usize>> = HashMap::new();

        // 1. Iterate through every hash in our query snippet
        for query_fp in query_fingerprints {
            // Check if this hash exists anywhere in our database in O(1) time
            if let Some(db_matches) = self.hashes.get(&query_fp.hash) {

                // If it exists, iterate through all songs that contain this hash
                for &(song_id, db_time_offset) in db_matches {

                    // Calculate the relative time difference
                    // Delta = Database Time - Query Time
                    let delta = db_time_offset as i64 - query_fp.time_offset as i64;

                    // Increment the histogram for this specific song and delta
                    let song_histogram = match_counts.entry(song_id).or_insert_with(HashMap::new);
                    let count = song_histogram.entry(delta).or_insert(0);
                    *count += 1;
                }
            }
        }

        // 2. Analyze the histograms to find the highest peak (max coherence)
        let mut best_song_id = None;
        let mut max_aligned_matches = 0;
        let mut best_delta = 0;

        for (song_id, histogram) in match_counts {
            for (delta, count) in histogram {
                if count > max_aligned_matches {
                    max_aligned_matches = count;
                    best_song_id = Some(song_id);
                    best_delta = delta;
                }
            }
        }

        // 3. Apply a confidence threshold
        // If we only have 3 or 4 random hashes align, it could be a coincidence.
        // 10+ aligned hashes is statistically impossible to happen by chance.
        let threshold = 10;

        if max_aligned_matches >= threshold {
            if let Some(id) = best_song_id {
                if let Some(song_name) = self.songs.get(&id) {
                    println!(
                        "Match found! '{}' with {} aligned hashes at time offset delta {}.",
                        song_name, max_aligned_matches, best_delta
                    );
                    return Some(song_name.clone());
                }
            }
        }

        println!("No match found. (Highest coherence was {} hashes)", max_aligned_matches);
        None
    }

}