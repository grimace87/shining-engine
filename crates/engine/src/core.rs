
use crate::{internals::EngineInternals, SceneFactory};
use window::{
    Window, WindowCommand, WindowStateEvent,
    RenderCycleEvent, KeyCode, KeyState, MessageProxy, WindowEventLooper,
    Event, WindowEvent, KeyboardInput, ControlFlow,
    RenderEventHandler, WindowEventHandler
};
use vk_renderer::PresentResult;
use std::fmt::Debug;

pub struct Engine<M: 'static + Send + Debug> {
    app_title: &'static str,
    looper: Option<WindowEventLooper<M>>
}

impl<M: 'static + Send + Debug> Engine<M> {

    pub fn new(app_title: &'static str) -> Self {
        Self {
            app_title,
            looper: Some(WindowEventLooper::new())
        }
    }

    pub fn new_message_proxy(&self) -> MessageProxy<WindowCommand<M>> {
        let Some(looper) = &self.looper else {
            panic!("Internal error");
        };
        looper.create_proxy()
    }

    pub fn run<A>(self, app: A) where
        A: 'static + WindowEventHandler<M> + RenderEventHandler + SceneFactory
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
        A: 'static + WindowEventHandler<M> + RenderEventHandler + SceneFactory
    {
        let Some(looper) = self.looper.take() else {
            panic!("Internal error");
        };
        let mut internals = {
            let scene = app.get_scene();
            let resource_bearer = scene.get_resource_bearer();
            let internals = EngineInternals::new(&window, &resource_bearer).unwrap();
            internals.record_graphics_commands(&scene).unwrap();
            internals
        };
        let running_window_id = window.get_window_id();
        app.on_window_state_event(WindowStateEvent::Starting);
        let mut scene = app.get_scene();
        let code = looper.run_loop(move |event, _, control_flow| {
            *control_flow = match *control_flow {
                ControlFlow::ExitWithCode(_) => return,
                _ => ControlFlow::Wait
            };
            match event {
                Event::UserEvent(command) => {
                    match command {
                        WindowCommand::RequestClose => {
                            internals.engine_teardown();
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
                                    internals.engine_teardown();
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
                            internals.engine_teardown();
                            *control_flow = ControlFlow::Exit;
                        },
                        WindowEvent::Resized(client_area_dimensions) => {
                            // TODO - this recreates swapchain after first init; is it safe to not init swapchain until this?
                            let last_known_size = internals.get_last_known_size();
                            if last_known_size != client_area_dimensions {
                                let aspect_ratio = client_area_dimensions.width as f32 /
                                    client_area_dimensions.height as f32;
                                app.on_render_cycle_event(
                                    RenderCycleEvent::RecreatingSurface(aspect_ratio));
                                internals.recreate_surface(&window, client_area_dimensions, &scene)
                                    .unwrap();
                            }
                        },
                        _ => {}
                    };
                },
                Event::MainEventsCleared => {
                    // TODO: v-sync?
                    let time_passed_millis = internals.pull_time_step_millis();
                    app.on_render_cycle_event(
                        RenderCycleEvent::PrepareUpdate(time_passed_millis));
                    scene.update(time_passed_millis as f64);
                    window.request_redraw();
                },
                Event::RedrawRequested(_) => {
                    app.on_render_cycle_event(RenderCycleEvent::RenderingFrame);
                    match internals.render_frame(&scene) {
                        Ok(PresentResult::Ok) => {},
                        Ok(PresentResult::SwapchainOutOfDate) => {
                            let last_known_size = internals.get_last_known_size();
                            let aspect_ratio = last_known_size.width as f32 /
                                last_known_size.height as f32;
                            app.on_render_cycle_event(
                                RenderCycleEvent::RecreatingSurface(aspect_ratio));
                            internals.recreate_surface(&window, last_known_size, &scene)
                                .unwrap();
                        },
                        Err(e) => {
                            println!("Rendering error: {:?}", e);
                            internals.engine_teardown();
                            *control_flow = ControlFlow::Exit
                        }
                    }
                },
                _ => ()
            }
        });
        println!("Window exited with code {}", code);
    }
}
