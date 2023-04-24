pub struct Timer {
    name: &'static str,
    timer: std::time::Instant,
}

impl Timer {
    pub fn start(name: &'static str) -> Self {
        Self {
            name,
            timer: std::time::Instant::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        println!(
            "{} took {} microseconds",
            self.name,
            self.timer.elapsed().as_micros()
        );
    }
}
