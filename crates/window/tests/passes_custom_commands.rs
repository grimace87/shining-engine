
use window::{
    RenderCycleEvent, RenderEventHandler,
    WindowEventHandler, WindowStateEvent, Window, MessageProxy, WindowCommand
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
        if event == WindowStateEvent::FocusGained {
            self.message_proxy.send_event(WindowCommand::Custom(TestAppMessage::RequestQuit))
                .unwrap();
        }
    }

    fn on_window_custom_event(&self, event: TestAppMessage) {
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
    let window = Window::<TestAppMessage>::new("App Closes");
    let message_proxy = window.new_message_proxy();
    let app = QuitsQuicklyApp::new(message_proxy.clone());
    window.run(app);
}
