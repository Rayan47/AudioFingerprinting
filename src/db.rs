use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::pipeline::extract_features;
use crate::types::types::Fingerprint;


use serde::{Serialize, Deserialize};
use std::io::{BufReader, BufWriter};
use std::sync::mpsc;
use rayon::prelude::*;
#[derive(Serialize, Deserialize)]
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
    pub fn index_directory(&mut self, directory: &str) {
        let path = Path::new(directory);
        if !path.is_dir() {
            eprintln!("Error: {} is not a directory", directory);
            return;
        }

        // 1. Gather all file paths sequentially first
        let mut audio_files = Vec::new();
        self.collect_files(path, &mut audio_files);

        let total_files = audio_files.len();
        println!("Found {} audio files. Processing in parallel...", total_files);

        // 2. Set up a message channel
        // tx (Transmitter) can be cloned and given to many threads.
        // rx (Receiver) stays on the main thread.
        let (tx, rx) = mpsc::channel();

        // 3. Process files in the background using Rayon's thread pool
        // We use thread::spawn so the main thread isn't blocked and can start
        // receiving data immediately.
        std::thread::spawn(move || {
            // into_par_iter() automatically distributes the workload across all CPU cores
            audio_files.into_par_iter().for_each(|file_path| {

                let filename = file_path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();

                println!("Thread {:?} started processing: {}", std::thread::current().id(), filename);

                // Run the heavy audio pipeline (Decoding -> FFT -> Hashing)
                let fingerprints = extract_features(&file_path, 1.0f32);

                // Send the result back to the main thread
                // If the receiver is dropped, send() fails, so we just ignore errors here
                let _ = tx.send((filename, fingerprints));
            });
            // The `tx` is dropped here when the parallel iterator finishes.
            // This signals the `rx` channel to close.
        });

        // 4. Listen on the main thread and insert into the database sequentially
        let mut processed_count = 0;

        // This loop will block and wait for messages. It automatically exits
        // when all transmitters (`tx`) are dropped (i.e., when all files are done).
        for (filename, fingerprints) in rx {
            processed_count += 1;

            let song_id = self.next_song_id;
            self.songs.insert(song_id, filename.clone());
            self.next_song_id += 1;

            for fp in fingerprints {
                self.hashes
                    .entry(fp.hash)
                    .or_insert_with(Vec::new)
                    .push((song_id, fp.time_offset));
            }

            println!("Merged {}/{} into DB: {}", processed_count, total_files, filename);
        }

        println!("Indexing complete! Database contains {} songs.", self.songs.len());
    }
    pub fn update_db(&mut self, path_to_song: &str){

    }

    /// Helper to recursively gather file paths (Runs quickly on the main thread)
    fn collect_files(&self, dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    self.collect_files(&path, files);
                } else {
                    let is_audio = path.extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == "mp3" || ext == "wav")
                        .unwrap_or(false);

                    if is_audio {
                        files.push(path);
                    }
                }
            }
        }
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
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = fs::File::create(path)?;
        // Wrap the file in a BufWriter for performance
        let mut writer = BufWriter::new(file);

        // Serialize the database directly into the file stream
        rmp_serde::encode::write(&mut writer, &self)?;

        println!("Successfully saved database to {}", path);
        Ok(())
    }

    /// Loads the database from a binary file on disk.
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = fs::File::open(path)?;
        // Wrap the file in a BufReader for performance
        let reader = BufReader::new(file);

        // Deserialize the bytes back into our AudioDatabase struct
        let db: AudioDatabase = rmp_serde::decode::from_read(reader)?;

        println!(
            "Successfully loaded database from {}. ({} songs, {} unique hashes)",
            path, db.songs.len(), db.hashes.len()
        );
        Ok(db)
    }
}