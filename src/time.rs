use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeltaTime(pub f32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElapsedTime(pub f32);

pub struct Time {
    start_time: Instant,
    previous_time: Instant,
}

impl Time {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            previous_time: Instant::now(),
        }
    }

    pub fn elapsed_time(&self) -> ElapsedTime {
        ElapsedTime(self.start_time.elapsed().as_secs_f32())
    }

    pub fn delta_time(&mut self) -> DeltaTime {
        let dt = DeltaTime(self.previous_time.elapsed().as_secs_f32());
        self.previous_time = Instant::now();
        dt
    }
}
