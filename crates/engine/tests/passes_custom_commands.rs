
use window::{
    RenderCycleEvent, WindowStateEvent, Window, WindowCommand, WindowEventLooper, MessageProxy,
    Event, WindowEvent, ControlFlow,
    RenderEventHandler, WindowEventHandler
};

#[derive(PartialEq, Debug)]
pub enum TestAppMessage {
    RequestQuit
}

struct QuitsQuicklyApp {
    message_proxy: MessageProxy<WindowCommand<TestAppMessage>>
}

impl QuitsQuicklyApp {
    fn new(message_proxy: MessageProxy<WindowCommand<TestAppMessage>>) -> Self {
        Self { message_proxy }
    }
}

impl WindowEventHandler<TestAppMessage> for QuitsQuicklyApp {

    fn on_window_state_event(&mut self, event: WindowStateEvent) {
        if event == WindowStateEvent::FocusGained {
            self.message_proxy.send_event(WindowCommand::Custom(TestAppMessage::RequestQuit))
                .unwrap();
        }
    }

    fn on_window_custom_event(&mut self, event: TestAppMessage) {
        if event == TestAppMessage::RequestQuit {
            self.message_proxy.send_event(WindowCommand::RequestClose)
                .unwrap();
        }
    }
}

impl RenderEventHandler for QuitsQuicklyApp {
    fn on_render_cycle_event(&self, _event: RenderCycleEvent) {}
}

/// Test: intercept window event, and request for the window to exit.
/// Expected: window opens and then exits very quickly without user interaction.
fn main() {
    let looper = WindowEventLooper::<TestAppMessage>::new();
    let message_proxy = looper.create_proxy();
    let mut app = QuitsQuicklyApp::new(message_proxy);
    let window = Window::new("App Closes", &looper);
    let running_window_id = window.get_window_id();
    let _code = looper.run_loop(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(command) => {
                match command {
                    WindowCommand::RequestClose => {
                        *control_flow = ControlFlow::Exit
                    },
                    WindowCommand::Custom(e) => {
                        app.on_window_custom_event(e);
                        ()
                    },
                    _ => {}
                }
            },
            Event::WindowEvent { event, window_id }
            if window_id == running_window_id => {
                match event {
                    WindowEvent::CloseRequested => { *control_flow = ControlFlow::Exit; },
                    WindowEvent::Focused(focused) => {
                        match focused {
                            true => app.on_window_state_event(WindowStateEvent::FocusGained),
                            false => app.on_window_state_event(WindowStateEvent::FocusLost)
                        };
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    });
}
