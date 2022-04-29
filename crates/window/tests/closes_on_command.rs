
use window::{WindowEventHandler, WindowStateEvent, Window, Command};

#[derive(PartialEq, Debug)]
pub enum TestAppMessage {
    RequestQuit
}

struct DoesNothingApp {}

impl DoesNothingApp {
    fn new() -> Self {
        Self {}
    }
}

impl WindowEventHandler<TestAppMessage> for DoesNothingApp {
    fn on_window_state_event(&self, _event: WindowStateEvent) {}
    fn on_custom_event(&self, _event: TestAppMessage) {}
}

/// Test: send a RequestClose command via the event loop proxy after 1 second.
/// Expected: window opens and then exits after 1 second without user interaction.
fn main() {
    let window = Window::<TestAppMessage>::new();
    let message_proxy = window.new_message_proxy();
    let app = DoesNothingApp::new();
    let join_handle = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        message_proxy.send_event(Command::RequestClose)
            .unwrap();
    });
    window.run(app);
    join_handle.join().unwrap();
}
