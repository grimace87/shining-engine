
use window::{Window, WindowCommand, WindowEventLooper};
use winit::{event::{Event}, event_loop::ControlFlow};

/// Test: send a RequestClose command via the event loop proxy after 1 second.
/// Expected: window opens and then exits after 1 second without user interaction.
fn main() {
    let looper = WindowEventLooper::<()>::new();
    let message_proxy = looper.create_proxy();
    let _window = Window::new("Window Test", &looper);
    let join_handle = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        message_proxy.send_event(WindowCommand::RequestClose)
            .unwrap();
    });
    let _code = looper.run_loop(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(command) => {
                match command {
                    WindowCommand::RequestClose => {
                        *control_flow = ControlFlow::Exit
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    });
    join_handle.join().unwrap();
}
