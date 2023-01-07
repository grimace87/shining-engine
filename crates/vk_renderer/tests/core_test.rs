
/// Test features in the core module.
/// The core relies on having an actual window to use for the instance and the surface.
///
/// The test creates a window, then creates and destroys a VkCore.

use vk_renderer::VkCore;
use window::{
    RenderCycleEvent, RenderEventHandler,
    WindowEventHandler, WindowStateEvent, Window, MessageProxy, WindowCommand
};
use std::fmt::Debug;

struct VulkanTestApp {
    message_proxy: MessageProxy<WindowCommand<()>>
}

impl VulkanTestApp {

    fn new<T: Send + Debug>(
        window: &Window<T>,
        message_proxy: MessageProxy<WindowCommand<()>>
    ) -> Self {
        unsafe {
            VkCore::new(window, vec![]).unwrap();
        }
        Self { message_proxy }
    }
}

impl WindowEventHandler<()> for VulkanTestApp {

    fn on_window_state_event(&mut self, event: WindowStateEvent) {
        if event == WindowStateEvent::FocusGained {
            self.message_proxy.send_event(WindowCommand::RequestClose)
                .unwrap();
        }
    }

    fn on_window_custom_event(&mut self, _event: ()) {}
}

impl RenderEventHandler for VulkanTestApp {
    fn on_render_cycle_event(&self, _event: RenderCycleEvent) {}
}

/// Test: send a RequestClose command via the event loop proxy after the window has gained focus.
/// Expected: window opens and then exits very quickly without issue.
fn main() {
    let window = Window::<()>::new("Vulkan Core Test");
    let message_proxy = window.new_message_proxy();
    let app = VulkanTestApp::new(&window, message_proxy.clone());
    window.run(app);
}
