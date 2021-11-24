use std::time::Instant;

pub fn clamp<T: std::cmp::PartialOrd>(x: T, min: T, max: T) -> T {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

pub struct Timer {
    pub last: std::time::Instant,
}

impl Default for Timer {
    fn default() -> Self {
        Timer { last: Instant::now() }
    }
}

impl Timer {
    pub fn dt(&mut self) -> f32 {
        let current = Instant::now();
        let dt = (current - self.last).as_secs_f32();
        self.last = current;
        dt
    }
}
