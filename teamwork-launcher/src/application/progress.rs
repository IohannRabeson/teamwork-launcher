#[derive(Default)]
pub struct Progress {
    current: u64,
    total: u64,
}

impl Progress {
    pub fn reset(&mut self) {
        self.current = 0;
        self.total = 0;
    }

    pub fn increment_total(&mut self) {
        self.total += 1;
    }

    pub fn increment_current(&mut self) {
        self.current += 1;
    }

    pub fn current_progress(&self) -> f32 {
        if self.total == 0 {
            return 0.0
        }

        self.current as f32 / self.total as f32
    }

    pub fn is_finished(&self) -> bool {
        self.current >= self.total
    }
}