
pub use winit::event_loop::EventLoopProxy as MessageProxy;
pub use winit::event::VirtualKeyCode as KeyCode;
pub use winit::event::ElementState as KeyState;

use winit::{
    event_loop::{EventLoop, ControlFlow},
    event::{Event, KeyboardInput, WindowEvent}
};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, HasRawDisplayHandle, RawDisplayHandle};
use std::fmt::Debug;

#[derive(PartialEq)]
pub enum WindowStateEvent {
    Starting,
    FocusGained,
    FocusLost,
    Closing,
    KeyEvent(KeyCode, KeyState)
}

#[derive(Debug)]
pub enum WindowCommand<T> {
    Custom(T),
    RequestRedraw,
    RequestClose
}

#[derive(PartialEq)]
pub enum RenderCycleEvent {
    PrepareUpdate,
    RenderFrame,
    RecreateSurface
}

pub trait WindowEventHandler<T: 'static> {
    fn on_window_state_event(&mut self, event: WindowStateEvent);
    fn on_window_custom_event(&mut self, event: T);
}

pub trait RenderEventHandler {
    fn on_render_cycle_event(&self, event: RenderCycleEvent);
}

pub struct Window<T: 'static + Send + Debug> {
    event_loop: EventLoop<WindowCommand<T>>,
    window: winit::window::Window
}

impl<T: 'static + Send + Debug> Window<T> {

    pub fn new(app_title: &str) -> Self {
        let event_loop = EventLoop::with_user_event();
        let window = winit::window::WindowBuilder::new()
            .with_title(app_title)
            .build(&event_loop)
            .unwrap();
        Self { event_loop, window }
    }

    pub fn new_message_proxy(&self) -> MessageProxy<WindowCommand<T>> {
        self.event_loop.create_proxy()
    }

    pub fn run<A: 'static + WindowEventHandler<T> + RenderEventHandler>(self, mut app: A) {
        let running_window_id = self.window.id();
        app.on_window_state_event(WindowStateEvent::Starting);
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::UserEvent(command) => {
                    match command {
                        WindowCommand::RequestClose => {
                            *control_flow = ControlFlow::Exit
                        },
                        WindowCommand::RequestRedraw => {
                            self.window.request_redraw();
                        },
                        WindowCommand::Custom(e) => {
                            app.on_window_custom_event(e);
                            ()
                        }
                    }
                },
                Event::WindowEvent { event, window_id }
                    if window_id == running_window_id =>
                {
                    match event {
                        WindowEvent::KeyboardInput { input, .. } => {
                            let KeyboardInput { virtual_keycode, state, .. } = input;
                            match (virtual_keycode, state) {
                                (Some(KeyCode::Escape), KeyState::Pressed) => {
                                    *control_flow = ControlFlow::Exit;
                                },
                                (Some(keycode), state) => {
                                    app.on_window_state_event(
                                        WindowStateEvent::KeyEvent(
                                            keycode,
                                            state));
                                },
                                _ => {}
                            };
                        },
                        WindowEvent::Focused(focused) => {
                            match focused {
                                true => app.on_window_state_event(WindowStateEvent::FocusGained),
                                false => app.on_window_state_event(WindowStateEvent::FocusLost)
                            };
                        },
                        WindowEvent::CloseRequested => {
                            app.on_window_state_event(WindowStateEvent::Closing);
                            *control_flow = ControlFlow::Exit;
                        },
                        WindowEvent::Resized(_) => {
                            app.on_render_cycle_event(RenderCycleEvent::RecreateSurface);
                        },
                        _ => {}
                    };
                },
                Event::MainEventsCleared => {
                    app.on_render_cycle_event(RenderCycleEvent::PrepareUpdate);
                },
                Event::RedrawRequested(_) => {
                    app.on_render_cycle_event(RenderCycleEvent::RenderFrame);
                },
                _ => ()
            }
        });
    }
}

unsafe impl<T: Send + Debug> HasRawDisplayHandle for Window<T> {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.window.raw_display_handle()
    }
}

unsafe impl<T: Send + Debug> HasRawWindowHandle for Window<T> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.raw_window_handle()
    }
}
