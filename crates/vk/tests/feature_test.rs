use std::fmt::Debug;
use window::{
    RenderCycleEvent, RenderEventHandler,
    WindowEventHandler, WindowStateEvent, Window, WindowCommand
};

#[derive(PartialEq, Debug)]
pub enum TestAppMessage {
    RequestQuit
}

struct VulkanTestApp {}

impl VulkanTestApp {

    fn new<T: Send + Debug>(window: &Window<T>) -> Self {
        unsafe {
            vk::VkCore::new(window, vec![]).unwrap();
        }
        Self {}
    }
}

impl WindowEventHandler<TestAppMessage> for VulkanTestApp {
    fn on_window_state_event(&self, _event: WindowStateEvent) {}
    fn on_window_custom_event(&self, _event: TestAppMessage) {}
}

impl RenderEventHandler for VulkanTestApp {
    fn on_render_cycle_event(&self, _event: RenderCycleEvent) {}
}

/// Test: send a RequestClose command via the event loop proxy after 1 second.
/// Expected: window opens and then exits after 1 second without user interaction.
fn main() {
    assert!(true);
    let window = Window::<TestAppMessage>::new("Vulkan Feature Test");
    let message_proxy = window.new_message_proxy();
    let app = VulkanTestApp::new(&window);
    let join_handle = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        message_proxy.send_event(WindowCommand::RequestClose)
            .unwrap();
    });
    window.run(app);
    join_handle.join().unwrap();
}
