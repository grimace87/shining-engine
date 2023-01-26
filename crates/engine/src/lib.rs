mod renderable;
mod scene;

pub use renderable::{
    Renderable,
    stock::StockRenderable,
    null::NullRenderable
};
pub use scene::{
    Scene, SceneFactory,
    stock::{StockScene, StockSceneFactory}
};

use vk_renderer::{VkError, VkCore, VkContext, RenderpassWrapper, PipelineWrapper};
use window::{
    Window, WindowCommand, WindowStateEvent,
    RenderCycleEvent, KeyCode, KeyState, MessageProxy, WindowEventLooper,
    Event, WindowEvent, KeyboardInput, ControlFlow,
    event::{RenderEventHandler, WindowEventHandler}
};
use resource::{ResourceManager, RawResourceBearer};
use std::fmt::Debug;

pub struct Engine<M: 'static + Send + Debug> {
    app_title: &'static str,
    looper: WindowEventLooper<M>
}

impl<M: 'static + Send + Debug> Engine<M> {

    pub fn new(app_title: &'static str) -> Self {
        Self {
            app_title,
            looper: WindowEventLooper::new()
        }
    }

    pub fn new_message_proxy(&self) -> MessageProxy<WindowCommand<M>> {
        self.looper.create_proxy()
    }

    pub fn run<A>(self, app: A) where
        A: 'static + WindowEventHandler<M> + RenderEventHandler + RawResourceBearer + SceneFactory
    {
        // Create the window
        let window = Window::new(self.app_title, &self.looper);

        // Creation of required components
        let core = unsafe { VkCore::new(&window, vec![]).unwrap() };
        let mut context = VkContext::new(&core, &window).unwrap();
        let mut resource_manager = ResourceManager::new();
        resource_manager.load_resources_from(&context, &app).unwrap();

        // Create the pipelines
        let scene = app.get_scene();
        let renderable = scene.get_renderable();
        let mut pipelines = unsafe {
            Self::create_pipelines(&context, &resource_manager, &renderable).unwrap()
        };

        // Run main loop until completion
        self.run_main_loop(window, app);

        // Release resources
        for (renderpass, pipeline) in pipelines.iter_mut() {
            pipeline.destroy_resources(&context);
            renderpass.destroy_resources(&context);
        }
        resource_manager.free_resources(&mut context).unwrap();
    }

    unsafe fn create_pipelines(
        context: &VkContext,
        resource_manager: &ResourceManager<VkContext>,
        renderable: &Box<dyn Renderable>
    ) -> Result<Vec<(RenderpassWrapper, PipelineWrapper)>, VkError> {

        let swapchain_size = context.get_swapchain_image_count();
        let mut pipeline_set = Vec::new();
        for image_index in 0..swapchain_size {
            let (renderpass, pipeline) = renderable.make_pipeline(
                context,
                resource_manager,
                image_index)?;
            pipeline_set.push((renderpass, pipeline));
        }
        Ok(pipeline_set)
    }

    fn run_main_loop<A>(self, window: Window, mut app: A) where
        A: 'static + WindowEventHandler<M> + RenderEventHandler + RawResourceBearer + SceneFactory
    {
        let running_window_id = window.get_window_id();
        app.on_window_state_event(WindowStateEvent::Starting);
        let code = self.looper.run_loop(move |event, _, control_flow| {
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
                        WindowEvent::KeyboardInput { input, .. } => {
                            let KeyboardInput { virtual_keycode, state, .. } = input;
                            match (virtual_keycode, state) {
                                (Some(KeyCode::Escape), KeyState::Pressed) => {
                                    *control_flow = ControlFlow::Exit;
                                },
                                (Some(keycode), state) => {
                                    app.on_window_state_event(
                                        WindowStateEvent::KeyEvent(
                                            keycode,
                                            state));
                                },
                                _ => {}
                            };
                        },
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
                        WindowEvent::Resized(_) => {
                            app.on_render_cycle_event(RenderCycleEvent::RecreateSurface);
                        },
                        _ => {}
                    };
                },
                Event::MainEventsCleared => {
                    app.on_render_cycle_event(RenderCycleEvent::PrepareUpdate);
                },
                Event::RedrawRequested(_) => {
                    app.on_render_cycle_event(RenderCycleEvent::RenderFrame);
                },
                _ => ()
            }
        });
        println!("Window exited with code {}", code);
    }
}
