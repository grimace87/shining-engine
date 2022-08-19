
use window::{
    RenderEventHandler, RenderCycleEvent,
    WindowEventHandler, WindowStateEvent, Window, MessageProxy, WindowCommand,
    KeyCode, KeyState
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

    fn on_window_state_event(&self, event: WindowStateEvent) {
        if let WindowStateEvent::KeyEvent(KeyCode::Escape, KeyState::Pressed) = event {
            self.message_proxy.send_event(WindowCommand::RequestClose)
                .unwrap();
        }
    }

    fn on_window_custom_event(&self, _event: TestAppMessage) {}
}

impl RenderEventHandler for QuitsQuicklyApp {

    fn on_render_cycle_event(&self, event: RenderCycleEvent) {
        match event {
            RenderCycleEvent::PrepareUpdate => {
                self.message_proxy.send_event(WindowCommand::RequestRedraw)
                    .unwrap();
            },
            _ => {}
        }
    }
}

// Current setup will intercept a FocusGained state event, then post a custom message.
// This custom message will also be intercepted, at which point a RequestClose command is sent.
fn main() {
    let window = Window::<TestAppMessage>::new("Demo App");
    let message_proxy = window.new_message_proxy();
    let app = QuitsQuicklyApp::new(message_proxy.clone());
    window.run(app);
}
