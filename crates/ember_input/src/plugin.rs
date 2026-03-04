use crate::keyboard::{update_keyboard_state, KeyboardInputEvent, KeyboardState};
use crate::mouse::{
    update_mouse_state, MouseButtonEvent, MouseMoveEvent, MouseScrollEvent, MouseState,
};
use ember_core::app::App;
use ember_core::event::EventReader;
use ember_core::plugin::Plugin;
use ember_core::system::ResMut;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeyboardState::default());
        app.add_event::<KeyboardInputEvent>();
        app.add_system::<fn(ResMut<'static, KeyboardState>, EventReader<'static, 'static, KeyboardInputEvent>), _>(update_keyboard_state);

        app.insert_resource(MouseState::default());
        app.add_event::<MouseButtonEvent>();
        app.add_event::<MouseMoveEvent>();
        app.add_event::<MouseScrollEvent>();
        app.add_system::<fn(
            ResMut<'static, MouseState>,
            EventReader<'static, 'static, MouseButtonEvent>,
            EventReader<'static, 'static, MouseMoveEvent>,
            EventReader<'static, 'static, MouseScrollEvent>,
        ), _>(update_mouse_state);
    }
}
