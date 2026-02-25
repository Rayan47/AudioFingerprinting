use crate::types::types::SpectrogramPoint;



const BUFFER_SIZE:usize = 60;

const CHUNKS: [(usize, usize); 6]  = [(0, 10), (10, 20), (20, 40), (40, 80), (80, 160), (160, 512)];

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
    pub fn get_threshold(&self, modifier: f32) -> f32{
        self.mean() + modifier*self.std_dev()
    }
}


pub fn save_spectrogram_peaks(
    spectrogram: &[Vec<f32>],
    modifier : f32
) -> Vec<SpectrogramPoint> {
    let mut ret: Vec<SpectrogramPoint> = Vec::new();
    let mut track = RollingStats::new();

    for (x, column) in spectrogram.iter().enumerate() {
        let mut inter: Vec<SpectrogramPoint> = Vec::new();

        for (start, end) in CHUNKS.iter() {
            let mut max_j = 0;
            let mut max_mag = f32::MIN;
            for j in *start..*end {
                if max_mag < column[j] {
                    max_mag = column[j];
                    max_j = j;
                }
            }
            inter.push(SpectrogramPoint { freq_bin: max_j, magnitude: max_mag, time_idx: x });
            track.push(max_mag);
        }
        for pt in inter {
            if pt.magnitude > track.get_threshold(modifier) {
                ret.push(pt);
            }
        }
    }
    ret
}


