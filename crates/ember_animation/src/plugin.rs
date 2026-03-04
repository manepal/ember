use ember_core::app::App;
use ember_core::plugin::Plugin;

/// Plugin that registers animation systems.
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, _app: &mut App) {
        // Animation systems will be registered here once the ECS
        // supports system ordering / stages.
        // For now, SpriteAnimator::update() and AnimationStateMachine::evaluate()
        // are called directly by the user or by dedicated systems.
    }
}
