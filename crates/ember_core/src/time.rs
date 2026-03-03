use std::time::Duration;

/// The central Time resource for tracking application execution duration and frame deltas.
pub struct Time {
    delta: Duration,
    elapsed: Duration,
    fixed_step: Duration,
    accumulator: Duration,
    frame_count: u64,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            delta: Duration::ZERO,
            elapsed: Duration::ZERO,
            // Standard 60 FPS fixed timestep default
            fixed_step: Duration::from_secs_f32(1.0 / 60.0),
            accumulator: Duration::ZERO,
            frame_count: 0,
        }
    }
}

impl Time {
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates the time tracking system with the given duration since the last frame.
    pub fn update(&mut self, delta: Duration) {
        self.delta = delta;
        self.elapsed += delta;
        self.accumulator += delta;
        self.frame_count += 1;
    }

    /// The duration since the last update.
    pub fn delta(&self) -> Duration {
        self.delta
    }

    /// The total duration since the app started.
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// The delta duration in seconds.
    pub fn delta_seconds(&self) -> f32 {
        self.delta.as_secs_f32()
    }

    /// The total elapsed duration in seconds.
    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed.as_secs_f32()
    }

    /// Fixed duration timestep for systems like physics execution.
    pub fn fixed_step(&self) -> Duration {
        self.fixed_step
    }

    /// Sets the fixed logic timestep dynamically.
    pub fn set_fixed_step(&mut self, step: Duration) {
        self.fixed_step = step;
    }

    /// Total number of frames generated since startup.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Returns true if enough time has accumulated to run a fixed timestep update.
    pub fn should_run_fixed_step(&self) -> bool {
        self.accumulator >= self.fixed_step
    }

    /// Consumes one fixed step from the accumulator, returning true if successful.
    pub fn consume_fixed_step(&mut self) -> bool {
        if self.should_run_fixed_step() {
            self.accumulator -= self.fixed_step;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_delta_and_elapsed() {
        let mut time = Time::new();
        
        time.update(Duration::from_millis(16));
        assert_eq!(time.delta().as_millis(), 16);
        assert_eq!(time.elapsed().as_millis(), 16);
        assert_eq!(time.frame_count(), 1);

        time.update(Duration::from_millis(20));
        assert_eq!(time.delta().as_millis(), 20);
        assert_eq!(time.elapsed().as_millis(), 36);
        assert_eq!(time.frame_count(), 2);
    }

    #[test]
    fn time_fixed_timestep_accumulator() {
        let mut time = Time::default();
        time.set_fixed_step(Duration::from_millis(50));

        // Frame 1: 30ms elapsed (total 30ms, not enough for fixed step)
        time.update(Duration::from_millis(30));
        assert!(!time.should_run_fixed_step());
        assert!(!time.consume_fixed_step());

        // Frame 2: 30ms elapsed (total 60ms, enough for one fixed step)
        time.update(Duration::from_millis(30));
        assert!(time.should_run_fixed_step());
        assert!(time.consume_fixed_step());
        
        // After consuming 50ms, 10ms remains
        assert!(!time.should_run_fixed_step());
        assert_eq!(time.accumulator.as_millis(), 10);
    }
}
