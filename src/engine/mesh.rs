use super::{Camera, Canvas};
use glam::Mat4;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	pub position: [f32; 3],
	pub color: [f32; 3],
}

pub struct Mesh {
	rpf: wgpu::RenderPipeline,
	rpb: wgpu::RenderPipeline,
	vb: wgpu::Buffer,
	ib: wgpu::Buffer,
	tb: wgpu::Buffer,
	tg: wgpu::BindGroup,
	ilen: u32,
	pub transform: Mat4,
}

impl Mesh {
	pub fn new(
		canvas: &Canvas,
		camera: &Camera,
		vertices: Vec<Vertex>,
		indices: Vec<u16>,
	) -> Self {
		let vb = canvas
			.device
			.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some("mesh vertex_buffer"),
				usage: wgpu::BufferUsages::VERTEX,
				contents: bytemuck::cast_slice(&vertices),
			});
		let ib = canvas
			.device
			.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some("mesh index_buffer"),
				usage: wgpu::BufferUsages::INDEX,
				contents: bytemuck::cast_slice(&indices),
			});
		let tb =
			canvas
				.device
				.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label: Some("mesh transform_buffer"),
					usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
					contents: bytemuck::cast_slice(&[Mat4::IDENTITY]),
				});

		let transform_layout =
			canvas
				.device
				.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					label: Some("mesh transform_layout"),
					entries: &[wgpu::BindGroupLayoutEntry {
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX,
						count: None,
					}],
				});

		let tg = canvas.device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("mesh transform_group"),
			layout: &transform_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: tb.as_entire_binding(),
			}],
		});

		let pipeline_layout =
			canvas
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: Some("mesh pipeline_layout"),
					bind_group_layouts: &[&camera.layout, &transform_layout],
					push_constant_ranges: &[],
				});

		let shader = canvas
			.device
			.create_shader_module(wgpu::include_wgsl!("mesh.wgsl"));

		let targets = [Some(wgpu::ColorTargetState {
			format: canvas.config.format,
			blend: Some(wgpu::BlendState {
				color: wgpu::BlendComponent {
					src_factor: wgpu::BlendFactor::One,
					dst_factor: wgpu::BlendFactor::One,
					operation: wgpu::BlendOperation::Add,
				},
				alpha: wgpu::BlendComponent {
					src_factor: wgpu::BlendFactor::Zero,
					dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
					operation: wgpu::BlendOperation::Add,
				},
			}),
			write_mask: wgpu::ColorWrites::ALL,
		})];

		let mut rp_descriptor = wgpu::RenderPipelineDescriptor {
			label: Some("mesh render_pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[wgpu::VertexBufferLayout {
					step_mode: wgpu::VertexStepMode::Vertex,
					array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
					attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
				}],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &targets,
			}),
			depth_stencil: Some(wgpu::DepthStencilState {
				format: wgpu::TextureFormat::Depth32Float,
				depth_write_enabled: false,
				depth_compare: wgpu::CompareFunction::Less,
				stencil: wgpu::StencilState::default(),
				bias: wgpu::DepthBiasState::default(),
			}),
			primitive: wgpu::PrimitiveState::default(),
			multisample: wgpu::MultisampleState::default(),
			multiview: None,
		};

		rp_descriptor.primitive.cull_mode = Some(wgpu::Face::Front); 
		let rpf = canvas.device.create_render_pipeline(&rp_descriptor);
		rp_descriptor.primitive.cull_mode = Some(wgpu::Face::Back); 
		let rpb = canvas.device.create_render_pipeline(&rp_descriptor);
		Self {
			rpf, rpb,
			vb, ib, tb,
			tg,
			ilen: indices.len() as u32,
			transform: Mat4::IDENTITY,
		}
	}

	pub fn render<'r>(&'r self, render_pass: &mut wgpu::RenderPass<'r>, camera: &'r Camera) {
		render_pass.set_bind_group(0, &camera.group, &[]);
		render_pass.set_bind_group(1, &self.tg, &[]);
		render_pass.set_vertex_buffer(0, self.vb.slice(..));
		render_pass.set_index_buffer(self.ib.slice(..), wgpu::IndexFormat::Uint16);

		render_pass.set_pipeline(&self.rpb);
		render_pass.draw_indexed(0..self.ilen, 0, 0..1);
		render_pass.set_pipeline(&self.rpf);
		render_pass.draw_indexed(0..self.ilen, 0, 0..1);
	}

	pub fn update_transform_buffer(&self, queue: &wgpu::Queue) {
		queue.write_buffer(&self.tb, 0, bytemuck::cast_slice(&[self.transform]));
	}
}

pub struct Quad {
	render_pipeline: wgpu::RenderPipeline,
	transform_buffer: wgpu::Buffer,
	transform_group: wgpu::BindGroup,
}

impl Quad {
	pub fn new(canvas: &Canvas, camera: &Camera) -> Self {
		let transform_buffer =
			canvas
				.device
				.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label: Some("mesh transform_buffer"),
					usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
					contents: bytemuck::cast_slice(&[Mat4::IDENTITY]),
				});

		let transform_layout =
			canvas
				.device
				.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					label: Some("mesh transform_layout"),
					entries: &[wgpu::BindGroupLayoutEntry {
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX,
						count: None,
					}],
				});

		let transform_group = canvas.device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("mesh transform_group"),
			layout: &transform_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: transform_buffer.as_entire_binding(),
			}],
		});

		let pipeline_layout =
			canvas
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: Some("mesh pipeline_layout"),
					bind_group_layouts: &[&camera.layout, &transform_layout],
					push_constant_ranges: &[],
				});

		let shader = canvas
			.device
			.create_shader_module(wgpu::include_wgsl!("quad.wgsl"));

		let render_pipeline =
			canvas
				.device
				.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
					label: Some("mesh render_pipeline"),
					layout: Some(&pipeline_layout),
					vertex: wgpu::VertexState {
						module: &shader,
						entry_point: "vs_main",
						buffers: &[],
					},
					fragment: Some(wgpu::FragmentState {
						module: &shader,
						entry_point: "fs_main",
						targets: &[Some(wgpu::ColorTargetState {
							format: canvas.config.format,
							blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
							write_mask: wgpu::ColorWrites::ALL,
						})],
					}),
					depth_stencil: Some(wgpu::DepthStencilState {
						format: wgpu::TextureFormat::Depth32Float,
						depth_write_enabled: true,
						depth_compare: wgpu::CompareFunction::Less,
						stencil: wgpu::StencilState::default(),
						bias: wgpu::DepthBiasState::default(),
					}),
					primitive: wgpu::PrimitiveState {
						topology: wgpu::PrimitiveTopology::TriangleStrip,
						..Default::default()
					},
					multisample: wgpu::MultisampleState::default(),
					multiview: None,
				});
		Self {
			render_pipeline,
			transform_buffer,
			transform_group,
		}
	}

	pub fn set_transform(&self, queue: &wgpu::Queue, transform_matrix: Mat4) {
		queue.write_buffer(
			&self.transform_buffer,
			0,
			bytemuck::cast_slice(&[transform_matrix]),
		);
	}

	pub fn render<'r>(&'r self, render_pass: &mut wgpu::RenderPass<'r>, camera: &'r Camera) {
		render_pass.set_bind_group(0, &camera.group, &[]);
		render_pass.set_bind_group(1, &self.transform_group, &[]);
		render_pass.set_pipeline(&self.render_pipeline);
		render_pass.draw(0..4, 0..1);
	}
}
