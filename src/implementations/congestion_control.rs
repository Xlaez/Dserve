use std::time::{Duration, Instant};

use crate::CongestionControl;

impl CongestionControl {
    fn new() -> Self {
        Self {
            window_size: 1,
            threshold: 16,
            rtt: Duration::from_millis(100),
            rtt_var: Duration::from_millis(50),
            last_window_decrease: Instant::now(),
        }
    }

    fn on_ack(&mut self) {
        if self.window_size < self.threshold {
            self.window_size += 1;
        } else {
            self.window_size += 1 / self.window_size
        }
    }

    fn on_loss(&mut self) {
        self.threshold = self.window_size / 2;
        self.window_size = 1;
        self.last_window_decrease = Instant::now();
    }

    fn update_rrt(&mut self, measured_rtt: Duration) {
        const ALPHA: f32 = 0.125;
        const BETA: f32 = 0.25;

        let rrt_ms = self.rtt.as_secs_f32() * 1000.0;
        let measured_rtt_ms = measured_rtt.as_secs_f32() * 1000.0;
        let diff = measured_rtt_ms - rrt_ms;

        self.rtt = Duration::from_secs_f32((rrt_ms + ALPHA * diff) / 1000.0);
        self.rtt_var = Duration::from_secs_f32(
            (self.rtt_var.as_secs_f32() + BETA * (diff.abs() - self.rtt_var.as_secs_f32()))
                / 1000.0,
        );
    }
}
