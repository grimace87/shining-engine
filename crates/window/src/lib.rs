
pub use winit::event_loop::EventLoopProxy as MessageProxy;
use winit::{
    event_loop::{EventLoop, ControlFlow},
    event::{Event, WindowEvent}
};
use std::fmt::Debug;

#[derive(PartialEq)]
pub enum WindowStateEvent {
    Starting,
    FocusGained,
    FocusLost,
    Closing
}

#[derive(Debug)]
pub enum Command<T> {
    Custom(T),
    RequestClose
}

pub trait WindowEventHandler<T: 'static> {
    fn on_window_state_event(&self, event: WindowStateEvent);
    fn on_custom_event(&self, event: T);
}

pub struct Window<T: 'static + Send + Debug> {
    event_loop: EventLoop<Command<T>>,
    window: winit::window::Window
}

impl<T: 'static + Send + Debug> Window<T> {
    pub fn new() -> Self {
        let event_loop = EventLoop::with_user_event();
        let window = winit::window::WindowBuilder::new()
            .build(&event_loop)
            .unwrap();
        Self { event_loop, window }
    }

    pub fn new_message_proxy(&self) -> MessageProxy<Command<T>> {
        self.event_loop.create_proxy()
    }

    pub fn run<A: 'static + WindowEventHandler<T>>(self, app: A) {
        let running_window_id = self.window.id();
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::UserEvent(command) => {
                    match command {
                        Command::RequestClose => {
                            *control_flow = ControlFlow::Exit
                        },
                        Command::Custom(e) => {
                            app.on_custom_event(e);
                            ()
                        }
                    }
                },
                Event::WindowEvent { event: WindowEvent::Focused(focused), window_id }
                        if window_id == running_window_id => {
                    match focused {
                        true => app.on_window_state_event(WindowStateEvent::FocusGained),
                        false => app.on_window_state_event(WindowStateEvent::FocusLost)
                    };
                    ()
                },
                Event::WindowEvent { event: WindowEvent::CloseRequested, window_id }
                        if window_id == running_window_id => {
                    app.on_window_state_event(WindowStateEvent::Closing);
                    *control_flow = ControlFlow::Exit
                },
                _ => ()
            }
        });
    }
}
