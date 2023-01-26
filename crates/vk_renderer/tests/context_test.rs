
/// Test features in the context module.
/// The context relies on the VkCore, and the core relies on having an actual window to use for the
/// instance and the surface.
///
/// The test creates a window, then creates and destroys a VkCore and VkContext.

use vk_renderer::{VkCore, VkContext};
use window::{
    WindowEventLooper, RenderCycleEvent, RenderEventHandler, ControlFlow, Event, WindowEvent,
    WindowEventHandler, WindowStateEvent, Window, MessageProxy, WindowCommand
};
use std::fmt::Debug;

struct VulkanTestApp {
    message_proxy: MessageProxy<WindowCommand<()>>
}

impl VulkanTestApp {

    fn new<T: Send + Debug>(
        window: &Window,
        message_proxy: MessageProxy<WindowCommand<()>>
    ) -> Self {
        unsafe {
            let mut core = VkCore::new(window, vec![]).unwrap();
            let mut context = VkContext::new(&core, window).unwrap();
            context.teardown();
            core.teardown();
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
    let looper = WindowEventLooper::<()>::new();
    let message_proxy = looper.create_proxy();
    let window = Window::new("Vulkan Context Test", &looper);
    let mut app = VulkanTestApp::new::<()>(&window, message_proxy.clone());
    let running_window_id = window.get_window_id();
    let _code = looper.run_loop(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(command) => {
                match command {
                    WindowCommand::RequestClose => {
                        *control_flow = ControlFlow::Exit
                    },
                    WindowCommand::RequestRedraw => {
                        window.request_redraw();
                    },
                    WindowCommand::Custom(e) => {
                        app.on_window_custom_event(e);
                        ()
                    }
                }
            },
            Event::WindowEvent { event, window_id }
            if window_id == running_window_id => {
                match event {
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
                    _ => {}
                };
            },
            _ => ()
        }
    });
}
