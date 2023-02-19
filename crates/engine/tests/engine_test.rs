
/// Test features in the engine crate.
/// This heavily relies on the Vulkan crate, which is tested more in isolation elsewhere (though it
/// does depend on a window).
///
/// This test creates a more-or-less functioning graphics application.

use engine::{Engine, SceneFactory, Scene, StockScene};
use vk_renderer::VkContext;
use window::{
    RenderCycleEvent, RenderEventHandler, WindowEventHandler, WindowStateEvent, WindowCommand
};

struct EngineTestApp {}

impl WindowEventHandler<()> for EngineTestApp {
    fn on_window_state_event(&mut self, _event: WindowStateEvent) {}
    fn on_window_custom_event(&mut self, _event: ()) {}
}

impl RenderEventHandler for EngineTestApp {
    fn on_render_cycle_event(&self, _event: RenderCycleEvent) {}
}

impl SceneFactory<VkContext> for EngineTestApp {
    fn get_scene(&self) -> Box<dyn Scene<VkContext>> {
        Box::new(StockScene::new())
    }
}

impl EngineTestApp {
    fn new() -> Self {
        Self {}
    }
}

/// Test: send a RequestClose command via the event loop proxy after the window has gained focus.
/// Expected: window opens and then exits very quickly without issue.
fn main() {
    let engine = Engine::<()>::new("Engine Test");
    let message_proxy = engine.new_message_proxy();
    let app = EngineTestApp::new();
    let join_handle = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(3000));
        message_proxy.send_event(WindowCommand::RequestClose)
            .unwrap();
    });
    engine.run(app);
    join_handle.join().unwrap();
}
