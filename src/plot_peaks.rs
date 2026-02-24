use plotters::prelude::*;
use crate::types::types::SpectrogramPoint;

/// Plots a spectrogram heatmap to a PNG file.
///
/// # Arguments
/// * `data` - A slice of SpectrogramPoint structs.
/// * `filename` - The output path for the image (e.g., "spectrogram.png").
/// * `width` & `height` - Dimensions of the output image.
pub fn plot_spectrogram(
    data: &[SpectrogramPoint],
    filename: &str,
    width: u32,
    height: u32
) -> Result<(), Box<dyn std::error::Error>> {

    // 1. Setup the backend
    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    // 2. Determine bounds for axes and color scale
    if data.is_empty() {
        return Err("No data provided to plot".into());
    }

    let max_time = data.iter().map(|p| p.time_idx).max().unwrap_or(0);
    let max_freq = 255usize;

    // Find min/max magnitude for color normalization
    let min_mag = data.iter().map(|p| p.magnitude).fold(f32::INFINITY, |a, b| a.min(b));
    let max_mag = data.iter().map(|p| p.magnitude).fold(f32::NEG_INFINITY, |a, b| a.max(b));

    // 3. Build the chart
    let mut chart = ChartBuilder::on(&root)
        .caption("Spectrogram Analysis", ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0..max_time + 1, 0..max_freq + 1)?;

    chart
        .configure_mesh()
        .x_desc("Time Index")
        .y_desc("Frequency Bin")
        .draw()?;

    // 4. Draw the Heatmap
    // We represent each point as a rectangle of size 1x1.
    // The color is interpolated between Blue (Low) and Red (High).
    chart.draw_series(data.iter().map(|point| {
        let x = point.time_idx;
        let y = point.freq_bin;

        // Normalize magnitude between 0.0 and 1.0
        let norm_mag = if max_mag == min_mag {
            0.5
        } else {
            (point.magnitude - min_mag) / (max_mag - min_mag)
        };

        // Create a color style: Blue -> Cyan -> Yellow -> Red
        let color = HSLColor(
            (1.0 - norm_mag) as f64 * 0.66, // Hue: 0.66 (Blue) down to 0.0 (Red)
            0.8, // Saturation
            0.5  // Lightness
        );

        Circle::new((x, y), 2, color.filled())
    }))?;

    // To prevent saving partial files on error
    root.present()?;
    println!("Spectrogram saved to {}", filename);

    Ok(())
}

