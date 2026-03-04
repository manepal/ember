//! Ember GUI — Immediate-mode GUI system for in-game UI
//!
//! Provides panels, buttons, labels, sliders, and a layout engine.
//! All rendering goes through the engine's own 2D pipeline.

pub mod context;
pub mod font;
pub mod layout;
pub mod overlay;
pub mod plugin;
pub mod render;
pub mod style;
pub mod widgets;

pub use context::*;
pub use layout::*;
pub use plugin::*;
pub use style::*;
pub use widgets::*;
