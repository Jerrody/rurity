#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unstable_features)]

use engine::Engine;
use winit::event::{Event, KeyboardInput, WindowEvent};

fn main() {
    // TODO: Change to the SDL2 due to more feature availability and capabilities, but maybe it's unreasonable.
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Rurity Alpha Build 0.1.0")
        .with_min_inner_size(winit::dpi::LogicalSize::new(640, 480))
        .with_resizable(false) // TODO: Make resizable.
        .build(&event_loop)
        .unwrap();

    let mut engine = Engine::new(
        &window,
        window.inner_size().width,
        window.inner_size().height,
    )
    .unwrap();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: winit::event::ElementState::Pressed,
                        virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,
            _ => (),
        },
        Event::MainEventsCleared => window.request_redraw(),
        Event::RedrawRequested(_) => engine.draw_call().unwrap(),
        _ => (),
    });
}
