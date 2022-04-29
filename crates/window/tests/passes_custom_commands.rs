
use window::{WindowEventHandler, WindowStateEvent, Window, MessageProxy, Command};

#[derive(PartialEq, Debug)]
pub enum TestAppMessage {
    RequestQuit
}

struct QuitsQuicklyApp {
    message_proxy: MessageProxy<Command<TestAppMessage>>
}

impl QuitsQuicklyApp {
    fn new(message_proxy: MessageProxy<Command<TestAppMessage>>) -> Self {
        Self { message_proxy }
    }
}

impl WindowEventHandler<TestAppMessage> for QuitsQuicklyApp {

    fn on_window_state_event(&self, event: WindowStateEvent) {
        if event == WindowStateEvent::FocusGained {
            self.message_proxy.send_event(Command::Custom(TestAppMessage::RequestQuit))
                .unwrap();
        }
    }

    fn on_custom_event(&self, event: TestAppMessage) {
        if event == TestAppMessage::RequestQuit {
            self.message_proxy.send_event(Command::RequestClose)
                .unwrap();
        }
    }
}

/// Test: intercept window event, and request for the window to exit.
/// Expected: window opens and then exits very quickly without user interaction.
fn main() {
    let window = Window::<TestAppMessage>::new();
    let message_proxy = window.new_message_proxy();
    let app = QuitsQuicklyApp::new(message_proxy.clone());
    window.run(app);
}
