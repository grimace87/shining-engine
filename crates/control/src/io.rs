
use window::{KeyCode, KeyState};

/// ControlIo trait
/// Abstraction for an entity that polls and receives input states.
pub trait ControlIo {

    /// Instruct this control to update itself
    fn update(&mut self);

    /// Process a keyboard event
    fn process_keyboard_event(&mut self, keycode: KeyCode, state: KeyState);

    /// Retrieve the left/right direction currently being input
    fn get_dx(&self) -> f32;

    /// Retrieve the up/down direction currently being input
    fn get_dy(&self) -> f32;
}
