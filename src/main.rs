use glam::*;
use std::{f32::consts::FRAC_PI_2, time::Instant};
use winit::{
	event::{
		DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta,
		VirtualKeyCode, WindowEvent,
	},
	event_loop::EventLoop,
	window::Window,
};

mod engine;
use engine::*;

struct State {
	clock: Instant,
	canvas: Canvas,
	camera: Camera,
	surface: Surface,
	quad: Quad,
	camera_transform: (f32, f32, f32),
	pressed: bool,
	mesh_delta: Vec3,
	quad_elapsed: f32,
	play: bool,
	show: bool,
}

impl State {
	fn new(event_loop: &EventLoop<()>) -> Self {
		let window = Window::new(event_loop).unwrap();
		window.set_inner_size(winit::dpi::PhysicalSize::new(500., 500.));
		let canvas = Canvas::new(window);
		let camera = Camera::new(&canvas);

		let surface = Surface::new(
			&canvas,
			&camera,
			"assets/default.rhai"
		);
		let quad = Quad::new(&canvas, &camera);

		Self {
			clock: Instant::now(),
			canvas,
			camera,
			surface,
			quad,
			camera_transform: (1.0, 0.0, 5.),
			pressed: false,
			mesh_delta: Vec3::Z,
			quad_elapsed: 0.0,
			play: false,
			show: true,
		}
	}
}

impl engine::EventHandler for State {
	fn event<'a>(&'a mut self, event: &Event<'a, ()>) {
		match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::Resized(_) => self.camera.resize(&self.canvas),
				WindowEvent::MouseInput { state, button, .. } => {
					self.pressed = *button == MouseButton::Left && *state == ElementState::Pressed;
				}
				WindowEvent::MouseWheel { delta, .. } => {
					let y = match delta {
						MouseScrollDelta::LineDelta(_, y) => *y,
						MouseScrollDelta::PixelDelta(p) => p.y as f32,
					} / self.camera_transform.2;

					if self.camera_transform.2 - y > 0.0 {
						self.camera_transform.2 -= y;
					}
				}
				WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							state: ElementState::Released,
							virtual_keycode: Some(key),
							..
						},
					..
				} => match key {
					VirtualKeyCode::Space => self.play = !self.play,
					VirtualKeyCode::P => self.show = !self.show,
					VirtualKeyCode::Key1 => self.mesh_delta = Vec3::X,
					VirtualKeyCode::Key2 => self.mesh_delta = Vec3::Y,
					VirtualKeyCode::Key3 => self.mesh_delta = Vec3::Z,
					//VirtualKeyCode::T => self.surface.mesh.transform = Mat4::IDENTITY,
					VirtualKeyCode::R => {
						self.surface.update(&self.canvas, &self.camera, "assets/default.rhai");
						self.quad_elapsed = 0.0;
					},
					_ => {}
				},
				_ => {}
			},
			Event::DeviceEvent { event, .. } => match event {
				DeviceEvent::MouseMotion { delta } => {
					if self.pressed {
						let (yaw, pitch, _) = &mut self.camera_transform;

						*yaw -= delta.0 as f32 / 100.;
						*pitch += delta.1 as f32 / 100.;

						if *pitch >= FRAC_PI_2 {
							*pitch = FRAC_PI_2 - f32::EPSILON;
						} else if *pitch < -FRAC_PI_2 {
							*pitch = f32::EPSILON - FRAC_PI_2;
						}
					}
				}
				_ => {}
			},
			_ => {}
		}
	}

	fn update(&mut self) {
		let dt = self.clock.elapsed().as_secs_f32();
		if self.play {
			self.surface.mesh.transform *= Mat4::from_quat(Quat::from_scaled_axis(self.mesh_delta*dt));
			self.quad_elapsed += dt;
		}

		self.clock = Instant::now();
	}

	fn render(&mut self) {
		self.surface.mesh.update_transform_buffer(&self.canvas.queue);
		self.quad.set_transform(
			&self.canvas.queue,
			Mat4::from_translation(Vec3::X * self.quad_elapsed.sin()),
		);
		self.camera.set_transform(
			&self.canvas.queue,
			self.camera_transform.0,
			self.camera_transform.1,
			self.camera_transform.2,
		);
		let frame = self.canvas.surface.get_current_texture().unwrap();
		let view = frame.texture.create_view(&Default::default());
		let mut encoder = self
			.canvas
			.device
			.create_command_encoder(&Default::default());

		{
			let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("render_pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.3,
							g: 0.3,
							b: 0.3,
							a: 1.0,
						}),
						store: true,
					},
				})],
				depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
					view: &self.canvas.depth_view,
					depth_ops: Some(wgpu::Operations {
						load: wgpu::LoadOp::Clear(1.0),
						store: true,
					}),
					stencil_ops: None,
				}),
				..Default::default()
			});
			if self.show {
				self.quad.render(&mut rp, &self.camera);
			}
			self.surface.mesh.render(&mut rp, &self.camera);
		}
		self.canvas.queue.submit(Some(encoder.finish()));
		frame.present();
	}

	fn get_canvas(&mut self) -> &mut Canvas {
		&mut self.canvas
	}
}

fn main() {
	let event_loop = EventLoop::new();
	let state = State::new(&event_loop);
	engine::run(event_loop, state);
}
