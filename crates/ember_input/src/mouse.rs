use ember_core::event::EventReader;
use ember_core::system::ResMut;
use glam::Vec2;
use std::collections::HashSet;
pub use winit::event::MouseButton;

/// ECS Event sent by the window handler for mouse button presses/releases.
#[derive(Debug, Clone)]
pub struct MouseButtonEvent {
    pub button: MouseButton,
    pub state: winit::event::ElementState,
}

/// ECS Event sent by the window handler when the mouse moves.
#[derive(Debug, Clone)]
pub struct MouseMoveEvent {
    pub position: Vec2,
    pub delta: Vec2,
}

/// ECS Event sent by the window handler when the mouse scrolls.
#[derive(Debug, Clone)]
pub struct MouseScrollEvent {
    pub delta: Vec2,
}

/// Stores the state of the mouse for the current frame.
#[derive(Debug, Clone, Default)]
pub struct MouseState {
    pressed: HashSet<MouseButton>,
    just_pressed: HashSet<MouseButton>,
    just_released: HashSet<MouseButton>,
    pub position: Vec2,
    pub delta: Vec2,
    pub scroll: Vec2,
}

impl MouseState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.pressed.contains(&button)
    }

    pub fn is_just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed.contains(&button)
    }

    pub fn is_just_released(&self, button: MouseButton) -> bool {
        self.just_released.contains(&button)
    }

    /// Update the state based on a button press or release.
    pub fn process_button_event(&mut self, event: &MouseButtonEvent) {
        match event.state {
            winit::event::ElementState::Pressed => {
                if !self.pressed.contains(&event.button) {
                    self.pressed.insert(event.button);
                    self.just_pressed.insert(event.button);
                }
            }
            winit::event::ElementState::Released => {
                if self.pressed.contains(&event.button) {
                    self.pressed.remove(&event.button);
                    self.just_released.insert(event.button);
                }
            }
        }
    }

    /// Update the state based on a mouse move event.
    pub fn process_move_event(&mut self, event: &MouseMoveEvent) {
        self.position = event.position;
        self.delta += event.delta;
    }

    /// Update the state based on a mouse scroll event.
    pub fn process_scroll_event(&mut self, event: &MouseScrollEvent) {
        self.scroll += event.delta;
    }

    /// Clear the `just_` states and framewise deltas. Should be called at the start of every frame.
    pub fn clear_frame_state(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.delta = Vec2::ZERO;
        self.scroll = Vec2::ZERO;
    }
}

/// System to poll mouse events and update the `MouseState` resource.
pub fn update_mouse_state(
    mut mouse: ResMut<MouseState>,
    mut button_events: EventReader<MouseButtonEvent>,
    mut move_events: EventReader<MouseMoveEvent>,
    mut scroll_events: EventReader<MouseScrollEvent>,
) {
    mouse.clear_frame_state();

    for event in button_events.iter() {
        mouse.process_button_event(event);
    }

    for event in move_events.iter() {
        mouse.process_move_event(event);
    }

    for event in scroll_events.iter() {
        mouse.process_scroll_event(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_state_transitions() {
        let mut mouse = MouseState::new();

        // 1. Initial press
        mouse.process_button_event(&MouseButtonEvent {
            button: MouseButton::Left,
            state: winit::event::ElementState::Pressed,
        });

        assert!(mouse.is_pressed(MouseButton::Left));
        assert!(mouse.is_just_pressed(MouseButton::Left));
        assert!(!mouse.is_just_released(MouseButton::Left));

        // 2. Next frame (hold)
        mouse.clear_frame_state();
        assert!(mouse.is_pressed(MouseButton::Left));
        assert!(!mouse.is_just_pressed(MouseButton::Left));
        assert!(!mouse.is_just_released(MouseButton::Left));

        // 3. Move and Scroll
        mouse.process_move_event(&MouseMoveEvent {
            position: Vec2::new(100.0, 200.0),
            delta: Vec2::new(10.0, -5.0),
        });
        mouse.process_scroll_event(&MouseScrollEvent {
            delta: Vec2::new(0.0, 1.0),
        });

        assert_eq!(mouse.position, Vec2::new(100.0, 200.0));
        assert_eq!(mouse.delta, Vec2::new(10.0, -5.0));
        assert_eq!(mouse.scroll, Vec2::new(0.0, 1.0));

        // 4. Release
        mouse.process_button_event(&MouseButtonEvent {
            button: MouseButton::Left,
            state: winit::event::ElementState::Released,
        });

        assert!(!mouse.is_pressed(MouseButton::Left));
        assert!(!mouse.is_just_pressed(MouseButton::Left));
        assert!(mouse.is_just_released(MouseButton::Left));

        // 5. Next frame (idle)
        mouse.clear_frame_state();
        assert!(!mouse.is_pressed(MouseButton::Left));
        assert!(!mouse.is_just_pressed(MouseButton::Left));
        assert!(!mouse.is_just_released(MouseButton::Left));
        assert_eq!(mouse.delta, Vec2::ZERO);
        assert_eq!(mouse.scroll, Vec2::ZERO);
    }
}
