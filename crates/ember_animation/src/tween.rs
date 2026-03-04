/// Easing function type.
pub type EasingFn = fn(f32) -> f32;

/// Standard easing functions for tweening.
pub mod easing {
    use std::f32::consts::PI;

    pub fn linear(t: f32) -> f32 {
        t
    }

    pub fn ease_in_quad(t: f32) -> f32 {
        t * t
    }

    pub fn ease_out_quad(t: f32) -> f32 {
        1.0 - (1.0 - t) * (1.0 - t)
    }

    pub fn ease_in_out_quad(t: f32) -> f32 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
        }
    }

    pub fn ease_in_cubic(t: f32) -> f32 {
        t * t * t
    }

    pub fn ease_out_cubic(t: f32) -> f32 {
        1.0 - (1.0 - t).powi(3)
    }

    pub fn ease_in_out_cubic(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }

    pub fn bounce_out(t: f32) -> f32 {
        let n1 = 7.5625;
        let d1 = 2.75;
        if t < 1.0 / d1 {
            n1 * t * t
        } else if t < 2.0 / d1 {
            let t = t - 1.5 / d1;
            n1 * t * t + 0.75
        } else if t < 2.5 / d1 {
            let t = t - 2.25 / d1;
            n1 * t * t + 0.9375
        } else {
            let t = t - 2.625 / d1;
            n1 * t * t + 0.984375
        }
    }

    pub fn bounce_in(t: f32) -> f32 {
        1.0 - bounce_out(1.0 - t)
    }

    pub fn elastic_out(t: f32) -> f32 {
        if t == 0.0 || t == 1.0 {
            return t;
        }
        let p = 0.3;
        (2.0f32).powf(-10.0 * t) * ((t - p / 4.0) * (2.0 * PI) / p).sin() + 1.0
    }

    pub fn elastic_in(t: f32) -> f32 {
        if t == 0.0 || t == 1.0 {
            return t;
        }
        1.0 - elastic_out(1.0 - t)
    }

    pub fn back_in(t: f32) -> f32 {
        let c = 1.70158;
        (c + 1.0) * t * t * t - c * t * t
    }

    pub fn back_out(t: f32) -> f32 {
        let c = 1.70158;
        let t = t - 1.0;
        1.0 + (c + 1.0) * t * t * t + c * t * t
    }
}

/// What happens when a tween completes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweenMode {
    /// Play once and stop.
    Once,
    /// Loop from start.
    Loop,
    /// Ping-pong back and forth.
    PingPong,
}

/// A single tween tracking interpolation between two f32 values.
#[derive(Debug, Clone)]
pub struct Tween {
    pub from: f32,
    pub to: f32,
    pub duration: f32,
    pub elapsed: f32,
    pub easing: EasingFn,
    pub mode: TweenMode,
    pub finished: bool,
    /// Direction for PingPong: true = forward, false = reverse.
    forward: bool,
}

impl Tween {
    pub fn new(from: f32, to: f32, duration: f32) -> Self {
        Self {
            from,
            to,
            duration,
            elapsed: 0.0,
            easing: easing::linear,
            mode: TweenMode::Once,
            finished: false,
            forward: true,
        }
    }

    pub fn with_easing(mut self, easing: EasingFn) -> Self {
        self.easing = easing;
        self
    }

    pub fn with_mode(mut self, mode: TweenMode) -> Self {
        self.mode = mode;
        self
    }

    /// Get the current interpolated value.
    pub fn value(&self) -> f32 {
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        let eased = (self.easing)(if self.forward { t } else { 1.0 - t });
        self.from + (self.to - self.from) * eased
    }

