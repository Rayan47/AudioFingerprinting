use image::{ImageBuffer, Rgb};
use std::cmp;

pub fn save_spectrogram_image(
    spectrogram: &[Vec<f32>],
    path: &str,
    height_limit: usize
) -> Result<(), Box<dyn std::error::Error>> {
    if spectrogram.is_empty() {
        return Ok(());
    }

    let width = spectrogram.len();
    // Assuming all time slices have the same number of frequency bins
    let full_height = spectrogram[0].len();

    // We might crop the height because high frequencies (top of FFT) are often empty/noise
    let height = cmp::min(full_height, height_limit);

    // Create a new image buffer
    let mut imgbuf = ImageBuffer::new(width as u32, height as u32);

    // Find min/max for normalization (Log scale is best for audio)
    let mut max_val = -f32::INFINITY;
    let mut min_val = f32::INFINITY;

    // Pre-calculate log magnitudes for better visualization
    let log_spec: Vec<Vec<f32>> = spectrogram.iter()
        .map(|col| {
            col.iter().take(height).map(|&x| {
                let log_x = (x + 1e-6).ln(); // Avoid log(0)
                if log_x > max_val { max_val = log_x; }
                if log_x < min_val { min_val = log_x; }
                log_x
            }).collect()
        })
        .collect();

    let range = max_val - min_val;

    // Draw pixels
    // x = time, y = frequency
    for (x, column) in log_spec.iter().enumerate() {
        for (y, &val) in column.iter().enumerate() {
            // Normalize 0.0 to 1.0
            let normalized = (val - min_val) / range;

            // Get color from gradient (Inferno is great for audio)
            let color = colorous::INFERNO.eval_continuous(normalized as f64);

            // Image coordinates: (0,0) is Top-Left.
            // We want low freq at bottom, so we flip Y: (height - 1 - y)
            let pixel_y = (height - 1 - y) as u32;

            if pixel_y < height as u32 {
                imgbuf.put_pixel(x as u32, pixel_y, Rgb([color.r, color.g, color.b]));
            }
        }
    }

    imgbuf.save(path)?;
    println!("Spectrogram saved to {}", path);
    Ok(())
}

