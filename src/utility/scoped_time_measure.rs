pub struct ScopedTimeMeasure {
    start: std::time::Instant,
}

impl ScopedTimeMeasure {
    fn _new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

impl Drop for ScopedTimeMeasure {
    fn drop(&mut self) {
        println!("Duration: {}", self.start.elapsed().as_micros());
    }
}
