mod camera;
mod canvas;
mod surface;
mod mesh;

use winit::{
	event::{ElementState, Event, KeyboardInput, VirtualKeyCode as Key, WindowEvent},
	event_loop::EventLoop,
};

pub use camera::*;
pub use canvas::*;
pub use mesh::*;
pub use surface::*;

pub trait EventHandler {
	fn event<'a>(&'a mut self, _event: &Event<'a, ()>) {}
	fn update(&mut self) {}
	fn render(&mut self) {}
	fn get_canvas(&mut self) -> &mut Canvas;
}

pub fn run<S: EventHandler + 'static>(event_loop: EventLoop<()>, mut state: S) {
	event_loop.run(move |event, _, control_flow| {
		control_flow.set_poll();
		let canvas = state.get_canvas();
		match event {
			Event::WindowEvent { ref event, .. } => match event {
				WindowEvent::Resized(size) => {
					canvas.resize(*size);
				}
				WindowEvent::CloseRequested
				| WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							virtual_keycode: Some(Key::Escape),
							state: ElementState::Pressed,
							..
						},
					..
				} => control_flow.set_exit(),
				_ => {}
			},
			Event::RedrawRequested(_) => state.render(),
			Event::RedrawEventsCleared => canvas.window.request_redraw(),
			_ => {}
		}
		state.event(&event);
		state.update();
	});
}
