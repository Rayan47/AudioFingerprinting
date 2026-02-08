use crate::types::types::SpectrogramPoint;

use std::cmp::Ordering;
use std::collections::VecDeque;


const TIME_STEP: usize = 16;

const STRIDE:usize = 4;
const PEAKS_PER_WINDOW:usize = 1;

const CHUNKS: [usize; 10]  = [8, 8, 16, 32, 64, 128, 128, 128, 256, 256];
struct FixedLengthQueue {
    queue: VecDeque<f32>,
    capacity: usize,
    top: usize,
}

impl FixedLengthQueue {
    fn new(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity), // Pre-allocate memory
            capacity,
            top: 0,
        }
    }

    fn push(&mut self, item: f32) {
        if self.queue.len() == self.capacity {
            // If full, remove the oldest element (from the front)
            self.queue.pop_front();
        }
        self.queue.push_back(item);
        self.top += 1;
    }

    fn pop(&mut self) -> Option<f32> {
        self.queue.pop_front()
    }
    
    fn top(&self) -> usize {
        self.top
    }

    fn len(&self) -> usize {
        self.queue.len()
    }
    fn top_n_with_indices(&self, n:usize) -> Vec<(usize, f32)> {
        // 1. Create a vector of (index, value) tuples
        let mut items: Vec<(usize, f32)> = self.queue
            .iter()
            .enumerate()
            .map(|(i, &val)| (i, val))
            .collect();

        // 2. Sort descending by value.
        // We use a custom comparator because f32 doesn't implement Ord (due to NaN).
        items.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal)
        });

        // 3. Take the first 3 items
        items.into_iter().take(n).collect()
    }
    
    

}


pub fn save_spectrogram_peaks(
    spectrogram: &[Vec<f32>]
) -> Vec<SpectrogramPoint> {
    let mut ret: Vec<SpectrogramPoint> = Vec::new();
    let mut qs = CHUNKS.map(|t| {FixedLengthQueue::new(t*TIME_STEP)});
    for (x, column) in spectrogram.iter().enumerate() {
        //Might be able to optimize if vector pop is used instead of n
        let mut n = 0;
        let chunks = CHUNKS;
        for (i, index) in chunks.iter().enumerate() {
            for _ in 0..*index{
                qs[i].push(column[n]);
                n += 1;
            }

        }
        if (x >= TIME_STEP-1) && ((x+ 1 -TIME_STEP)%STRIDE == 0){
            for (i, q) in qs.iter().enumerate() {
                let top = q.top();
                for (ind, freq) in q.top_n_with_indices(PEAKS_PER_WINDOW) {
                    let band_size = q.capacity/TIME_STEP;
                    let time = (top - q.capacity + ind + 1) / band_size;
                    let bin = (i*band_size) + (ind % band_size);
                    ret.push(SpectrogramPoint { freq_bin: bin, magnitude: freq, time_idx: time });
                }
            }
        }
    }

    ret
}