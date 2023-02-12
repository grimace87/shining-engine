
use crate::io::ControlIo;
use window::{KeyCode, KeyState};

/// UserControl struct
/// Handles left/right/up/down inputs from a keyboard
pub struct UserControl {
    dx: f32,
    dy: f32
}

impl UserControl {

    /// Construct new instance, initially with no inputs signalled
    pub fn new() -> Self {
        Self {
            dx: 0.0,
            dy: 0.0
        }
    }
}

impl ControlIo for UserControl {

    /// No-op; nothing can be done internally here
    fn update(&mut self) {}

    /// Update internal fields in response to individual keystroke events
    fn process_keyboard_event(&mut self, keycode: KeyCode, state: KeyState) {
        match keycode {
            KeyCode::Left => {
                self.dx = match state {
                    KeyState::Pressed => -1.0,
                    KeyState::Released => 0.0
                };
            },
            KeyCode::Right => {
                self.dx = match state {
                    KeyState::Pressed => 1.0,
                    KeyState::Released => 0.0
                };
            },
            KeyCode::Up => {
                self.dy = match state {
                    KeyState::Pressed => 1.0,
                    KeyState::Released => 0.0
                };
            },
            KeyCode::Down => {
                self.dy = match state {
                    KeyState::Pressed => -1.0,
                    KeyState::Released => 0.0
                };
            },
            _ => {}
        }
    }

    /// Retrieve the left/right input position
    fn get_dx(&self) -> f32 {
        self.dx
    }

    /// Retrieve the up/down input position; positive is considered to be 'up'
    fn get_dy(&self) -> f32 {
        self.dy
    }
}