    /// Advance the tween by `dt` seconds. Returns the new value.
    pub fn update(&mut self, dt: f32) -> f32 {
        if self.finished {
            return self.value();
        }

        self.elapsed += dt;

        if self.elapsed >= self.duration {
            match self.mode {
                TweenMode::Once => {
                    self.elapsed = self.duration;
                    self.finished = true;
                }
                TweenMode::Loop => {
                    self.elapsed %= self.duration;
                }
                TweenMode::PingPong => {
                    self.elapsed %= self.duration;
                    self.forward = !self.forward;
                }
            }
        }

        self.value()
    }

    /// Reset the tween to its initial state.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.finished = false;
        self.forward = true;
    }
}

/// A sequence of tweens executed one after another.
#[derive(Debug, Clone)]
pub struct TweenSequence {
    pub tweens: Vec<Tween>,
    pub current_index: usize,
    pub finished: bool,
}

impl TweenSequence {
    pub fn new(tweens: Vec<Tween>) -> Self {
        Self {
            tweens,
            current_index: 0,
            finished: false,
        }
    }

    pub fn value(&self) -> f32 {
        self.tweens
            .get(self.current_index)
            .map(|t| t.value())
            .unwrap_or(0.0)
    }

    pub fn update(&mut self, dt: f32) -> f32 {
        if self.finished || self.tweens.is_empty() {
            return self.value();
        }

        let tween = &mut self.tweens[self.current_index];
        let val = tween.update(dt);

        if tween.finished {
            self.current_index += 1;
            if self.current_index >= self.tweens.len() {
                self.current_index = self.tweens.len() - 1;
                self.finished = true;
            }
        }

        val
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
        self.finished = false;
        for tween in &mut self.tweens {
            tween.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_tween_interpolates() {
        let mut tween = Tween::new(0.0, 100.0, 1.0);
        tween.update(0.5);
        assert!((tween.value() - 50.0).abs() < 0.01);
    }

    #[test]
    fn tween_finishes_once() {
        let mut tween = Tween::new(0.0, 1.0, 0.5);
        tween.update(1.0);
        assert!(tween.finished);
        assert!((tween.value() - 1.0).abs() < 0.001);
    }

    #[test]
    fn tween_loops() {
        let mut tween = Tween::new(0.0, 1.0, 0.5).with_mode(TweenMode::Loop);
        tween.update(0.75); // Loops: 0.75 % 0.5 = 0.25 → t = 0.5
        assert!(!tween.finished);
        assert!((tween.value() - 0.5).abs() < 0.01);
    }

    #[test]
    fn tween_pingpong() {
        let mut tween = Tween::new(0.0, 1.0, 1.0).with_mode(TweenMode::PingPong);
        tween.update(1.0); // Complete forward, flip direction
        assert!(!tween.forward);

        // Now going reverse from 1.0 back to 0.0
        tween.update(0.5); // Half reverse → should be around 0.5
        assert!((tween.value() - 0.5).abs() < 0.1);
    }

    #[test]
    fn easing_ease_in_quad() {
        let mut tween = Tween::new(0.0, 1.0, 1.0).with_easing(easing::ease_in_quad);
        tween.update(0.5);
        // ease_in_quad(0.5) = 0.25
        assert!((tween.value() - 0.25).abs() < 0.01);
    }

    #[test]
    fn bounce_easing_endpoints() {
        assert!((easing::bounce_out(0.0)).abs() < 0.001);
        assert!((easing::bounce_out(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn sequence_advances() {
        let mut seq =
            TweenSequence::new(vec![Tween::new(0.0, 1.0, 0.5), Tween::new(1.0, 2.0, 0.5)]);

        seq.update(0.5); // Complete first tween
        assert_eq!(seq.current_index, 1);

        seq.update(0.5); // Complete second tween
        assert!(seq.finished);
        assert!((seq.value() - 2.0).abs() < 0.01);
    }

    #[test]
    fn reset_tween() {
        let mut tween = Tween::new(0.0, 1.0, 0.5);
        tween.update(1.0);
        assert!(tween.finished);
        tween.reset();
        assert!(!tween.finished);
        assert!((tween.value()).abs() < 0.001);
    }
}
