use std::collections::HashMap;

/// A single animation clip defining a sequence of frames.
#[derive(Debug, Clone)]
pub struct SpriteAnimationClip {
    /// Name of this clip (e.g., "idle", "walk", "attack").
    pub name: String,
    /// Frame indices into a TextureAtlas.
    pub frames: Vec<usize>,
    /// Duration of each frame in seconds.
    pub frame_duration: f32,
    /// Whether the animation loops.
    pub looping: bool,
}

impl SpriteAnimationClip {
    pub fn new(name: &str, frames: Vec<usize>, frame_duration: f32, looping: bool) -> Self {
        Self {
            name: name.to_string(),
            frames,
            frame_duration,
            looping,
        }
    }

    /// Total duration of one playthrough.
    pub fn total_duration(&self) -> f32 {
        self.frames.len() as f32 * self.frame_duration
    }
}

/// Playback state for a sprite animator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Finished,
}

/// Component that drives sprite sheet animation on an entity.
#[derive(Debug, Clone)]
pub struct SpriteAnimator {
    /// Available animation clips, keyed by name.
    pub clips: HashMap<String, SpriteAnimationClip>,
    /// Name of the currently active clip.
    pub current_clip: String,
    /// Accumulated time within the current clip.
    pub timer: f32,
    /// Current frame index within the clip's frames array.
    pub frame_index: usize,
    /// Playback speed multiplier (1.0 = normal).
    pub speed: f32,
    /// Current playback state.
    pub state: PlaybackState,
}

impl SpriteAnimator {
    pub fn new(clips: Vec<SpriteAnimationClip>, initial_clip: &str) -> Self {
        let clip_map: HashMap<String, SpriteAnimationClip> =
            clips.into_iter().map(|c| (c.name.clone(), c)).collect();

        Self {
            clips: clip_map,
            current_clip: initial_clip.to_string(),
            timer: 0.0,
            frame_index: 0,
            speed: 1.0,
            state: PlaybackState::Playing,
        }
    }

    /// Get the current atlas frame index.
    pub fn current_atlas_index(&self) -> Option<usize> {
        self.clips
            .get(&self.current_clip)
            .and_then(|clip| clip.frames.get(self.frame_index).copied())
    }

    /// Switch to a different clip, resetting the timer.
    pub fn set_clip(&mut self, name: &str) {
        if self.current_clip != name && self.clips.contains_key(name) {
            self.current_clip = name.to_string();
            self.timer = 0.0;
            self.frame_index = 0;
            self.state = PlaybackState::Playing;
        }
    }

    pub fn play(&mut self) {
        self.state = PlaybackState::Playing;
    }

    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    /// Advance the animation by `dt` seconds. Returns the new atlas frame index.
    pub fn update(&mut self, dt: f32) -> Option<usize> {
        if self.state != PlaybackState::Playing {
            return self.current_atlas_index();
        }

        let clip = self.clips.get(&self.current_clip)?;

        if clip.frames.is_empty() {
            return None;
        }

        self.timer += dt * self.speed;

        // Calculate which frame we should be on
        let frame_count = clip.frames.len();
        let total_frames_elapsed = (self.timer / clip.frame_duration) as usize;

        if clip.looping {
            self.frame_index = total_frames_elapsed % frame_count;
        } else if total_frames_elapsed >= frame_count {
            self.frame_index = frame_count - 1;
            self.state = PlaybackState::Finished;
        } else {
            self.frame_index = total_frames_elapsed;
        }

        self.current_atlas_index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_animator() -> SpriteAnimator {
        let walk = SpriteAnimationClip::new("walk", vec![0, 1, 2, 3], 0.1, true);
        let attack = SpriteAnimationClip::new("attack", vec![4, 5, 6], 0.15, false);
        SpriteAnimator::new(vec![walk, attack], "walk")
    }

    #[test]
    fn initial_frame_is_first() {
        let anim = make_test_animator();
        assert_eq!(anim.current_atlas_index(), Some(0));
        assert_eq!(anim.frame_index, 0);
    }

    #[test]
    fn advances_frame_over_time() {
        let mut anim = make_test_animator();
        anim.update(0.1); // Should move to frame 1
        assert_eq!(anim.frame_index, 1);
    }

    #[test]
    fn looping_wraps_around() {
        let mut anim = make_test_animator();
        anim.update(0.45); // 4 frames × 0.1s = 0.4s total, 0.45 → wraps to frame 0
        assert_eq!(anim.frame_index, 0); // Wrapped
    }

    #[test]
    fn non_looping_finishes() {
        let mut anim = make_test_animator();
        anim.set_clip("attack");
        anim.update(0.5); // 3 frames × 0.15s = 0.45s, exceed → finished
        assert_eq!(anim.state, PlaybackState::Finished);
        assert_eq!(anim.frame_index, 2); // Last frame
    }

    #[test]
    fn pause_stops_advancing() {
        let mut anim = make_test_animator();
        anim.pause();
        anim.update(0.5);
        assert_eq!(anim.frame_index, 0); // Didn't advance
    }

    #[test]
    fn speed_multiplier_works() {
        let mut anim = make_test_animator();
        anim.set_speed(2.0);
        anim.update(0.05); // Effective: 0.1s → frame 1
        assert_eq!(anim.frame_index, 1);
    }

    #[test]
    fn set_clip_resets_state() {
        let mut anim = make_test_animator();
        anim.update(0.2);
        assert!(anim.frame_index > 0);
        anim.set_clip("attack");
        assert_eq!(anim.frame_index, 0);
        assert_eq!(anim.timer, 0.0);
    }
}
