mod window;
pub mod event;

pub use crate::window::Window;
pub use crate::event::{
    WindowEventLooper, RenderCycleEvent, WindowStateEvent, RenderEventHandler, WindowEventHandler
};

pub use winit::dpi::PhysicalSize;
pub use winit::event::VirtualKeyCode as KeyCode;
pub use winit::event::ElementState as KeyState;
pub use winit::event_loop::EventLoopProxy as MessageProxy;
pub use winit::event::{Event, WindowEvent, KeyboardInput};
pub use winit::event_loop::ControlFlow;

use std::fmt::Debug;

#[derive(Debug)]
pub enum WindowCommand<T> {
    Custom(T),
    RequestRedraw,
    RequestClose
}
