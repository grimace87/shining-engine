
use crate::WindowEventLooper;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, HasRawDisplayHandle, RawDisplayHandle};
use winit::window::WindowId;
use std::fmt::Debug;

pub struct Window {
    window: winit::window::Window
}

impl Window {

    pub fn new<M: 'static + Send + Debug>(app_title: &str, looper: &WindowEventLooper<M>) -> Self {
        let window = winit::window::WindowBuilder::new()
            .with_title(app_title)
            .build(&looper.event_loop)
            .unwrap();
        Self { window }
    }

    pub fn get_window_id(&self) -> WindowId {
        self.window.id()
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

unsafe impl HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.window.raw_display_handle()
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.raw_window_handle()
    }
}
