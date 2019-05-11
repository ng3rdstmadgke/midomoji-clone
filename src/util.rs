use std::time::{Duration, Instant};

pub struct Timer {
    instant: Option<Instant>,
    duration: Duration
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            instant: None,
            duration: Duration::new(0, 0),
        }
    }

    pub fn start(&mut self) {
        self.instant = Some(Instant::now());
    }

    pub fn stop(&mut self) {
        if let Some(instant) = self.instant {
            self.duration += instant.elapsed();
        }
        self.instant  = None;
    }

    pub fn reset(&mut self) {
        self.instant = None;
        self.duration = Duration::new(0, 0);
    }

    pub fn print(&self) {
        println!("{:?}", self.duration)
    }
}
