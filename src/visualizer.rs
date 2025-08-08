use eframe::egui;
use egui_plot::{Bar, BarChart, Plot};
use std::sync::{Arc, Mutex};

use crate::histogram_tracker::HistogramTracker;

pub struct HistogramApp {
    histogram_tracker: Arc<Mutex<HistogramTracker>>,
    update_counter: u32,
}

impl HistogramApp {
    pub fn new(histogram_tracker: Arc<Mutex<HistogramTracker>>) -> Self {
        Self {
            histogram_tracker,
            update_counter: 0,
        }
    }
}

impl eframe::App for HistogramApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaints for live updates
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            let tracker = self.histogram_tracker.lock().unwrap();
            let ((pending_data, non_pending_data), _) = tracker.get_aligned_histogram_data();
            let (pending_count, non_pending_count, pending_avg, non_pending_avg) = tracker.get_stats();
            drop(tracker);

            ui.heading("Transaction Confirmation Time Histogram");
            
            ui.horizontal(|ui| {
                ui.label(format!("With Pending: {} txs (avg: {:.1}ms)", pending_count, pending_avg));
                ui.separator();
                ui.label(format!("Without Pending: {} txs (avg: {:.1}ms)", non_pending_count, non_pending_avg));
            });

            if pending_data.is_empty() && non_pending_data.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("Waiting for transaction data...");
                });
                return;
            }

            // Convert data to bar charts
            let mut pending_bars = Vec::new();
            let mut non_pending_bars = Vec::new();

            let max_len = pending_data.len().max(non_pending_data.len());
            for i in 0..max_len {
                let bin_center = if i < pending_data.len() {
                    pending_data[i].0
                } else if i < non_pending_data.len() {
                    non_pending_data[i].0
                } else {
                    continue;
                };

                // Calculate bar width based on spacing
                let bar_width = if pending_data.len() > 1 {
                    (pending_data[1].0 - pending_data[0].0) * 0.35
                } else {
                    5.0 // default width
                };

                // Left bar (red) - pending data
                if i < pending_data.len() && pending_data[i].1 > 0 {
                    pending_bars.push(
                        Bar::new(bin_center - bar_width/2.0, pending_data[i].1 as f64)
                            .width(bar_width)
                            .fill(egui::Color32::from_rgb(255, 0, 0))
                    );
                }

                // Right bar (blue) - non-pending data
                if i < non_pending_data.len() && non_pending_data[i].1 > 0 {
                    non_pending_bars.push(
                        Bar::new(bin_center + bar_width/2.0, non_pending_data[i].1 as f64)
                            .width(bar_width)
                            .fill(egui::Color32::from_rgb(0, 0, 255))
                    );
                }
            }

            Plot::new("histogram")
                .height(600.0)
                .show(ui, |plot_ui| {
                    if !pending_bars.is_empty() {
                        plot_ui.bar_chart(
                            BarChart::new(pending_bars)
                                .color(egui::Color32::from_rgb(255, 0, 0))
                                .name("With Pending Tag")
                        );
                    }

                    if !non_pending_bars.is_empty() {
                        plot_ui.bar_chart(
                            BarChart::new(non_pending_bars)
                                .color(egui::Color32::from_rgb(0, 0, 255))
                                .name("Without Pending Tag")
                        );
                    }
                });

            self.update_counter += 1;
            if self.update_counter % 60 == 0 { // Every ~2 seconds at 30 FPS
                println!(
                    "GUI Update #{} - Pending: {} txs ({:.1}ms avg) | Non-pending: {} txs ({:.1}ms avg)",
                    self.update_counter, pending_count, pending_avg, non_pending_count, non_pending_avg
                );
            }
        });
    }
}

pub struct Visualizer {
    histogram_tracker: Arc<Mutex<HistogramTracker>>,
}

impl Visualizer {
    pub fn new(histogram_tracker: Arc<Mutex<HistogramTracker>>) -> Self {
        Self { histogram_tracker }
    }

    pub fn run(self) -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
            ..Default::default()
        };

        eframe::run_native(
            "Transaction Confirmation Time Histogram",
            options,
            Box::new(|_cc| Ok(Box::new(HistogramApp::new(self.histogram_tracker)))),
        )
    }
}