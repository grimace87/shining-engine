mod internals;
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

use internals::EngineInternals;
use window::{
    Window, WindowCommand, WindowStateEvent,
    RenderCycleEvent, KeyCode, KeyState, MessageProxy, WindowEventLooper,
    Event, WindowEvent, KeyboardInput, ControlFlow,
    event::{RenderEventHandler, WindowEventHandler}
};
use resource::RawResourceBearer;
use std::fmt::Debug;

pub struct Engine<M: 'static + Send + Debug> {
    app_title: &'static str,
    looper: Option<WindowEventLooper<M>>,
    internals: Option<EngineInternals>
}

impl<M: 'static + Send + Debug> Engine<M> {

    pub fn new(app_title: &'static str) -> Self {
        Self {
            app_title,
            looper: Some(WindowEventLooper::new()),
            internals: None
        }
    }

    pub fn new_message_proxy(&self) -> MessageProxy<WindowCommand<M>> {
        let Some(looper) = &self.looper else {
            panic!("Internal error");
        };
        looper.create_proxy()
    }

    pub fn run<A>(self, app: A) where
        A: 'static + WindowEventHandler<M> + RenderEventHandler + RawResourceBearer + SceneFactory
    {
        // Create the window
        let Some(looper) = &self.looper else {
            panic!("Internal error");
        };
        let window = Window::new(self.app_title, looper);

        // Run main loop until completion
        self.run_main_loop(window, app);
    }

    fn run_main_loop<A>(mut self, window: Window, mut app: A) where
        A: 'static + WindowEventHandler<M> + RenderEventHandler + RawResourceBearer + SceneFactory
    {
        let Some(looper) = self.looper.take() else {
            panic!("Internal error");
        };
        let running_window_id = window.get_window_id();
        app.on_window_state_event(WindowStateEvent::Starting);
        let code = looper.run_loop(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::UserEvent(command) => {
                    match command {
                        WindowCommand::RequestClose => {
                            if let Some(internals) = &mut self.internals {
                                internals.engine_teardown();
                            };
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
                                    if let Some(internals) = &mut self.internals {
                                        internals.engine_teardown();
                                    };
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
                                true => {
                                    if self.internals.is_none() {
                                        self.internals = Some(EngineInternals::new(&window, &app).unwrap());
                                    }
                                    app.on_window_state_event(WindowStateEvent::FocusGained)
                                },
                                false => app.on_window_state_event(WindowStateEvent::FocusLost)
                            };
                        },
                        WindowEvent::CloseRequested => {
                            app.on_window_state_event(WindowStateEvent::Closing);
                            if let Some(internals) = &mut self.internals {
                                internals.engine_teardown();
                            };
                            *control_flow = ControlFlow::Exit;
                        },
                        WindowEvent::Resized(client_area_dimensions) => {
                            if let Some(internals) = &mut self.internals {
                                // TODO - this recreates swapchain after first init; is it safe to not init swapchain until this?
                                let last_known_size = internals.get_last_known_size();
                                if last_known_size != client_area_dimensions {
                                    let aspect_ratio = client_area_dimensions.width as f32 /
                                        client_area_dimensions.height as f32;
                                    app.on_render_cycle_event(
                                        RenderCycleEvent::RecreatingSurface(aspect_ratio));
                                    internals.recreate_surface(&window, client_area_dimensions, &app)
                                        .unwrap();
                                }
                            }
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
