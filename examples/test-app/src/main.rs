
use engine::{Engine, StockScene, SceneFactory, Scene};
use vk_renderer::VkContext;
use window::{
    RenderEventHandler, RenderCycleEvent, MessageProxy,
    WindowEventHandler, WindowStateEvent, WindowCommand,
    KeyCode, KeyState
};
use std::fmt::Debug;

#[derive(PartialEq, Debug)]
pub enum TestAppMessage {
    RequestQuit
}

struct QuitsQuicklyApp {
    message_proxy: MessageProxy<WindowCommand<TestAppMessage>>
}

impl QuitsQuicklyApp {
    fn new<T: Send + Debug>(message_proxy: MessageProxy<WindowCommand<TestAppMessage>>) -> Self {
        Self { message_proxy }
    }
}

impl WindowEventHandler<TestAppMessage> for QuitsQuicklyApp {

    fn on_window_state_event(&mut self, event: WindowStateEvent) {
        if let WindowStateEvent::KeyEvent(KeyCode::Escape, KeyState::Pressed) = event {
            self.message_proxy.send_event(WindowCommand::RequestClose)
                .unwrap();
        }
    }

    fn on_window_custom_event(&mut self, _event: TestAppMessage) {}
}

impl SceneFactory<VkContext> for QuitsQuicklyApp {
    fn get_scene(&self) -> Box<dyn Scene<VkContext>> {
        Box::new(StockScene::new())
    }
}

impl RenderEventHandler for QuitsQuicklyApp {

    fn on_render_cycle_event(&self, event: RenderCycleEvent) {
        match event {
            RenderCycleEvent::PrepareUpdate(_) => {
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
    let engine = Engine::<TestAppMessage>::new("Demo App");
    let message_proxy = engine.new_message_proxy();
    let app = QuitsQuicklyApp::new::<WindowCommand<TestAppMessage>>(
        message_proxy.clone());
    engine.run(app);
}
