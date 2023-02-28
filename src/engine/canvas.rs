use futures::executor::block_on;
use winit::{dpi::PhysicalSize, window::Window};

pub fn depth(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
	let texture = device.create_texture(&wgpu::TextureDescriptor {
		label: Some("depth_texture"),
		size: wgpu::Extent3d {
			width,
			height,
			depth_or_array_layers: 1,
		},
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
		format: wgpu::TextureFormat::Depth32Float,
		dimension: wgpu::TextureDimension::D2,
		sample_count: 1,
		mip_level_count: 1,
		view_formats: &[],
	});
	texture.create_view(&Default::default())
}

pub struct Canvas {
	pub window: Window,
	pub surface: wgpu::Surface,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub config: wgpu::SurfaceConfiguration,
	pub depth_view: wgpu::TextureView,
}

impl Canvas {
	pub fn new(window: Window) -> Self {
		let instance = wgpu::Instance::default();
		let surface = unsafe { instance.create_surface(&window).unwrap() };

		let (adapter, device, queue) = block_on(async {
			let adapter = instance
				.request_adapter(&wgpu::RequestAdapterOptions {
					compatible_surface: Some(&surface),
					..Default::default()
				})
				.await
				.unwrap();
			let (device, queue) = adapter
				.request_device(
					&wgpu::DeviceDescriptor {
						label: Some("Canvas::Device"),
						limits: wgpu::Limits::downlevel_defaults(),
						..Default::default()
					},
					None,
				)
				.await
				.unwrap();

			(adapter, device, queue)
		});

		let capabilites = surface.get_capabilities(&adapter);
		let format = capabilites.formats[0];
		let alpha_mode = capabilites.alpha_modes[0];

		let PhysicalSize { width, height } = window.inner_size();

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			present_mode: wgpu::PresentMode::Fifo,
			format,
			alpha_mode,
			width,
			height,
			view_formats: vec![],
		};

		surface.configure(&device, &config);

		let depth_view = depth(&device, config.width, config.height);

		Self {
			window,
			surface,
			device,
			queue,
			config,
			depth_view,
		}
	}

	pub fn resize(&mut self, PhysicalSize { width, height }: PhysicalSize<u32>) {
		if width == 0 || height == 0 {
			return;
		}
		self.config.width = width;
		self.config.height = height;
		self.surface.configure(&self.device, &self.config);
		self.depth_view = depth(&self.device, width, height);
	}
}
