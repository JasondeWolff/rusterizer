use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct Timer {
    start: SystemTime
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            start: SystemTime::now()
        }
    }

    pub fn elapsed(&self) -> f64 {
        self.start.elapsed().unwrap().as_secs_f64()
    }

    pub fn reset(&mut self) {
        self.start = SystemTime::now();
    }
}