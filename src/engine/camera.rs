use glam::*;
use wgpu::util::DeviceExt;

use super::Canvas;

pub struct Camera {
	proj_buf: wgpu::Buffer,
	view_buf: wgpu::Buffer,
	pub(super) layout: wgpu::BindGroupLayout,
	pub(super) group: wgpu::BindGroup,
}

impl Camera {
	pub fn new(canvas: &Canvas) -> Self {
		let proj_mat = Mat4::perspective_rh(
			std::f32::consts::FRAC_PI_4,
			canvas.config.width as f32 / canvas.config.height as f32,
			0.1,
			100.0,
		);
		let view_mat = Mat4::look_at_rh(Vec3::X * 10.0, Vec3::ZERO, Vec3::Z);

		let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
		let proj_buf = canvas
			.device
			.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some("camera proj_buffer"),
				usage,
				contents: bytemuck::cast_slice(&[proj_mat]),
			});
		let view_buf = canvas
			.device
			.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some("camera view_buffer"),
				usage,
				contents: bytemuck::cast_slice(&[view_mat]),
			});

		let layout = canvas
			.device
			.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("camera layout"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX,
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						binding: 1,
						visibility: wgpu::ShaderStages::VERTEX,
						count: None,
					},
				],
			});
		let group = canvas.device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("camera group"),
			layout: &layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: proj_buf.as_entire_binding(),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: view_buf.as_entire_binding(),
				},
			],
		});

		Self {
			proj_buf,
			view_buf,
			layout,
			group,
		}
	}

	pub fn resize(&self, canvas: &Canvas) {
		let proj_mat = Mat4::perspective_rh(
			std::f32::consts::FRAC_PI_4,
			canvas.config.width as f32 / canvas.config.height as f32,
			0.1,
			100.0,
		);
		canvas
			.queue
			.write_buffer(&self.proj_buf, 0, bytemuck::cast_slice(&[proj_mat]));
	}

	pub fn set_transform(&self, queue: &wgpu::Queue, yaw: f32, pitch: f32, distance: f32) {
		let (sin_y, cos_y) = yaw.sin_cos();
		let (sin_p, cos_p) = pitch.sin_cos();
		let view_mat = Mat4::look_at_rh(
			vec3(cos_p * cos_y, cos_p * sin_y, sin_p).normalize_or_zero() * distance,
			Vec3::ZERO,
			Vec3::Z,
		);
		queue.write_buffer(&self.view_buf, 0, bytemuck::cast_slice(&[view_mat]));
	}
}
