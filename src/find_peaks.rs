use crate::types::types::SpectrogramPoint;

use std::cmp::Ordering;
use std::collections::VecDeque;


const BAND_SIZE: usize = 16;
const TIME_STEP: usize = 16;

const CAPACITY: usize = BAND_SIZE * TIME_STEP;
const Q_SIZE: usize = 2048 / BAND_SIZE;
const STRIDE:usize = 4;
const PEAKS_PER_WINDOW:usize = 1;

struct FixedLengthQueue {
    queue: VecDeque<f32>,
    capacity: usize,
    top: usize,
}

impl FixedLengthQueue {
    fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(CAPACITY), // Pre-allocate memory
            capacity: CAPACITY,
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
    let mut qs: [FixedLengthQueue; Q_SIZE] = std::array::from_fn(|_| {
        let deque = FixedLengthQueue::new();
        deque
    });
    for (x, column) in spectrogram.iter().enumerate() {
        let mut i = 0;
        for slice in column.chunks(BAND_SIZE) {
            for &y in slice {
                qs[i].push(y);
            }
            i += 1;
        }
        if (x >= BAND_SIZE-1) && ((x+ 1 -BAND_SIZE)%STRIDE == 0){
            for (i, q) in qs.iter().enumerate() {
                let top = q.top();
                for (ind, freq) in q.top_n_with_indices(PEAKS_PER_WINDOW) {
                    let time = (top - CAPACITY + ind + 1) / BAND_SIZE;
                    let bin = (i*BAND_SIZE) + (ind % BAND_SIZE);
                    ret.push(SpectrogramPoint { freq_bin: bin, magnitude: freq, time_idx: time });
                }
            }
        }
    }

    ret
}