use ember_core::event::EventReader;
use ember_core::system::ResMut;
use std::collections::HashSet;
use winit::keyboard::KeyCode as WinitKeyCode;

/// Re-export KeyCode from winit for convenience.
pub type KeyCode = WinitKeyCode;

/// ECS Event sent by the window handler when a keyboard key is pressed or released.
#[derive(Debug, Clone)]
pub struct KeyboardInputEvent {
    pub key_code: KeyCode,
    pub state: winit::event::ElementState,
}

/// Stores the state of the keyboard for the current frame.
#[derive(Debug, Clone, Default)]
pub struct KeyboardState {
    pressed: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    just_released: HashSet<KeyCode>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn is_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    pub fn is_just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }

    /// Update the state based on a key press or release.
    pub fn process_event(&mut self, event: &KeyboardInputEvent) {
        match event.state {
            winit::event::ElementState::Pressed => {
                if !self.pressed.contains(&event.key_code) {
                    self.pressed.insert(event.key_code);
                    self.just_pressed.insert(event.key_code);
                }
            }
            winit::event::ElementState::Released => {
                if self.pressed.contains(&event.key_code) {
                    self.pressed.remove(&event.key_code);
                    self.just_released.insert(event.key_code);
                }
            }
        }
    }

    /// Clear the `just_` states. Should be called at the start of every frame.
    pub fn clear_just_states(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

/// System to poll `KeyboardInputEvent`s and update the `KeyboardState` resource.
pub fn update_keyboard_state(
    mut keyboard: ResMut<KeyboardState>,
    mut events: EventReader<KeyboardInputEvent>,
) {
    keyboard.clear_just_states();

    for event in events.iter() {
        keyboard.process_event(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_state_transitions() {
        let mut kb = KeyboardState::new();

        // 1. Initial press
        kb.process_event(&KeyboardInputEvent {
            key_code: KeyCode::KeyA,
            state: winit::event::ElementState::Pressed,
        });

        assert!(kb.is_pressed(KeyCode::KeyA));
        assert!(kb.is_just_pressed(KeyCode::KeyA));
        assert!(!kb.is_just_released(KeyCode::KeyA));

        // 2. Next frame (hold)
        kb.clear_just_states();
        assert!(kb.is_pressed(KeyCode::KeyA));
        assert!(!kb.is_just_pressed(KeyCode::KeyA));
        assert!(!kb.is_just_released(KeyCode::KeyA));

        // 3. Release
        kb.process_event(&KeyboardInputEvent {
            key_code: KeyCode::KeyA,
            state: winit::event::ElementState::Released,
        });

        assert!(!kb.is_pressed(KeyCode::KeyA));
        assert!(!kb.is_just_pressed(KeyCode::KeyA));
        assert!(kb.is_just_released(KeyCode::KeyA));

        // 4. Next frame (idle)
        kb.clear_just_states();
        assert!(!kb.is_pressed(KeyCode::KeyA));
        assert!(!kb.is_just_pressed(KeyCode::KeyA));
        assert!(!kb.is_just_released(KeyCode::KeyA));
    }
}
