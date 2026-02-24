use crate::types::types::SpectrogramPoint;

use std::cmp::Ordering;
use std::collections::VecDeque;


const TIME_STEP: usize = 16;

const STRIDE:usize = 4;
const BUFFER_SIZE:usize = 60;

const CHUNKS: [(usize, usize); 6]  = [(0, 10), (10, 20), (20, 40), (40, 80), (80, 160), (160, 512)];
struct FixedLengthQueue {
    queue: VecDeque<f32>,
    capacity: usize,
    top: usize,
    offset: usize,
}

impl FixedLengthQueue {
    fn new(capacity: usize, offset: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity), // Pre-allocate memory
            capacity,
            top: 0,
            offset,

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
    fn push_slice(&mut self, items: &[f32]) {
        self.queue.extend(items.iter().cloned());
        self.top += items.len();
        if self.len() > self.capacity {
            self.queue.drain(0..items.len());
        }
    }
    fn top(&self) -> usize {
        self.top
    }
    fn max_with_index(&self) -> Option<(usize, f32)> {
        self.queue
            .iter()
            .enumerate()
            .map(|(i, &val)| (i, val))
            // partial_cmp handles the fact that f32 isn't fully "Ord"
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    fn len(&self) -> usize{
        self.queue.len()
    }



    

}
pub struct RollingStats {
    buffer: [f32; BUFFER_SIZE],
    cursor: usize,
    count: usize,
    sum: f32,
    sum_sq: f32,
}

impl RollingStats {
    /// Creates a new, empty RollingStats structure.
    pub fn new() -> Self {
        Self{
            buffer: [0.0; BUFFER_SIZE],
            cursor: 0,
            count: 0,
            sum: 0.0,
            sum_sq: 0.0,
        }
    }

    /// Adds a new value to the rolling window.
    pub fn push(&mut self, value: f32) {
        // If the buffer is full, we must remove the oldest value from our running sums
        let old_value = if self.count == BUFFER_SIZE {
            self.buffer[self.cursor]
        } else {
            0.0
        };

        // Update the running sum and sum of squares
        self.sum = self.sum + value - old_value;
        self.sum_sq = self.sum_sq + (value * value) - (old_value * old_value);

        // Overwrite the oldest value and move the cursor
        self.buffer[self.cursor] = value;
        self.cursor = (self.cursor + 1) % BUFFER_SIZE;

        if self.count < BUFFER_SIZE {
            self.count += 1;
        }
    }

    /// Returns the mean of the current values.
    pub fn mean(&self) -> f32 {
        if self.count == 0 {
            return 10f32;
        }
        self.sum / self.count as f32
    }

    /// Returns the sample standard deviation of the current values.
    pub fn std_dev(&self) -> f32 {
        if self.count < 2 {
            // Standard deviation requires at least 2 data points for a sample.
            // If you prefer Population Standard Deviation, you can change this logic.
            return 0f32;
        }

        let n = self.count as f32;

        // Calculate variance.
        // We use `.max(0.0)` because floating point inaccuracies can
        // occasionally cause the numerator to drop infinitesimally below zero.
        let variance = (self.sum_sq - (self.sum * self.sum) / n) / (n - 1.0);

        variance.max(0.0).sqrt()
    }
    pub fn get_threshold(&self) -> f32{
        self.mean() + 0.5*self.std_dev()
    }
}


pub fn save_spectrogram_peaks(
    spectrogram: &[Vec<f32>]
) -> Vec<SpectrogramPoint> {
    let mut ret: Vec<SpectrogramPoint> = Vec::new();
    let mut qs = CHUNKS.map(|(start, end)| {FixedLengthQueue::new((end-start)*TIME_STEP, start)});
    let mut track = RollingStats::new();
    for (x, column) in spectrogram.iter().enumerate() {
        for (i, (start, end)) in CHUNKS.iter().enumerate() {
            qs[i].push_slice(&column[*start..*end]);

        }
        if (x >= TIME_STEP-1) && ((x+ 1 -TIME_STEP)%STRIDE == 0){
            let mut inter: Vec<SpectrogramPoint> = Vec::new();
            let mut sum = 0.0f32;
            for i in 0..qs.len() {
                let q = &mut qs[i];
                let top = q.top();
                let (ind, freq) = q.max_with_index().unwrap();
                let band_size = q.capacity/TIME_STEP;
                let time = (top - q.capacity + ind) / band_size;
                let bin = q.offset + (ind % band_size);
                track.push(freq);
                inter.push(SpectrogramPoint { freq_bin: bin, magnitude: freq, time_idx: time });


            }
            for pt in inter {
                if pt.magnitude > track.get_threshold() {
                    ret.push(pt);
                }
            }
            }
        }
    ret
    }


