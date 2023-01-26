
use window::{Window, WindowCommand, WindowEventLooper};
use winit::{event::{Event, WindowEvent}, event_loop::ControlFlow};

/// Test: send a RequestClose command via the event loop proxy after 1 second.
/// Expected: window opens and then exits after 1 second without user interaction.
fn main() {
    let looper = WindowEventLooper::<()>::new();
    let message_proxy = looper.create_proxy();
    let window = Window::new("App Does Nothing", &looper);
    let join_handle = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        message_proxy.send_event(WindowCommand::RequestClose)
            .unwrap();
    });
    let running_window_id = window.get_window_id();
    let _code = looper.run_loop(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, window_id }
            if window_id == running_window_id => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    });
    join_handle.join().unwrap();
}
