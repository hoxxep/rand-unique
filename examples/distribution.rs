use std::error::Error;

use plotters::prelude::*;
use rand::rngs::OsRng;
use rand_sequence::RandomSequence;

/// Generate histogram and scatter plots for a u16 sequence to demonstrate uniformity.
fn main() -> Result<(), Box<dyn Error>> {
    plot_histogram("charts/histogram-u16.png", 50, 5000)?;
    plot_scatter("charts/scatter-u16.png", 1000)?;
    Ok(())
}

/// Plot the distribution of sequence outputs.
fn plot_histogram(file_name: &str, bins: u32, count: u32) -> Result<(), Box<dyn Error>> {
    type T = u16;

    let root = BitMapBackend::new(file_name, (1280, 960)).into_drawing_area();
    root.fill(&WHITE)?;

    let sequence = RandomSequence::<T>::rand(&mut OsRng);
    let data: Vec<T> = sequence.take(count as usize).collect();
    let binned: Vec<(u32, u32)> =
        data.iter().map(|x| ((*x as f64 / (T::MAX as f64 / bins as f64)) as u32, 1)).collect();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .margin(25)
        .caption("RandomSequence distribution (n=5000, bins=50)", ("sans-serif", 25.0))
        .build_cartesian_2d((0u32..bins).into_segmented(), 0u32..(1.5 * (count / bins) as f64) as u32)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Count")
        .x_desc("Bucket")
        .axis_desc_style(("sans-serif", 20))
        .draw()?;

    chart.draw_series(Histogram::vertical(&chart).style(RED.filled()).data(binned))?;

    root.present().expect(&format!("Unable to write result to file {file_name}"));
    println!("Histogram has been saved to {file_name}");

    Ok(())
}

/// Plot the outputs in a scatter.
fn plot_scatter(file_name: &str, count: u32) -> Result<(), Box<dyn Error>> {
    type T = u16;

    let root = BitMapBackend::new(file_name, (1280, 960)).into_drawing_area();
    root.fill(&WHITE)?;

    let sequence = RandomSequence::<T>::rand(&mut OsRng);
    let data: Vec<(u32, u32)> = sequence.take(count as usize).enumerate().map(|(i, o)| (i as u32, o as u32)).collect();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .margin(25)
        .caption("RandomSequence Outputs (n=1000)", ("sans-serif", 25))
        .build_cartesian_2d(0u32..count, 0u32..(1.1 * T::MAX as f64) as u32)?;

    chart
        .configure_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Iteration")
        .x_desc("Output")
        .axis_desc_style(("sans-serif", 20))
        .draw()
        .unwrap();

    chart.draw_series(data.iter().map(|point| Circle::new(*point, 2, &BLUE)))?;

    root.present().expect(&format!("Unable to write result to file {file_name}"));
    println!("Scatter has been saved to {file_name}");

    Ok(())
}
