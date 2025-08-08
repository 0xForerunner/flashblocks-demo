use plotters::prelude::*;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

use crate::histogram_tracker::HistogramTracker;

pub struct Visualizer {
    histogram_tracker: Arc<Mutex<HistogramTracker>>,
}

impl Visualizer {
    pub fn new(histogram_tracker: Arc<Mutex<HistogramTracker>>) -> Self {
        Self { histogram_tracker }
    }

    pub async fn run(&self) {
        let mut iteration = 0u32;
        
        loop {
            if let Err(e) = self.update_plots(iteration).await {
                eprintln!("Error updating plots: {}", e);
            }
            
            iteration += 1;
            sleep(Duration::from_secs(2)).await;
        }
    }

    async fn update_plots(&self, iteration: u32) -> Result<(), Box<dyn std::error::Error>> {
        let tracker = self.histogram_tracker.lock().unwrap();
        let pending_data = tracker.get_pending_histogram_data();
        let non_pending_data = tracker.get_non_pending_histogram_data();
        let (pending_count, non_pending_count, pending_avg, non_pending_avg) = tracker.get_stats();
        drop(tracker);

        if pending_data.is_empty() && non_pending_data.is_empty() {
            return Ok(());
        }

        let filename = format!("histogram_plot_{:04}.png", iteration);
        let root = BitMapBackend::new(&filename, (1200, 800)).into_drawing_area();
        root.fill(&WHITE)?;

        let areas = root.split_evenly((2, 1));
        let upper = &areas[0];
        let lower = &areas[1];

        self.draw_histogram(
            upper.clone(),
            &pending_data,
            "Transaction Confirmation Time (With Pending Tag)",
            RED,
            pending_count,
            pending_avg,
        )?;

        self.draw_histogram(
            lower.clone(),
            &non_pending_data,
            "Transaction Confirmation Time (Without Pending Tag)",
            BLUE,
            non_pending_count,
            non_pending_avg,
        )?;

        root.present()?;

        if iteration % 10 == 0 {
            println!(
                "Stats - Pending: {} txs (avg: {:.1}ms), Non-pending: {} txs (avg: {:.1}ms)",
                pending_count, pending_avg, non_pending_count, non_pending_avg
            );
        }

        Ok(())
    }

    fn draw_histogram<DB: DrawingBackend>(
        &self,
        drawing_area: DrawingArea<DB, plotters::coord::Shift>,
        data: &[(f64, usize)],
        title: &str,
        color: RGBColor,
        count: usize,
        avg: f64,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        DB::ErrorType: 'static,
    {
        if data.is_empty() {
            let mut chart = ChartBuilder::on(&drawing_area)
                .caption(
                    &format!("{} (No data yet)", title),
                    ("sans-serif", 30).into_font(),
                )
                .build_cartesian_2d(0f64..100f64, 0..1)?;
            chart.configure_mesh().draw()?;
            return Ok(());
        }

        let max_time = data.iter().map(|(time, _)| *time).fold(0.0, f64::max);
        let max_count = data.iter().map(|(_, count)| *count).max().unwrap_or(1);

        let full_title = format!(
            "{} (Count: {}, Avg: {:.1}ms)",
            title, count, avg
        );

        let mut chart = ChartBuilder::on(&drawing_area)
            .caption(&full_title, ("sans-serif", 30).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0f64..max_time * 1.1, 0..max_count)?;

        chart
            .configure_mesh()
            .x_desc("Time (ms)")
            .y_desc("Count")
            .draw()?;

        let histogram_data: Vec<_> = data
            .iter()
            .map(|&(time, count)| {
                let bar_width = if data.len() > 1 {
                    (data[1].0 - data[0].0) * 0.8
                } else {
                    max_time * 0.05
                };
                Rectangle::new([(time - bar_width / 2.0, 0), (time + bar_width / 2.0, count)], color.filled())
            })
            .collect();

        chart.draw_series(histogram_data)?;

        Ok(())
    }
}