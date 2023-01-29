
use crate::{WindowCommand, KeyCode, KeyState};
use winit::event::Event;
use winit::event_loop::{
    ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget
};
use winit::platform::run_return::EventLoopExtRunReturn;
use std::fmt::Debug;

#[derive(PartialEq)]
pub enum WindowStateEvent {
    Starting,
    FocusGained,
    FocusLost,
    Closing,
    KeyEvent(KeyCode, KeyState)
}

#[derive(PartialEq)]
pub enum RenderCycleEvent {
    PrepareUpdate(u64),
    RenderingFrame,
    RecreatingSurface(f32) // Aspect ratio passed
}

pub trait WindowEventHandler<T: 'static> {
    fn on_window_state_event(&mut self, event: WindowStateEvent);
    fn on_window_custom_event(&mut self, event: T);
}

pub trait RenderEventHandler {
    fn on_render_cycle_event(&self, event: RenderCycleEvent);
}

pub struct WindowEventLooper<M: 'static + Send + Debug> {
    pub(crate) event_loop: EventLoop<WindowCommand<M>>
}

impl<M: 'static + Send + Debug> WindowEventLooper<M> {

    pub fn new() -> Self {
        Self {
            event_loop: EventLoopBuilder::with_user_event().build()
        }
    }

    pub fn create_proxy(&self) -> EventLoopProxy<WindowCommand<M>> {
        self.event_loop.create_proxy()
    }

    pub fn run_loop<F>(mut self, event_handler: F) -> i32
        where F: FnMut(Event<'_, WindowCommand<M>>, &EventLoopWindowTarget<WindowCommand<M>>, &mut ControlFlow)
    {
        self.event_loop.run_return(event_handler)
    }
}
