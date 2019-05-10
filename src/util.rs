use std::time::Instant;

pub struct Timer {
    instant: Instant,
}

impl Timer {
    pub fn start() -> Timer {
        Timer { instant: Instant::now() }
    }

    pub fn end(&self) -> String {
        let end = self.instant.elapsed();
        format!("{}.{:03} sec", end.as_secs(), end.subsec_nanos() / 1_000_000)
    }
}
