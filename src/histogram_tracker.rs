use std::time::Duration;

pub struct HistogramTracker {
    pending_times: Vec<f64>,
    non_pending_times: Vec<f64>,
}

impl HistogramTracker {
    pub fn new() -> Self {
        Self {
            pending_times: Vec::new(),
            non_pending_times: Vec::new(),
        }
    }

    pub fn record_transaction(&mut self, duration: Duration, use_pending: bool) {
        let duration_ms = duration.as_millis() as f64;
        
        if use_pending {
            self.pending_times.push(duration_ms);
        } else {
            self.non_pending_times.push(duration_ms);
        }

        if self.pending_times.len() > 1000 {
            self.pending_times.remove(0);
        }
        if self.non_pending_times.len() > 1000 {
            self.non_pending_times.remove(0);
        }
    }


    pub fn get_aligned_histogram_data(&self) -> ((Vec<(f64, usize)>, Vec<(f64, usize)>), (f64, f64)) {
        self.create_aligned_histogram_bins()
    }

    fn create_aligned_histogram_bins(&self) -> ((Vec<(f64, usize)>, Vec<(f64, usize)>), (f64, f64)) {
        let all_data: Vec<f64> = self.pending_times.iter()
            .chain(self.non_pending_times.iter())
            .cloned()
            .collect();

        if all_data.is_empty() {
            return ((Vec::new(), Vec::new()), (0.0, 0.0));
        }

        let min_val = all_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = all_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if min_val == max_val {
            let pending_count = if self.pending_times.contains(&min_val) { self.pending_times.len() } else { 0 };
            let non_pending_count = if self.non_pending_times.contains(&min_val) { self.non_pending_times.len() } else { 0 };
            return ((vec![(min_val, pending_count)], vec![(min_val, non_pending_count)]), (min_val, max_val));
        }

        let num_bins = 20;
        let bin_width = (max_val - min_val) / num_bins as f64;
        let mut pending_bins = vec![0; num_bins];
        let mut non_pending_bins = vec![0; num_bins];

        // Fill pending bins
        for &value in &self.pending_times {
            let bin_index = ((value - min_val) / bin_width).floor() as usize;
            let bin_index = bin_index.min(num_bins - 1);
            pending_bins[bin_index] += 1;
        }

        // Fill non-pending bins
        for &value in &self.non_pending_times {
            let bin_index = ((value - min_val) / bin_width).floor() as usize;
            let bin_index = bin_index.min(num_bins - 1);
            non_pending_bins[bin_index] += 1;
        }

        let pending_data: Vec<(f64, usize)> = pending_bins.into_iter()
            .enumerate()
            .map(|(i, count)| {
                let bin_center = min_val + (i as f64 + 0.5) * bin_width;
                (bin_center, count)
            })
            .collect();

        let non_pending_data: Vec<(f64, usize)> = non_pending_bins.into_iter()
            .enumerate()
            .map(|(i, count)| {
                let bin_center = min_val + (i as f64 + 0.5) * bin_width;
                (bin_center, count)
            })
            .collect();

        ((pending_data, non_pending_data), (min_val, max_val))
    }


    pub fn get_stats(&self) -> (usize, usize, f64, f64) {
        let pending_count = self.pending_times.len();
        let non_pending_count = self.non_pending_times.len();
        
        let pending_avg = if pending_count > 0 {
            self.pending_times.iter().sum::<f64>() / pending_count as f64
        } else {
            0.0
        };
        
        let non_pending_avg = if non_pending_count > 0 {
            self.non_pending_times.iter().sum::<f64>() / non_pending_count as f64
        } else {
            0.0
        };

        (pending_count, non_pending_count, pending_avg, non_pending_avg)
    }
}